//! Transient simulation state — mud, weather, damage, congestion, etc.
//!
//! **Does not** replace fact [`TagSet`](super::material::TagSet) or static [`MaterialDef`](super::material::MaterialDef).
//! Persistence is optional per game mode. See `ontology/refactor_execution_plan_v1.md` (implementation tranche B).
//!
//! **Systems:** [`stub_accumulate_overlay_from_chunk_fields`] + [`decay_dynamic_terrain_overlay`] are prototype
//! writers registered with **`MaterialUnificationPlugin`**; replace with game-specific sim when ready.

use std::collections::HashMap;

use bevy::math::IVec2;
use bevy::prelude::{Query, Res, ResMut, Resource, Time};

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

/// Exponential decay so overlay maps stay sparse and bounded during long editor sessions.
pub fn decay_dynamic_terrain_overlay(
    time: Option<Res<Time>>,
    mut overlay: ResMut<DynamicTerrainOverlay>,
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
}

/// Prototype **writer:** pushes mud/snow from per-cell moisture + temperature. Does not mutate tags or materials.
pub fn stub_accumulate_overlay_from_chunk_fields(
    time: Option<Res<Time>>,
    mut overlay: ResMut<DynamicTerrainOverlay>,
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
        for y in 0..h {
            for x in 0..w {
                let i = matrix.idx(x, y);
                let moist = matrix.moisture[i];
                let temp = matrix.temperature[i];
                let key = ChunkCellKey::new(chunk.coord, i as u32);

                let snow_add = ((1.0 - temp).max(0.0) * moist * 0.025 * dt).min(0.008);
                if snow_add > 1e-7 {
                    let e = overlay.snow.entry(key).or_insert(0.0);
                    *e = (*e + snow_add).min(2.0);
                }
                let mud_add =
                    (moist * moist * (temp + 0.15).max(0.0) * 0.02 * dt).min(0.008);
                if mud_add > 1e-7 {
                    let e = overlay.mud.entry(key).or_insert(0.0);
                    *e = (*e + mud_add).min(2.0);
                }
            }
        }
    }
}
