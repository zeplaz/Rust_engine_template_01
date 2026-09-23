//! Weather precip Core2d vertex-pull draw (EFFECTS-SYSTEM ES-5-002).
//!
//! Instance storage → `WEATHER_PRECIP_INSTANCES_BUFFER` → anisotropic streak raster.
//! Does **not** share FIRE_* buffers or fire expand WGSL. Reject Hanabi world rain.

use std::borrow::Cow;

use bevy::asset::AssetServer;
use bevy::core_pipeline::{core_2d::CORE_2D_DEPTH_FORMAT, Core2d};
use bevy::prelude::*;
use bevy::render::{
    camera::ExtractedCamera,
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_resource::{
        binding_types::uniform_buffer, BindGroup, BindGroupEntry, BindGroupLayoutDescriptor,
        BindGroupLayoutEntries, BindGroupLayoutEntry, BindingResource, BindingType,
        BlendComponent, BlendFactor, BlendOperation, BlendState, BufferBinding, BufferBindingType,
        CachedPipelineState, CachedRenderPipelineId, ColorTargetState, ColorWrites,
        CompareFunction, DepthBiasState, DepthStencilState, FragmentState, FrontFace, LoadOp,
        MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
        RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages, ShaderType,
        StencilFaceState, StencilState, StoreOp, TextureFormat, UniformBuffer, VertexState,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
    view::{ExtractedView, Msaa, ViewDepthTexture, ViewTarget},
    Render, RenderApp, RenderStartup, RenderSystems,
};

use crate::gui::{MainWorldCamera, TileDebugRenderHost};
use crate::render::core::gpu_buffer_registry::{
    BufferVisibility, GPUBufferRegistry, RegisteredBufferDescriptor, WEATHER_PRECIP_INSTANCES_BUFFER,
};
use crate::render::extraction::extracted_camera_metrics::{
    ExtractedCameraMetrics, ExtractedCameraMetricsSet,
};
use crate::render::particle_domains::ParticleDomainRegistry;
use crate::render::pipelines::core2d_overlay_order::{
    core2d_overlay_pipeline_hdr_index, Core2dOverlaySet, CORE2D_OVERLAY_SDR_FORMAT,
};
use crate::render::pipelines::gpu_instanced_quad::GpuInstancedQuadInstance;

use super::frame::WeatherPrecipFrame;
use super::witness::cpu_weather_mesh_precip_enabled;

pub const WEATHER_PRECIP_DRAW_WGSL: &str = "shaders/weather/weather_precip_draw.wgsl";

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

/// Main-world draw intent / evidence for witnesses (single Update writer).
#[derive(Resource, Clone, Copy, Debug, Default, ExtractResource)]
pub struct WeatherPrecipDrawStatus {
    pub upload_active: bool,
    pub draw_instances: u32,
    pub camera_centered: bool,
}

/// Extracted globals for the precip raster pass.
#[derive(Resource, Clone, Copy, Debug, Default, ExtractResource, ShaderType)]
pub struct WeatherPrecipDrawGlobals {
    pub view_proj: Mat4,
    pub instance_count: u32,
    pub time_secs: f32,
    pub zoom_alpha: f32,
    /// Half of visible world height — rain fall wrap band.
    pub view_half_y: f32,
    /// px-per-tile — streak world size scales as `1/camera_zoom` for screen-stable rain.
    pub camera_zoom: f32,
}

#[derive(Resource)]
struct WeatherPrecipPipeline {
    globals_layout: BindGroupLayoutDescriptor,
    instances_layout: BindGroupLayoutDescriptor,
    pipelines: [[CachedRenderPipelineId; 4]; 2],
}

#[derive(Resource, Default)]
struct WeatherPrecipBindGpu {
    uniform: UniformBuffer<WeatherPrecipDrawGlobals>,
    bind_group_0: Option<BindGroup>,
    bind_group_1: Option<BindGroup>,
    storage_version: u64,
}

/// Register extract + Update sync + RenderApp upload/draw (no-op draw without RenderApp).
pub fn register_weather_precip_draw(app: &mut App) {
    app.init_resource::<WeatherPrecipDrawStatus>()
        .init_resource::<WeatherPrecipDrawGlobals>()
        .add_systems(
            Update,
            sync_weather_precip_draw_globals
                .after(ExtractedCameraMetricsSet::Sync)
                .after(super::extract::fill_weather_precip_frame_from_climate),
        )
        .add_systems(
            PostUpdate,
            sync_weather_precip_draw_globals
                .after(ExtractedCameraMetricsSet::Sync)
                .after(crate::gui::sync_main_world_camera_viewport_and_projection)
                .run_if(crate::gui::in_simulation_or_editor_map),
        );

    if app.get_sub_app(RenderApp).is_none() {
        return;
    }

    app.add_plugins((
        ExtractResourcePlugin::<WeatherPrecipFrame>::default(),
        ExtractResourcePlugin::<WeatherPrecipDrawGlobals>::default(),
        ExtractResourcePlugin::<WeatherPrecipDrawStatus>::default(),
    ));

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<WeatherPrecipBindGpu>()
        .add_systems(RenderStartup, init_weather_precip_pipeline)
        .add_systems(
            Render,
            (
                prepare_weather_precip_instance_storage,
                prepare_weather_precip_bind_groups.after(prepare_weather_precip_instance_storage),
            )
                .in_set(RenderSystems::PrepareBindGroups),
        )
        .add_systems(
            Core2d,
            weather_precip_pass.in_set(Core2dOverlaySet::WeatherPrecipRaster),
        );
}

fn sync_weather_precip_draw_globals(
    time: Res<Time>,
    cam: Res<ExtractedCameraMetrics>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
    domains: Res<ParticleDomainRegistry>,
    mut frame: ResMut<WeatherPrecipFrame>,
    mut globals: ResMut<WeatherPrecipDrawGlobals>,
    mut status: ResMut<WeatherPrecipDrawStatus>,
) {
    let cpu_on = cpu_weather_mesh_precip_enabled();
    let domain_active = domains.weather_precip_active();
    let count = frame.instance_count as u32;
    // Mesh path retired — upload whenever domain has prepared streaks.
    let upload = !cpu_on && domain_active && count > 0;

    status.upload_active = upload;
    status.draw_instances = if upload { count } else { 0 };
    status.camera_centered = frame.camera_centered;
    frame.gpu_authority = upload;

    let view_proj = if let Ok((camera, gt)) = cam_q.single() {
        let view_from_world = Mat4::from(gt.affine().inverse());
        camera.clip_from_view() * view_from_world
    } else {
        cam.view_proj
    };

    *globals = WeatherPrecipDrawGlobals {
        view_proj,
        instance_count: status.draw_instances,
        time_secs: time.elapsed_secs(),
        zoom_alpha: cam.zoom_alpha,
        view_half_y: {
            let z = cam.zoom_level.max(0.08);
            cam.view_pixels.y.max(1.0) * 0.5 / z
        },
        camera_zoom: cam.zoom_level.max(0.08),
    };
}

fn prepare_weather_precip_instance_storage(
    mut local_frame: Local<u64>,
    frame: Res<WeatherPrecipFrame>,
    status: Res<WeatherPrecipDrawStatus>,
    mut registry: ResMut<GPUBufferRegistry>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    *local_frame = local_frame.wrapping_add(1);
    if !status.upload_active || frame.streaks.is_empty() {
        let _ = registry.upload_pod_slice(
            &render_device,
            &queue,
            RegisteredBufferDescriptor {
                id: WEATHER_PRECIP_INSTANCES_BUFFER,
                size_bytes: 0,
                usage: bevy::render::render_resource::BufferUsages::COPY_DST
                    | bevy::render::render_resource::BufferUsages::STORAGE,
                visibility: BufferVisibility::RenderAndCompute,
                stride: std::mem::size_of::<GpuInstancedQuadInstance>() as u32,
            },
            1,
            &[] as &[GpuInstancedQuadInstance],
            *local_frame,
        );
        return;
    }
    let stride = std::mem::size_of::<GpuInstancedQuadInstance>() as u32;
    let reserve_rows = frame.streaks.len().max(1);
    let _ = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: WEATHER_PRECIP_INSTANCES_BUFFER,
            size_bytes: 0,
            usage: bevy::render::render_resource::BufferUsages::COPY_DST
                | bevy::render::render_resource::BufferUsages::STORAGE,
            visibility: BufferVisibility::RenderAndCompute,
            stride,
        },
        reserve_rows,
        &frame.streaks,
        *local_frame,
    );
}

fn init_weather_precip_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let globals_layout = BindGroupLayoutDescriptor::new(
        "weather_precip_globals_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (uniform_buffer::<WeatherPrecipDrawGlobals>(false),),
        ),
    );
    let instances_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("weather_precip_instances_layout"),
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

    let shader = asset_server.load(WEATHER_PRECIP_DRAW_WGSL);
    let pipelines = std::array::from_fn(|hdr| {
        let fmt = if hdr == 0 {
            CORE2D_OVERLAY_SDR_FORMAT
        } else {
            TextureFormat::Rgba16Float
        };
        std::array::from_fn(|si| {
            let samples = MSAA_SAMPLES[si];
            let desc = RenderPipelineDescriptor {
                label: Some(Cow::Borrowed("weather_precip_draw")),
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
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::SrcAlpha,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
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

    commands.insert_resource(WeatherPrecipPipeline {
        globals_layout,
        instances_layout,
        pipelines,
    });
}

fn prepare_weather_precip_bind_groups(
    globals: Res<WeatherPrecipDrawGlobals>,
    pipeline: Res<WeatherPrecipPipeline>,
    registry: Res<GPUBufferRegistry>,
    mut bind_gpu: ResMut<WeatherPrecipBindGpu>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
) {
    if globals.instance_count == 0 {
        bind_gpu.bind_group_0 = None;
        bind_gpu.bind_group_1 = None;
        bind_gpu.storage_version = 0;
        return;
    }

    bind_gpu.uniform.set(*globals);
    bind_gpu.uniform.write_buffer(&render_device, &queue);

    let globals_layout = pipeline_cache.get_bind_group_layout(&pipeline.globals_layout);
    if let Some(binding) = bind_gpu.uniform.binding() {
        bind_gpu.bind_group_0 = Some(render_device.create_bind_group(
            "weather_precip_globals_bind_group",
            &globals_layout,
            &[BindGroupEntry {
                binding: 0,
                resource: binding,
            }],
        ));
    }

    let Some(storage) = registry.get(WEATHER_PRECIP_INSTANCES_BUFFER) else {
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
        "weather_precip_instances_bind_group",
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

fn weather_precip_pass(
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
    let globals = world.resource::<WeatherPrecipDrawGlobals>();
    if globals.instance_count == 0 {
        return;
    }
    let bind = world.resource::<WeatherPrecipBindGpu>();
    let Some(bg0) = bind.bind_group_0.as_ref() else {
        return;
    };
    let Some(bg1) = bind.bind_group_1.as_ref() else {
        return;
    };

    let pipeline_res = world.resource::<WeatherPrecipPipeline>();
    let cache = world.resource::<PipelineCache>();
    let hdr = core2d_overlay_pipeline_hdr_index(extracted_view.target_format);
    let si = msaa_index(msaa.samples());
    let pipeline_id = pipeline_res.pipelines[hdr][si];
    if let CachedPipelineState::Err(e) = cache.get_render_pipeline_state(pipeline_id) {
        let detail = format!("{e:?}");
        if !detail.contains("ShaderNotLoaded") && !detail.contains("ShaderImportNotYetAvailable") {
            panic!("weather_precip pipeline ({WEATHER_PRECIP_DRAW_WGSL}): {e}");
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
        label: Some("weather_precip"),
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
