//! Instanced tile LOD / fire debug: one `draw(0..6, 0..N)` from storage instances
//! (`TILE_DEBUG_INSTANCES_BUFFER`) + globals uniform. Core2d overlay pass after main transparent.

use std::borrow::Cow;

use bevy::asset::AssetServer;
use bevy::core_pipeline::{Core2d, core_2d::CORE_2D_DEPTH_FORMAT};
use bevy::prelude::*;
use bevy::render::{
    camera::ExtractedCamera,
    render_resource::{
        binding_types::uniform_buffer,
        BindGroup, BlendState, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
        BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding, BufferBindingType,
        CachedPipelineState, CachedRenderPipelineId, ColorTargetState, ColorWrites,
        CompareFunction, DepthBiasState, DepthStencilState, FragmentState, FrontFace, LoadOp,
        MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
        RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages, StencilFaceState,
        StencilState, StoreOp, TextureFormat, UniformBuffer, VertexState,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
    view::{ExtractedView, Msaa, ViewDepthTexture, ViewTarget},
    Render, RenderApp, RenderStartup, RenderSystems,
};

use crate::gui::{TileDebugDrawGlobals, TileDebugRenderHost};
use crate::render::core2d_overlay_order::{
    core2d_overlay_pipeline_hdr_index, Core2dOverlaySet, CORE2D_OVERLAY_SDR_FORMAT,
};
use crate::render::gpu_buffer_registry::{GPUBufferRegistry, TILE_DEBUG_INSTANCES_BUFFER};
use crate::render::gpu_tile_debug_buffer::prepare_tile_debug_instance_storage;

pub const TILE_DEBUG_INSTANCED_WGSL: &str = "shaders/debug/tile_debug_instanced.wgsl";

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

#[derive(Resource)]
struct TileDebugInstancedPipeline {
    globals_layout: BindGroupLayoutDescriptor,
    instances_layout: BindGroupLayoutDescriptor,
    /// `[LDR=0 / HDR=1][msaa 1,2,4,8]`
    pipelines: [[CachedRenderPipelineId; 4]; 2],
}

#[derive(Resource, Default)]
struct TileDebugInstancedBindGpu {
    uniform: UniformBuffer<TileDebugDrawGlobals>,
    bind_group_0: Option<BindGroup>,
    bind_group_1: Option<BindGroup>,
    storage_version: u64,
}

pub fn register_tile_debug_instanced_draw(app: &mut App) {
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };

    render_app
        .init_resource::<TileDebugInstancedBindGpu>()
        .add_systems(RenderStartup, init_tile_debug_instanced_pipeline)
        .add_systems(
            Render,
            prepare_tile_debug_instanced_bind_groups
                .after(prepare_tile_debug_instance_storage)
                .in_set(RenderSystems::PrepareBindGroups),
        )
        .add_systems(
            Core2d,
            tile_debug_instanced_pass.in_set(Core2dOverlaySet::TileDebug),
        );
}

fn init_tile_debug_instanced_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let globals_layout = BindGroupLayoutDescriptor::new(
        "tile_debug_globals_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX,
            (uniform_buffer::<TileDebugDrawGlobals>(false),),
        ),
    );
    let instances_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("tile_debug_instances_layout"),
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

    let shader = asset_server.load(TILE_DEBUG_INSTANCED_WGSL);

    let pipelines = std::array::from_fn(|hdr| {
        let fmt = if hdr == 0 {
            CORE2D_OVERLAY_SDR_FORMAT
        } else {
            TextureFormat::Rgba16Float
        };
        std::array::from_fn(|si| {
            let samples = MSAA_SAMPLES[si];
            let desc = RenderPipelineDescriptor {
                label: Some(Cow::Borrowed("tile_debug_instanced")),
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
                        blend: Some(BlendState::ALPHA_BLENDING),
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

    commands.insert_resource(TileDebugInstancedPipeline {
        globals_layout,
        instances_layout,
        pipelines,
    });
}

fn prepare_tile_debug_instanced_bind_groups(
    globals: Res<TileDebugDrawGlobals>,
    pipeline: Res<TileDebugInstancedPipeline>,
    registry: Res<GPUBufferRegistry>,
    mut bind_gpu: ResMut<TileDebugInstancedBindGpu>,
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
        "tile_debug_globals_bind_group",
        &globals_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: globals_binding,
        }],
    ));

    let Some(storage) = registry.get(TILE_DEBUG_INSTANCES_BUFFER) else {
        bind_gpu.bind_group_1 = None;
        bind_gpu.storage_version = 0;
        return;
    };

    if bind_gpu.storage_version == storage.version && bind_gpu.bind_group_1.is_some() {
        return;
    }

    bind_gpu.storage_version = storage.version;
    let instances_layout = pipeline_cache.get_bind_group_layout(&pipeline.instances_layout);
    bind_gpu.bind_group_1 = Some(render_device.create_bind_group(
        "tile_debug_instances_bind_group",
        &instances_layout,
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

fn tile_debug_instanced_pass(
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
) {
    let (_camera, extracted_view, view_target, depth, msaa, host) = view.into_inner();
    if !host {
        return;
    }
    let globals = world.resource::<TileDebugDrawGlobals>();
    if globals.instance_count == 0 {
        return;
    }
    let bind = world.resource::<TileDebugInstancedBindGpu>();
    let Some(bg0) = bind.bind_group_0.as_ref() else {
        return;
    };
    let Some(bg1) = bind.bind_group_1.as_ref() else {
        return;
    };

    let pipeline_res = world.resource::<TileDebugInstancedPipeline>();
    let cache = world.resource::<PipelineCache>();
    let hdr = core2d_overlay_pipeline_hdr_index(extracted_view.target_format);
    let si = msaa_index(msaa.samples());
    let pipeline_id = pipeline_res.pipelines[hdr][si];
    if let CachedPipelineState::Err(e) = cache.get_render_pipeline_state(pipeline_id) {
        let detail = format!("{e:?}");
        if !detail.contains("ShaderNotLoaded") && !detail.contains("ShaderImportNotYetAvailable") {
            panic!("tile_debug_instanced pipeline ({TILE_DEBUG_INSTANCED_WGSL}): {e}");
        }
        return;
    }
    let Some(pl) = cache.get_render_pipeline(pipeline_id) else {
        return;
    };

    let mut color = view_target.get_color_attachment();
    color.ops.load = LoadOp::Load;
    let depth_stencil = Some(depth.get_attachment(StoreOp::Store));

    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("tile_debug_instanced"),
        color_attachments: &[Some(color)],
        depth_stencil_attachment: depth_stencil,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    pass.set_render_pipeline(pl);
    pass.set_bind_group(0, bg0, &[]);
    pass.set_bind_group(1, bg1, &[]);
    pass.draw(0..6, 0..globals.instance_count);
}
