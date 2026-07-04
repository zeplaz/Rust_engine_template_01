//! GPU instanced terrain atlas draw (P0-C′) — one quad per world tile from chunk matrices.
//!
//! **P0-C′-PRIME interim (plan Q3):** Simulation default uses dirty-gated **sprite texture bake**
//! via [`super::terrain_render_authority::TerrainRenderAuthority::uses_gpu_sprite_display`].
//! The Core2d instanced pass remains wired for a future flip when per-tile instancing replaces
//! the sprite bake; [`sync_terrain_instances_from_chunks`] clears instances while sprite display
//! is active.

use std::borrow::Cow;

use bevy::asset::AssetServer;
use bevy::core_pipeline::{Core2d, core_2d::CORE_2D_DEPTH_FORMAT};
use bevy::prelude::*;
use bevy::render::{
    camera::ExtractedCamera,
    render_resource::{
        binding_types::{sampler, texture_2d, uniform_buffer},
        BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
        BindGroupLayoutEntry, BindingResource, BindingType, BufferBinding, BufferBindingType,
        CachedPipelineState, CachedRenderPipelineId, ColorTargetState, ColorWrites,
        CompareFunction, DepthBiasState, DepthStencilState, FragmentState, FrontFace, LoadOp,
        MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
        RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages, StencilFaceState,
        StencilState, StoreOp, TextureFormat, TextureSampleType, UniformBuffer, VertexState,
    },
    renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
    view::{ExtractedView, Msaa, ViewDepthTexture, ViewTarget},
    Render, RenderApp, RenderStartup, RenderSystems,
};
use bevy::render::{
    render_asset::RenderAssets,
    texture::GpuImage,
};
use bevy::render::extract_component::ExtractComponent;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bytemuck::{Pod, Zeroable};

use crate::gui::MainWorldCamera;
use crate::render::core2d_overlay_order::{
    core2d_overlay_pipeline_hdr_index, Core2dOverlaySet, CORE2D_OVERLAY_SDR_FORMAT,
};
use crate::render::gpu_buffer_registry::{
    BufferVisibility, GPUBufferRegistry, RegisteredBufferDescriptor, TERRAIN_INSTANCES_BUFFER,
};
use crate::render::terrain_material_atlas::TerrainMaterialAtlasGpu;
use crate::render::terrain_render_authority::TerrainRenderAuthority;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::{family_default_material_def, MaterialId, MaterializedChunk, MaterialRegistry};

pub const TERRAIN_INSTANCED_WGSL: &str = "shaders/terrain/terrain_instanced.wgsl";

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

/// Core2d host marker — terrain instanced pass runs on views with this component.
#[derive(Component, Clone, Copy, Default, Reflect, ExtractComponent)]
#[reflect(Component)]
pub struct TerrainInstancedRenderHost;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct TerrainTileInstance {
    pub world_pos: [f32; 2],
    pub material_index: u32,
    pub _pad: u32,
}

#[derive(Resource, Clone, Copy, Default, ExtractResource, bevy::render::render_resource::ShaderType)]
pub struct TerrainInstancedDrawGlobals {
    pub view_proj: Mat4,
    pub instance_count: u32,
    pub atlas_cols: u32,
    pub atlas_rows: u32,
    pub cell_uv: Vec2,
    pub _pad: f32,
}

#[derive(Resource, Default, Debug, Clone, ExtractResource)]
pub struct TerrainInstanceMap {
    pub instances: Vec<TerrainTileInstance>,
    pub revision: u64,
}

#[derive(Resource)]
struct TerrainInstancedPipeline {
    globals_layout: BindGroupLayoutDescriptor,
    instances_layout: BindGroupLayoutDescriptor,
    pipelines: [[CachedRenderPipelineId; 4]; 2],
}

#[derive(Resource, Default)]
struct TerrainInstancedBindGpu {
    uniform: UniformBuffer<TerrainInstancedDrawGlobals>,
    bind_group_0: Option<BindGroup>,
    bind_group_1: Option<BindGroup>,
    storage_version: u64,
    atlas_revision: u64,
}

#[derive(Resource, Default)]
struct TerrainInstancedPassReady {
    pipeline_ready: bool,
}

pub struct TerrainInstancedDrawPlugin;

impl Plugin for TerrainInstancedDrawPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainInstanceMap>()
            .init_resource::<TerrainInstancedDrawGlobals>()
            .add_plugins((
                ExtractResourcePlugin::<TerrainInstanceMap>::default(),
                ExtractResourcePlugin::<TerrainInstancedDrawGlobals>::default(),
            ))
            .add_systems(
                Update,
                (sync_terrain_instances_from_chunks, sync_terrain_instanced_draw_globals).chain(),
            );
        register_terrain_instanced_draw(app);
    }
}

pub fn register_terrain_instanced_draw(app: &mut App) {
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<TerrainInstancedBindGpu>()
        .init_resource::<TerrainInstancedPassReady>()
        .add_systems(RenderStartup, init_terrain_instanced_pipeline)
        .add_systems(
            Render,
            (
                prepare_terrain_instance_storage,
                prepare_terrain_instanced_bind_groups.after(prepare_terrain_instance_storage),
            )
                .in_set(RenderSystems::PrepareBindGroups),
        )
        .add_systems(
            Core2d,
            (
                ensure_terrain_instanced_pipeline_ready,
                terrain_instanced_pass.after(ensure_terrain_instanced_pipeline_ready),
            )
                .chain()
                .in_set(Core2dOverlaySet::TerrainInstanced),
        );
}

fn sync_terrain_instances_from_chunks(
    authority: Res<TerrainRenderAuthority>,
    params: Res<WorldGenParams>,
    handles: Option<Res<TerrainRegistriesHandles>>,
    materials: Res<Assets<MaterialRegistry>>,
    gpu_stamps: Res<crate::gui::map_tile_atlas_stamp::TerrainGpuStampIndices>,
    chunks: Query<(&Chunk, &ChunkCellMatrix, Option<&MaterializedChunk>)>,
    mut map: ResMut<TerrainInstanceMap>,
) {
    // P0-C′ display uses one dirty-gated sprite texture — not per-tile instancing.
    if !authority.is_gpu() || authority.uses_gpu_sprite_display() {
        if !map.instances.is_empty() {
            map.instances.clear();
            map.revision = map.revision.wrapping_add(1);
        }
        return;
    }

    let Some(handles) = handles else {
        return;
    };
    let reg = materials.get(&handles.material_registry);
    let mut out = Vec::new();
    let tex_w = params.width;
    let tex_h = params.height;

    for (chunk, matrix, mat_chunk) in &chunks {
        let sx = matrix.size.x as usize;
        let sy = matrix.size.y as usize;
        if sx == 0 || sy == 0 {
            continue;
        }
        for y in 0..sy {
            for x in 0..sx {
                let wx = chunk.coord.x as isize * sx as isize + x as isize;
                let wy = chunk.coord.y as isize * sy as isize + y as isize;
                if wx < 0 || wy < 0 {
                    continue;
                }
                let xu = wx as u32;
                let yu = wy as u32;
                if xu >= tex_w || yu >= tex_h {
                    continue;
                }
                let i = matrix.idx(x as u32, y as u32);
                let material_index = if let Some(mc) = mat_chunk {
                    mc.materials.get(i).copied().unwrap_or(MaterialId(0)).0 as u32
                } else if let Some(reg) = reg {
                    family_default_material_def(reg, matrix.family[i])
                        .map(|d| {
                            reg.materials
                                .iter()
                                .position(|m| m.name == d.name)
                                .unwrap_or(0) as u32
                        })
                        .unwrap_or(0)
                } else {
                    0
                };
                out.push(TerrainTileInstance {
                    world_pos: [xu as f32 + 0.5, yu as f32 + 0.5],
                    material_index,
                    _pad: 0,
                });
            }
        }
    }

    map.instances = out;
    crate::gui::map_tile_atlas_stamp::apply_gpu_stamps_to_terrain_instances(
        gpu_stamps.as_ref(),
        &mut map.instances,
    );
    map.revision = map.revision.wrapping_add(1);
}

fn sync_terrain_instanced_draw_globals(
    authority: Res<crate::render::terrain_render_authority::TerrainRenderAuthority>,
    atlas: Res<TerrainMaterialAtlasGpu>,
    map: Res<TerrainInstanceMap>,
    mut globals: ResMut<TerrainInstancedDrawGlobals>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
) {
    *globals = TerrainInstancedDrawGlobals::default();
    if !authority.is_gpu() || map.instances.is_empty() {
        return;
    }
    let Ok((camera, gt)) = cam_q.single() else {
        return;
    };
    let view_from_world = Mat4::from(gt.affine().inverse());
    globals.view_proj = camera.clip_from_view() * view_from_world;
    globals.instance_count = map.instances.len() as u32;
    globals.atlas_cols = atlas.cols.max(1);
    globals.atlas_rows = atlas.rows.max(1);
    globals.cell_uv = atlas.cell_uv.into();
}

fn prepare_terrain_instance_storage(
    mut local_frame: Local<u64>,
    map: Res<TerrainInstanceMap>,
    mut registry: ResMut<GPUBufferRegistry>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    *local_frame = local_frame.wrapping_add(1);
    let stride = std::mem::size_of::<TerrainTileInstance>() as u32;
    let reserve_rows = map.instances.len().max(1);
    let _ = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: TERRAIN_INSTANCES_BUFFER,
            size_bytes: 0,
            usage: bevy::render::render_resource::BufferUsages::COPY_DST
                | bevy::render::render_resource::BufferUsages::STORAGE,
            visibility: BufferVisibility::RenderAndCompute,
            stride,
        },
        reserve_rows,
        &map.instances,
        *local_frame,
    );
}

fn init_terrain_instanced_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let globals_layout = BindGroupLayoutDescriptor::new(
        "terrain_instanced_globals_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                uniform_buffer::<TerrainInstancedDrawGlobals>(false),
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(bevy::render::render_resource::SamplerBindingType::Filtering),
            ),
        ),
    );
    let instances_layout = BindGroupLayoutDescriptor {
        label: Cow::Borrowed("terrain_instanced_instances_layout"),
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

    let shader = asset_server.load(TERRAIN_INSTANCED_WGSL);
    let pipelines = std::array::from_fn(|hdr| {
        let fmt = if hdr == 0 {
            CORE2D_OVERLAY_SDR_FORMAT
        } else {
            TextureFormat::Rgba16Float
        };
        std::array::from_fn(|si| {
            let samples = MSAA_SAMPLES[si];
            let desc = RenderPipelineDescriptor {
                label: Some(Cow::Borrowed("terrain_instanced")),
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
                        blend: None,
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
                    depth_compare: Some(CompareFunction::LessEqual),
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

    commands.insert_resource(TerrainInstancedPipeline {
        globals_layout,
        instances_layout,
        pipelines,
    });
}

fn prepare_terrain_instanced_bind_groups(
    globals: Res<TerrainInstancedDrawGlobals>,
    pipeline: Res<TerrainInstancedPipeline>,
    registry: Res<GPUBufferRegistry>,
    atlas: Res<TerrainMaterialAtlasGpu>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    mut bind_gpu: ResMut<TerrainInstancedBindGpu>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
) {
    bind_gpu.uniform.set(*globals);
    bind_gpu.uniform.write_buffer(&render_device, &queue);

    let globals_layout = pipeline_cache.get_bind_group_layout(&pipeline.globals_layout);

    let atlas_gpu = gpu_images.get(atlas.image.id());
    let atlas_revision = atlas.revision;
    let need_atlas_rebind =
        atlas_revision_changed(bind_gpu.atlas_revision, atlas_revision) || bind_gpu.bind_group_0.is_none();

    if need_atlas_rebind {
        bind_gpu.atlas_revision = atlas_revision;
        if let (Some(atlas_gpu), Some(globals_binding)) =
            (atlas_gpu, bind_gpu.uniform.binding())
        {
            let view = &atlas_gpu.texture_view;
            let sampler = &atlas_gpu.sampler;
            bind_gpu.bind_group_0 = Some(render_device.create_bind_group(
                "terrain_instanced_globals_bind_group",
                &globals_layout,
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: globals_binding,
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(sampler),
                    },
                ],
            ));
        }
    }

    let Some(storage) = registry.get(TERRAIN_INSTANCES_BUFFER) else {
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
        "terrain_instanced_instances_bind_group",
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

#[inline]
fn atlas_revision_changed(prev: u64, next: u64) -> bool {
    prev != next
}

fn ensure_terrain_instanced_pipeline_ready(
    pipeline: Option<Res<TerrainInstancedPipeline>>,
    cache: Res<PipelineCache>,
    mut ready: ResMut<TerrainInstancedPassReady>,
) {
    if ready.pipeline_ready {
        return;
    }
    let Some(pl) = pipeline else {
        return;
    };
    let mut all_ok = true;
    let mut saw_shader_loading = false;
    for row in &pl.pipelines {
        for id in row {
            match cache.get_render_pipeline_state(*id) {
                CachedPipelineState::Ok(_) => {}
                CachedPipelineState::Queued | CachedPipelineState::Creating(_) => {
                    all_ok = false;
                }
                CachedPipelineState::Err(e) => {
                    let detail = format!("{e:?}");
                    if detail.contains("ShaderNotLoaded")
                        || detail.contains("ShaderImportNotYetAvailable")
                    {
                        saw_shader_loading = true;
                        all_ok = false;
                    } else {
                        panic!("terrain_instanced pipeline ({TERRAIN_INSTANCED_WGSL}): {e}");
                    }
                }
            }
        }
    }
    if all_ok {
        ready.pipeline_ready = true;
    } else if !saw_shader_loading {
        // Queued / creating — wait.
    }
}

fn terrain_instanced_pass(
    world: &World,
    view: ViewQuery<(
        &ExtractedCamera,
        &ExtractedView,
        &ViewTarget,
        &ViewDepthTexture,
        &Msaa,
        Has<TerrainInstancedRenderHost>,
    )>,
    mut ctx: RenderContext,
    ready: Res<TerrainInstancedPassReady>,
) {
    let (_camera, extracted_view, view_target, depth, msaa, host) = view.into_inner();
    if !host || !ready.pipeline_ready {
        return;
    }
    let globals = world.resource::<TerrainInstancedDrawGlobals>();
    if globals.instance_count == 0 {
        return;
    }
    let bind = world.resource::<TerrainInstancedBindGpu>();
    let Some(bg0) = bind.bind_group_0.as_ref() else {
        return;
    };
    let Some(bg1) = bind.bind_group_1.as_ref() else {
        return;
    };

    let pipeline_res = world.resource::<TerrainInstancedPipeline>();
    let cache = world.resource::<PipelineCache>();
    let hdr = core2d_overlay_pipeline_hdr_index(extracted_view.target_format);
    let si = msaa_index(msaa.samples());
    let pipeline_id = pipeline_res.pipelines[hdr][si];
    let Some(pl) = cache.get_render_pipeline(pipeline_id) else {
        return;
    };

    let mut color = view_target.get_color_attachment();
    color.ops.load = LoadOp::Load;
    let depth_stencil = Some(depth.get_attachment(StoreOp::Store));

    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("terrain_instanced"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_tile_instance_pod_layout() {
        assert_eq!(std::mem::size_of::<TerrainTileInstance>(), 16);
    }
}
