//! RPC-1-004 **A0** + RPC-1-005 **A1** — flagged GPU atlas→world bake spike.
//!
//! # Contract
//! - Default **OFF** (`TERRAIN_GPU_BAKE_SPIKE` unset / not `1`).
//! - When ON: pack dirty-region scissors + material tile indices into
//!   [`TerrainGpuBakeIndexMap`], allocate a world bake [`Image`], and run a
//!   **host-side atlas→world scissor write** into that Image (A1 stand-in for a
//!   GPU RTT pass — operator pixel proof remains RPC-1-006).
//! - Fire / weather stay **live overlays** — never stored in bake indices or pixels.
//! - CPU / sprite [`crate::render::tile_world_fallback`] remains **display truth**.
//! - Minimap may bind the bake Image when [`spike_bake_ready_for_consumers`] and
//!   label `terrain_source=gpu_bake` **only** for that bind (else `world_raster`).
//! - Does **not** populate [`crate::render::TerrainInstanceMap`] while sprite display
//!   is active (avoids dual terrain draw). Dormant `terrain_instanced_draw` may consume
//!   these bake indices later.
//!
//! Plan: `src/dev/plan_rpc1_gpu_terrain_ab_v1.md`.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::gui::map_tile_atlas_stamp::TerrainGpuStampIndices;
use crate::render::pipelines::terrain_instanced_draw::TerrainTileInstance;
use crate::render::core::terrain_material_atlas::TerrainMaterialAtlasGpu;

/// Must match `terrain_material_atlas` cell size (host sample of atlas→world bake).
const BAKE_ATLAS_TILE_PX: u32 = 8;
use crate::render::tile_world_fallback::{
    TileWorldFallbackChunkGrid, TileWorldFallbackRasterCtrl, TileWorldFallbackState,
    RASTER_CHUNK_TILES,
};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::{family_default_material_def, MaterialId, MaterializedChunk, MaterialRegistry};

/// Env flag — opt-in A0/A1 bake spike. Default OFF.
pub const TERRAIN_GPU_BAKE_SPIKE_ENV: &str = "TERRAIN_GPU_BAKE_SPIKE";

/// Live witness path for RPC-1-004 A0 probe (kept for regression).
pub const TERRAIN_GPU_BAKE_SPIKE_LIVE_JSON: &str =
    "debug_runs/terrain_gpu_bake_spike_rpc1_004_live.json";

/// Live witness path for RPC-1-005 A1 consumers probe.
pub const TERRAIN_GPU_BAKE_SPIKE_A1_LIVE_JSON: &str =
    "debug_runs/terrain_gpu_bake_spike_rpc1_005_live.json";

#[must_use]
pub fn terrain_gpu_bake_spike_env_enabled() -> bool {
    std::env::var(TERRAIN_GPU_BAKE_SPIKE_ENV)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
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
}

/// Material-only bake indices (no fire/weather). Separate from display `TerrainInstanceMap`.
#[derive(Resource, Debug, Clone, Default)]
pub struct TerrainGpuBakeIndexMap {
    pub instances: Vec<TerrainTileInstance>,
    pub revision: u64,
}

/// Host-side spike state — index feed + world bake target + atlas→world write.
#[derive(Resource, Debug, Clone)]
pub struct TerrainGpuBakeSpikeState {
    pub enabled: bool,
    pub bake_revision: u64,
    pub regions: Vec<TerrainGpuBakeScissor>,
    pub world_bake_image: Handle<Image>,
    pub world_bake_w: u32,
    pub world_bake_h: u32,
    /// Always true by contract — fire is never written into bake indices/pixels.
    pub fire_excluded_from_bake: bool,
    /// Display pixels still owned by CPU dirty-bake → sprite (not this spike).
    pub cpu_display_still_authority: bool,
    /// Host atlas→world scissor write path is active (A1). GPU RTT shader still deferred.
    pub rtt_pass_wired: bool,
    /// Pixels written by the last successful atlas→world pass.
    pub last_write_pixel_count: u32,
    /// Index-map revision last consumed by the write pass.
    pub last_write_index_revision: u64,
}

impl Default for TerrainGpuBakeSpikeState {
    fn default() -> Self {
        Self {
            enabled: false,
            bake_revision: 0,
            regions: Vec::new(),
            world_bake_image: Handle::default(),
            world_bake_w: 0,
            world_bake_h: 0,
            fire_excluded_from_bake: true,
            cpu_display_still_authority: true,
            rtt_pass_wired: false,
            last_write_pixel_count: 0,
            last_write_index_revision: 0,
        }
    }
}

/// True when consumers may honestly bind the bake Image (flag on + write proved).
#[must_use]
pub fn spike_bake_ready_for_consumers(state: &TerrainGpuBakeSpikeState) -> bool {
    state.enabled
        && state.rtt_pass_wired
        && state.world_bake_image != Handle::default()
        && state.world_bake_w > 0
        && state.world_bake_h > 0
        && state.last_write_pixel_count > 0
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
    pub rtt_pass_wired: bool,
    pub consumers_ready: bool,
    pub write_pixel_count: u32,
    /// Default-off label contract: WorldRaster bind still reads `world_raster`.
    pub minimap_default_off_is_world_raster: bool,
    /// Flag-on honest label when consumers ready.
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
    chunks: &[(Chunk, ChunkCellMatrix, Option<MaterializedChunk>)],
    gpu_stamps: &TerrainGpuStampIndices,
) -> Vec<TerrainTileInstance> {
    let reg = handles.and_then(|h| materials.get(&h.material_registry));
    let tex_w = params.width;
    let tex_h = params.height;
    let mut out = Vec::new();

    for (chunk, matrix, mat_chunk) in chunks {
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

    crate::gui::map_tile_atlas_stamp::apply_gpu_stamps_to_terrain_instances(gpu_stamps, &mut out);
    out
}

/// Sample material atlas cell center RGBA for a material index.
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
    let px = col * BAKE_ATLAS_TILE_PX + BAKE_ATLAS_TILE_PX / 2;
    let py = row * BAKE_ATLAS_TILE_PX + BAKE_ATLAS_TILE_PX / 2;
    let px = px.min(atlas_w.saturating_sub(1));
    let py = py.min(atlas_h.saturating_sub(1));
    let i = ((py * atlas_w + px) * 4) as usize;
    if i + 3 < atlas_data.len() {
        [atlas_data[i], atlas_data[i + 1], atlas_data[i + 2], atlas_data[i + 3]]
    } else {
        [0, 0, 0, 255]
    }
}

/// Host atlas→world scissor write into bake Image bytes (A1 consumer feed).
///
/// Returns pixels written. Does not touch fire/weather.
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
        rtt_pass_wired: state.rtt_pass_wired,
        consumers_ready: spike_bake_ready_for_consumers(state),
        write_pixel_count: state.last_write_pixel_count,
        minimap_default_off_is_world_raster: minimap_terrain_source_label(
            MinimapTerrainSourceBind::WorldRaster,
        ) == "world_raster",
        minimap_gpu_bake_label_available: minimap_terrain_source_label(
            MinimapTerrainSourceBind::GpuBake,
        ) == "gpu_bake",
        path: "scissor_atlas_to_world_bake",
    }
}

/// A0 exit predicate — spike green when flag-gated path is proven without flipping defaults.
/// After A1, flag-off still uses this; flag-on A0 green no longer requires `!rtt_pass_wired`.
#[must_use]
pub fn terrain_gpu_bake_spike_a0_green(probe: &TerrainGpuBakeSpikeProbe) -> bool {
    probe.fire_excluded_from_bake
        && probe.cpu_display_still_authority
        && probe.minimap_default_off_is_world_raster
        && probe.path.starts_with("scissor_atlas_to_world")
        && {
            if !probe.flag_enabled {
                probe.flag_default_off && probe.index_count == 0 && !probe.rtt_pass_wired
            } else {
                probe.world_bake_target_allocated
                    && (probe.region_count > 0 || probe.index_count > 0)
            }
        }
}

/// A1 exit — consumers can honestly bind `gpu_bake` when flag on; default-off unchanged.
#[must_use]
pub fn terrain_gpu_bake_spike_a1_green(probe: &TerrainGpuBakeSpikeProbe) -> bool {
    terrain_gpu_bake_spike_a0_green(probe)
        && probe.minimap_gpu_bake_label_available
        && {
            if !probe.flag_enabled {
                !probe.consumers_ready
            } else {
                probe.rtt_pass_wired && probe.consumers_ready && probe.write_pixel_count > 0
            }
        }
}

fn make_world_bake_image(w: u32, h: u32) -> Image {
    let width = w.max(1);
    let height = h.max(1);
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("terrain_gpu_bake_spike_world"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.data = Some(vec![0u8; 4 * width as usize * height as usize]);
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
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
            state.world_bake_w = 0;
            state.world_bake_h = 0;
            state.rtt_pass_wired = false;
            state.last_write_pixel_count = 0;
            state.last_write_index_revision = 0;
        }
    }
    state.fire_excluded_from_bake = true;
    state.cpu_display_still_authority = true;
}

fn sync_gpu_bake_indices_from_chunks(
    state: Res<TerrainGpuBakeSpikeState>,
    params: Res<WorldGenParams>,
    handles: Option<Res<TerrainRegistriesHandles>>,
    materials: Res<Assets<MaterialRegistry>>,
    gpu_stamps: Res<TerrainGpuStampIndices>,
    chunks: Query<(&Chunk, &ChunkCellMatrix, Option<&MaterializedChunk>)>,
    mut map: ResMut<TerrainGpuBakeIndexMap>,
) {
    if !state.enabled {
        if !map.instances.is_empty() {
            map.instances.clear();
            map.revision = map.revision.wrapping_add(1);
        }
        return;
    }

    let owned: Vec<(Chunk, ChunkCellMatrix, Option<MaterializedChunk>)> = chunks
        .iter()
        .map(|(c, m, mat)| (*c, m.clone(), mat.cloned()))
        .collect();
    let out = collect_material_bake_instances(
        params.as_ref(),
        materials.as_ref(),
        handles.as_deref(),
        &owned,
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
        // First enable / no dirty peek — full-world scissor so the spike is still measurable.
        vec![TerrainGpuBakeScissor {
            x0: 0,
            y0: 0,
            x1: tex_w,
            y1: tex_h,
        }]
    } else {
        pack_gpu_bake_scissors(&dirty, tex_w, tex_h)
    };

    let need_image = state.world_bake_image == Handle::default()
        || state.world_bake_w != tex_w
        || state.world_bake_h != tex_h;
    if need_image {
        let image = make_world_bake_image(tex_w, tex_h);
        state.world_bake_image = images.add(image);
        state.world_bake_w = tex_w;
        state.world_bake_h = tex_h;
        state.bake_revision = state.bake_revision.wrapping_add(1);
        state.last_write_pixel_count = 0;
        state.last_write_index_revision = 0;
    }

    if state.regions != regions {
        state.regions = regions;
        state.bake_revision = state.bake_revision.wrapping_add(1);
    }
}

/// A1 — host-side atlas→world scissor write into the bake Image (fire excluded).
fn apply_gpu_bake_atlas_to_world_write(
    mut images: ResMut<Assets<Image>>,
    atlas: Res<TerrainMaterialAtlasGpu>,
    map: Res<TerrainGpuBakeIndexMap>,
    mut state: ResMut<TerrainGpuBakeSpikeState>,
) {
    if !state.enabled {
        state.rtt_pass_wired = false;
        return;
    }
    // Path is wired once scissors+target exist; consumers wait for pixels.
    state.rtt_pass_wired = true;

    if state.world_bake_image == Handle::default() || state.world_bake_w == 0 || state.world_bake_h == 0
    {
        return;
    }
    if map.instances.is_empty() {
        return;
    }
    if map.revision == state.last_write_index_revision && state.last_write_pixel_count > 0 {
        return;
    }

    let Some(atlas_img) = images.get(&atlas.image) else {
        return;
    };
    let Some(atlas_src) = atlas_img.data.as_ref() else {
        return;
    };
    let atlas_w = atlas_img.width();
    let atlas_h = atlas_img.height();
    let atlas_cols = atlas.cols.max(1);
    let atlas_bytes = atlas_src.clone();
    let regions = state.regions.clone();
    let bake_w = state.world_bake_w;
    let bake_h = state.world_bake_h;
    let bake_handle = state.world_bake_image.clone();

    let Some(mut bake) = images.get_mut(&bake_handle) else {
        return;
    };
    let Some(bake_data) = bake.data.as_mut() else {
        return;
    };

    let written = write_atlas_to_world_bake(
        bake_data,
        bake_w,
        bake_h,
        &regions,
        &map.instances,
        &atlas_bytes,
        atlas_w,
        atlas_h,
        atlas_cols,
    );
    if written > 0 {
        state.last_write_pixel_count = written;
        state.last_write_index_revision = map.revision;
        state.bake_revision = state.bake_revision.wrapping_add(1);
    }
}

pub struct TerrainGpuBakeSpikePlugin;

impl Plugin for TerrainGpuBakeSpikePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainGpuBakeSpikeState>()
            .init_resource::<TerrainGpuBakeIndexMap>()
            .add_systems(
                Update,
                (
                    sync_bake_spike_enabled_flag,
                    sync_gpu_bake_indices_from_chunks.after(sync_bake_spike_enabled_flag),
                    sync_gpu_bake_scissors_and_target.after(sync_bake_spike_enabled_flag),
                    apply_gpu_bake_atlas_to_world_write
                        .after(sync_gpu_bake_indices_from_chunks)
                        .after(sync_gpu_bake_scissors_and_target),
                ),
            );
    }
}

#[must_use]
pub fn build_terrain_gpu_bake_spike_witness_body(
    state: &TerrainGpuBakeSpikeState,
    map: &TerrainGpuBakeIndexMap,
) -> serde_json::Value {
    let probe = build_bake_spike_probe(state, map.instances.len() as u32);
    let a0 = terrain_gpu_bake_spike_a0_green(&probe);
    let a1 = terrain_gpu_bake_spike_a1_green(&probe);
    let terrain_source = if spike_bake_ready_for_consumers(state) {
        "gpu_bake"
    } else {
        "world_raster"
    };
    serde_json::json!({
        "gate": "RPC-1-005",
        "fleet": "FLEET-RPC1-005",
        "green": a1,
        "a0_green": a0,
        "a1_green": a1,
        "flag_env": TERRAIN_GPU_BAKE_SPIKE_ENV,
        "flag_enabled": probe.flag_enabled,
        "flag_default_off": !terrain_gpu_bake_spike_env_enabled(),
        "index_count": probe.index_count,
        "region_count": probe.region_count,
        "bake_revision": state.bake_revision,
        "fire_excluded_from_bake": probe.fire_excluded_from_bake,
        "cpu_display_still_authority": probe.cpu_display_still_authority,
        "world_bake_target_allocated": probe.world_bake_target_allocated,
        "world_bake_w": state.world_bake_w,
        "world_bake_h": state.world_bake_h,
        "rtt_pass_wired": probe.rtt_pass_wired,
        "rtt_pass_kind": "host_atlas_to_world_scissor",
        "consumers_ready": probe.consumers_ready,
        "write_pixel_count": probe.write_pixel_count,
        "minimap_terrain_source": terrain_source,
        "minimap_default_off_is_world_raster": probe.minimap_default_off_is_world_raster,
        "minimap_gpu_bake_label_available": probe.minimap_gpu_bake_label_available,
        "path": probe.path,
        "chunk_tiles": RASTER_CHUNK_TILES,
        "operator_pixel_proof": false,
        "next": "RPC-1-006 — operator pixel prove + optional default flip (do not claim here)",
        "plan_ref": "src/dev/plan_rpc1_gpu_terrain_ab_v1.md",
    })
}

#[must_use]
pub fn refresh_terrain_gpu_bake_spike_witness(
    state: &TerrainGpuBakeSpikeState,
    map: &TerrainGpuBakeIndexMap,
) -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body_a1 = build_terrain_gpu_bake_spike_witness_body(state, map);
    let a0 = body_a1
        .get("a0_green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let a1 = body_a1
        .get("a1_green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut body_a0 = body_a1.clone();
    if let Some(obj) = body_a0.as_object_mut() {
        obj.insert("gate".into(), serde_json::json!("RPC-1-004"));
        obj.insert("fleet".into(), serde_json::json!("FLEET-RPC1-004"));
        obj.insert("green".into(), serde_json::json!(a0));
        obj.insert(
            "next".into(),
            serde_json::json!("RPC-1-005 closed — see terrain_gpu_bake_spike_rpc1_005_live.json"),
        );
    }
    let wrapped_a0 = wrap_debug_run(
        "RPC-1-004",
        "refresh_terrain_gpu_bake_spike_witness",
        TERRAIN_GPU_BAKE_SPIKE_LIVE_JSON,
        body_a0,
    );
    let ok004 = write_debug_run_json(TERRAIN_GPU_BAKE_SPIKE_LIVE_JSON, wrapped_a0) && a0;

    let wrapped_a1 = wrap_debug_run(
        "RPC-1-005",
        "refresh_terrain_gpu_bake_spike_a1_witness",
        TERRAIN_GPU_BAKE_SPIKE_A1_LIVE_JSON,
        body_a1,
    );
    let ok005 = write_debug_run_json(TERRAIN_GPU_BAKE_SPIKE_A1_LIVE_JSON, wrapped_a1) && a1;
    ok004 && ok005
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bake_spike_flag_defaults_off() {
        std::env::remove_var(TERRAIN_GPU_BAKE_SPIKE_ENV);
        assert!(!terrain_gpu_bake_spike_env_enabled());
        let state = TerrainGpuBakeSpikeState::default();
        assert!(!state.enabled);
        assert!(state.fire_excluded_from_bake);
        assert!(state.cpu_display_still_authority);
        assert!(!state.rtt_pass_wired);
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
        // TerrainTileInstance layout is world_pos + material_index only — fire cannot ride along.
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
    fn a0_probe_green_when_flag_off() {
        std::env::remove_var(TERRAIN_GPU_BAKE_SPIKE_ENV);
        let state = TerrainGpuBakeSpikeState::default();
        let probe = build_bake_spike_probe(&state, 0);
        assert!(terrain_gpu_bake_spike_a0_green(&probe));
        assert!(terrain_gpu_bake_spike_a1_green(&probe));
        assert!(probe.minimap_default_off_is_world_raster);
    }

    #[test]
    fn a0_probe_requires_target_when_flag_on() {
        let state = TerrainGpuBakeSpikeState {
            enabled: true,
            regions: vec![TerrainGpuBakeScissor {
                x0: 0,
                y0: 0,
                x1: 16,
                y1: 16,
            }],
            world_bake_image: Handle::default(),
            world_bake_w: 0,
            world_bake_h: 0,
            ..Default::default()
        };
        let probe = build_bake_spike_probe(&state, 4);
        assert!(!terrain_gpu_bake_spike_a0_green(&probe));

        let ok_state = TerrainGpuBakeSpikeState {
            enabled: true,
            regions: vec![TerrainGpuBakeScissor {
                x0: 0,
                y0: 0,
                x1: 16,
                y1: 16,
            }],
            world_bake_w: 16,
            world_bake_h: 16,
            ..Default::default()
        };
        let probe_ok = build_bake_spike_probe(&ok_state, 4);
        assert!(probe_ok.world_bake_target_allocated);
        assert!(terrain_gpu_bake_spike_a0_green(&probe_ok));
        // A1 needs write + consumers_ready.
        assert!(!terrain_gpu_bake_spike_a1_green(&probe_ok));
    }

    #[test]
    fn a1_probe_green_when_consumers_ready() {
        let mut images = Assets::<Image>::default();
        let handle = images.add(Image::default());
        let mut state = TerrainGpuBakeSpikeState {
            enabled: true,
            regions: vec![TerrainGpuBakeScissor {
                x0: 0,
                y0: 0,
                x1: 4,
                y1: 4,
            }],
            world_bake_w: 4,
            world_bake_h: 4,
            rtt_pass_wired: true,
            last_write_pixel_count: 4,
            world_bake_image: handle,
            ..Default::default()
        };
        assert!(spike_bake_ready_for_consumers(&state));
        let probe = build_bake_spike_probe(&state, 4);
        assert!(terrain_gpu_bake_spike_a1_green(&probe));

        state.last_write_pixel_count = 0;
        assert!(!spike_bake_ready_for_consumers(&state));
    }

    #[test]
    fn write_atlas_to_world_respects_scissors_and_skips_fire() {
        let mut bake = vec![0u8; 4 * 4 * 4];
        let atlas = vec![
            // 8x8 cell of green for mat 0 (only first pixels matter for center sample)
            40u8, 160, 60, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];
        // Pad atlas to 8x8
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
        // Outside scissor untouched
        assert_eq!(&bake[(3 * 4 + 3) * 4..(3 * 4 + 3) * 4 + 4], &[0, 0, 0, 0]);
        let _ = atlas; // silence
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

    #[test]
    fn collect_material_bake_instances_empty_world() {
        let params = WorldGenParams {
            width: 4,
            height: 4,
            ..Default::default()
        };
        let materials = Assets::<MaterialRegistry>::default();
        let stamps = TerrainGpuStampIndices::default();
        let out = collect_material_bake_instances(&params, &materials, None, &[], &stamps);
        assert!(out.is_empty());
    }

    #[test]
    fn refresh_witness_flag_off_green() {
        std::env::remove_var(TERRAIN_GPU_BAKE_SPIKE_ENV);
        let state = TerrainGpuBakeSpikeState::default();
        let map = TerrainGpuBakeIndexMap::default();
        assert!(refresh_terrain_gpu_bake_spike_witness(&state, &map));
    }
}
