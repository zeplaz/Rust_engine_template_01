//! W1 water overlay raster pass — river ribbons + lake/ocean motion on Core2d.

use std::borrow::Cow;

use bevy::asset::AssetServer;
use bevy::core_pipeline::{Core2d, core_2d::CORE_2D_DEPTH_FORMAT};
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::{
    camera::ExtractedCamera,
    extract_resource::ExtractResourcePlugin,
    render_resource::{
        binding_types::uniform_buffer,
        BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
        BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding,
        BufferBindingType, BufferDescriptor, BufferUsages, CachedPipelineState, CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
        DepthStencilState, FragmentState, FrontFace, LoadOp, MultisampleState, PipelineCache,
        PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassDescriptor,
        RenderPipelineDescriptor, ShaderStages, StencilFaceState, StencilState, StoreOp,
        TextureFormat, UniformBuffer, VertexState,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
    view::{ExtractedView, Msaa, ViewDepthTexture, ViewTarget},
    Render, RenderApp, RenderStartup, RenderSystems,
};

use crate::gui::{MainWorldCamera, TileDebugRenderHost};
use crate::render::core2d_overlay_order::{
    core2d_overlay_pipeline_hdr_index, Core2dOverlaySet, CORE2D_OVERLAY_SDR_FORMAT,
};
use crate::render::water_surface_visual::{WaterOverlayDrawFrame, WaterOverlayGpuInstance};

pub const WATER_OVERLAY_WGSL: &str = "shaders/water/water_overlay.wgsl";

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
pub struct WaterOverlayDrawGlobals {
    pub view_proj: Mat4,
    pub instance_count: u32,
    pub time_secs: f32,
    pub zoom_alpha: f32,
}

#[derive(Resource)]
struct WaterSurfaceDrawPipeline {
    globals_layout: BindGroupLayoutDescriptor,
    instances_layout: BindGroupLayoutDescriptor,
    pipelines: [[CachedRenderPipelineId; 4]; 2],
}

#[derive(Resource, Default)]
struct WaterSurfaceDrawBindGpu {
    uniform: UniformBuffer<WaterOverlayDrawGlobals>,
    instance_buffer: Option<Buffer>,
    instance_bytes: usize,
    bind_group_0: Option<BindGroup>,
    bind_group_1: Option<BindGroup>,
}

#[derive(Resource, Default)]
struct WaterSurfaceDrawPassReady {
    pipeline_ready: bool,
}

pub fn register_water_surface_draw(app: &mut App) {
    app.init_resource::<WaterOverlayDrawGlobals>()
        .add_plugins(ExtractResourcePlugin::<WaterOverlayDrawGlobals>::default())
        .add_systems(
            Update,
            sync_water_overlay_draw_globals
                .after(crate::render::water_surface_visual::sync_water_overlay_draw_frame),
        );

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app
        .init_resource::<WaterSurfaceDrawBindGpu>()
        .init_resource::<WaterSurfaceDrawPassReady>()
        .add_systems(RenderStartup, init_water_surface_draw_pipeline)
        .add_systems(
            Render,
            prepare_water_surface_draw_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        )
        .add_systems(
            Core2d,
            (
                ensure_water_surface_draw_pipeline_ready,
                water_surface_draw_pass.after(ensure_water_surface_draw_pipeline_ready),
            )
                .chain()
                .in_set(Core2dOverlaySet::WaterSurface),
        );
}

pub fn sync_water_overlay_draw_globals(
    frame: Res<WaterOverlayDrawFrame>,
    mut globals: ResMut<WaterOverlayDrawGlobals>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
) {
    globals.instance_count = 0;
    globals.time_secs = frame.anim_time_secs;
    globals.zoom_alpha = frame.zoom_alpha;
    // W1 (D-W09): overlay motion always on — do not gate on particle instanced_draw.
    if frame.instances.is_empty() {
        return;
    }
    let Ok((camera, gt)) = cam_q.single() else {
        return;
    };
    let view_from_world = Mat4::from(gt.affine().inverse());
    globals.view_proj = camera.clip_from_view() * view_from_world;
    globals.instance_count = frame.instances.len() as u32;
}

fn init_water_surface_draw_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let globals_layout = BindGroupLayoutDescriptor::new(
        "water_overlay_globals_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (uniform_buffer::<WaterOverlayDrawGlobals>(false),),
        ),
    );
    let instances_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("water_overlay_instances_layout"),
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

    let shader = asset_server.load(WATER_OVERLAY_WGSL);
    let pipelines = std::array::from_fn(|hdr| {
        let fmt = if hdr == 0 {
            CORE2D_OVERLAY_SDR_FORMAT
        } else {
            TextureFormat::Rgba16Float
        };
        std::array::from_fn(|si| {
            let samples = MSAA_SAMPLES[si];
            let desc = RenderPipelineDescriptor {
                label: Some(Cow::Borrowed("water_surface_overlay")),
                layout: vec![globals_layout.clone(), instances_layout.clone()],
                immediate_size: 0,
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
                        blend: Some(bevy::render::render_resource::BlendState::ALPHA_BLENDING),
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
                    depth_write_enabled: Some(false),
                    depth_compare: Some(CompareFunction::Always),
                    stencil: StencilState {
                        front: StencilFaceState::IGNORE,
                        back: StencilFaceState::IGNORE,
                        read_mask: 0,
                        write_mask: 0,
                    },
                    bias: DepthBiasState::default(),
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

    commands.insert_resource(WaterSurfaceDrawPipeline {
        globals_layout,
        instances_layout,
        pipelines,
    });
}

fn prepare_water_surface_draw_bind_groups(
    pipeline: Res<WaterSurfaceDrawPipeline>,
    globals: Res<WaterOverlayDrawGlobals>,
    frame: Res<WaterOverlayDrawFrame>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    mut bind_gpu: ResMut<WaterSurfaceDrawBindGpu>,
) {
    bind_gpu.uniform.set(*globals);
    bind_gpu.uniform.write_buffer(&render_device, &render_queue);
    bind_gpu.bind_group_0 = None;
    bind_gpu.bind_group_1 = None;

    if globals.instance_count == 0 || frame.instances.is_empty() {
        return;
    }

    let bytes = frame.instances.len() * std::mem::size_of::<WaterOverlayGpuInstance>();
    let buf = match bind_gpu.instance_buffer.as_ref() {
        Some(b) if bind_gpu.instance_bytes >= bytes => b.clone(),
        _ => {
            let alloc = render_device.create_buffer(&BufferDescriptor {
                label: Some("water_overlay_instances"),
                size: (bytes.max(std::mem::size_of::<WaterOverlayGpuInstance>() * 64)) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            bind_gpu.instance_bytes = bytes.max(bind_gpu.instance_bytes);
            bind_gpu.instance_buffer = Some(alloc.clone());
            alloc
        }
    };
    render_queue.write_buffer(&buf, 0, bytemuck::cast_slice(&frame.instances));

    let globals_layout = pipeline_cache.get_bind_group_layout(&pipeline.globals_layout);
    let Some(globals_binding) = bind_gpu.uniform.binding() else {
        return;
    };
    let globals_bind = render_device.create_bind_group(
        "water_overlay_globals_bind_group",
        &globals_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: globals_binding,
        }],
    );
    let instances_bind = render_device.create_bind_group(
        "water_overlay_instances_bind_group",
        &pipeline_cache.get_bind_group_layout(&pipeline.instances_layout),
        &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(BufferBinding {
                buffer: &buf,
                offset: 0,
                size: None,
            }),
        }],
    );
    bind_gpu.bind_group_0 = Some(globals_bind);
    bind_gpu.bind_group_1 = Some(instances_bind);
}

fn ensure_water_surface_draw_pipeline_ready(
    pipeline: Option<Res<WaterSurfaceDrawPipeline>>,
    cache: Res<PipelineCache>,
    mut ready: ResMut<WaterSurfaceDrawPassReady>,
) {
    if ready.pipeline_ready {
        return;
    }
    let Some(pipeline) = pipeline else {
        return;
    };
    let mut all_ok = true;
    for row in &pipeline.pipelines {
        for id in row {
            if !matches!(cache.get_render_pipeline_state(*id), CachedPipelineState::Ok(_)) {
                all_ok = false;
            }
        }
    }
    if all_ok {
        ready.pipeline_ready = true;
    }
}

fn water_surface_draw_pass(
    world: &World,
    view: ViewQuery<(
        &ExtractedCamera,
        &ExtractedView,
        &ViewTarget,
        &ViewDepthTexture,
        &Msaa,
        Has<TileDebugRenderHost>,
    )>,
    mut ctx: RenderContext,
    ready: Res<WaterSurfaceDrawPassReady>,
) {
    let (_camera, extracted_view, target, depth, msaa, host) = view.into_inner();
    if !host || !ready.pipeline_ready {
        return;
    }
    let pipeline = world.resource::<WaterSurfaceDrawPipeline>();
    let bind_gpu = world.resource::<WaterSurfaceDrawBindGpu>();
    let globals = world.resource::<WaterOverlayDrawGlobals>();
    if globals.instance_count == 0 {
        return;
    }
    let (Some(bg0), Some(bg1)) = (bind_gpu.bind_group_0.as_ref(), bind_gpu.bind_group_1.as_ref()) else {
        return;
    };

    let pipeline_cache = world.resource::<PipelineCache>();
    let hdr = core2d_overlay_pipeline_hdr_index(extracted_view.target_format);
    let pid = pipeline.pipelines[hdr][msaa_index(msaa.samples())];
    let Some(p) = pipeline_cache.get_render_pipeline(pid) else {
        return;
    };

    let mut color = target.get_color_attachment();
    color.ops.load = LoadOp::Load;
    let depth_stencil = Some(depth.get_attachment(StoreOp::Store));

    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("water_surface_overlay"),
        color_attachments: &[Some(color)],
        depth_stencil_attachment: depth_stencil,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    pass.set_render_pipeline(p);
    pass.set_bind_group(0, bg0, &[]);
    pass.set_bind_group(1, bg1, &[]);
    let verts = globals.instance_count.saturating_mul(6);
    pass.draw(0..verts, 0..1);
}
