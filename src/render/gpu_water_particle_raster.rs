//! Raster pass for world water particles (D-W05/D-W10 — fire spine clone).

use std::borrow::Cow;

use bevy::asset::AssetServer;
use bevy::core_pipeline::core_2d::{
    graph::Core2d,
    CORE_2D_DEPTH_FORMAT,
};
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bevy::render::{
    camera::ExtractedCamera,
    extract_resource::ExtractResource,
    render_graph::{self, RenderGraph, RenderLabel, ViewNode, ViewNodeRunner},
    render_resource::{
        binding_types::uniform_buffer,
        BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
        BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding, BufferBindingType,
        BlendComponent, BlendFactor, BlendOperation, BlendState, CachedPipelineState,
        CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
        DepthStencilState, FragmentState, FrontFace, LoadOp, MultisampleState, PipelineCache,
        PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassDescriptor,
        RenderPipelineDescriptor, ShaderStages, StencilFaceState, StencilState, StoreOp,
        TextureFormat, UniformBuffer, VertexState,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue},
    view::{Msaa, ViewDepthTexture, ViewTarget},
    Render, RenderApp, RenderStartup, RenderSystems,
};

use crate::gui::{MainWorldCamera, RepresentationResult, TileDebugRenderHost};
use crate::render::gpu_buffer_registry::{GPUBufferRegistry, WATER_PARTICLE_EXPANDED_VERTICES_BUFFER};
use crate::render::gpu_water_particles::WorldWaterParticleFrame;

pub const WATER_PARTICLE_DRAW_WGSL: &str = "shaders/water/water_particle_draw.wgsl";

const MSAA_SAMPLES: [u32; 4] = [1, 2, 4, 8];

#[inline]
fn msaa_index(samples: u32) -> usize {
    match samples {
        1 => 0,
        2 => 1,
        4 => 2,
        8 => 3,
        _ => 2,
    }
}

#[derive(Resource, Clone, Copy, Default, ExtractResource, ShaderType)]
pub struct WaterParticleDrawGlobals {
    pub view_proj: Mat4,
    pub vertex_count: u32,
    pub time_secs: f32,
    pub zoom_alpha: f32,
    pub _pad: f32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(crate) struct WorldWaterParticleRasterPassLabel;

#[derive(Resource)]
struct WaterParticleRasterPipeline {
    globals_layout: BindGroupLayoutDescriptor,
    expanded_layout: BindGroupLayoutDescriptor,
    pipelines: [[CachedRenderPipelineId; 4]; 2],
}

#[derive(Resource, Default)]
struct WaterParticleRasterBindGpu {
    uniform: UniformBuffer<WaterParticleDrawGlobals>,
    bind_group_0: Option<BindGroup>,
    bind_group_1: Option<BindGroup>,
    storage_version: u64,
}

#[derive(Default)]
struct WaterParticleRasterNode {
    pipeline_ready: bool,
}

pub fn register_world_water_particle_raster(app: &mut App) {
    app.init_resource::<WaterParticleDrawGlobals>()
        .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            WaterParticleDrawGlobals,
        >::default())
        .add_systems(
            Update,
            sync_water_particle_draw_globals
                .after(crate::render::gpu_water_particles::emit_world_water_particles_from_catalog)
                .run_if(crate::gui::in_simulation_or_editor_map),
        );

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app
        .init_resource::<WaterParticleRasterBindGpu>()
        .add_systems(
            RenderStartup,
            (init_water_particle_raster_pipeline, install_water_particle_raster_graph_node).chain(),
        )
        .add_systems(
            Render,
            prepare_water_particle_raster_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        );
}

fn sync_water_particle_draw_globals(
    policy: Res<RepresentationResult>,
    particles: Res<WorldWaterParticleFrame>,
    cam_scale: Res<crate::render::gpu_particles::FireParticleCameraScale>,
    mut globals: ResMut<WaterParticleDrawGlobals>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
) {
    globals.vertex_count = 0;
    globals.time_secs = particles.anim_time_secs;
    globals.zoom_alpha = cam_scale.zoom_alpha;
    if !policy.particle_policy.instanced_draw {
        return;
    }
    let cap = particles.instances.len();
    if cap == 0 {
        return;
    }
    let Ok((camera, gt)) = cam_q.single() else {
        return;
    };
    let view_from_world = Mat4::from(gt.affine().inverse());
    globals.view_proj = camera.clip_from_view() * view_from_world;
    globals.vertex_count = (cap as u32).saturating_mul(6);
}

fn init_water_particle_raster_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let globals_layout = BindGroupLayoutDescriptor::new(
        "water_particle_globals_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (uniform_buffer::<WaterParticleDrawGlobals>(false),),
        ),
    );
    let expanded_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("water_particle_expanded_layout"),
        entries: vec![BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };

    let shader = asset_server.load(WATER_PARTICLE_DRAW_WGSL);
    let pipelines = std::array::from_fn(|hdr| {
        let fmt = if hdr == 0 {
            TextureFormat::bevy_default()
        } else {
            ViewTarget::TEXTURE_FORMAT_HDR
        };
        std::array::from_fn(|si| {
            let samples = MSAA_SAMPLES[si];
            let desc = RenderPipelineDescriptor {
                label: Some(Cow::Borrowed("water_particle_raster")),
                layout: vec![globals_layout.clone(), expanded_layout.clone()],
                push_constant_ranges: vec![],
                vertex: VertexState {
                    shader: shader.clone(),
                    entry_point: Some(Cow::Borrowed("vs_main")),
                    shader_defs: vec![],
                    buffers: vec![],
                },
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    entry_point: Some(Cow::Borrowed("fs_main")),
                    shader_defs: vec![],
                    targets: vec![Some(ColorTargetState {
                        format: fmt,
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::SrcAlpha,
                                dst_factor: BlendFactor::One,
                                operation: BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::One,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                        }),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: CORE_2D_DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: CompareFunction::Always,
                    stencil: StencilState {
                        front: StencilFaceState::IGNORE,
                        back: StencilFaceState::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    },
                    bias: DepthBiasState {
                        constant: -2,
                        slope_scale: -0.5,
                        clamp: 0.0,
                    },
                }),
                multisample: MultisampleState {
                    count: samples,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                zero_initialize_workgroup_memory: true,
            };
            pipeline_cache.queue_render_pipeline(desc)
        })
    });

    commands.insert_resource(WaterParticleRasterPipeline {
        globals_layout,
        expanded_layout,
        pipelines,
    });
}

fn install_water_particle_raster_graph_node(world: &mut World) {
    let runner = ViewNodeRunner::<WaterParticleRasterNode>::from_world(world);
    let mut graph = world.resource_mut::<RenderGraph>();
    let Some(sub) = graph.get_sub_graph_mut(Core2d) else {
        return;
    };
    sub.add_node(WorldWaterParticleRasterPassLabel, runner);
    crate::render::gpu_fire_particle_raster::relink_core2d_transparent_overlay_order(sub);
}

fn prepare_water_particle_raster_bind_groups(
    globals: Res<WaterParticleDrawGlobals>,
    pipeline: Res<WaterParticleRasterPipeline>,
    registry: Res<GPUBufferRegistry>,
    mut bind_gpu: ResMut<WaterParticleRasterBindGpu>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
) {
    bind_gpu.uniform.set(*globals);
    bind_gpu.uniform.write_buffer(&render_device, &queue);

    let globals_layout = pipeline_cache.get_bind_group_layout(&pipeline.globals_layout);
    let Some(globals_binding) = bind_gpu.uniform.binding() else {
        return;
    };
    bind_gpu.bind_group_0 = Some(render_device.create_bind_group(
        "water_particle_globals_bind_group",
        &globals_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: globals_binding,
        }],
    ));

    if globals.vertex_count == 0 {
        bind_gpu.bind_group_1 = None;
        bind_gpu.storage_version = 0;
        return;
    }

    let Some(storage) = registry.get(WATER_PARTICLE_EXPANDED_VERTICES_BUFFER) else {
        bind_gpu.bind_group_1 = None;
        bind_gpu.storage_version = 0;
        return;
    };

    if bind_gpu.storage_version == storage.version && bind_gpu.bind_group_1.is_some() {
        return;
    }

    bind_gpu.storage_version = storage.version;
    let expanded_layout = pipeline_cache.get_bind_group_layout(&pipeline.expanded_layout);
    bind_gpu.bind_group_1 = Some(render_device.create_bind_group(
        "water_particle_expanded_bind_group",
        &expanded_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(BufferBinding {
                buffer: &storage.buffer,
                offset: 0,
                size: None,
            }),
        }],
    ));
}

impl ViewNode for WaterParticleRasterNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static Msaa,
        Has<TileDebugRenderHost>,
    );

    fn update(&mut self, world: &mut World) {
        if self.pipeline_ready {
            return;
        }
        let Some(pl) = world.get_resource::<WaterParticleRasterPipeline>() else {
            return;
        };
        let cache = world.resource::<PipelineCache>();
        let mut all_ok = true;
        for row in &pl.pipelines {
            for id in row {
                match cache.get_render_pipeline_state(*id) {
                    CachedPipelineState::Ok(_) => {}
                    _ => all_ok = false,
                }
            }
        }
        if all_ok {
            self.pipeline_ready = true;
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        (camera, view_target, depth, msaa, host): bevy::ecs::query::QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if !host || !self.pipeline_ready {
            return Ok(());
        }
        let globals = world.resource::<WaterParticleDrawGlobals>();
        if globals.vertex_count == 0 {
            return Ok(());
        }
        let bind = world.resource::<WaterParticleRasterBindGpu>();
        let Some(bg0) = bind.bind_group_0.as_ref() else {
            return Ok(());
        };
        let Some(bg1) = bind.bind_group_1.as_ref() else {
            return Ok(());
        };

        let pipeline_res = world.resource::<WaterParticleRasterPipeline>();
        let cache = world.resource::<PipelineCache>();
        let hdr = usize::from(view_target.is_hdr());
        let si = msaa_index(msaa.samples());
        let pipeline_id = pipeline_res.pipelines[hdr][si];
        let Some(pl) = cache.get_render_pipeline(pipeline_id) else {
            return Ok(());
        };

        let mut color = view_target.get_color_attachment();
        color.ops.load = LoadOp::Load;
        let depth_stencil = Some(depth.get_attachment(StoreOp::Store));

        let mut pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("water_particle_raster"),
            color_attachments: &[Some(color)],
            depth_stencil_attachment: depth_stencil,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if let Some(viewport) = camera.viewport.as_ref() {
            pass.set_camera_viewport(viewport);
        }

        pass.set_render_pipeline(pl);
        pass.set_bind_group(0, bg0, &[]);
        pass.set_bind_group(1, bg1, &[]);
        pass.draw(0..globals.vertex_count, 0..1);
        Ok(())
    }
}
