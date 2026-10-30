//! Transient simulation state — mud, weather, damage, congestion, etc.
//!
//! **Does not** replace fact [`TagSet`](super::material::TagSet) or static [`MaterialDef`](super::material::MaterialDef).
//! **PR4-2:** HashMap writes dual-write into [`WorldChunkState::dynamic`](crate::substrate::types::WorldChunkState)
//! when [`WorldSubstrateRegistry`](crate::substrate::registry::WorldSubstrateRegistry) is present.

use std::collections::HashMap;

use bevy::math::IVec2;
use bevy::prelude::{Query, Res, ResMut, Resource, Time};

use crate::substrate::{DualWriteShimState, WorldSubstrateRegistry};
use crate::substrate::{mirror_overlay_cell_to_slab, ChunkKey};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Chunk coordinate + linear index in row-major order (same as [`ChunkCellMatrix::idx`](super::generation::cell_matrix::ChunkCellMatrix)).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkCellKey {
    pub chunk: IVec2,
    pub cell_index: u32,
}

impl ChunkCellKey {
    #[inline]
    pub fn new(chunk: IVec2, cell_index: u32) -> Self {
        Self { chunk, cell_index }
    }
}

/// Number of chunk slices the dynamic-overlay writers are spread over.
///
/// The overlay is a slow-moving field (mud / snow integrate over tens of seconds), but each full
/// pass touches every cell of every resident chunk (hash-map churn + slab mirror) — tens of ms.
/// Each frame now processes one chunk slice (`1 / DYNAMIC_OVERLAY_SLICES` of the map) with the
/// `dt` accumulated since that slice was last visited, so the cost is flat instead of spiking.
/// 32 slices (was 8): at 8 the three passes still cost ~7 ms/frame combined on a 320x320 world;
/// a slice is revisited about once per second, which is far below the field's own timescale.
pub const DYNAMIC_OVERLAY_SLICES: usize = 32;

/// Chunk -> slice index (interleaved so neighbouring chunks land on different frames).
#[inline]
fn overlay_slice_of(chunk: IVec2) -> usize {
    let n = DYNAMIC_OVERLAY_SLICES as i32;
    (chunk.x.rem_euclid(n) + 3 * chunk.y.rem_euclid(n)).rem_euclid(n) as usize
}

/// Sparse transient scalars. Hot paths may later use chunk-partitioned `Vec` slabs instead.
#[derive(Resource, Debug)]
pub struct DynamicTerrainOverlay {
    pub mud: HashMap<ChunkCellKey, f32>,
    pub snow: HashMap<ChunkCellKey, f32>,
    pub danger: HashMap<ChunkCellKey, f32>,
    pub congestion: HashMap<ChunkCellKey, f32>,
    /// Bumped whenever a writer pass mutates the maps (lets slab migration skip idle frames).
    pub generation: u64,
    /// Accumulated sim dt per writer pass (`[accumulate, weather, decay]`) and slice.
    slice_acc: [[f32; DYNAMIC_OVERLAY_SLICES]; 3],
    /// Next slice each writer pass visits.
    slice_cursor: [usize; 3],
}

impl Default for DynamicTerrainOverlay {
    fn default() -> Self {
        Self {
            mud: HashMap::new(),
            snow: HashMap::new(),
            danger: HashMap::new(),
            congestion: HashMap::new(),
            generation: 0,
            slice_acc: [[0.0; DYNAMIC_OVERLAY_SLICES]; 3],
            // Offset so the three passes do not all hit the same slice on the same frame.
            slice_cursor: [0, 3, 5],
        }
    }
}

impl DynamicTerrainOverlay {
    /// Advance writer pass `slot` by `dt`; returns `(slice, batched_dt)` to process this frame.
    fn begin_slice_pass(&mut self, slot: usize, dt: f32) -> (usize, f32) {
        for a in self.slice_acc[slot].iter_mut() {
            *a = (*a + dt).min(4.0);
        }
        let slice = self.slice_cursor[slot];
        self.slice_cursor[slot] = (slice + 1) % DYNAMIC_OVERLAY_SLICES;
        let due = std::mem::take(&mut self.slice_acc[slot][slice]);
        self.generation = self.generation.wrapping_add(1);
        (slice, due)
    }
}

#[inline]
fn mirror_cell_to_slab(
    substrate: Option<&mut WorldSubstrateRegistry>,
    key: ChunkCellKey,
    overlay: &DynamicTerrainOverlay,
) {
    let Some(reg) = substrate else {
        return;
    };
    let mud = overlay.mud.get(&key).copied().unwrap_or(0.0);
    let snow = overlay.snow.get(&key).copied().unwrap_or(0.0);
    let danger = overlay.danger.get(&key).copied().unwrap_or(0.0);
    let congestion = overlay.congestion.get(&key).copied().unwrap_or(0.0);
    mirror_overlay_cell_to_slab(reg, &key, mud, snow, danger, congestion);
}

/// Slab-first read when PR-2 dual-write shim is enabled and chunk is resident.
#[must_use]
pub fn overlay_mud_at(
    overlay: &DynamicTerrainOverlay,
    substrate: Option<&WorldSubstrateRegistry>,
    dual: Option<&DualWriteShimState>,
    key: &ChunkCellKey,
) -> f32 {
    if dual.is_some_and(|s| s.enabled) {
        if let Some(reg) = substrate {
            let slab_key = ChunkKey::from(key.chunk);
            if reg.chunks.is_resident(slab_key) {
                if let Some(state) = reg.chunks.get(slab_key) {
                    let i = key.cell_index as usize;
                    if i < state.dynamic.mud.len() {
                        return state.dynamic.mud[i];
                    }
                }
            }
        }
    }
    overlay.mud.get(key).copied().unwrap_or(0.0)
}

/// Exponential decay so overlay maps stay sparse and bounded during long editor sessions.
pub fn decay_dynamic_terrain_overlay(
    time: Option<Res<Time>>,
    mut overlay: ResMut<DynamicTerrainOverlay>,
    mut substrate: Option<ResMut<WorldSubstrateRegistry>>,
) {
    let Some(time) = time else {
        return;
    };
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let (slice, dt) = overlay.begin_slice_pass(2, dt);
    if dt <= 0.0 {
        return;
    }
    let k = (-0.12 * dt).exp();
    fn decay_slice(map: &mut HashMap<ChunkCellKey, f32>, k: f32, slice: usize) {
        for (key, v) in map.iter_mut() {
            if overlay_slice_of(key.chunk) == slice {
                *v *= k;
            }
        }
        map.retain(|key, v| overlay_slice_of(key.chunk) != slice || *v > 1e-5);
    }
    decay_slice(&mut overlay.mud, k, slice);
    decay_slice(&mut overlay.snow, k, slice);
    decay_slice(&mut overlay.danger, k, slice);
    decay_slice(&mut overlay.congestion, k, slice);
    if substrate.is_some() {
        // Mirror each distinct cell of this slice once (a cell commonly lives in several maps).
        let mut keys: Vec<ChunkCellKey> = overlay
            .mud
            .keys()
            .chain(overlay.snow.keys())
            .chain(overlay.danger.keys())
            .chain(overlay.congestion.keys())
            .filter(|key| overlay_slice_of(key.chunk) == slice)
            .copied()
            .collect();
        keys.sort_unstable_by_key(|k| (k.chunk.x, k.chunk.y, k.cell_index));
        keys.dedup();
        for key in keys {
            mirror_cell_to_slab(substrate.as_deref_mut(), key, &overlay);
        }
    }
}

/// Prototype **writer:** pushes mud/snow from per-cell moisture + temperature. Does not mutate tags or materials.
pub fn stub_accumulate_overlay_from_chunk_fields(
    time: Option<Res<Time>>,
    mut overlay: ResMut<DynamicTerrainOverlay>,
    mut substrate: Option<ResMut<WorldSubstrateRegistry>>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
) {
    let Some(time) = time else {
        return;
    };
    let dt = time.delta_secs().clamp(0.0, 0.1);
    if dt <= 0.0 {
        return;
    }
    let (slice, dt) = overlay.begin_slice_pass(0, dt);
    if dt <= 0.0 {
        return;
    }
    // Per-step caps below were tuned for ~60 Hz steps; scale with the batched step so the
    // per-second rate ceiling is unchanged.
    let step_scale = (dt / (1.0 / 60.0)).max(1.0);
    for (chunk, matrix) in chunks.iter() {
        if overlay_slice_of(chunk.coord) != slice {
            continue;
        }
        let w = matrix.size.x;
        let h = matrix.size.y;
        let expected = (w * h) as usize;
        if matrix.moisture.len() != expected || matrix.temperature.len() != expected {
            continue;
        }
        for dz in 0..h {
            for dx in 0..w {
                let i = matrix.idx(dx, dz);
                let key = ChunkCellKey::new(chunk.coord, i as u32);

                let snow_add = ((1.0 - matrix.temperature[i]).max(0.0) * matrix.moisture[i] * 0.025 * dt)
                    .min(0.008 * step_scale);
                if snow_add > 1e-7 {
                    let e = overlay.snow.entry(key).or_insert(0.0);
                    *e = (*e + snow_add).min(2.0);
                }
                let mud_add =
                    (matrix.moisture[i] * matrix.moisture[i] * (matrix.temperature[i] + 0.15).max(0.0)
                        * 0.02
                        * dt)
                        .min(0.008 * step_scale);
                if mud_add > 1e-7 {
                    let e = overlay.mud.entry(key).or_insert(0.0);
                    *e = (*e + mud_add).min(2.0);
                }
                if substrate.is_some() && (snow_add > 1e-7 || mud_add > 1e-7) {
                    mirror_cell_to_slab(substrate.as_deref_mut(), key, &overlay);
                }
            }
        }
    }
}

/// Mud/snow overlay reads [`ChunkWeather`] (rain/snow depth) after field accumulation (S2 → dynamic overlay).
pub fn apply_chunk_weather_to_dynamic_overlay(
    time: Option<Res<Time>>,
    mut overlay: ResMut<DynamicTerrainOverlay>,
    mut substrate: Option<ResMut<WorldSubstrateRegistry>>,
    chunks: Query<(&Chunk, &ChunkCellMatrix, Option<&crate::systems::weather::ChunkWeather>)>,
) {
    let Some(time) = time else {
        return;
    };
    let dt = time.delta_secs().clamp(0.0, 0.1);
    if dt <= 0.0 {
        return;
    }
    let (slice, dt) = overlay.begin_slice_pass(1, dt);
    if dt <= 0.0 {
        return;
    }
    for (chunk, matrix, w_opt) in chunks.iter() {
        if overlay_slice_of(chunk.coord) != slice {
            continue;
        }
        let Some(w) = w_opt else {
            continue;
        };
        let dim_x = matrix.size.x;
        let dim_y = matrix.size.y;
        let expected = (dim_x * dim_y) as usize;
        if matrix.moisture.len() != expected {
            continue;
        }
        let boost_mud = 1.0 + 0.55 * w.rain_intensity + 0.2 * w.snow_depth.min(1.0);
        let boost_snow = 1.0 + 0.45 * w.snow_depth + 0.12 * w.rain_intensity;
        // Boosts were tuned as per-60 Hz-step multipliers; compound them over the batched step.
        let steps = (dt * 60.0).clamp(1.0, 120.0);
        let boost_mud_step = boost_mud.powf(steps);
        let boost_snow_step = boost_snow.powf(steps);
        let rain_mud = w.rain_intensity * 0.014 * dt;
        let fall_snow = w.snow_depth * 0.011 * dt;
        for y in 0..dim_y {
            for x in 0..dim_x {
                let i = matrix.idx(x, y);
                let key = ChunkCellKey::new(chunk.coord, i as u32);
                let mut touched = false;
                if boost_mud > 1.001 {
                    if let Some(v) = overlay.mud.get_mut(&key) {
                        *v = (*v * boost_mud_step).min(2.0);
                        touched = true;
                    }
                }
                if boost_snow > 1.001 {
                    if let Some(v) = overlay.snow.get_mut(&key) {
                        *v = (*v * boost_snow_step).min(2.0);
                        touched = true;
                    }
                }
                if rain_mud > 1e-7 {
                    let e = overlay.mud.entry(key).or_insert(0.0);
                    *e = (*e + rain_mud).min(2.0);
                    touched = true;
                }
                if fall_snow > 1e-7 {
                    let e = overlay.snow.entry(key).or_insert(0.0);
                    *e = (*e + fall_snow).min(2.0);
                    touched = true;
                }
                if touched && substrate.is_some() {
                    mirror_cell_to_slab(substrate.as_deref_mut(), key, &overlay);
                }
            }
        }
    }
}
