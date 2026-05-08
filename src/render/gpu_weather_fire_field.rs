//! Ping-pong **GPU field** for weather + fire **visuals** (compute on `Rgba32Float` textures).
//!
//! - **CPU** uploads [`WeatherFireFieldUniforms`] from [`ChunkWeather`](crate::systems::weather::ChunkWeather)
//!   and [`ChunkSurfaceFire`](crate::systems::fire::ChunkSurfaceFire).
//! - **WGSL** (`assets/shaders/weather_fire_field.wgsl`) relaxes the field each frame.
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

use crate::systems::fire::chunk_surface_fire_tick;
use crate::systems::fire::ChunkSurfaceFire;
use crate::systems::weather::ChunkWeather;

const SHADER_PATH: &str = "shaders/weather_fire_field.wgsl";
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
    pub means: Vec4,
    pub time_secs: f32,
    pub blend_rate: f32,
    pub decay: f32,
    pub _pad: f32,
}

impl Default for WeatherFireFieldUniforms {
    fn default() -> Self {
        Self {
            means: Vec4::ZERO,
            time_secs: 0.0,
            blend_rate: 0.14,
            decay: 0.004,
            _pad: 0.0,
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

fn sync_weather_fire_uniforms(
    time: Res<Time>,
    wx: Query<&ChunkWeather>,
    fire: Query<&ChunkSurfaceFire>,
    mut u: ResMut<WeatherFireFieldUniforms>,
) {
    let mut nw = 0u32;
    let mut r = 0f32;
    let mut s = 0f32;
    let mut fg = 0f32;
    for w in &wx {
        nw += 1;
        r += w.rain_intensity;
        s += w.snow_depth;
        fg += w.fog_density;
    }
    let mut nf = 0u32;
    let mut h_sum = 0f32;
    for f in &fire {
        nf += 1;
        h_sum += f.heat;
    }

    let nf_w = nw.max(1) as f32;
    u.means = Vec4::new(
        r / nf_w,
        s / nf_w,
        if nf > 0 { h_sum / nf.max(1) as f32 } else { 0.0 },
        fg / nf_w,
    );
    u.time_secs = time.elapsed_secs();
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
            .add_systems(Startup, startup_field_textures)
            .add_systems(
                Update,
                (
                    cleanup_debug_sprite,
                    maybe_spawn_debug_sprite.after(cleanup_debug_sprite),
                    sync_weather_fire_uniforms.after(chunk_surface_fire_tick),
                    flip_debug_sprite_texture.after(sync_weather_fire_uniforms),
                ),
            );

        app.add_plugins((
            ExtractResourcePlugin::<WeatherFireFieldUniforms>::default(),
            ExtractResourcePlugin::<WeatherFireFieldTextures>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(RenderStartup, init_weather_fire_pipeline)
            .add_systems(
                Render,
                prepare_field_bind_groups.in_set(RenderSystems::PrepareBindGroups),
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
struct WeatherFireFieldPipeline {
    layout: BindGroupLayoutDescriptor,
    update_pipeline: CachedComputePipelineId,
}

fn prepare_field_bind_groups(
    mut commands: Commands,
    pipeline: Res<WeatherFireFieldPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    textures: Res<WeatherFireFieldTextures>,
    uniforms: Res<WeatherFireFieldUniforms>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
) {
    let va = gpu_images.get(&textures.texture_a).unwrap();
    let vb = gpu_images.get(&textures.texture_b).unwrap();

    let mut ub = UniformBuffer::from(uniforms.clone());
    ub.write_buffer(&render_device, &queue);

    let bg0 = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((&va.texture_view, &vb.texture_view, &ub)),
    );
    let bg1 = render_device.create_bind_group(
        None,
        &pipeline_cache.get_bind_group_layout(&pipeline.layout),
        &BindGroupEntries::sequential((&vb.texture_view, &va.texture_view, &ub)),
    );
    commands.insert_resource(WeatherFireFieldBindGroups([bg0, bg1]));
}

fn init_weather_fire_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
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

    let shader = asset_server.load(SHADER_PATH);
    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![layout.clone()],
        shader,
        entry_point: Some(Cow::from("update")),
        ..default()
    });

    commands.insert_resource(WeatherFireFieldPipeline {
        layout,
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
                pass.set_pipeline(pl);
                let dx = WEATHER_FIRE_FIELD_SIZE.x.div_ceil(WORKGROUP);
                let dy = WEATHER_FIRE_FIELD_SIZE.y.div_ceil(WORKGROUP);
                pass.dispatch_workgroups(dx, dy, 1);
            }
            WfState::PingB => {
                let pl = cache.get_compute_pipeline(pipeline.update_pipeline).unwrap();
                pass.set_bind_group(0, &groups[1], &[]);
                pass.set_pipeline(pl);
                let dx = WEATHER_FIRE_FIELD_SIZE.x.div_ceil(WORKGROUP);
                let dy = WEATHER_FIRE_FIELD_SIZE.y.div_ceil(WORKGROUP);
                pass.dispatch_workgroups(dx, dy, 1);
            }
        }
        Ok(())
    }
}
