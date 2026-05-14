//! Ping-pong **GPU field** for weather + fire **visuals** (compute on `Rgba32Float` textures).
//!
//! - **CPU** uploads [`WeatherFireFieldUniforms`] from [`ClimateVisualAggregate`](crate::render::ClimateVisualAggregate),
//! [`RenderProjectionGraph`](crate::render::extraction::RenderProjectionGraph) (fire node) / [`SimChunkSmokeVisualExtract`](crate::render::sim_visual_extract::SimChunkSmokeVisualExtract)
//!   (via [`crate::systems::atmosphere::gpu_field_bridge`]). No direct [`ChunkWeather`](crate::systems::weather::ChunkWeather) /
//!   [`ChunkEcology`](crate::systems::ecology::ChunkEcology) queries in the bridge.
//! - Packed fire instances are extracted to the render world and uploaded each frame through
//!   [`GPUBufferRegistry`](crate::render::GPUBufferRegistry) ([`FIRE_VISUAL_INSTANCES_BUFFER`](crate::render::FIRE_VISUAL_INSTANCES_BUFFER)).
//! - **WGSL** (`assets/shaders/post/weather_fire_field.wgsl`) relaxes the field each frame.
//! - P2-H partial dirty-rect uploads are planned on the main world ([`crate::systems::atmosphere::AtmosphereGpuFieldBridge`]),
//!   extracted to [`super::atmosphere_partial_gpu::AtmospherePartialGpuExtract`], and written into both ping-pong
//!   textures via [`super::atmosphere_partial_gpu::apply_partial_texture_writes`].
//!   Until [`crate::systems::atmosphere::P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE`] is true, this node still dispatches the full
//!   `WEATHER_FIRE_FIELD_SIZE` compute pass each frame.
//! - Optional **debug sprite** (see [`WeatherFireFieldDebugOverlay`]).
//!
//! This is **not** gameplay state; do not sample into sim without explicit readback.

use std::borrow::Cow;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{
            binding_types::{texture_storage_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderStartup, RenderSystems,
    },
    shader::PipelineCacheError,
};

use crate::gui::representation_band_from_world_lod;
use super::atmosphere_partial_gpu::{
    apply_partial_texture_writes, sync_atmosphere_partial_gpu_extract, AtmospherePartialGpuExtract,
};
use super::extraction::RenderProjectionGraph;
use super::gpu_representation_metrics::GpuRepresentationMetrics;
use super::gpu_buffer_registry::{
    BufferVisibility, GPUBufferRegistry, RegisteredBufferDescriptor,
};
use super::gpu_bind_group_registry::{
    BindGroupBufferBinding, GPUBindGroupRegistry, WEATHER_FIRE_FIELD_FIRE_BIND_GROUP,
};
use super::domain_overlay_gpu::DomainOverlayGpuFrame;
use super::gpu_packed_formats::{
    ecology_overlay_row_format, fire_particle_instance_format, fire_visual_instance_format,
    logistics_overlay_row_format, packed_byte_size,
};
use super::fire_smoke_shader_handles::load_fire_smoke_shader_handles;
use super::gpu_particles::{WorldFireParticleFrame, WorldFireParticleGpuStorage};
use super::sim_visual_extract::SimChunkSmokeVisualExtract;

use crate::systems::atmosphere::{mirror_partial_write_metrics, AtmospherePipelineSet, P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE, WEATHER_FIRE_FIELD_WGSL};

const SHADER_PATH: &str = WEATHER_FIRE_FIELD_WGSL;
pub const WEATHER_FIRE_FIELD_SIZE: UVec2 = UVec2::splat(128);
const WORKGROUP: u32 = 8;

/// Show a small sprite (bottom-left) with the latest field—F3 **Weather / fire GPU field**.
#[derive(Resource, Debug, Clone)]
pub struct WeatherFireFieldDebugOverlay {
    pub show: bool,
}

impl Default for WeatherFireFieldDebugOverlay {
    fn default() -> Self {
        Self { show: false }
    }
}

#[derive(Resource, Clone, ExtractResource, ShaderType)]
pub struct WeatherFireFieldUniforms {
    /// rain, snow, fire_heat_mean, fog
    pub means: Vec4,
    /// biomass_mean, fire_risk_mean, wind_speed_mean, lightning_mean
    pub extra_means: Vec4,
    pub time_secs: f32,
    pub blend_rate: f32,
    pub decay: f32,
    pub _pad: f32,
    /// Number of valid rows in the fire visual storage buffer (WGSL reads `fire_instances[0..count)`).
    pub fire_instance_count: u32,
    pub _fire_pad: UVec3,
    pub partial_origin: UVec2,
    pub partial_extent: UVec2,
    pub partial_active: u32,
    pub _partial_dispatch_pad: u32,
}

impl Default for WeatherFireFieldUniforms {
    fn default() -> Self {
        Self {
            means: Vec4::ZERO,
            extra_means: Vec4::ZERO,
            time_secs: 0.0,
            blend_rate: 0.14,
            decay: 0.004,
            _pad: 0.0,
            fire_instance_count: 0,
            _fire_pad: UVec3::ZERO,
            partial_origin: UVec2::ZERO,
            partial_extent: UVec2::ZERO,
            partial_active: 0,
            _partial_dispatch_pad: 0,
        }
    }
}

#[derive(Resource, Clone, ExtractResource)]
pub struct WeatherFireFieldTextures {
    pub texture_a: Handle<Image>,
    pub texture_b: Handle<Image>,
}

#[derive(Component)]
pub(crate) struct DebugFieldSpriteTag;

#[derive(Resource, Default)]
struct WeatherFieldDebugSpawned(bool);

/// Render-world view of the latest fire instance upload (count only; [`Buffer`] lives in [`GPUBufferRegistry`]).
#[derive(Resource, Default)]
pub struct FireVisualGpuInstanceStorage {
    pub instance_count: u32,
}

fn prepare_fire_visual_gpu_storage(
    extracted: Res<RenderProjectionGraph>,
    mut storage: ResMut<FireVisualGpuInstanceStorage>,
    mut metrics: ResMut<GpuRepresentationMetrics>,
    mut registry: ResMut<GPUBufferRegistry>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let format = fire_visual_instance_format();
    let rows = extracted.fire.instance_buffer.as_slice();
    let lod_cap = extracted.fire.gpu_instance_capacity;
    let alloc_rows = if lod_cap == usize::MAX {
        rows.len().max(1)
    } else {
        lod_cap.max(1)
    };
    let needed = packed_byte_size(format, alloc_rows);
    let id = extracted.fire.buffer_id;
    let stats = registry
        .upload_pod_slice(
            &render_device,
            &queue,
            RegisteredBufferDescriptor {
                id,
                size_bytes: needed,
                usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
                visibility: BufferVisibility::RenderOnly,
                stride: format.stride,
            },
            alloc_rows,
            rows,
            extracted.fire.snapshot_stamp,
        )
        .expect("fire visual GPU buffer registration failed");
    storage.instance_count = stats.active_rows;
    let band = representation_band_from_world_lod(extracted.fire.lod);
    metrics.record_fire_upload(
        band,
        stats.active_rows,
        stats.upload_bytes,
        stats.reserved_rows,
        stats.active_rows,
        stats.reserved_bytes,
    );
}

fn prepare_fire_particle_gpu_storage(
    extracted: Res<WorldFireParticleFrame>,
    mut storage: ResMut<WorldFireParticleGpuStorage>,
    mut metrics: ResMut<GpuRepresentationMetrics>,
    mut registry: ResMut<GPUBufferRegistry>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let format = fire_particle_instance_format();
    let rows = extracted.instances.as_slice();
    let lod_cap = extracted.gpu_capacity;
    let active_rows = if lod_cap == usize::MAX {
        rows.len()
    } else {
        rows.len().min(lod_cap)
    };
    let upload_rows = &rows[..active_rows];
    let alloc_rows = if lod_cap == usize::MAX {
        upload_rows.len().max(1)
    } else {
        lod_cap.max(1)
    };
    let needed = packed_byte_size(format, alloc_rows);
    let stats = registry
        .upload_pod_slice(
            &render_device,
            &queue,
            RegisteredBufferDescriptor {
                id: format.buffer_id,
                size_bytes: needed,
                usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
                visibility: BufferVisibility::RenderOnly,
                stride: format.stride,
            },
            alloc_rows,
            upload_rows,
            extracted.snapshot_stamp,
        )
        .expect("fire particle GPU buffer registration failed");
    storage.instance_count = stats.active_rows;
    storage.expanded_vertex_count = stats.active_rows.saturating_mul(4);
    metrics.record_particle_upload(
        stats.active_rows,
        stats.upload_bytes,
        stats.reserved_bytes,
    );
}

fn prepare_domain_overlay_gpu_storage(
    extracted: Res<DomainOverlayGpuFrame>,
    mut registry: ResMut<GPUBufferRegistry>,
    mut metrics: ResMut<GpuRepresentationMetrics>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let stamp = extracted.stamp.tick;
    let logistics_format = logistics_overlay_row_format();
    let logistics_rows = extracted.logistics_rows.as_slice();
    let logistics_needed = packed_byte_size(logistics_format, logistics_rows.len().max(1));
    if let Ok(stats) = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: logistics_format.buffer_id,
            size_bytes: logistics_needed,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            visibility: BufferVisibility::RenderOnly,
            stride: logistics_format.stride,
        },
        logistics_rows.len().max(1),
        logistics_rows,
        stamp,
    ) {
        metrics.record_domain_overlay_upload(
            stats.upload_bytes,
            stats.reserved_bytes,
            stats.active_rows,
        );
    }

    let ecology_format = ecology_overlay_row_format();
    let ecology_rows = extracted.ecology_rows.as_slice();
    let ecology_needed = packed_byte_size(ecology_format, ecology_rows.len().max(1));
    if let Ok(stats) = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: ecology_format.buffer_id,
            size_bytes: ecology_needed,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            visibility: BufferVisibility::RenderOnly,
            stride: ecology_format.stride,
        },
        ecology_rows.len().max(1),
        ecology_rows,
        stamp,
    ) {
        metrics.record_domain_overlay_upload(
            stats.upload_bytes,
            stats.reserved_bytes,
            stats.active_rows,
        );
    }
}

fn make_field_image() -> Image {
    let mut img = Image::new_target_texture(
        WEATHER_FIRE_FIELD_SIZE.x,
        WEATHER_FIRE_FIELD_SIZE.y,
        TextureFormat::Rgba32Float,
        None,
    );
    img.asset_usage = RenderAssetUsages::RENDER_WORLD;
    img.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    img
}

fn startup_field_textures(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let ha = images.add(make_field_image());
    let hb = images.add(make_field_image());
    commands.insert_resource(WeatherFireFieldTextures {
        texture_a: ha,
        texture_b: hb,
    });
}

fn cleanup_debug_sprite(
    mut commands: Commands,
    overlay: Res<WeatherFireFieldDebugOverlay>,
    q: Query<Entity, With<DebugFieldSpriteTag>>,
    mut spawn_gate: ResMut<WeatherFieldDebugSpawned>,
) {
    if overlay.show {
        return;
    }
    for e in &q {
        commands.entity(e).despawn();
    }
    spawn_gate.0 = false;
}

fn maybe_spawn_debug_sprite(
    mut commands: Commands,
    overlay: Res<WeatherFireFieldDebugOverlay>,
    tex: Res<WeatherFireFieldTextures>,
    mut gate: ResMut<WeatherFieldDebugSpawned>,
    existing: Query<(), With<DebugFieldSpriteTag>>,
) {
    if !overlay.show {
        return;
    }
    if !existing.is_empty() {
        gate.0 = true;
        return;
    }
    if gate.0 {
        return;
    }
    commands.spawn((
        DebugFieldSpriteTag,
        Name::new("WeatherFireFieldDebug"),
        Sprite {
            image: tex.texture_a.clone(),
            custom_size: Some(WEATHER_FIRE_FIELD_SIZE.as_vec2() * 3.0),
            ..default()
        },
        Transform::from_translation(Vec3::new(-580.0, -300.0, 2000.0)),
    ));
    gate.0 = true;
}

/// Match ping-pong write target (same pattern as Bevy `compute_shader_game_of_life` example).
fn flip_debug_sprite_texture(
    tex: Res<WeatherFireFieldTextures>,
    overlay: Res<WeatherFireFieldDebugOverlay>,
    mut q: Query<&mut Sprite, With<DebugFieldSpriteTag>>,
) {
    if !overlay.show {
        return;
    }
    for mut spr in &mut q {
        if spr.image == tex.texture_a {
            spr.image = tex.texture_b.clone();
        } else {
            spr.image = tex.texture_a.clone();
        }
    }
}

pub struct GpuWeatherFireFieldPlugin;

impl Plugin for GpuWeatherFireFieldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherFireFieldUniforms>()
            .init_resource::<WeatherFireFieldDebugOverlay>()
            .init_resource::<WeatherFieldDebugSpawned>()
            .add_systems(
                Startup,
                (startup_field_textures, load_fire_smoke_shader_handles).chain(),
            )
            .add_systems(
                Update,
                (
                    cleanup_debug_sprite,
                    maybe_spawn_debug_sprite.after(cleanup_debug_sprite),
                    flip_debug_sprite_texture,
                ),
            );

        app.init_resource::<AtmospherePartialGpuExtract>()
            .add_systems(
                Update,
                sync_atmosphere_partial_gpu_extract
                    .after(mirror_partial_write_metrics)
                    .in_set(AtmospherePipelineSet::FieldFill),
            );

        app.add_plugins((
            ExtractResourcePlugin::<WeatherFireFieldUniforms>::default(),
            ExtractResourcePlugin::<WeatherFireFieldTextures>::default(),
            ExtractResourcePlugin::<AtmospherePartialGpuExtract>::default(),
            ExtractResourcePlugin::<RenderProjectionGraph>::default(),
            ExtractResourcePlugin::<WorldFireParticleFrame>::default(),
            ExtractResourcePlugin::<DomainOverlayGpuFrame>::default(),
            ExtractResourcePlugin::<SimChunkSmokeVisualExtract>::default(),
        ));

        crate::render::register_world_fire_particle_draw(app);
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<GPUBufferRegistry>()
            .init_resource::<GPUBindGroupRegistry>()
            .init_resource::<FireVisualGpuInstanceStorage>()
            .init_resource::<WorldFireParticleGpuStorage>()
            .init_resource::<GpuRepresentationMetrics>()
            .add_systems(RenderStartup, init_weather_fire_pipeline)
            .add_systems(
                Render,
                (
                    apply_partial_texture_writes,
                    prepare_fire_visual_gpu_storage.after(apply_partial_texture_writes),
                    prepare_fire_particle_gpu_storage.after(prepare_fire_visual_gpu_storage),
                    prepare_domain_overlay_gpu_storage.after(prepare_fire_particle_gpu_storage),
                    prepare_field_bind_groups.after(prepare_domain_overlay_gpu_storage),
                )
                    .chain()
                    .in_set(RenderSystems::PrepareBindGroups),
            );

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(WeatherFireFieldLabel, WeatherFireFieldNode::default());
        graph.add_node_edge(WeatherFireFieldLabel, bevy::render::graph::CameraDriverLabel);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct WeatherFireFieldLabel;

#[derive(Resource)]
struct WeatherFireFieldBindGroups([BindGroup; 2]);

#[derive(Resource)]
struct WeatherFireFieldFireBindGroup(BindGroup);

#[derive(Resource)]
struct WeatherFireFieldPipeline {
    field_layout: BindGroupLayoutDescriptor,
    fire_layout: BindGroupLayoutDescriptor,
    update_pipeline: CachedComputePipelineId,
}

fn prepare_field_bind_groups(
    mut commands: Commands,
    pipeline: Res<WeatherFireFieldPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    textures: Res<WeatherFireFieldTextures>,
    mut uniforms: ResMut<WeatherFireFieldUniforms>,
    partial: Res<AtmospherePartialGpuExtract>,
    extracted: Res<RenderProjectionGraph>,
    registry: Res<GPUBufferRegistry>,
    mut bind_registry: ResMut<GPUBindGroupRegistry>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
) {
    let va = gpu_images.get(&textures.texture_a).unwrap();
    let vb = gpu_images.get(&textures.texture_b).unwrap();

    if P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE && partial.partial_dispatch_active {
        uniforms.partial_origin = partial.partial_dispatch_origin;
        uniforms.partial_extent = partial.partial_dispatch_extent;
        uniforms.partial_active = 1;
    } else {
        uniforms.partial_origin = UVec2::ZERO;
        uniforms.partial_extent = UVec2::ZERO;
        uniforms.partial_active = 0;
    }

    let mut ub = UniformBuffer::from((*uniforms).clone());
    ub.write_buffer(&render_device, &queue);

    let field_gpu_layout = pipeline_cache.get_bind_group_layout(&pipeline.field_layout);
    let fire_gpu_layout = pipeline_cache.get_bind_group_layout(&pipeline.fire_layout);

    let bg0 = render_device.create_bind_group(
        None,
        &field_gpu_layout,
        &BindGroupEntries::sequential((&va.texture_view, &vb.texture_view, &ub)),
    );
    let bg1 = render_device.create_bind_group(
        None,
        &field_gpu_layout,
        &BindGroupEntries::sequential((&vb.texture_view, &va.texture_view, &ub)),
    );
    commands.insert_resource(WeatherFireFieldBindGroups([bg0, bg1]));

    let buffer_id = extracted.fire.buffer_id;
    if !bind_registry.is_valid(WEATHER_FIRE_FIELD_FIRE_BIND_GROUP, &registry) {
        let fire_buf = registry
            .get(buffer_id)
            .expect("fire visual GPU buffer must exist after prepare_fire_visual_gpu_storage");
        let fire_bg = render_device.create_bind_group(
            None,
            &fire_gpu_layout,
            &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &fire_buf.buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        );
        bind_registry.insert(
            WEATHER_FIRE_FIELD_FIRE_BIND_GROUP,
            fire_bg,
            vec![BindGroupBufferBinding {
                buffer_id,
                buffer_version: fire_buf.version,
            }],
        );
    }
    let fire_entry = bind_registry
        .get(WEATHER_FIRE_FIELD_FIRE_BIND_GROUP)
        .expect("fire bind group must exist after prepare");
    commands.insert_resource(WeatherFireFieldFireBindGroup(fire_entry.bind_group.clone()));
}

fn init_weather_fire_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let field_layout = BindGroupLayoutDescriptor::new(
        "WeatherFireField",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
                texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                uniform_buffer::<WeatherFireFieldUniforms>(false),
            ),
        ),
    );

    let fire_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("WeatherFireFieldFireInstances"),
        entries: vec![BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };

    let shader = asset_server.load(SHADER_PATH);
    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![field_layout.clone(), fire_layout.clone()],
        shader,
        entry_point: Some(Cow::from("update")),
        ..default()
    });

    commands.insert_resource(WeatherFireFieldPipeline {
        field_layout,
        fire_layout,
        update_pipeline,
    });
}

enum WfState {
    Loading,
    PingA,
    PingB,
}

struct WeatherFireFieldNode {
    state: WfState,
}

impl Default for WeatherFireFieldNode {
    fn default() -> Self {
        Self {
            state: WfState::Loading,
        }
    }
}

impl render_graph::Node for WeatherFireFieldNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<WeatherFireFieldPipeline>();
        let cache = world.resource::<PipelineCache>();
        match self.state {
            WfState::Loading => match cache.get_compute_pipeline_state(pipeline.update_pipeline) {
                CachedPipelineState::Ok(_) => self.state = WfState::PingA,
                CachedPipelineState::Err(PipelineCacheError::ShaderNotLoaded(_)) => {}
                CachedPipelineState::Err(e) => {
                    panic!("Loading assets/{SHADER_PATH} for weather/fire field:\n{e}")
                }
                _ => {}
            },
            WfState::PingA => self.state = WfState::PingB,
            WfState::PingB => self.state = WfState::PingA,
        }
    }

    fn run(
        &self,
        _ctx: &mut render_graph::RenderGraphContext,
        render_ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let groups = &world.resource::<WeatherFireFieldBindGroups>().0;
        let fire_bg = world.resource::<WeatherFireFieldFireBindGroup>();
        let cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<WeatherFireFieldPipeline>();
        let partial = world.resource::<AtmospherePartialGpuExtract>();
        let uniforms = world.resource::<WeatherFireFieldUniforms>();

        let mut pass = render_ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        let (dx, dy) = if P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE && partial.partial_dispatch_active {
            (
                partial.partial_dispatch_extent.x.div_ceil(WORKGROUP),
                partial.partial_dispatch_extent.y.div_ceil(WORKGROUP),
            )
        } else if P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE && partial.full_field_fallback {
            (
                WEATHER_FIRE_FIELD_SIZE.x.div_ceil(WORKGROUP),
                WEATHER_FIRE_FIELD_SIZE.y.div_ceil(WORKGROUP),
            )
        } else if P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE {
            (0, 0)
        } else {
            (
                WEATHER_FIRE_FIELD_SIZE.x.div_ceil(WORKGROUP),
                WEATHER_FIRE_FIELD_SIZE.y.div_ceil(WORKGROUP),
            )
        };

        if dx == 0 || dy == 0 {
            return Ok(());
        }

        match self.state {
            WfState::Loading => {}
            WfState::PingA => {
                let pl = cache.get_compute_pipeline(pipeline.update_pipeline).unwrap();
                pass.set_bind_group(0, &groups[0], &[]);
                pass.set_bind_group(1, &fire_bg.0, &[]);
                pass.set_pipeline(pl);
                pass.dispatch_workgroups(dx, dy, 1);
            }
            WfState::PingB => {
                let pl = cache.get_compute_pipeline(pipeline.update_pipeline).unwrap();
                pass.set_bind_group(0, &groups[1], &[]);
                pass.set_bind_group(1, &fire_bg.0, &[]);
                pass.set_pipeline(pl);
                pass.dispatch_workgroups(dx, dy, 1);
            }
        }
        let _ = uniforms;
        Ok(())
    }
}
