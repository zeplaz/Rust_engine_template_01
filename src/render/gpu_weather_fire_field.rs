//! Ping-pong **GPU field** for weather + fire **visuals** (compute on `Rgba32Float` textures).
//!
//! - **CPU** uploads [`WeatherFireFieldUniforms`] from [`ClimateVisualAggregate`](crate::render::ClimateVisualAggregate),
//! [`FireVisualFrame`](crate::render::extraction::FireVisualFrame) / [`SimChunkSmokeVisualExtract`](crate::render::sim_visual_extract::SimChunkSmokeVisualExtract)
//!   (via [`crate::systems::atmosphere::gpu_field_bridge`]). No direct [`ChunkWeather`](crate::systems::weather::ChunkWeather) /
//!   [`ChunkEcology`](crate::systems::ecology::ChunkEcology) queries in the bridge.
//! - Packed fire instances are extracted to the render world and uploaded each frame into
//!   [`FireVisualGpuInstanceStorage`] (wgpu storage buffer, group `@group(1)` in `weather_fire_field.wgsl`).
//! - **WGSL** (`assets/shaders/post/weather_fire_field.wgsl`) relaxes the field each frame.
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

use bytemuck;

use super::extraction::FireVisualFrame;
use super::fire_smoke_shader_handles::load_fire_smoke_shader_handles;
use super::fx_burst_request::{enqueue_fx_bursts_from_hot_emitters, FxParticleBurstRequest};
use super::sim_visual_extract::{FireVisualGpuInstance, SimChunkSmokeVisualExtract};

use crate::systems::atmosphere::{
    update_fire_emitters_from_heat, AtmospherePipelineSet, WEATHER_FIRE_FIELD_WGSL,
};

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

/// GPU copy of [`FireVisualFrame::instances`] (render world). Bound at `@group(1) @binding(0)`.
#[derive(Resource, Default)]
pub struct FireVisualGpuInstanceStorage {
    pub buffer: Option<Buffer>,
    pub capacity_bytes: u64,
    pub instance_count: u32,
}

fn prepare_fire_visual_gpu_storage(
    extracted: Res<FireVisualFrame>,
    mut storage: ResMut<FireVisualGpuInstanceStorage>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    const STRIDE: usize = std::mem::size_of::<FireVisualGpuInstance>();
    let rows = extracted.instances.as_slice();
    let needed = (rows.len().max(1) * STRIDE) as u64;
    let must_grow = storage
        .buffer
        .as_ref()
        .map_or(true, |_| storage.capacity_bytes < needed);
    if must_grow {
        let size = needed.max(256).next_multiple_of(256);
        let buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("fire_visual_instances"),
            size,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        storage.buffer = Some(buffer);
        storage.capacity_bytes = size;
    }
    storage.instance_count = rows.len() as u32;
    if let Some(buf) = storage.buffer.as_ref() {
        if rows.is_empty() {
            queue.write_buffer(buf, 0, &[0u8; STRIDE]);
        } else {
            queue.write_buffer(buf, 0, bytemuck::cast_slice(rows));
        }
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

fn stub_drain_fx_burst_requests(mut _reader: MessageReader<FxParticleBurstRequest>) {}

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
            .add_message::<FxParticleBurstRequest>()
            .add_systems(
                Update,
                (
                    cleanup_debug_sprite,
                    maybe_spawn_debug_sprite.after(cleanup_debug_sprite),
                    flip_debug_sprite_texture,
                    enqueue_fx_bursts_from_hot_emitters
                        .in_set(AtmospherePipelineSet::Emitters)
                        .after(update_fire_emitters_from_heat),
                    stub_drain_fx_burst_requests.after(enqueue_fx_bursts_from_hot_emitters),
                ),
            );

        app.add_plugins((
            ExtractResourcePlugin::<WeatherFireFieldUniforms>::default(),
            ExtractResourcePlugin::<WeatherFireFieldTextures>::default(),
            ExtractResourcePlugin::<FireVisualFrame>::default(),
            ExtractResourcePlugin::<SimChunkSmokeVisualExtract>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<FireVisualGpuInstanceStorage>()
            .add_systems(RenderStartup, init_weather_fire_pipeline)
            .add_systems(
                Render,
                (
                    prepare_fire_visual_gpu_storage,
                    prepare_field_bind_groups.after(prepare_fire_visual_gpu_storage),
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
    uniforms: Res<WeatherFireFieldUniforms>,
    storage: Res<FireVisualGpuInstanceStorage>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
) {
    let va = gpu_images.get(&textures.texture_a).unwrap();
    let vb = gpu_images.get(&textures.texture_b).unwrap();

    let mut ub = UniformBuffer::from(uniforms.clone());
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

    let fire_buf = storage.buffer.as_ref().expect("fire visual GPU buffer must exist after prepare_fire_visual_gpu_storage");
    let fire_bg = render_device.create_bind_group(
        None,
        &fire_gpu_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(BufferBinding {
                buffer: fire_buf,
                offset: 0,
                size: None,
            }),
        }],
    );
    commands.insert_resource(WeatherFireFieldFireBindGroup(fire_bg));
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

        let mut pass = render_ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        match self.state {
            WfState::Loading => {}
            WfState::PingA => {
                let pl = cache.get_compute_pipeline(pipeline.update_pipeline).unwrap();
                pass.set_bind_group(0, &groups[0], &[]);
                pass.set_bind_group(1, &fire_bg.0, &[]);
                pass.set_pipeline(pl);
                let dx = WEATHER_FIRE_FIELD_SIZE.x.div_ceil(WORKGROUP);
                let dy = WEATHER_FIRE_FIELD_SIZE.y.div_ceil(WORKGROUP);
                pass.dispatch_workgroups(dx, dy, 1);
            }
            WfState::PingB => {
                let pl = cache.get_compute_pipeline(pipeline.update_pipeline).unwrap();
                pass.set_bind_group(0, &groups[1], &[]);
                pass.set_bind_group(1, &fire_bg.0, &[]);
                pass.set_pipeline(pl);
                let dx = WEATHER_FIRE_FIELD_SIZE.x.div_ceil(WORKGROUP);
                let dy = WEATHER_FIRE_FIELD_SIZE.y.div_ceil(WORKGROUP);
                pass.dispatch_workgroups(dx, dy, 1);
            }
        }
        Ok(())
    }
}
