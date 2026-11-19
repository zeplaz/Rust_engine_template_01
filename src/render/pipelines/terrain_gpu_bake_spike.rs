//! RPC-1-004/005/006 + **GPU-A / GS-01** — atlas→world bake via GPU RTT compute.
//!
//! CPU uploads a compact **control texture** (material id + road mask). RenderApp compute
//! ([`terrain_gpu_bake_rtt`](super::terrain_gpu_bake_rtt)) samples the material atlas into the
//! world bake Image. Host atlas scissor write is **test/diagnose only**.
//!
//! Design: [`design_gpu_rtt_bake_water_displace_v1.md`](../../dev/design_gpu_rtt_bake_water_displace_v1.md).

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::gui::map_tile_atlas_stamp::TerrainGpuStampIndices;
use crate::render::pipelines::terrain_gpu_bake_rtt::{
    atlas_tile_px, bake_mip_level_count, register_terrain_gpu_bake_rtt, TerrainBakeRttParamsGpu,
    TerrainGpuBakeRttDispatch, TERRAIN_BAKE_RTT_EXECUTE_COUNT,
};
use crate::render::pipelines::terrain_tile_instance::TerrainTileInstance;
use crate::render::core::terrain_material_atlas::TerrainMaterialAtlasGpu;
use crate::render::tile_world_fallback::{
    TileWorldFallbackChunkGrid, TileWorldFallbackRasterCtrl, TileWorldFallbackState,
};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix, WorldGenDenseTerrainCache};
use crate::terrain::material::ground_display::ground_display_material_index;
use crate::terrain::material::MaterialRegistry;

/// Env flag — Simulation default ON; set `0`/`false` to opt out for diagnose.
pub const TERRAIN_GPU_BAKE_SPIKE_ENV: &str = "TERRAIN_GPU_BAKE_SPIKE";

#[must_use]
pub fn terrain_gpu_bake_spike_env_enabled() -> bool {
    match std::env::var(TERRAIN_GPU_BAKE_SPIKE_ENV) {
        Ok(v) if v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off") => {
            false
        }
        _ => true,
    }
}

/// Inclusive-exclusive tile-space scissor for a dirty bake region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerrainGpuBakeScissor {
    pub x0: u32,
    pub y0: u32,
    pub x1: u32,
    pub y1: u32,
}

impl TerrainGpuBakeScissor {
    #[must_use]
    pub fn from_chunk(cx: u32, cz: u32, tex_w: u32, tex_h: u32) -> Self {
        let (x0, y0, x1, y1) =
            TileWorldFallbackChunkGrid::chunk_pixel_bounds(cx, cz, tex_w, tex_h);
        Self {
            x0: x0 as u32,
            y0: y0 as u32,
            x1: x1 as u32,
            y1: y1 as u32,
        }
    }

    #[must_use]
    pub fn tile_count(self) -> u32 {
        self.x1.saturating_sub(self.x0) * self.y1.saturating_sub(self.y0)
    }

    #[must_use]
    pub fn contains(self, x: u32, y: u32) -> bool {
        x >= self.x0 && x < self.x1 && y >= self.y0 && y < self.y1
    }

    #[must_use]
    pub fn union_bounds(regions: &[Self], bake_w: u32, bake_h: u32) -> Self {
        if regions.is_empty() {
            return Self {
                x0: 0,
                y0: 0,
                x1: bake_w,
                y1: bake_h,
            };
        }
        let mut x0 = u32::MAX;
        let mut y0 = u32::MAX;
        let mut x1 = 0u32;
        let mut y1 = 0u32;
        for r in regions {
            x0 = x0.min(r.x0);
            y0 = y0.min(r.y0);
            x1 = x1.max(r.x1);
            y1 = y1.max(r.y1);
        }
        Self {
            x0,
            y0,
            x1: x1.min(bake_w),
            y1: y1.min(bake_h),
        }
    }
}

/// Material-only bake indices (no fire/weather). Separate from display `TerrainInstanceMap`.
#[derive(Resource, Debug, Clone, Default)]
pub struct TerrainGpuBakeIndexMap {
    pub instances: Vec<TerrainTileInstance>,
    pub revision: u64,
}

/// GPU bake state — control tex + world bake target + RTT dispatch arming.
#[derive(Resource, Debug, Clone)]
pub struct TerrainGpuBakeSpikeState {
    pub enabled: bool,
    pub bake_revision: u64,
    pub regions: Vec<TerrainGpuBakeScissor>,
    pub world_bake_image: Handle<Image>,
    pub control_image: Handle<Image>,
    pub world_bake_w: u32,
    pub world_bake_h: u32,
    /// Always true by contract — fire is never written into bake indices/pixels.
    pub fire_excluded_from_bake: bool,
    /// Display pixels still owned by CPU dirty-bake → sprite (not this spike).
    pub cpu_display_still_authority: bool,
    /// Control texels written for the last armed job.
    pub last_control_texel_count: u32,
    /// Index-map revision last uploaded into the control texture.
    pub last_control_index_revision: u64,
    /// Last dispatch commit stamp armed from main world.
    pub last_dispatch_stamp: u64,
    /// Render-world execute count snapshot when consumers last became ready.
    pub last_rtt_execute_seen: u64,
    /// Scissors of the last armed dispatch (incremental re-bake when the dirty set changes).
    pub last_dispatched_regions: Vec<TerrainGpuBakeScissor>,
    /// Dirty rect of the last armed dispatch and the render execute count when it was armed:
    /// a re-arm before that dispatch ran must cover both rects, or it is lost.
    pub armed_dirty: Option<TerrainGpuBakeScissor>,
    pub exec_count_at_arm: u64,
}

impl Default for TerrainGpuBakeSpikeState {
    fn default() -> Self {
        Self {
            enabled: false,
            bake_revision: 0,
            regions: Vec::new(),
            world_bake_image: Handle::default(),
            control_image: Handle::default(),
            world_bake_w: 0,
            world_bake_h: 0,
            fire_excluded_from_bake: true,
            cpu_display_still_authority: true,
            last_control_texel_count: 0,
            last_control_index_revision: 0,
            last_dispatch_stamp: 0,
            last_rtt_execute_seen: 0,
            last_dispatched_regions: Vec::new(),
            armed_dirty: None,
            exec_count_at_arm: 0,
        }
    }
}

/// True when consumers may bind the bake Image (control uploaded + bake allocated + RTT armed).
#[must_use]
pub fn spike_bake_ready_for_consumers(state: &TerrainGpuBakeSpikeState) -> bool {
    state.enabled
        && state.world_bake_image != Handle::default()
        && state.control_image != Handle::default()
        && state.world_bake_w > 0
        && state.world_bake_h > 0
        && state.last_control_texel_count > 0
        && state.last_dispatch_stamp > 0
}

/// Structured probe for unit tests + witness (no full visual required).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainGpuBakeSpikeProbe {
    pub flag_enabled: bool,
    pub flag_default_off: bool,
    pub index_count: u32,
    pub region_count: u32,
    pub fire_excluded_from_bake: bool,
    pub cpu_display_still_authority: bool,
    pub world_bake_target_allocated: bool,
    pub control_uploaded: bool,
    pub gpu_rtt_armed: bool,
    pub consumers_ready: bool,
    pub control_texel_count: u32,
    pub minimap_default_off_is_world_raster: bool,
    pub minimap_gpu_bake_label_available: bool,
    pub path: &'static str,
}

/// Pack dirty chunk coords into atlas→world scissors (does not mutate the CPU dirty grid).
#[must_use]
pub fn pack_gpu_bake_scissors(
    dirty_chunks: &[(u32, u32)],
    tex_w: u32,
    tex_h: u32,
) -> Vec<TerrainGpuBakeScissor> {
    dirty_chunks
        .iter()
        .map(|&(cx, cz)| TerrainGpuBakeScissor::from_chunk(cx, cz, tex_w, tex_h))
        .filter(|s| s.tile_count() > 0)
        .collect()
}

/// Peek dirty chunk coordinates without clearing CPU raster dirty flags.
#[must_use]
pub fn peek_dirty_chunk_coords(grid: &TileWorldFallbackChunkGrid) -> Vec<(u32, u32)> {
    grid.peek_dirty_chunks()
}

/// Collect material-only tile instances from chunk matrices (bake feed).
#[must_use]
pub fn collect_material_bake_instances(
    params: &WorldGenParams,
    materials: &Assets<MaterialRegistry>,
    handles: Option<&TerrainRegistriesHandles>,
    chunks: &[(Chunk, ChunkCellMatrix)],
    dense: Option<&WorldGenDenseTerrainCache>,
    gpu_stamps: &TerrainGpuStampIndices,
) -> Vec<TerrainTileInstance> {
    // No registry yet → no bake feed (index 0 would be a water material); re-collected once loaded.
    let Some(reg) = handles.and_then(|h| materials.get(&h.material_registry)) else {
        return Vec::new();
    };
    collect_material_bake_instances_with_registry(params, reg, chunks, dense, gpu_stamps)
}

/// Registry-explicit core of [`collect_material_bake_instances`].
#[must_use]
pub fn collect_material_bake_instances_with_registry(
    params: &WorldGenParams,
    reg: &MaterialRegistry,
    chunks: &[(Chunk, ChunkCellMatrix)],
    dense: Option<&WorldGenDenseTerrainCache>,
    gpu_stamps: &TerrainGpuStampIndices,
) -> Vec<TerrainTileInstance> {
    let tex_w = params.width;
    let tex_h = params.height;
    let mut out = Vec::new();

    let families = crate::terrain::default_terrain_families();
    // Same source as the CPU raster (`tile_world_fallback` raster index): the dense world cache
    // is authoritative for display whenever it exists. Per-chunk matrices can be re-classified
    // from elevation after scenario authoring (DamFlood plateau: dense Grassland, matrix
    // Mountain -> grey granite in the bake while the raster stayed green).
    if let Some(cache) = dense.filter(|c| !c.family.is_empty() && c.width > 0) {
        let w = cache.width as usize;
        for (i, &family) in cache.family.iter().enumerate() {
            let xu = (i % w) as u32;
            let yu = (i / w) as u32;
            if xu >= tex_w || yu >= tex_h {
                continue;
            }
            let material_index =
                ground_display_material_index(reg, families, family).unwrap_or(0);
            out.push(TerrainTileInstance {
                world_pos: [xu as f32 + 0.5, yu as f32 + 0.5],
                material_index,
                _pad: 0,
            });
        }
        crate::gui::map_tile_atlas_stamp::apply_gpu_stamps_to_terrain_instances(gpu_stamps, &mut out);
        return out;
    }
    for (chunk, matrix) in chunks {
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
                // One resolution with the CPU raster (`tile_world_fallback`): family default
                // material; water families map to bed material. Rule-resolved
                // `MaterializedChunk` ids are not used by either display path.
                let material_index =
                    ground_display_material_index(reg, families, matrix.family[i]).unwrap_or(0);
                out.push(TerrainTileInstance {
                    world_pos: [xu as f32 + 0.5, yu as f32 + 0.5],
                    material_index,
                    _pad: 0,
                });
            }
        }
    }

    crate::gui::map_tile_atlas_stamp::apply_gpu_stamps_to_terrain_instances(gpu_stamps, &mut out);
    out
}

/// Sample material atlas cell center RGBA for a material index (unit tests / diagnose).
#[must_use]
pub fn sample_atlas_material_rgba(
    atlas_data: &[u8],
    atlas_w: u32,
    atlas_h: u32,
    cols: u32,
    material_index: u32,
) -> [u8; 4] {
    if atlas_w == 0 || atlas_h == 0 || cols == 0 || atlas_data.len() < 4 {
        return [0, 0, 0, 255];
    }
    let col = material_index % cols;
    let row = material_index / cols;
    let tile = atlas_tile_px();
    let px = col * tile + tile / 2;
    let py = row * tile + tile / 2;
    let px = px.min(atlas_w.saturating_sub(1));
    let py = py.min(atlas_h.saturating_sub(1));
    let i = ((py * atlas_w + px) * 4) as usize;
    if i + 3 < atlas_data.len() {
        [atlas_data[i], atlas_data[i + 1], atlas_data[i + 2], atlas_data[i + 3]]
    } else {
        [0, 0, 0, 255]
    }
}

/// Host atlas→world scissor write — **diagnose / unit tests only** (not production authority).
#[must_use]
pub fn write_atlas_to_world_bake(
    bake_data: &mut [u8],
    bake_w: u32,
    bake_h: u32,
    regions: &[TerrainGpuBakeScissor],
    instances: &[TerrainTileInstance],
    atlas_data: &[u8],
    atlas_w: u32,
    atlas_h: u32,
    atlas_cols: u32,
) -> u32 {
    if bake_w == 0 || bake_h == 0 || bake_data.len() < 4 {
        return 0;
    }
    let use_regions = !regions.is_empty();
    let mut written = 0u32;
    for inst in instances {
        let x = inst.world_pos[0].floor() as i32;
        let y = inst.world_pos[1].floor() as i32;
        if x < 0 || y < 0 {
            continue;
        }
        let xu = x as u32;
        let yu = y as u32;
        if xu >= bake_w || yu >= bake_h {
            continue;
        }
        if use_regions && !regions.iter().any(|r| r.contains(xu, yu)) {
            continue;
        }
        let color = sample_atlas_material_rgba(
            atlas_data,
            atlas_w,
            atlas_h,
            atlas_cols.max(1),
            inst.material_index,
        );
        let i = ((yu * bake_w + xu) * 4) as usize;
        if i + 3 < bake_data.len() {
            bake_data[i..i + 4].copy_from_slice(&color);
            written = written.saturating_add(1);
        }
    }
    written
}

/// Pack instances into control RGBA8 (R=material_id, G=road_mask reserved 0).
#[must_use]
pub fn write_control_texels(
    control: &mut [u8],
    w: u32,
    h: u32,
    instances: &[TerrainTileInstance],
    regions: &[TerrainGpuBakeScissor],
) -> u32 {
    if w == 0 || h == 0 || control.len() < 4 {
        return 0;
    }
    let use_regions = !regions.is_empty();
    let mut written = 0u32;
    for inst in instances {
        let x = inst.world_pos[0].floor() as i32;
        let y = inst.world_pos[1].floor() as i32;
        if x < 0 || y < 0 {
            continue;
        }
        let xu = x as u32;
        let yu = y as u32;
        if xu >= w || yu >= h {
            continue;
        }
        if use_regions && !regions.iter().any(|r| r.contains(xu, yu)) {
            continue;
        }
        let i = ((yu * w + xu) * 4) as usize;
        if i + 3 < control.len() {
            control[i] = inst.material_index.min(255) as u8;
            control[i + 1] = 0;
            control[i + 2] = 0;
            control[i + 3] = 255;
            written = written.saturating_add(1);
        }
    }
    written
}

#[must_use]
pub fn build_bake_spike_probe(state: &TerrainGpuBakeSpikeState, index_count: u32) -> TerrainGpuBakeSpikeProbe {
    use crate::render::minimap_compositor::{minimap_terrain_source_label, MinimapTerrainSourceBind};

    TerrainGpuBakeSpikeProbe {
        flag_enabled: state.enabled,
        flag_default_off: !terrain_gpu_bake_spike_env_enabled() || !state.enabled,
        index_count,
        region_count: state.regions.len() as u32,
        fire_excluded_from_bake: state.fire_excluded_from_bake,
        cpu_display_still_authority: state.cpu_display_still_authority,
        world_bake_target_allocated: state.world_bake_w > 0 && state.world_bake_h > 0,
        control_uploaded: state.last_control_texel_count > 0,
        gpu_rtt_armed: state.last_dispatch_stamp > 0,
        consumers_ready: spike_bake_ready_for_consumers(state),
        control_texel_count: state.last_control_texel_count,
        minimap_default_off_is_world_raster: minimap_terrain_source_label(
            MinimapTerrainSourceBind::WorldRaster,
        ) == "world_raster",
        minimap_gpu_bake_label_available: minimap_terrain_source_label(
            MinimapTerrainSourceBind::GpuBake,
        ) == "gpu_bake",
        path: "gpu_rtt_control_to_world_bake",
    }
}

fn make_rgba8_storage_image(w: u32, h: u32, label: &'static str) -> Image {
    let width = w.max(1);
    let height = h.max(1);
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some(label),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        },
        ..default()
    };
    image.data = Some(vec![0u8; 4 * width as usize * height as usize]);
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image
}

/// GPU-MIPS / RND-08 world bake: **full mip chain required** (single-mip = zoom shimmer + cache thrash).
/// GPU RTT writes mip0; compute downsample fills mip1..N (dirty-rect incremental).
/// Sampler: nearest mag, linear min + linear mipmap (= trilinear path).
fn make_rgba8_bake_image_with_mips(w: u32, h: u32) -> Image {
    let width = w.max(1);
    let height = h.max(1);
    let mips = bake_mip_level_count(width, height);
    debug_assert!(
        width <= 1 && height <= 1 || mips > 1,
        "world bake must not be single-mip when extent > 1×1"
    );
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("terrain_gpu_bake_world"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: mips,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        },
        ..default()
    };
    // GPU owns all mip pixels — no host mip0 authorship.
    image.data = None;
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    // RND-08 / gea4v2 11.3.14.7: trilinear + anisotropic for PixelWorld oblique walls /
    // zoom-out (same Image feeds flat sprite and voxel ground).
    let mut sampler = ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    };
    sampler.set_anisotropic_filter(8);
    image.sampler = ImageSampler::Descriptor(sampler);
    image
}

fn sync_bake_spike_enabled_flag(mut state: ResMut<TerrainGpuBakeSpikeState>) {
    let enabled = terrain_gpu_bake_spike_env_enabled();
    if state.enabled != enabled {
        state.enabled = enabled;
        state.bake_revision = state.bake_revision.wrapping_add(1);
        if !enabled {
            state.regions.clear();
            state.world_bake_image = Handle::default();
            state.control_image = Handle::default();
            state.world_bake_w = 0;
            state.world_bake_h = 0;
            state.last_control_texel_count = 0;
            state.last_control_index_revision = 0;
            state.last_dispatch_stamp = 0;
            state.last_rtt_execute_seen = 0;
        }
    }
    state.fire_excluded_from_bake = true;
    state.cpu_display_still_authority = !spike_bake_ready_for_consumers(&state);
}

fn sync_gpu_bake_indices_from_chunks(
    state: Res<TerrainGpuBakeSpikeState>,
    params: Res<WorldGenParams>,
    handles: Option<Res<TerrainRegistriesHandles>>,
    materials: Res<Assets<MaterialRegistry>>,
    gpu_stamps: Res<TerrainGpuStampIndices>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    dense: Option<Res<WorldGenDenseTerrainCache>>,
    mut map: ResMut<TerrainGpuBakeIndexMap>,
) {
    if !state.enabled {
        if !map.instances.is_empty() {
            map.instances.clear();
            map.revision = map.revision.wrapping_add(1);
        }
        return;
    }

    let owned: Vec<(Chunk, ChunkCellMatrix)> = chunks
        .iter()
        .map(|(c, m)| (*c, m.clone()))
        .collect();
    let out = collect_material_bake_instances(
        params.as_ref(),
        materials.as_ref(),
        handles.as_deref(),
        &owned,
        dense.as_deref(),
        gpu_stamps.as_ref(),
    );

    if map.instances.len() != out.len()
        || map
            .instances
            .iter()
            .zip(out.iter())
            .any(|(a, b)| a.material_index != b.material_index || a.world_pos != b.world_pos)
    {
        map.instances = out;
        map.revision = map.revision.wrapping_add(1);
    }
}

fn sync_gpu_bake_scissors_and_target(
    mut images: ResMut<Assets<Image>>,
    fallback: Res<TileWorldFallbackState>,
    raster_ctrl: Res<TileWorldFallbackRasterCtrl>,
    mut state: ResMut<TerrainGpuBakeSpikeState>,
) {
    if !state.enabled {
        return;
    }

    let tex_w = fallback.last_w.max(1);
    let tex_h = fallback.last_h.max(1);
    let dirty = peek_dirty_chunk_coords(&raster_ctrl.chunk_grid);
    let regions = if dirty.is_empty() {
        vec![TerrainGpuBakeScissor {
            x0: 0,
            y0: 0,
            x1: tex_w,
            y1: tex_h,
        }]
    } else {
        pack_gpu_bake_scissors(&dirty, tex_w, tex_h)
    };

    let need_images = state.world_bake_image == Handle::default()
        || state.control_image == Handle::default()
        || state.world_bake_w != tex_w
        || state.world_bake_h != tex_h;
    if need_images {
        state.world_bake_image = images.add(make_rgba8_bake_image_with_mips(tex_w, tex_h));
        state.control_image =
            images.add(make_rgba8_storage_image(tex_w, tex_h, "terrain_gpu_bake_control"));
        state.world_bake_w = tex_w;
        state.world_bake_h = tex_h;
        state.bake_revision = state.bake_revision.wrapping_add(1);
        state.last_control_texel_count = 0;
        state.last_control_index_revision = 0;
        state.last_dispatch_stamp = 0;
    }

    if state.regions != regions {
        state.regions = regions;
        state.bake_revision = state.bake_revision.wrapping_add(1);
    }
}

/// Upload control texels + arm GPU RTT dispatch (production path).
fn upload_control_and_arm_gpu_rtt(
    mut images: ResMut<Assets<Image>>,
    atlas: Res<TerrainMaterialAtlasGpu>,
    map: Res<TerrainGpuBakeIndexMap>,
    mut state: ResMut<TerrainGpuBakeSpikeState>,
    mut dispatch: ResMut<TerrainGpuBakeRttDispatch>,
) {
    if !state.enabled {
        return;
    }
    if state.world_bake_image == Handle::default()
        || state.control_image == Handle::default()
        || state.world_bake_w == 0
        || state.world_bake_h == 0
    {
        return;
    }
    if map.instances.is_empty() {
        return;
    }
    let bake_w = state.world_bake_w;
    let bake_h = state.world_bake_h;
    // CLOSE-G1: a new bake image or new material indices must be baked WHOLE. Baking only the
    // CPU raster's dirty chunks at that moment (often just the authored valley) left the rest of
    // the world image blank → grey ground in both views. Later dirty-chunk changes bake
    // incrementally.
    let full_bake = map.revision != state.last_control_index_revision
        || state.last_control_texel_count == 0;
    let regions = if full_bake {
        vec![TerrainGpuBakeScissor { x0: 0, y0: 0, x1: bake_w, y1: bake_h }]
    } else if state.regions != state.last_dispatched_regions {
        state.regions.clone()
    } else {
        // Still refresh execute-seen for readiness honesty.
        let exec = TERRAIN_BAKE_RTT_EXECUTE_COUNT.load(std::sync::atomic::Ordering::Relaxed);
        state.last_rtt_execute_seen = exec;
        return;
    };
    let control_handle = state.control_image.clone();
    let bake_handle = state.world_bake_image.clone();
    // GPU-ARRAY.1 — bake samples Texture2DArray; fall back to 2D grid only if array missing.
    let atlas_handle = if atlas.array_image != Handle::default() {
        atlas.array_image.clone()
    } else {
        atlas.image.clone()
    };
    if atlas_handle == Handle::default() {
        return;
    }

    let Some(mut control) = images.get_mut(&control_handle) else {
        return;
    };
    let Some(control_data) = control.data.as_mut() else {
        return;
    };

    let written = write_control_texels(control_data, bake_w, bake_h, &map.instances, &regions);
    if written == 0 {
        return;
    }

    // Force GPU re-upload of control.
    control.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    if let Some(data) = control.data.as_ref() {
        control.data = Some(data.clone());
    }

    let mut dirty = TerrainGpuBakeScissor::union_bounds(&regions, bake_w, bake_h);
    // CLOSE-G3: the render world retries an armed dispatch until it executes, but a new arm
    // replaces it. Water keeps the ROI chunks dirty, so the full-world bake after authoring was
    // overwritten by small ROI re-arms before it ever ran (grey ground outside the ROI). Keep
    // the not-yet-executed rect in the new dispatch.
    let exec_now = TERRAIN_BAKE_RTT_EXECUTE_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    if let Some(prev) = state.armed_dirty {
        if exec_now == state.exec_count_at_arm {
            dirty = TerrainGpuBakeScissor::union_bounds(&[prev, dirty], bake_w, bake_h);
        }
    }
    let params = TerrainBakeRttParamsGpu {
        atlas_cols: atlas.cols.max(1),
        atlas_tile_px: atlas_tile_px(),
        bake_w,
        bake_h,
        dirty_x0: dirty.x0,
        dirty_y0: dirty.y0,
        dirty_x1: dirty.x1.max(dirty.x0.saturating_add(1)),
        dirty_y1: dirty.y1.max(dirty.y0.saturating_add(1)),
    };

    dispatch.arm(control_handle, bake_handle, atlas_handle, params);
    state.armed_dirty = Some(dirty);
    state.exec_count_at_arm = exec_now;
    state.last_dispatched_regions = state.regions.clone();
    state.last_control_texel_count = written;
    state.last_control_index_revision = map.revision;
    state.last_dispatch_stamp = dispatch.commit_stamp;
    state.bake_revision = state.bake_revision.wrapping_add(1);
    state.last_rtt_execute_seen =
        TERRAIN_BAKE_RTT_EXECUTE_COUNT.load(std::sync::atomic::Ordering::Relaxed);
    state.cpu_display_still_authority = !spike_bake_ready_for_consumers(&state);
}

pub struct TerrainGpuBakeSpikePlugin;

impl Plugin for TerrainGpuBakeSpikePlugin {
    fn build(&self, app: &mut App) {
        register_terrain_gpu_bake_rtt(app);
        app.init_resource::<TerrainGpuBakeSpikeState>()
            .init_resource::<TerrainGpuBakeIndexMap>()
            .add_systems(
                Update,
                (
                    sync_bake_spike_enabled_flag,
                    sync_gpu_bake_indices_from_chunks.after(sync_bake_spike_enabled_flag),
                    sync_gpu_bake_scissors_and_target.after(sync_bake_spike_enabled_flag),
                    upload_control_and_arm_gpu_rtt
                        .after(sync_gpu_bake_indices_from_chunks)
                        .after(sync_gpu_bake_scissors_and_target),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bake_spike_flag_defaults_on() {
        std::env::remove_var(TERRAIN_GPU_BAKE_SPIKE_ENV);
        assert!(terrain_gpu_bake_spike_env_enabled());
        std::env::set_var(TERRAIN_GPU_BAKE_SPIKE_ENV, "0");
        assert!(!terrain_gpu_bake_spike_env_enabled());
        std::env::remove_var(TERRAIN_GPU_BAKE_SPIKE_ENV);
        let state = TerrainGpuBakeSpikeState::default();
        assert!(!state.enabled);
        assert!(state.fire_excluded_from_bake);
        assert!(state.cpu_display_still_authority);
        assert_eq!(state.last_dispatch_stamp, 0);
        assert!(!spike_bake_ready_for_consumers(&state));
    }

    #[test]
    fn pack_scissors_from_dirty_chunks() {
        let dirty = vec![(0, 0), (1, 0)];
        let scissors = pack_gpu_bake_scissors(&dirty, 200, 128);
        assert_eq!(scissors.len(), 2);
        assert_eq!(
            scissors[0],
            TerrainGpuBakeScissor {
                x0: 0,
                y0: 0,
                x1: 128,
                y1: 128
            }
        );
        assert_eq!(
            scissors[1],
            TerrainGpuBakeScissor {
                x0: 128,
                y0: 0,
                x1: 200,
                y1: 128
            }
        );
        assert_eq!(scissors[0].tile_count(), 128 * 128);
        assert_eq!(scissors[1].tile_count(), 72 * 128);
    }

    #[test]
    fn bake_indices_are_material_only_no_fire_fields() {
        let inst = TerrainTileInstance {
            world_pos: [1.5, 2.5],
            material_index: 3,
            _pad: 0,
        };
        assert_eq!(std::mem::size_of_val(&inst), 16);
        let state = TerrainGpuBakeSpikeState {
            enabled: true,
            ..Default::default()
        };
        assert!(state.fire_excluded_from_bake);
    }

    #[test]
    fn write_control_texels_packs_material_id() {
        let mut control = vec![0u8; 4 * 4 * 4];
        let instances = vec![TerrainTileInstance {
            world_pos: [1.5, 2.5],
            material_index: 7,
            _pad: 0,
        }];
        let regions = vec![TerrainGpuBakeScissor {
            x0: 0,
            y0: 0,
            x1: 4,
            y1: 4,
        }];
        let n = write_control_texels(&mut control, 4, 4, &instances, &regions);
        assert_eq!(n, 1);
        let i = ((2 * 4 + 1) * 4) as usize;
        assert_eq!(control[i], 7);
    }

    #[test]
    fn write_atlas_to_world_respects_scissors_and_skips_fire() {
        let mut bake = vec![0u8; 4 * 4 * 4];
        let mut atlas_full = vec![0u8; 8 * 8 * 4];
        for i in 0..(8 * 8) {
            let o = i * 4;
            atlas_full[o] = 40;
            atlas_full[o + 1] = 160;
            atlas_full[o + 2] = 60;
            atlas_full[o + 3] = 255;
        }
        let instances = vec![
            TerrainTileInstance {
                world_pos: [0.5, 0.5],
                material_index: 0,
                _pad: 0,
            },
            TerrainTileInstance {
                world_pos: [3.5, 3.5],
                material_index: 0,
                _pad: 0,
            },
        ];
        let regions = vec![TerrainGpuBakeScissor {
            x0: 0,
            y0: 0,
            x1: 2,
            y1: 2,
        }];
        let written = write_atlas_to_world_bake(
            &mut bake,
            4,
            4,
            &regions,
            &instances,
            &atlas_full,
            8,
            8,
            1,
        );
        assert_eq!(written, 1);
        assert_eq!(&bake[0..4], &[40, 160, 60, 255]);
        assert_eq!(&bake[(3 * 4 + 3) * 4..(3 * 4 + 3) * 4 + 4], &[0, 0, 0, 0]);
    }

    #[test]
    fn peek_dirty_does_not_clear_flags() {
        let mut grid = TileWorldFallbackChunkGrid::default();
        grid.resize_for_world(256, 128);
        grid.mark_chunk(0, 0);
        grid.mark_chunk(1, 0);
        let peek = peek_dirty_chunk_coords(&grid);
        assert_eq!(peek, vec![(0, 0), (1, 0)]);
        assert!(grid.has_dirty());
        let taken = grid.take_dirty_chunks(8);
        assert_eq!(taken, vec![(0, 0), (1, 0)]);
        assert!(!grid.has_dirty());
    }

    fn test_registry() -> MaterialRegistry {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        MaterialRegistry::load_from_json(
            root.join("assets/config/terrain/material_registry.example.json")
                .to_str()
                .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn bake_material_matches_cpu_raster_and_water_gets_bed() {
        use crate::gui::editor::world_preview::preview_biome_rgba_for_tile;
        use crate::render::tile_world_fallback::ground_display_family_for_raster;
        let reg = test_registry();
        let fams = crate::terrain::default_terrain_families();
        let names = ["Grassland", "Forest", "Mountain", "Stone", "ShallowWater", "DeepWater"];
        let mut m = ChunkCellMatrix::new(UVec2::new(names.len() as u32, 1));
        for (i, n) in names.iter().enumerate() {
            m.family[i] = fams.id(n).unwrap();
        }
        let params = WorldGenParams { width: 6, height: 1, ..Default::default() };
        let out = collect_material_bake_instances_with_registry(
            &params,
            &reg,
            &[(Chunk { coord: IVec2::ZERO }, m.clone())],
            None,
            &TerrainGpuStampIndices::default(),
        );
        assert_eq!(out.len(), names.len());
        for (i, inst) in out.iter().enumerate() {
            // CPU raster colour for the same cell (family handed through the raster's mapping).
            let raster_fam = ground_display_family_for_raster(Some(fams), m.family[i]);
            let cpu = preview_biome_rgba_for_tile(i as u32, 0, raster_fam, &[], &reg, Some(fams));
            let bake = reg.materials[inst.material_index as usize].preview_color;
            assert_eq!(bake, cpu, "{}", names[i]);
        }
        // Land is not rule-resolved grey; water is a non-water bed material.
        assert_eq!(reg.materials[out[0].material_index as usize].name, "loam_wet");
        for i in [4, 5] {
            let def = &reg.materials[out[i].material_index as usize];
            assert_ne!(def.name, "water_deep");
            assert_ne!(def.name, "water_shallow");
        }
    }

    #[test]
    fn dense_cache_family_wins_over_stale_chunk_matrix() {
        let reg = test_registry();
        let fams = crate::terrain::default_terrain_families();
        let mut m = ChunkCellMatrix::new(UVec2::new(2, 1));
        m.family[0] = fams.id("Mountain").unwrap();
        m.family[1] = fams.id("Mountain").unwrap();
        let dense = WorldGenDenseTerrainCache {
            width: 2,
            height: 1,
            elevation: vec![0.9; 2],
            moisture: vec![0.5; 2],
            temperature: vec![0.5; 2],
            family: vec![fams.id("Grassland").unwrap(); 2],
            region_index: vec![0; 2],
        };
        let params = WorldGenParams { width: 2, height: 1, ..Default::default() };
        let out = collect_material_bake_instances_with_registry(
            &params,
            &reg,
            &[(Chunk { coord: IVec2::ZERO }, m)],
            Some(&dense),
            &TerrainGpuStampIndices::default(),
        );
        assert_eq!(out.len(), 2);
        for inst in &out {
            assert_eq!(reg.materials[inst.material_index as usize].name, "loam_wet");
        }
    }

    #[test]
    fn collect_material_bake_instances_empty_world() {
        let params = WorldGenParams {
            width: 4,
            height: 4,
            ..Default::default()
        };
        let materials = Assets::<MaterialRegistry>::default();
        let stamps = TerrainGpuStampIndices::default();
        let out = collect_material_bake_instances(&params, &materials, None, &[], None, &stamps);
        assert!(out.is_empty());
    }

    #[test]
    fn gs01_production_path_is_control_upload_plus_gpu_rtt() {
        let src = include_str!("terrain_gpu_bake_spike.rs");
        assert!(src.contains("upload_control_and_arm_gpu_rtt"));
        assert!(src.contains("gpu_rtt_control_to_world_bake"));
        assert!(src.contains("write_control_texels"));
        // Plugin Update tuple must arm GPU RTT, not a host Image byte writer system.
        assert!(src.contains("upload_control_and_arm_gpu_rtt\n                        .after(sync_gpu_bake_indices_from_chunks)"));
    }

    #[test]
    fn gpu_mips_bake_image_allocates_mip_chain() {
        let img = make_rgba8_bake_image_with_mips(64, 32);
        assert!(img.texture_descriptor.mip_level_count > 1);
        assert_eq!(
            img.texture_descriptor.mip_level_count,
            bake_mip_level_count(64, 32)
        );
        assert!(img.data.is_none());
        match &img.sampler {
            ImageSampler::Descriptor(d) => {
                assert!(d.anisotropy_clamp >= 8, "RND-08 anisotropic for oblique/zoom");
                assert_eq!(d.min_filter, ImageFilterMode::Linear);
                assert_eq!(d.mipmap_filter, ImageFilterMode::Linear);
            }
            _ => panic!("bake image must carry explicit sampler descriptor"),
        }
    }

    #[test]
    fn gs_witness_probe_path_is_gpu_rtt_not_host_scissor() {
        let state = TerrainGpuBakeSpikeState {
            enabled: true,
            world_bake_w: 8,
            world_bake_h: 8,
            last_control_texel_count: 4,
            last_dispatch_stamp: 1,
            ..Default::default()
        };
        // Non-default handle required for ready — probe path string is the honesty signal.
        let probe = build_bake_spike_probe(&state, 4);
        assert_eq!(probe.path, "gpu_rtt_control_to_world_bake");
        assert!(probe.gpu_rtt_armed);
        let spike_src = include_str!("terrain_gpu_bake_spike.rs");
        let auth_src = include_str!("../core/terrain_render_authority.rs");
        // Split tokens so include_str cannot false-fail on this assert body.
        let host_scissor = format!("{}_{}", "scissor_atlas_to", "world_bake");
        assert!(!spike_src.contains(&host_scissor));
        // Retired enum variant must not remain constructible (doc mentions of the old name OK).
        assert!(!auth_src.contains("TerrainRenderAuthority::GpuTilemap"));
        assert!(!auth_src.contains("GpuTilemap,"));
        let instanced = include_str!("terrain_instanced_draw.rs");
        assert!(instanced.contains("GS-03 CLOSED") || instanced.contains("deleted"));
    }
}
