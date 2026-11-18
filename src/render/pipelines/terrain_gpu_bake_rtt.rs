//! GPU-A / GS-01 — RenderApp compute bake: control tex + material atlas → world bake Image.
//! GPU-MIPS — after mip0 bake, box-filter downsample fills the mip chain.
//!
//! Main-world authoring uploads the compact control texture and arms
//! [`TerrainGpuBakeRttDispatch`]. This module never host-samples atlas pixels.

use std::borrow::Cow;

use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    binding_types::{texture_2d, texture_2d_array, texture_storage_2d, uniform_buffer},
    *,
};
use bevy::render::renderer::{RenderContext, RenderDevice, RenderGraph, RenderGraphSystems, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::ShaderCacheError;

pub const TERRAIN_BAKE_RTT_WGSL: &str = "shaders/terrain/terrain_bake_rtt.wgsl";
pub const TERRAIN_BAKE_MIPS_WGSL: &str = "shaders/terrain/terrain_bake_mips.wgsl";
const WORKGROUP: u32 = 8;
const STORAGE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
const ATLAS_TILE_PX: u32 = 8;
/// Cap mip depth so huge worlds stay bounded (still covers 2^11 = 2048+).
const MAX_BAKE_MIPS: u32 = 12;

#[derive(Clone, Copy, Debug, ShaderType, Default, PartialEq)]
pub struct TerrainBakeRttParamsGpu {
    pub atlas_cols: u32,
    pub atlas_tile_px: u32,
    pub bake_w: u32,
    pub bake_h: u32,
    pub dirty_x0: u32,
    pub dirty_y0: u32,
    pub dirty_x1: u32,
    pub dirty_y1: u32,
}

#[derive(Clone, Copy, Debug, ShaderType, Default, PartialEq)]
struct TerrainBakeMipParamsGpu {
    dst_w: u32,
    dst_h: u32,
    dirty_x0: u32,
    dirty_y0: u32,
    dirty_x1: u32,
    dirty_y1: u32,
}

/// Map a mip0 dirty rect into the next mip’s inclusive-exclusive bounds.
#[must_use]
pub fn scale_bake_dirty_rect_to_next_mip(
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    src_w: u32,
    src_h: u32,
) -> (u32, u32, u32, u32) {
    let dst_w = (src_w / 2).max(1);
    let dst_h = (src_h / 2).max(1);
    let dx0 = (x0 / 2).min(dst_w.saturating_sub(1));
    let dy0 = (y0 / 2).min(dst_h.saturating_sub(1));
    let dx1 = ((x1 + 1) / 2).clamp(dx0.saturating_add(1), dst_w);
    let dy1 = ((y1 + 1) / 2).clamp(dy0.saturating_add(1), dst_h);
    (dx0, dy0, dx1, dy1)
}

/// Main→render extract: armed bake job.
#[derive(Resource, Debug, Default, ExtractResource, Clone)]
#[extract_app(RenderApp)]
pub struct TerrainGpuBakeRttDispatch {
    pub commit_stamp: u64,
    pub control: Handle<Image>,
    pub bake: Handle<Image>,
    pub atlas: Handle<Image>,
    pub params: TerrainBakeRttParamsGpu,
}

impl TerrainGpuBakeRttDispatch {
    #[must_use]
    pub fn has_commit(&self) -> bool {
        self.commit_stamp > 0
            && self.control != Handle::default()
            && self.bake != Handle::default()
            && self.atlas != Handle::default()
            && self.params.bake_w > 0
            && self.params.bake_h > 0
    }

    pub fn arm(
        &mut self,
        control: Handle<Image>,
        bake: Handle<Image>,
        atlas: Handle<Image>,
        params: TerrainBakeRttParamsGpu,
    ) {
        self.control = control;
        self.bake = bake;
        self.atlas = atlas;
        self.params = params;
        self.commit_stamp = self.commit_stamp.wrapping_add(1).max(1);
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct TerrainBakeRttPassSet;

#[derive(Resource)]
struct TerrainBakeRttPipeline {
    layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}

#[derive(Resource)]
struct TerrainBakeMipPipeline {
    layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}

#[derive(Resource, Default)]
struct TerrainBakeRttBindGpu {
    bind_group: Option<BindGroup>,
    /// Single-mip storage view for bake mip0 (required when mip_level_count > 1).
    bake_mip0_view: Option<TextureView>,
    control_id: AssetId<Image>,
    bake_id: AssetId<Image>,
    atlas_id: AssetId<Image>,
    params: TerrainBakeRttParamsGpu,
    mip_level_count: u32,
}

#[derive(Resource, Default)]
struct TerrainBakeRttPassReady {
    bake_pipeline_ready: bool,
    mip_pipeline_ready: bool,
    last_executed_commit_stamp: u64,
    pub execute_count: u64,
    pub mip_pass_count: u64,
}

/// Atomic: render world saw a successful bake dispatch (main world may poll via probe later).
pub static TERRAIN_BAKE_RTT_EXECUTE_COUNT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
pub static TERRAIN_BAKE_RTT_SHADER_FAILED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
pub static TERRAIN_BAKE_MIP_PASS_COUNT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Full mip chain length for a bake extent (GPU-MIPS).
#[must_use]
pub fn bake_mip_level_count(w: u32, h: u32) -> u32 {
    let m = w.max(h).max(1);
    let levels = m.ilog2() + 1;
    levels.min(MAX_BAKE_MIPS).max(1)
}

pub fn register_terrain_gpu_bake_rtt(app: &mut App) {
    app.init_resource::<TerrainGpuBakeRttDispatch>().add_plugins(
        ExtractResourcePlugin::<TerrainGpuBakeRttDispatch>::default(),
    );

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<TerrainBakeRttBindGpu>()
        .init_resource::<TerrainBakeRttPassReady>()
        .add_systems(
            RenderStartup,
            (init_terrain_bake_rtt_pipeline, init_terrain_bake_mip_pipeline),
        )
        .add_systems(
            Render,
            prepare_terrain_bake_rtt_bind_groups.in_set(RenderSystems::PrepareBindGroups),
        )
        .add_systems(
            RenderGraph,
            terrain_bake_rtt_pass
                .in_set(RenderGraphSystems::Render)
                .in_set(TerrainBakeRttPassSet),
        );
}

fn init_terrain_bake_rtt_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "TerrainBakeRtt",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::ReadOnly),
                // GPU-ARRAY.1 — material layers (no 2D atlas UV bleed).
                texture_2d_array(TextureSampleType::Float { filterable: true }),
                binding_types::sampler(SamplerBindingType::Filtering),
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::WriteOnly),
                uniform_buffer::<TerrainBakeRttParamsGpu>(false),
            ),
        ),
    );

    let shader = asset_server.load(TERRAIN_BAKE_RTT_WGSL);
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![layout.clone()],
        shader,
        entry_point: Some(Cow::from("bake")),
        ..default()
    });

    commands.insert_resource(TerrainBakeRttPipeline { layout, pipeline });
}

fn init_terrain_bake_mip_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "TerrainBakeMips",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                texture_2d(TextureSampleType::Float { filterable: false }),
                texture_storage_2d(STORAGE_FORMAT, StorageTextureAccess::WriteOnly),
                uniform_buffer::<TerrainBakeMipParamsGpu>(false),
            ),
        ),
    );

    let shader = asset_server.load(TERRAIN_BAKE_MIPS_WGSL);
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        layout: vec![layout.clone()],
        shader,
        entry_point: Some(Cow::from("downsample")),
        ..default()
    });

    commands.insert_resource(TerrainBakeMipPipeline { layout, pipeline });
}

fn single_mip_view(texture: &Texture, base_mip_level: u32, label: &'static str) -> TextureView {
    texture.create_view(&TextureViewDescriptor {
        label: Some(label),
        format: Some(STORAGE_FORMAT),
        dimension: Some(TextureViewDimension::D2),
        aspect: TextureAspect::All,
        base_mip_level,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(1),
        usage: None,
    })
}

fn prepare_terrain_bake_rtt_bind_groups(
    pipeline: Res<TerrainBakeRttPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    dispatch: Res<TerrainGpuBakeRttDispatch>,
    mut bind_gpu: ResMut<TerrainBakeRttBindGpu>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
) {
    if !dispatch.has_commit() {
        return;
    }

    let control_id = dispatch.control.id();
    let bake_id = dispatch.bake.id();
    let atlas_id = dispatch.atlas.id();
    let same = bind_gpu.bind_group.is_some()
        && bind_gpu.bake_mip0_view.is_some()
        && bind_gpu.control_id == control_id
        && bind_gpu.bake_id == bake_id
        && bind_gpu.atlas_id == atlas_id
        && bind_gpu.params == dispatch.params;
    if same {
        return;
    }

    let Some(control) = gpu_images.get(&dispatch.control) else {
        return;
    };
    let Some(bake) = gpu_images.get(&dispatch.bake) else {
        return;
    };
    let Some(atlas) = gpu_images.get(&dispatch.atlas) else {
        return;
    };

    let mip_level_count = bake.texture_descriptor.mip_level_count.max(1);
    let bake_mip0 = single_mip_view(&bake.texture, 0, "terrain_bake_mip0_storage");

    let mut ub = UniformBuffer::from(dispatch.params);
    ub.write_buffer(&render_device, &queue);

    let layout = pipeline_cache.get_bind_group_layout(&pipeline.layout);
    let bind_group = render_device.create_bind_group(
        None,
        &layout,
        &BindGroupEntries::sequential((
            &control.texture_view,
            &atlas.texture_view,
            &atlas.sampler,
            &bake_mip0,
            &ub,
        )),
    );

    bind_gpu.bind_group = Some(bind_group);
    bind_gpu.bake_mip0_view = Some(bake_mip0);
    bind_gpu.control_id = control_id;
    bind_gpu.bake_id = bake_id;
    bind_gpu.atlas_id = atlas_id;
    bind_gpu.params = dispatch.params;
    bind_gpu.mip_level_count = mip_level_count;
}

fn terrain_bake_rtt_pass(
    mut ctx: RenderContext,
    bake_pipeline: Option<Res<TerrainBakeRttPipeline>>,
    mip_pipeline: Option<Res<TerrainBakeMipPipeline>>,
    cache: Res<PipelineCache>,
    mut ready: ResMut<TerrainBakeRttPassReady>,
    dispatch: Res<TerrainGpuBakeRttDispatch>,
    bind_gpu: Res<TerrainBakeRttBindGpu>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    if !ready.bake_pipeline_ready {
        if let Some(pipeline) = bake_pipeline.as_ref() {
            match cache.get_compute_pipeline_state(pipeline.pipeline) {
                CachedPipelineState::Ok(_) => ready.bake_pipeline_ready = true,
                CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_)) => {}
                CachedPipelineState::Err(e) => {
                    TERRAIN_BAKE_RTT_SHADER_FAILED.store(true, std::sync::atomic::Ordering::Relaxed);
                    bevy::log::error!(
                        target: "terrain_gpu_bake_rtt",
                        "terrain bake RTT shader failed: {e}"
                    );
                }
                _ => {}
            }
        }
        if !ready.bake_pipeline_ready {
            return;
        }
    }

    if !ready.mip_pipeline_ready {
        if let Some(pipeline) = mip_pipeline.as_ref() {
            match cache.get_compute_pipeline_state(pipeline.pipeline) {
                CachedPipelineState::Ok(_) => ready.mip_pipeline_ready = true,
                CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_)) => {}
                CachedPipelineState::Err(e) => {
                    bevy::log::error!(
                        target: "terrain_gpu_bake_rtt",
                        "terrain bake mip shader failed: {e}"
                    );
                }
                _ => {}
            }
        }
    }

    if !dispatch.has_commit() {
        return;
    }
    if dispatch.commit_stamp == ready.last_executed_commit_stamp {
        return;
    }
    let Some(bind_group) = bind_gpu.bind_group.as_ref() else {
        return;
    };
    let Some(bake_pipeline) = bake_pipeline.as_ref() else {
        return;
    };
    let Some(pl) = cache.get_compute_pipeline(bake_pipeline.pipeline) else {
        return;
    };

    let w = dispatch
        .params
        .dirty_x1
        .saturating_sub(dispatch.params.dirty_x0)
        .max(1);
    let h = dispatch
        .params
        .dirty_y1
        .saturating_sub(dispatch.params.dirty_y0)
        .max(1);
    let dx = w.div_ceil(WORKGROUP);
    let dy = h.div_ceil(WORKGROUP);

    {
        let mut pass = ctx
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(pl);
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(dx, dy, 1);
    }

    // GPU-MIPS / RND-08: never leave a single-mip world bake when zoom-out is possible.
    // Incremental: only downsample the dirty subrect at each level (not the full map).
    if ready.mip_pipeline_ready && bind_gpu.mip_level_count > 1 {
        if let (Some(mip_pipeline), Some(bake)) = (mip_pipeline.as_ref(), gpu_images.get(&dispatch.bake))
        {
            if let Some(mip_pl) = cache.get_compute_pipeline(mip_pipeline.pipeline) {
                let layout = cache.get_bind_group_layout(&mip_pipeline.layout);
                let mut src_w = dispatch.params.bake_w.max(1);
                let mut src_h = dispatch.params.bake_h.max(1);
                let mut dirty_x0 = dispatch.params.dirty_x0.min(src_w);
                let mut dirty_y0 = dispatch.params.dirty_y0.min(src_h);
                let mut dirty_x1 = dispatch.params.dirty_x1.clamp(dirty_x0.saturating_add(1), src_w);
                let mut dirty_y1 = dispatch.params.dirty_y1.clamp(dirty_y0.saturating_add(1), src_h);
                let mut mip_dispatches = 0u64;
                for level in 1..bind_gpu.mip_level_count {
                    let dst_w = (src_w / 2).max(1);
                    let dst_h = (src_h / 2).max(1);
                    let (dx0, dy0, dx1, dy1) = scale_bake_dirty_rect_to_next_mip(
                        dirty_x0, dirty_y0, dirty_x1, dirty_y1, src_w, src_h,
                    );
                    let src_view = single_mip_view(
                        &bake.texture,
                        level - 1,
                        "terrain_bake_mip_src",
                    );
                    let dst_view =
                        single_mip_view(&bake.texture, level, "terrain_bake_mip_dst");
                    let params = TerrainBakeMipParamsGpu {
                        dst_w,
                        dst_h,
                        dirty_x0: dx0,
                        dirty_y0: dy0,
                        dirty_x1: dx1,
                        dirty_y1: dy1,
                    };
                    let mut ub = UniformBuffer::from(params);
                    ub.write_buffer(&render_device, &queue);
                    let mip_bg = render_device.create_bind_group(
                        None,
                        &layout,
                        &BindGroupEntries::sequential((&src_view, &dst_view, &ub)),
                    );
                    let dw = dx1.saturating_sub(dx0).max(1);
                    let dh = dy1.saturating_sub(dy0).max(1);
                    {
                        let mut pass = ctx
                            .command_encoder()
                            .begin_compute_pass(&ComputePassDescriptor::default());
                        pass.set_pipeline(mip_pl);
                        pass.set_bind_group(0, &mip_bg, &[]);
                        pass.dispatch_workgroups(
                            dw.div_ceil(WORKGROUP),
                            dh.div_ceil(WORKGROUP),
                            1,
                        );
                    }
                    mip_dispatches = mip_dispatches.wrapping_add(1);
                    dirty_x0 = dx0;
                    dirty_y0 = dy0;
                    dirty_x1 = dx1;
                    dirty_y1 = dy1;
                    src_w = dst_w;
                    src_h = dst_h;
                }
                ready.mip_pass_count = ready.mip_pass_count.wrapping_add(mip_dispatches);
                TERRAIN_BAKE_MIP_PASS_COUNT.fetch_add(
                    mip_dispatches,
                    std::sync::atomic::Ordering::Relaxed,
                );
            }
        }
    }

    ready.last_executed_commit_stamp = dispatch.commit_stamp;
    ready.execute_count = ready.execute_count.wrapping_add(1);
    TERRAIN_BAKE_RTT_EXECUTE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

#[must_use]
pub fn atlas_tile_px() -> u32 {
    ATLAS_TILE_PX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_mips_level_count_covers_extent() {
        assert_eq!(bake_mip_level_count(1, 1), 1);
        assert_eq!(bake_mip_level_count(8, 8), 4);
        assert_eq!(bake_mip_level_count(256, 128), 9);
        assert!(bake_mip_level_count(8192, 8192) <= MAX_BAKE_MIPS);
        // Zoom-out maps must never ship as single-mip (RND-08 shimmer / cache thrash).
        assert!(bake_mip_level_count(64, 64) > 1);
        assert!(bake_mip_level_count(2, 2) > 1);
    }

    #[test]
    fn gpu_mips_dirty_rect_scales_to_next_level() {
        let (x0, y0, x1, y1) = scale_bake_dirty_rect_to_next_mip(10, 20, 40, 50, 128, 128);
        assert_eq!((x0, y0), (5, 10));
        assert!(x1 > x0 && y1 > y0);
        assert!(x1 <= 64 && y1 <= 64);
    }

    #[test]
    fn gpu_mips_shader_and_pass_are_wired() {
        let src = include_str!("terrain_gpu_bake_rtt.rs");
        assert!(src.contains("TERRAIN_BAKE_MIPS_WGSL"));
        assert!(src.contains("init_terrain_bake_mip_pipeline"));
        assert!(src.contains("single_mip_view"));
        assert!(src.contains("scale_bake_dirty_rect_to_next_mip"));
        let wgsl = include_str!("../../../assets/shaders/terrain/terrain_bake_mips.wgsl");
        assert!(wgsl.contains("fn downsample"));
        assert!(wgsl.contains("dirty_x0"));
    }
}
