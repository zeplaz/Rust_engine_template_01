//! DF-3 — GPU field pass: samples sim-packed depth/flow texture over the ROI.

use std::borrow::Cow;

use bevy::asset::{AssetId, AssetServer};
use bevy::render::render_asset::RenderAssets;
use bevy::core_pipeline::{Core2d, core_2d::CORE_2D_DEPTH_FORMAT};
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::ShaderType;
use bevy::render::texture::GpuImage;
use bevy::render::{
    camera::ExtractedCamera,
    render_resource::{
        binding_types::{sampler, texture_2d, uniform_buffer},
        BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
        BindingResource, CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction,
        DepthBiasState, DepthStencilState, FragmentState, FrontFace, LoadOp, MultisampleState,
        PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPassDescriptor,
        RenderPipelineDescriptor, SamplerBindingType, ShaderStages, StencilFaceState, StencilState,
        StoreOp, TextureFormat, TextureSampleType, UniformBuffer, VertexState,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
    view::{ExtractedView, Msaa, ViewDepthStencilTexture, ViewTarget},
    Render, RenderApp, RenderStartup, RenderSystems,
};

use crate::gui::{MainWorldCamera, TileDebugRenderHost};
use crate::render::pipelines::core2d_overlay_order::{
    core2d_overlay_pipeline_hdr_index, Core2dOverlaySet, CORE2D_OVERLAY_SDR_FORMAT,
};
use crate::systems::water::{SurfaceWaterGpuField, SurfaceWaterGpuFieldMeta};

pub const WATER_FIELD_WGSL: &str = "shaders/water/water_field.wgsl";

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

#[derive(Resource, Clone, Copy, Default, ShaderType, ExtractResource)]
#[extract_app(bevy::render::RenderApp)]
pub struct WaterFieldDrawGlobals {
    pub view_proj: Mat4,
    pub origin_extent: Vec4,
    pub time_secs: f32,
    pub zoom_alpha: f32,
    pub wet_texels: u32,
    pub _pad: u32,
}

#[derive(Resource)]
struct WaterFieldDrawPipeline {
    layout: BindGroupLayoutDescriptor,
    pipelines: [[CachedRenderPipelineId; 4]; 2],
}

#[derive(Resource, Default)]
struct WaterFieldDrawBindGpu {
    uniform: UniformBuffer<WaterFieldDrawGlobals>,
    bind_group: Option<BindGroup>,
    image_id: Option<AssetId<Image>>,
    revision: u64,
}

pub fn register_water_field_draw(app: &mut App) {
    app.init_resource::<WaterFieldDrawGlobals>()
        .init_resource::<SurfaceWaterGpuField>()
        .init_resource::<SurfaceWaterGpuFieldMeta>()
        .add_plugins(ExtractResourcePlugin::<SurfaceWaterGpuFieldMeta>::default())
        .add_plugins(ExtractResourcePlugin::<WaterFieldDrawGlobals>::default())
        .add_systems(Update, sync_water_field_draw_globals);

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<WaterFieldDrawBindGpu>()
        .init_resource::<WaterFieldDrawGlobals>()
        .add_systems(RenderStartup, init_water_field_draw_pipeline)
        .add_systems(
            Render,
            prepare_water_field_draw_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        )
        .add_systems(
            Core2d,
            water_field_draw_pass.in_set(Core2dOverlaySet::WaterSurface),
        );
}

fn sync_water_field_draw_globals(
    meta: Res<SurfaceWaterGpuFieldMeta>,
    mut globals: ResMut<WaterFieldDrawGlobals>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
    cam_scale: Option<Res<crate::render::pipelines::gpu_particles::FireParticleCameraScale>>,
    world_mode: Option<Res<crate::gui::WorldPresentationMode>>,
    time: Res<Time>,
) {
    globals.time_secs = time.elapsed_secs();
    globals.zoom_alpha = cam_scale
        .as_deref()
        .map(|c| c.zoom_alpha)
        .unwrap_or(0.5);
    globals.wet_texels = meta.wet_texels;
    globals.origin_extent = Vec4::ZERO;
    // PixelWorld draws water via `pixel_world::water_gpu` (same field Image).
    // Keep this Core2d pass for tactical flat only — avoids the floating Z=0 plane.
    if world_mode.is_some_and(|m| m.is_pixel_world()) {
        return;
    }
    if !meta.wired || meta.width == 0 || meta.height == 0 {
        return;
    }
    let Ok((camera, gt)) = cam_q.single() else {
        return;
    };
    let view_from_world = Mat4::from(gt.affine().inverse());
    globals.view_proj = camera.clip_from_view() * view_from_world;
    let step = meta.step.max(1) as f32;
    let world_w = meta.width as f32 * step;
    let world_h = meta.height as f32 * step;
    globals.origin_extent = Vec4::new(
        meta.origin_x as f32,
        meta.origin_z as f32,
        world_w,
        world_h,
    );
}

fn init_water_field_draw_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "water_field_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                uniform_buffer::<WaterFieldDrawGlobals>(false),
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    );
    let shader = asset_server.load(WATER_FIELD_WGSL);
    let pipelines = std::array::from_fn(|hdr| {
        let fmt = if hdr == 0 {
            CORE2D_OVERLAY_SDR_FORMAT
        } else {
            TextureFormat::Rgba16Float
        };
        std::array::from_fn(|si| {
            let samples = MSAA_SAMPLES[si];
            let desc = RenderPipelineDescriptor {
                label: Some(Cow::Borrowed("water_field_surface")),
                layout: vec![layout.clone()],
                immediate_size: 0,
                vertex: VertexState {
                    shader: shader.clone(),
                    entry_point: Some(Cow::Borrowed("vs_main")),
                    shader_defs: vec![],
                    constants: vec![],
                    buffers: vec![],
                },
                fragment: Some(FragmentState {
                    shader: shader.clone(),
                    entry_point: Some(Cow::Borrowed("fs_main")),
                    shader_defs: vec![],
                    constants: vec![],
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
    commands.insert_resource(WaterFieldDrawPipeline { layout, pipelines });
}

fn prepare_water_field_draw_bind_groups(
    pipeline: Option<Res<WaterFieldDrawPipeline>>,
    meta: Option<Res<SurfaceWaterGpuFieldMeta>>,
    globals: Option<Res<WaterFieldDrawGlobals>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    mut bind_gpu: ResMut<WaterFieldDrawBindGpu>,
) {
    let Some(globals) = globals else {
        return;
    };
    bind_gpu.uniform.set(*globals);
    bind_gpu.uniform.write_buffer(&render_device, &render_queue);
    bind_gpu.bind_group = None;

    let Some(pipeline) = pipeline else {
        return;
    };
    let Some(meta) = meta else {
        return;
    };
    if !meta.wired || meta.width == 0 || meta.wet_texels == 0 {
        return;
    }
    let Some(gpu) = gpu_images.get(&meta.image) else {
        return;
    };
    let Some(globals_binding) = bind_gpu.uniform.binding() else {
        return;
    };
    let layout = pipeline_cache.get_bind_group_layout(&pipeline.layout);
    bind_gpu.bind_group = Some(render_device.create_bind_group(
        "water_field_bind_group",
        &layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: globals_binding,
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&gpu.texture_view),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Sampler(&gpu.sampler),
            },
        ],
    ));
    bind_gpu.image_id = Some(meta.image.id());
    bind_gpu.revision = meta.revision;
}

fn water_field_draw_pass(
    world: &World,
    view: ViewQuery<(
        &ExtractedCamera,
        &ExtractedView,
        &ViewTarget,
        &ViewDepthStencilTexture,
        &Msaa,
        Has<TileDebugRenderHost>,
    )>,
    mut ctx: RenderContext,
) {
    let (_camera, extracted_view, target, depth, msaa, host) = view.into_inner();
    if !host {
        return;
    }
    let Some(pipeline) = world.get_resource::<WaterFieldDrawPipeline>() else {
        return;
    };
    let Some(bind_gpu) = world.get_resource::<WaterFieldDrawBindGpu>() else {
        return;
    };
    let Some(globals) = world.get_resource::<WaterFieldDrawGlobals>() else {
        return;
    };
    if globals.origin_extent.z <= 0.0 || globals.origin_extent.w <= 0.0 {
        return;
    }
    let Some(bg) = bind_gpu.bind_group.as_ref() else {
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
        label: Some("water_field_surface"),
        color_attachments: &[Some(color)],
        depth_stencil_attachment: depth_stencil,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_render_pipeline(p);
    pass.set_bind_group(0, bg, &[]);
    pass.draw(0..6, 0..1);
}

#[cfg(test)]
mod tests {
    #[test]
    fn water_field_wgsl_exists() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src = std::fs::read_to_string(root.join("assets/shaders/water/water_field.wgsl"))
            .expect("water_field.wgsl");
        assert!(src.contains("fn vs_main"));
        assert!(src.contains("fn fs_main"));
        assert!(src.contains("field_tex"));
        assert!(src.contains("shoreline") || src.contains("grad"));
    }
}
