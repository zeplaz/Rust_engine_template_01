//! Enhanced procedural world generation: parallel height/moisture/temperature sampling, Voronoi **region**
//! hierarchy, hydrology, river/lake spawn pipelines.
//!
//! **Polygon / graph-first design** (macro regions, semantic terrain, coarse sim layers) is documented in
//! [`voronoi_polygon_worlds_notes`](../../../prompts/guides/voronoi_polygon_worlds_notes.md.md).
//! Voronoi site count here still maps tiles → region entities; optional [`WorldGenParams::strategic_field_coupling`]
//! applies [`MacroTerrainSemantics`](super::polygon_world_semantics::MacroTerrainSemantics) nudges to moisture/temperature
//! after noise sampling (structure-informed fields without replacing the fractal core).

use std::collections::HashMap;
use std::time::Instant;

use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver, TryRecvError};
use noise::{Fbm, NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::terrain_noise::{
    build_fbm_perlin, build_height_noise, sample_height_field, HeightNoise, NoiseSamplingTuning,
};
pub use super::terrain_noise::TerrainNoiseProfile;

use super::tuning_io;
use crate::terrain::voronoi_enhanced::*;
use crate::terrain::world::GeoRegion;
use crate::terrain::biome::BiomeTuning;
use crate::terrain::family::{
    classify_biome, default_terrain_families, TerrainFamilyId, DEFAULT_TERRAIN_FAMILY_ID,
};
use crate::engine::WorldGenFlowState;
use crate::terrain::generation::hydrology::{compute_hydrology_world, HydrologyParams, HydrologyResult};
use crate::terrain::generation::world_gen_diagnostics::{
    summary_line, write_world_gen_debug_report, WorldGenLastDebugReport, WorldGenRunTiming,
};
use crate::terrain::material::TagSet;

pub use crate::terrain::generation::polygon_world_semantics::{
    apply_strategic_field_nudge, classify_strategic_tile, MacroStrategicKind,
};

// World generation parameters structure
#[derive(Resource, Clone)]
pub struct WorldGenParams {
    // General settings
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    
    /// Voronoi **site count** for tile→region assignment (ECS hierarchy / future sim hooks).
    /// This does **not** drive height, moisture, or temperature — those come entirely from fractal noise
    /// (`noise_scale`, profiles, island falloff, warp, etc.).
    pub num_regions: u32,
    pub region_method: RegionMethod,
    pub region_iterations: u32,  // For centroidal relaxation

    /// 0–~0.35: nudge moisture/temperature from [`MacroTerrainSemantics`](super::polygon_world_semantics::MacroTerrainSemantics) after noise; 0 still **classifies** every tile.
    pub strategic_field_coupling: f32,
    // Terrain / noise
    /// Multiplier on **world tile indices** into height noise (higher → higher spatial frequency → more features per map).
    pub noise_scale: f32,
    pub noise_octaves: u32,
    /// Lacunarity between octaves (frequency multiplier per octave).
    pub noise_lacunarity: f32,
    /// Persistence between octaves (amplitude falloff per octave).
    pub noise_persistence: f32,
    /// Which fractal family drives **height** (moisture/temperature use fBm·Perlin).
    pub height_noise_profile: TerrainNoiseProfile,
    /// >1 flattens lowlands; <1 lifts lows — artistic land/ocean contrast after base noise.
    pub height_curve_exponent: f32,
    /// Domain warp: displaces sample coordinates before height noise (0 = off). Natural coasts / folds.
    pub domain_warp_strength: f32,
    /// Blend in high-frequency fBm for small-scale breakup (0–1).
    pub terrain_detail_mix: f32,
    pub moisture_bias: f32,
    pub temperature_bias: f32,

    /// Channel frequencies / warp / detail — when loading chunk fields with [`crate::terrain::generation::passes::p1_fields::fill_fields`]
    /// and `tuning_overlay` is `None`, this is the tuning used (same path as [`crate::terrain::generation::world_generator_enhanced::generate_world`]).
    /// Optionally overridden by JSON via `tuning_io` merge on startup.
    pub noise_sampling: NoiseSamplingTuning,
    /// Biome weights + class thresholds — must stay aligned with [`crate::terrain::family::classify_biome`].
    pub biome_tuning: BiomeTuning,
    
    // Feature settings
    pub river_count: u32,
    pub lake_count: u32,
    pub mountain_threshold: f32,
    pub island_mode: bool,
    pub island_falloff: f32,
    /// Tags permitted in pass 2 / 4 (`ChunkCellMatrix`). Unchecked tags in the World Generator UI are cleared here.
    pub tag_pool: TagSet,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum RegionMethod {
    Regular,
    Manhattan,
    Weighted,
    Centroidal,
    Circular,
    Power,
}

impl Default for WorldGenParams {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            seed: rand::random(),
            num_regions: 24,
            region_method: RegionMethod::Centroidal,
            region_iterations: 3,
            strategic_field_coupling: 0.0,
            noise_scale: 0.024,
            noise_octaves: 6,
            noise_lacunarity: 2.0,
            noise_persistence: 0.5,
            height_noise_profile: TerrainNoiseProfile::default(),
            height_curve_exponent: 1.12,
            domain_warp_strength: 0.32,
            terrain_detail_mix: 0.22,
            moisture_bias: 0.0,
            temperature_bias: 0.03,
            noise_sampling: NoiseSamplingTuning::default(),
            biome_tuning: BiomeTuning::default(),
            river_count: 5,
            lake_count: 3,
            mountain_threshold: 0.7,
            island_mode: false,
            island_falloff: 3.0,
            tag_pool: TagSet::ALL,
        }
    }
}

/// Noise kernels for height / warp / detail / moisture / temperature — build once per full-world or chunk fill.
#[derive(Clone)]
pub struct WorldNoiseKernels {
    pub height: HeightNoise,
    pub warp: Fbm<Perlin>,
    pub detail: Fbm<Perlin>,
    pub moisture: Fbm<Perlin>,
    pub temperature: Fbm<Perlin>,
}

/// Same kernel construction as legacy `generate_world` (frequencies derived from `tuning`, usually `params.noise_sampling`).
pub fn build_world_noise_kernels(params: &WorldGenParams, tuning: &NoiseSamplingTuning) -> WorldNoiseKernels {
    let lac = params.noise_lacunarity as f64;
    let per = params.noise_persistence as f64;
    let height_noise = build_height_noise(
        params.height_noise_profile,
        params.noise_scale,
        params.noise_octaves,
        params.seed,
        lac,
        per,
    );
    let warp_noise = build_fbm_perlin(
        params.noise_scale * tuning.warp_noise_scale_mul,
        tuning.warp_noise_octaves,
        params.seed.wrapping_add(tuning.warp_seed_offset),
        lac,
        per,
    );
    let detail_noise = build_fbm_perlin(
        params.noise_scale * tuning.detail_noise_scale_mul,
        tuning.detail_noise_octaves,
        params.seed.wrapping_add(tuning.detail_seed_offset),
        lac,
        per * tuning.detail_persistence_mul,
    );
    let moisture_noise = build_fbm_perlin(
        params.noise_scale * tuning.moisture_noise_scale_mul,
        params.noise_octaves,
        params.seed.wrapping_add(1),
        lac,
        per,
    );
    let temperature_noise = build_fbm_perlin(
        params.noise_scale * tuning.temperature_noise_scale_mul,
        params.noise_octaves,
        params.seed.wrapping_add(2),
        lac,
        per,
    );
    WorldNoiseKernels {
        height: height_noise,
        warp: warp_noise,
        detail: detail_noise,
        moisture: moisture_noise,
        temperature: temperature_noise,
    }
}

/// One tile after parallel height-field sampling (ECS spawn reads this on the main thread).
#[derive(Clone, Copy)]
struct TileSpawnData {
    moisture: f32,
    temperature: f32,
    terrain_family: TerrainFamilyId,
    region_index: usize,
    strategic_kind: MacroStrategicKind,
}

/// Raster produced on a background thread (`rayon` per row), then consumed for batched ECS spawns.
struct PrecomputedTiling {
    height_grid: Vec<f32>,
    /// Same layout as `height_grid` — feeds moisture-weighted flow accumulation in hydrology.
    moisture_grid: Vec<f32>,
    cells: Vec<TileSpawnData>,
}

#[inline]
fn closest_voronoi_region_index(position: Vec2, regions: &[Vec<VoronoiSite>]) -> usize {
    let mut closest_region_index = 0usize;
    let mut min_distance = f32::MAX;
    for (region_index, region_points) in regions.iter().enumerate() {
        for point in region_points {
            let distance = position.distance(point.position);
            if distance < min_distance {
                min_distance = distance;
                closest_region_index = region_index;
            }
        }
    }
    closest_region_index
}

/// Parallel over world rows (CPU pool). Runs on a dedicated thread so the Bevy main thread can keep polling.
fn compute_tiling_parallel(
    params: WorldGenParams,
    regions: Vec<Vec<VoronoiSite>>,
    kernels: WorldNoiseKernels,
    w: usize,
    height: u32,
) -> PrecomputedTiling {
    let h = height as usize;
    let grid_len = w * h;
    let mut height_grid = vec![0.0f32; grid_len];
    let mut moisture_grid = vec![0.0f32; grid_len];
    let mut cells = vec![
        TileSpawnData {
            moisture: 0.0,
            temperature: 0.0,
            terrain_family: DEFAULT_TERRAIN_FAMILY_ID,
            region_index: 0,
            strategic_kind: MacroStrategicKind::default(),
        };
        grid_len
    ];
    let tuning = &params.noise_sampling;
    let families = default_terrain_families();

    cells
        .par_chunks_mut(w)
        .zip(height_grid.par_chunks_mut(w))
        .zip(moisture_grid.par_chunks_mut(w))
        .enumerate()
        .for_each(|(y, ((row_cells, row_heights), row_moist))| {
            for x in 0..w {
                let position = Vec2::new(x as f32, y as f32);
                let closest_region_index = closest_voronoi_region_index(position, &regions);
                let (hv, mv, tv) = sample_fields_at_world_tile(
                    x as i32,
                    y as i32,
                    &params,
                    &kernels,
                    tuning,
                );
                row_heights[x] = hv;
                let mut mv = mv;
                let mut tv = tv;
                let strategic_kind =
                    classify_strategic_tile(hv, mv, tv, params.mountain_threshold);
                apply_strategic_field_nudge(
                    strategic_kind,
                    params.strategic_field_coupling,
                    &mut mv,
                    &mut tv,
                );
                row_moist[x] = mv;
                let terrain_family =
                    classify_biome(hv, mv, tv, &params.biome_tuning, families).terrain_family;
                row_cells[x] = TileSpawnData {
                    moisture: mv,
                    temperature: tv,
                    terrain_family,
                    region_index: closest_region_index,
                    strategic_kind,
                };
            }
        });

    PrecomputedTiling {
        height_grid,
        moisture_grid,
        cells,
    }
}

/// Island edge falloff in world tile space — normalized by `params.width` / `height` like legacy tile spawn.
#[inline]
pub fn island_falloff_at_world_tile(wx: i32, wy: i32, params: &WorldGenParams) -> f32 {
    let w = params.width.max(1) as f32;
    let h = params.height.max(1) as f32;
    let normalized_x = wx as f32 / w;
    let normalized_y = wy as f32 / h;
    if params.island_mode {
        let dx = normalized_x * 2.0 - 1.0;
        let dy = normalized_y * 2.0 - 1.0;
        let distance_from_center = (dx * dx + dy * dy).sqrt();
        (1.0 - distance_from_center.powf(params.island_falloff)).max(0.0)
    } else {
        1.0
    }
}

/// Elevation (with island falloff), moisture, temperature for one world tile — shared by `generate_world` and pass 1.
pub fn sample_fields_at_world_tile(
    wx: i32,
    wy: i32,
    params: &WorldGenParams,
    kernels: &WorldNoiseKernels,
    tuning: &NoiseSamplingTuning,
) -> (f32, f32, f32) {
    let noise_x = wx as f64 * params.noise_scale as f64;
    let noise_y = wy as f64 * params.noise_scale as f64;
    let height_norm = sample_height_field(
        &kernels.height,
        &kernels.warp,
        &kernels.detail,
        noise_x,
        noise_y,
        params.height_curve_exponent,
        params.domain_warp_strength,
        params.terrain_detail_mix,
        tuning,
    );
    let height_value = height_norm * island_falloff_at_world_tile(wx, wy, params);
    let moisture_value = (kernels.moisture.get([
        noise_x * tuning.moisture_sample_freq_mul,
        noise_y * tuning.moisture_sample_freq_mul,
        0.0,
    ]) * 0.5
        + 0.5) as f32
        + params.moisture_bias;
    let temperature_value = (kernels.temperature.get([
        noise_x * tuning.temperature_sample_freq_mul,
        noise_y * tuning.temperature_sample_freq_mul,
        0.0,
    ]) * 0.5
        + 0.5) as f32
        + params.temperature_bias;
    (height_value, moisture_value, temperature_value)
}

// Component to tag entities as part of the world
#[derive(Component)]
pub struct WorldMarker;

/// Removes prior procedural worlds before regenerating (avoids duplicate `WorldMarker` hierarchies).
pub fn despawn_generated_world_entities(
    commands: &mut Commands,
    query: &Query<Entity, With<WorldMarker>>,
) {
    commands.remove_resource::<MacroRegionRaster>();
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// Component to tag entities as part of a region
#[derive(Component)]
pub struct RegionMarker;

// Component to tag entities as tiles
#[derive(Component)]
pub struct TileMarker;

/// Voronoi macro-region metadata on the region parent entity (`RegionMarker`).
#[derive(Component, Clone, Copy, Debug)]
pub struct MacroRegion {
    /// Index 0 .. `num_regions`−1 (matches [`TileRegionIndex`] on child tiles).
    pub index: u32,
    /// Centroid of Voronoi sites in **world tile** XY (same space as tile `Transform` x/z).
    pub site_center: Vec2,
    /// Tiles assigned to this region after generation.
    pub tile_count: u32,
}

/// Dense `width × height` Voronoi macro-region index per world tile (same layout as [`WorldGenActive::height_grid`]).
#[derive(Resource, Clone, Debug, Default)]
pub struct MacroRegionRaster {
    pub width: u32,
    pub height: u32,
    pub indices: Vec<u32>,
}

impl MacroRegionRaster {
    #[inline]
    pub fn region_at(&self, x: u32, y: u32) -> Option<u32> {
        if self.width == 0 || self.height == 0 {
            return None;
        }
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = (y as usize) * (self.width as usize) + (x as usize);
        self.indices.get(i).copied()
    }
}

/// Voronoi region index for this tile (site id 0..`num_regions`−1). Used by editor region preview.
#[derive(Component, Clone, Copy)]
pub struct TileRegionIndex(pub u32);

/// Marker for hydrology feature entities spawned during enhanced world gen.
#[derive(Component)]
pub struct RiverMarker;

/// Marker for hydrology feature entities spawned during enhanced world gen.
#[derive(Component)]
pub struct LakeMarker;

// Height component
#[derive(Component)]
pub struct Height(pub f32);

// Moisture component (for biome determination)
#[derive(Component)]
pub struct Moisture(pub f32);

// Temperature component (for biome determination)
#[derive(Component)]
pub struct Temperature(pub f32);

/// Legacy alias — dominant terrain **family** id (dense index into [`crate::terrain::TerrainFamilyRegistry`]).
#[deprecated(note = "Use terrain::family::TerrainFamilyId")]
pub type BiomeType = TerrainFamilyId;

/// Dominant terrain **family** on a tile (`MaterialDef.family` / chunk `ChunkCellMatrix.family`).
#[derive(Component)]
pub struct TerrainType(pub TerrainFamilyId);

/// Max world width / height (tiles) for **full** generation (`GenerateWorldEvent` full phase).
pub const MAX_WORLD_GEN_AXIS: u32 = 5120;

/// Hard cap on `width * height` for a **full** generation pass (prevents runaway ECS spawns / memory pressure).
pub const MAX_WORLD_GEN_TILES: u64 = MAX_WORLD_GEN_AXIS as u64 * MAX_WORLD_GEN_AXIS as u64;

/// Max width/height for the **preview** generation pass (smaller than full [`MAX_WORLD_GEN_AXIS`]).
pub const PREVIEW_WORLD_MAX_AXIS: u32 = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WorldGenPhase {
    Preview,
    Full,
}

fn preview_world_params(full: &WorldGenParams) -> WorldGenParams {
    let mut p = full.clone();
    p.width = p.width.clamp(128, PREVIEW_WORLD_MAX_AXIS);
    p.height = p.height.clamp(128, PREVIEW_WORLD_MAX_AXIS);
    p
}

fn effective_params_for_phase(stored: &WorldGenParams, phase: WorldGenPhase) -> WorldGenParams {
    match phase {
        WorldGenPhase::Preview => preview_world_params(stored),
        WorldGenPhase::Full => stored.clone(),
    }
}

/// Rows of tiles spawned per frame after parallel sampling (ECS work stays batched for responsive UI).
const WORLD_GEN_TILE_ROWS_PER_TICK: u32 = 32;

#[derive(Resource, Default)]
pub struct WorldGenProgress {
    pub running: bool,
    pub label: String,
    /// Approximate overall progress 0.0–1.0 (terrain ~0–0.9, hydrology tail).
    pub fraction: f32,
}

#[derive(Clone, Copy)]
enum WorldGenPipelineStep {
    /// Background thread: `rayon` row-parallel noise + Voronoi; main thread polls `tiling_compute_rx`.
    TilingComputePending,
    /// Batched `Commands::spawn` from precomputed raster.
    TilingSpawn,
    /// Waiting on hydrology result (compute off main thread; ECS spawn on receive).
    HydrologyPending,
}

struct WorldGenActive {
    request_phase: WorldGenPhase,
    run_params: WorldGenParams,
    region_entities: Vec<Entity>,
    geo_regions: Vec<GeoRegion>,
    w: usize,
    height_grid: Vec<f32>,
    moisture_grid: Vec<f32>,
    /// Filled when [`WorldGenPipelineStep::TilingComputePending`] receives [`PrecomputedTiling`].
    spawn_cells: Vec<TileSpawnData>,
    tile_lookup: HashMap<(u32, u32), Entity>,
    next_tile_row: u32,
    step: WorldGenPipelineStep,
    biome_counts: HashMap<String, u64>,
    timing: WorldGenRunTiming,
    tiling_compute_rx: Option<Receiver<PrecomputedTiling>>,
    hydro_rx: Option<Receiver<HydrologyResult>>,
    hydro_queued_at: Option<Instant>,
    /// Filled with Voronoi region index per tile; committed to [`MacroRegionRaster`] when generation finishes.
    region_index_raster: Vec<u32>,
}

#[derive(Resource, Default)]
pub struct WorldGenJobSlot(Option<WorldGenActive>);

impl WorldGenJobSlot {
    pub fn is_busy(&self) -> bool {
        self.0.is_some()
    }
}

fn finalize_world_gen_job(
    commands: &mut Commands,
    mut job: WorldGenActive,
    hydro: Option<&HydrologyResult>,
    last_debug: &mut WorldGenLastDebugReport,
    completed: &mut MessageWriter<WorldGenCompletedEvent>,
    progress: &mut WorldGenProgress,
) {
    let phase = job.request_phase;
    let p = &job.run_params;
    let expected = (p.width as usize).saturating_mul(p.height as usize);
    if job.region_index_raster.len() == expected && expected > 0 {
        commands.insert_resource(MacroRegionRaster {
            width: p.width,
            height: p.height,
            indices: std::mem::take(&mut job.region_index_raster),
        });
    } else if expected > 0 {
        warn!(
            "world_gen: region_index_raster len {} != expected {}; MacroRegionRaster not updated",
            job.region_index_raster.len(),
            expected
        );
    }

    for (i, &region_ent) in job.region_entities.iter().enumerate() {
        let Some(gr) = job.geo_regions.get(i) else {
            continue;
        };
        commands.entity(region_ent).insert(MacroRegion {
            index: i as u32,
            site_center: gr.center,
            tile_count: gr.tiles.len() as u32,
        });
    }

    let phase_str = format!("{phase:?}");
    let profile_str = format!("{:?}", p.height_noise_profile);
    let region_str = format!("{:?}", p.region_method);

    match write_world_gen_debug_report(
        &phase_str,
        p.seed,
        p.num_regions,
        &region_str,
        p.noise_scale,
        p.noise_octaves,
        &profile_str,
        p.island_mode,
        p.island_falloff,
        p.river_count,
        p.lake_count,
        p.width,
        p.height,
        &job.timing,
        &job.height_grid,
        &job.biome_counts,
        hydro,
    ) {
        Ok((path, report)) => {
            last_debug.path = Some(path.clone());
            last_debug.summary_one_line = summary_line(&path, &report);
        }
        Err(e) => {
            warn!("Could not write world_gen debug report: {e}");
            last_debug.path = None;
            last_debug.summary_one_line = format!("debug write failed: {e}");
        }
    }

    #[cfg(debug_assertions)]
    debug!(
        target: "world_gen",
        "Finished {:?} pass ({}×{})",
        phase,
        p.width,
        p.height
    );
    info!("World generation completed ({phase:?})");
    completed.write(WorldGenCompletedEvent(phase));
    progress.running = false;
    progress.label.clear();
    progress.fraction = 0.0;
}

fn world_gen_pipeline_tick(
    mut commands: Commands,
    params: Res<WorldGenParams>,
    mut pending: ResMut<WorldGenPending>,
    mut slot: ResMut<WorldGenJobSlot>,
    mut progress: ResMut<WorldGenProgress>,
    world_roots: Query<Entity, With<WorldMarker>>,
    mut completed: MessageWriter<WorldGenCompletedEvent>,
    mut last_debug: ResMut<WorldGenLastDebugReport>,
) {
    if let Some(job) = slot.0.as_mut() {
        if matches!(job.step, WorldGenPipelineStep::HydrologyPending) {
            let Some(rx) = job.hydro_rx.as_ref() else {
                error!("world_gen: HydrologyPending without receiver — resetting job");
                slot.0 = None;
                progress.running = false;
                progress.label.clear();
                progress.fraction = 0.0;
                return;
            };
            match rx.try_recv() {
                Ok(hydro) => {
                    let hydro_ms = job
                        .hydro_queued_at
                        .as_ref()
                        .expect("hydro queue time")
                        .elapsed()
                        .as_secs_f64()
                        * 1000.0;
                    let mut job = slot.0.take().expect("job");
                    job.timing.hydrology_compute_ms = hydro_ms;
                    job.hydro_rx = None;
                    job.hydro_queued_at = None;

                    progress.label = "Spawning rivers / lakes…".to_string();
                    progress.fraction = 0.95;

                    if job.run_params.river_count > 0 {
                        spawn_hydrology_rivers(
                            &mut commands,
                            &job.run_params,
                            &job.tile_lookup,
                            &hydro,
                        );
                    }
                    if job.run_params.lake_count > 0 {
                        spawn_hydrology_lakes(
                            &mut commands,
                            &job.run_params,
                            &job.tile_lookup,
                            &job.geo_regions,
                            &job.region_entities,
                            &hydro,
                        );
                    }
                    finalize_world_gen_job(
                        &mut commands,
                        job,
                        Some(&hydro),
                        &mut last_debug,
                        &mut completed,
                        &mut progress,
                    );
                    return;
                }
                Err(TryRecvError::Empty) => {
                    progress.label = "Rivers / lakes (background CPU)…".to_string();
                    progress.fraction = 0.92;
                    return;
                }
                Err(TryRecvError::Disconnected) => {
                    error!("Hydrology worker thread disconnected");
                    slot.0 = None;
                    progress.running = false;
                    progress.label.clear();
                    progress.fraction = 0.0;
                    return;
                }
            }
        } else if matches!(job.step, WorldGenPipelineStep::TilingComputePending) {
            let Some(rx) = job.tiling_compute_rx.as_ref() else {
                error!("world_gen: TilingComputePending without receiver — resetting job");
                slot.0 = None;
                progress.running = false;
                progress.label.clear();
                progress.fraction = 0.0;
                return;
            };
            match rx.try_recv() {
                Ok(pre) => {
                    job.height_grid = pre.height_grid;
                    job.moisture_grid = pre.moisture_grid;
                    job.spawn_cells = pre.cells;
                    job.region_index_raster = vec![0u32; job.height_grid.len()];
                    job.tiling_compute_rx = None;
                    job.step = WorldGenPipelineStep::TilingSpawn;
                    job.next_tile_row = 0;
                    progress.label = "Terrain (spawning tiles)…".to_string();
                    progress.fraction = 0.45;
                }
                Err(TryRecvError::Empty) => {
                    progress.label = "Terrain (sampling, parallel CPU)…".to_string();
                    progress.fraction = 0.25;
                    return;
                }
                Err(TryRecvError::Disconnected) => {
                    error!("Terrain sampling worker thread disconnected");
                    slot.0 = None;
                    progress.running = false;
                    progress.label.clear();
                    progress.fraction = 0.0;
                    return;
                }
            }
        }
    }

    // Start new job
    if slot.0.is_none() {
        let Some(phase) = pending.0.take() else {
            return;
        };

        let run_params = effective_params_for_phase(&params, phase);
        let tiles = (run_params.width as u64).saturating_mul(run_params.height as u64);

        if phase == WorldGenPhase::Full
            && (tiles > MAX_WORLD_GEN_TILES
                || run_params.width > MAX_WORLD_GEN_AXIS
                || run_params.height > MAX_WORLD_GEN_AXIS)
        {
            error!(
                "Full world generation aborted: {}×{} ({} tiles) exceeds cap {}×{} ({} tiles). Lower width/height under World Generator.",
                run_params.width,
                run_params.height,
                tiles,
                MAX_WORLD_GEN_AXIS,
                MAX_WORLD_GEN_AXIS,
                MAX_WORLD_GEN_TILES
            );
            progress.running = false;
            progress.label.clear();
            progress.fraction = 0.0;
            return;
        }

        #[cfg(debug_assertions)]
        debug!(
            target: "world_gen",
            "Starting {phase:?} pass: logical {}×{} (requested {}×{}), seed {}",
            run_params.width,
            run_params.height,
            params.width,
            params.height,
            run_params.seed
        );

        despawn_generated_world_entities(&mut commands, &world_roots);

        info!(
            "Generating {phase:?} world with seed: {} ({}×{})",
            run_params.seed, run_params.width, run_params.height
        );

        let wall_start = Instant::now();
        let t_regions = Instant::now();
        let mut rng = StdRng::seed_from_u64(run_params.seed);
        let ns = &run_params.noise_sampling;
        let kernels = build_world_noise_kernels(&run_params, ns);
        let regions = generate_regions(&run_params, &mut rng);
        let regions_ms = t_regions.elapsed().as_secs_f64() * 1000.0;

        let world_entity = commands
            .spawn((WorldMarker, Name::new("World")))
            .id();

        let mut geo_regions = Vec::new();
        let mut region_entities = Vec::new();

        for (region_index, region_points) in regions.iter().enumerate() {
            let mut center = Vec2::ZERO;
            for point in region_points {
                center += point.position;
            }
            center /= region_points.len() as f32;

            let mut geo_region = GeoRegion::new();
            geo_region.center = center;

            let region_entity = commands
                .spawn((
                    RegionMarker,
                    Transform::from_translation(Vec3::new(center.x, 0.0, center.y)),
                    Name::new(format!("Region {}", region_index)),
                ))
                .id();

            region_entities.push(region_entity);
            geo_regions.push(geo_region);
            commands.entity(world_entity).add_children(&[region_entity]);
        }

        let w = run_params.width as usize;
        let grid_len = w.saturating_mul(run_params.height as usize).max(1);
        let tiling_started = Instant::now();

        let run_params_thread = run_params.clone();
        let regions_thread = regions.clone();
        let kernels_thread = kernels;
        let height_thread = run_params.height;
        let (tx, rx) = unbounded();
        std::thread::spawn(move || {
            let pre = compute_tiling_parallel(
                run_params_thread,
                regions_thread,
                kernels_thread,
                w,
                height_thread,
            );
            let _ = tx.send(pre);
        });

        slot.0 = Some(WorldGenActive {
            request_phase: phase,
            run_params,
            region_entities,
            geo_regions,
            w,
            height_grid: Vec::new(),
            moisture_grid: Vec::new(),
            spawn_cells: Vec::new(),
            tile_lookup: HashMap::with_capacity(grid_len.max(256)),
            next_tile_row: 0,
            step: WorldGenPipelineStep::TilingComputePending,
            biome_counts: HashMap::new(),
            timing: WorldGenRunTiming::from_parts(wall_start, regions_ms, tiling_started),
            tiling_compute_rx: Some(rx),
            hydro_rx: None,
            hydro_queued_at: None,
            region_index_raster: Vec::new(),
        });

        progress.running = true;
        progress.label = "Terrain (sampling, parallel CPU)…".to_string();
        progress.fraction = 0.1;
    }

    let Some(job) = slot.0.as_mut() else {
        return;
    };

    if !matches!(job.step, WorldGenPipelineStep::TilingSpawn) {
        return;
    }

    let h = job.run_params.height;
    let y_end = (job.next_tile_row + WORLD_GEN_TILE_ROWS_PER_TICK).min(h);
    let w = job.w;
    debug_assert_eq!(
        job.spawn_cells.len(),
        w * h as usize,
        "spawn_cells must match raster size"
    );

    for y in job.next_tile_row..y_end {
        let row_off = y as usize * w;
        for x in 0..job.run_params.width {
            let idx = row_off + x as usize;
            let cell = job.spawn_cells[idx];
            let height_value = job.height_grid[idx];
            let position = Vec2::new(x as f32, y as f32);

            let fam = default_terrain_families();
            let biome_label = fam
                .def(cell.terrain_family)
                .map(|d| d.name.as_str())
                .unwrap_or("unknown_family");
            *job
                .biome_counts
                .entry(biome_label.to_string())
                .or_insert(0) += 1;

            if idx < job.region_index_raster.len() {
                job.region_index_raster[idx] = cell.region_index as u32;
            }

            let tile_entity = commands
                .spawn((
                    TileMarker,
                    TileRegionIndex(cell.region_index as u32),
                    Transform::from_translation(Vec3::new(
                        x as f32,
                        height_value * 20.0,
                        y as f32,
                    )),
                    Height(height_value),
                    Moisture(cell.moisture),
                    Temperature(cell.temperature),
                    TerrainType(cell.terrain_family),
                    cell.strategic_kind,
                    Name::new(format!("Tile ({}, {})", x, y)),
                ))
                .id();

            commands
                .entity(job.region_entities[cell.region_index])
                .add_children(&[tile_entity]);
            job.geo_regions[cell.region_index].add_tile(position, tile_entity);

            job.tile_lookup.insert((x, y), tile_entity);
        }
    }

    job.next_tile_row = y_end;
    progress.fraction = if y_end >= h {
        0.9
    } else {
        y_end as f32 / h as f32 * 0.9
    };
    progress.label = format!("Terrain (spawning tiles)… {y_end} / {h}");

    if y_end < h {
        return;
    }

    job.spawn_cells = Vec::new();

    job.timing.tiling_ms = job
        .timing
        .tiling_started
        .elapsed()
        .as_secs_f64()
        * 1000.0;

    if job.run_params.river_count > 0 || job.run_params.lake_count > 0 {
        let (tx, rx) = unbounded();
        let grid = job.height_grid.clone();
        let moist = job.moisture_grid.clone();
        let gw = job.run_params.width;
        let gh = job.run_params.height;
        let rc = job.run_params.river_count;
        let lc = job.run_params.lake_count;
        let tuning = job.run_params.biome_tuning.clone();
        std::thread::spawn(move || {
            let hp = HydrologyParams::from_biome_tuning(&tuning);
            let r = compute_hydrology_world(gw, gh, &grid, Some(moist.as_slice()), &hp, rc, lc);
            let _ = tx.send(r);
        });
        job.hydro_rx = Some(rx);
        job.hydro_queued_at = Some(Instant::now());
        job.step = WorldGenPipelineStep::HydrologyPending;
        progress.label = "Rivers / lakes (queued to CPU thread)…".to_string();
        progress.fraction = 0.9;
        return;
    }

    let job = slot.0.take().expect("tiling done");
    finalize_world_gen_job(
        &mut commands,
        job,
        None,
        &mut last_debug,
        &mut completed,
        &mut progress,
    );
}

// Generate regions using the specified method
fn generate_regions(params: &WorldGenParams, rng: &mut StdRng) -> Vec<Vec<VoronoiSite>> {
    match params.region_method {
        RegionMethod::Regular => voronoi_diagram_generation(
            params.num_regions, 
            params.width, 
            params.height
        ),
        RegionMethod::Manhattan => manhattan_voronoi_diagram_generation(
            params.num_regions, 
            params.width, 
            params.height
        ),
        RegionMethod::Centroidal => centroidal_voronoi_diagram_generation(
            params.num_regions, 
            params.width, 
            params.height, 
            params.region_iterations
        ),
        RegionMethod::Weighted => {
            // Generate random weights
            let weights = (0..params.num_regions)
                .map(|_| rng.gen_range(0.0..10.0))
                .collect::<Vec<f32>>();
                
            additively_weighted_voronoi_diagram_generation(
                params.num_regions, 
                params.width, 
                params.height,
                Some(weights)
            )
        },
        RegionMethod::Power => {
            // Generate random weights
            let weights = (0..params.num_regions)
                .map(|_| rng.gen_range(0.0..5.0))
                .collect::<Vec<f32>>();
                
            power_voronoi_diagram_generation(
                params.num_regions, 
                params.width, 
                params.height,
                Some(weights)
            )
        },
        RegionMethod::Circular => circular_voronoi_diagram_generation(
            params.num_regions, 
            params.width, 
            params.height,
            2.0
        ),
    }
}

fn region_entity_for_tile(
    geo_regions: &[GeoRegion],
    region_entities: &[Entity],
    x: u32,
    y: u32,
) -> Option<Entity> {
    let key = (x as usize, y as usize);
    let idx = geo_regions.iter().position(|r| r.tiles.contains_key(&key))?;
    Some(region_entities[idx])
}

fn apply_shallow_water_visual(
    commands: &mut Commands,
    tile: Entity,
    gx: f32,
    gy: f32,
    water_height: f32,
) {
    let shallow = default_terrain_families()
        .id("ShallowWater")
        .expect("terrain family registry must define ShallowWater");
    commands.entity(tile).insert((
        TerrainType(shallow),
        Height(water_height),
        Moisture(0.95),
        Transform::from_xyz(gx, water_height * 20.0, gy),
    ));
}

fn spawn_hydrology_rivers(
    commands: &mut Commands,
    params: &WorldGenParams,
    tile_lookup: &HashMap<(u32, u32), Entity>,
    hydro: &HydrologyResult,
) {
    let water_line = params.biome_tuning.shallow_water_height_max;
    let river_depth = (water_line * 0.92).clamp(0.02, 0.98);
    for (river_index, path) in hydro.rivers.iter().enumerate() {
        let river_entity = commands
            .spawn((RiverMarker, Name::new(format!("River {}", river_index))))
            .id();
        for &(tx, ty) in path {
            let Some(&tile_e) = tile_lookup.get(&(tx, ty)) else {
                continue;
            };
            apply_shallow_water_visual(
                commands,
                tile_e,
                tx as f32,
                ty as f32,
                river_depth,
            );
            commands.entity(tile_e).insert(ChildOf(river_entity));
        }
    }
}

fn spawn_hydrology_lakes(
    commands: &mut Commands,
    params: &WorldGenParams,
    tile_lookup: &HashMap<(u32, u32), Entity>,
    geo_regions: &[GeoRegion],
    region_entities: &[Entity],
    hydro: &HydrologyResult,
) {
    let tuning = &params.biome_tuning;
    let lake_depth = ((tuning.deep_water_height_max + tuning.shallow_water_height_max) * 0.5)
        .clamp(0.02, 0.98);

    for (lake_index, region) in hydro.lakes.iter().enumerate() {
        if region.cells.is_empty() {
            continue;
        }
        let (cx, cy) = region.cells[region.cells.len() / 2];
        let Some(region_parent) = region_entity_for_tile(geo_regions, region_entities, cx, cy)
        else {
            continue;
        };
        let lake_entity = commands
            .spawn((
                LakeMarker,
                Name::new(format!("Lake {}", lake_index)),
                ChildOf(region_parent),
            ))
            .id();
        for &(lx, ly) in &region.cells {
            let Some(&tile_e) = tile_lookup.get(&(lx, ly)) else {
                continue;
            };
            apply_shallow_water_visual(
                commands,
                tile_e,
                lx as f32,
                ly as f32,
                lake_depth,
            );
            commands.entity(tile_e).insert(ChildOf(lake_entity));
        }
    }
}

// Message to trigger world generation (buffered; processed before `generate_world` in chain).
#[derive(Message, Clone)]
pub struct GenerateWorldEvent {
    pub params: WorldGenParams,
    pub phase: WorldGenPhase,
}

/// When `Some`, `generate_world` will run once for that phase and then clear.
#[derive(Resource, Default)]
struct WorldGenPending(Option<WorldGenPhase>);

#[derive(Message)]
struct WorldGenCompletedEvent(WorldGenPhase);

fn world_gen_apply_completion(
    mut events: MessageReader<WorldGenCompletedEvent>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
) {
    for WorldGenCompletedEvent(phase) in events.read() {
        match phase {
            WorldGenPhase::Preview => {
                NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::PreviewReady);
            }
            WorldGenPhase::Full => {
                NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::FullReady);
            }
        }
    }
}

/// Default path for optional JSON overlay (`noise_sampling` + `biome_tuning`).
pub const WORLD_GEN_TUNING_JSON_PATH: &str = "assets/config/world_gen_tuning.json";

// Plugin to register world generation systems
pub struct WorldGeneratorPlugin;

impl Plugin for WorldGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenParams>()
            .init_resource::<WorldGenPending>()
            .init_resource::<WorldGenJobSlot>()
            .init_resource::<WorldGenProgress>()
            .init_resource::<WorldGenLastDebugReport>()
            .init_state::<WorldGenFlowState>()
            .add_message::<GenerateWorldEvent>()
            .add_message::<WorldGenCompletedEvent>()
            .add_systems(Startup, merge_world_gen_tuning_from_json)
            .add_systems(
                Update,
                (
                    apply_generate_world_request,
                    world_gen_pipeline_tick,
                    world_gen_apply_completion,
                )
                    .chain(),
            );
    }
}

fn merge_world_gen_tuning_from_json(mut params: ResMut<WorldGenParams>) {
    if let Ok(Some(overlay)) = tuning_io::load_overlay(WORLD_GEN_TUNING_JSON_PATH) {
        if let Some(n) = overlay.noise_sampling {
            params.noise_sampling = n;
        }
        if let Some(b) = overlay.biome_tuning {
            params.biome_tuning = b;
        }
    }
}

fn apply_generate_world_request(
    mut events: MessageReader<GenerateWorldEvent>,
    mut params: ResMut<WorldGenParams>,
    mut pending: ResMut<WorldGenPending>,
    flow: Res<State<WorldGenFlowState>>,
    job_slot: Res<WorldGenJobSlot>,
) {
    for ev in events.read() {
        if job_slot.0.is_some() {
            warn!("World generation already running; duplicate request ignored.");
            continue;
        }
        let allow = match (*flow.get(), ev.phase) {
            (WorldGenFlowState::Idle, _) => {
                warn!(
                    "Ignored {:?} world generation: use Main Menu → New World. \
                     (Debug builds: F8 from simulation opens the tool into the setup flow.)",
                    ev.phase
                );
                false
            }
            (WorldGenFlowState::LoadingSave, _) => {
                warn!("Ignored world generation while save load is active.");
                false
            }
            (_, WorldGenPhase::Full) if *flow.get() != WorldGenFlowState::PreviewReady => {
                warn!(
                    "Full generation requires a preview first: click “Generate preview”, then “Generate full world”."
                );
                false
            }
            (WorldGenFlowState::FullReady, _) => {
                warn!(
                    "Ignored generation: confirm “Enter world” or discard the current world first."
                );
                false
            }
            _ => true,
        };
        if !allow {
            continue;
        }
        params.clone_from(&ev.params);
        pending.0 = Some(ev.phase);
    }
}

#[cfg(test)]
mod hydrology_spawning_tests {
    use crate::terrain::generation::hydrology::{
        compute_hydrology_rect, compute_hydrology_world, HydrologyParams,
    };

    #[test]
    fn generate_rivers_uses_compute_hydrology() {
        let w = 16u32;
        let h = 16u32;
        let mut dem = vec![0.45f32; (w * h) as usize];
        for x in 0..w {
            dem[x as usize] = 0.95;
        }
        let p = HydrologyParams::default();
        let r = compute_hydrology_world(w, h, &dem, None, &p, 2, 1);
        assert!(r.rivers.len() <= 2);
        assert!(r.lakes.len() <= 1);
    }

    #[test]
    fn enhanced_generator_consumes_p4_hydrology_tags() {
        let w = 12u32;
        let h = 12u32;
        let mut dem = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let fx = x as f32 / (w.saturating_sub(1).max(1) as f32);
                let fy = y as f32 / (h.saturating_sub(1).max(1) as f32);
                dem.push(fx * 0.6 + fy * 0.4);
            }
        }
        let p = HydrologyParams::default();
        let rect = compute_hydrology_rect(w, h, &dem, &p, 3, None);
        let world = compute_hydrology_world(w, h, &dem, None, &p, 3, 0);
        assert_eq!(world.accumulation, rect.accumulation);
        assert_eq!(world.river_mask, rect.river_mask);
        assert_eq!(world.lake_mask, rect.lake_mask);
        assert!(world.lakes.is_empty());
    }
}