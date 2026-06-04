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

/// Sparse transient scalars. Hot paths may later use chunk-partitioned `Vec` slabs instead.
#[derive(Resource, Debug, Default)]
pub struct DynamicTerrainOverlay {
    pub mud: HashMap<ChunkCellKey, f32>,
    pub snow: HashMap<ChunkCellKey, f32>,
    pub danger: HashMap<ChunkCellKey, f32>,
    pub congestion: HashMap<ChunkCellKey, f32>,
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
    let k = (-0.12 * dt).exp();
    fn decay_one(map: &mut HashMap<ChunkCellKey, f32>, k: f32) {
        for v in map.values_mut() {
            *v *= k;
        }
        map.retain(|_, v| *v > 1e-5);
    }
    decay_one(&mut overlay.mud, k);
    decay_one(&mut overlay.snow, k);
    decay_one(&mut overlay.danger, k);
    decay_one(&mut overlay.congestion, k);
    if substrate.is_some() {
        let keys: Vec<ChunkCellKey> = overlay
            .mud
            .keys()
            .chain(overlay.snow.keys())
            .chain(overlay.danger.keys())
            .chain(overlay.congestion.keys())
            .copied()
            .collect();
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
    for (chunk, matrix) in chunks.iter() {
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
                    .min(0.008);
                if snow_add > 1e-7 {
                    let e = overlay.snow.entry(key).or_insert(0.0);
                    *e = (*e + snow_add).min(2.0);
                }
                let mud_add =
                    (matrix.moisture[i] * matrix.moisture[i] * (matrix.temperature[i] + 0.15).max(0.0)
                        * 0.02
                        * dt)
                        .min(0.008);
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
    for (chunk, matrix, w_opt) in chunks.iter() {
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
        let rain_mud = w.rain_intensity * 0.014 * dt;
        let fall_snow = w.snow_depth * 0.011 * dt;
        for y in 0..dim_y {
            for x in 0..dim_x {
                let i = matrix.idx(x, y);
                let key = ChunkCellKey::new(chunk.coord, i as u32);
                let mut touched = false;
                if boost_mud > 1.001 {
                    if let Some(v) = overlay.mud.get_mut(&key) {
                        *v = (*v * boost_mud).min(2.0);
                        touched = true;
                    }
                }
                if boost_snow > 1.001 {
                    if let Some(v) = overlay.snow.get_mut(&key) {
                        *v = (*v * boost_snow).min(2.0);
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
