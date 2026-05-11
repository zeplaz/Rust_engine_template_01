//! Copy authored [`TileMarker`](super::world_generator_enhanced::TileMarker) components into live [`ChunkCellMatrix`]
//! for chunks touched by an editor commit — keeps U7 SOA aligned with the map grid before passes rerun.

use std::collections::HashSet;

use bevy::prelude::{IVec2, Query, Transform, UVec2, With};

use super::world_generator_enhanced::{
    Height, Moisture, Temperature, TerrainType, TileMarker, WorldGenParams,
};
use super::{Chunk, ChunkCellMatrix};
use crate::terrain::material::ChunkDependency;

/// For every tile entity in `affected_chunks`, write height / moisture / temperature / terrain family into the
/// matching chunk slab cell. Chunks not in the set are untouched.
pub fn sync_tile_markers_into_affected_chunk_matrices(
    affected_chunks: &HashSet<IVec2>,
    cells_per_chunk: UVec2,
    params: &WorldGenParams,
    tiles: &Query<(&Transform, &Height, &Moisture, &Temperature, &TerrainType), With<TileMarker>>,
    chunks: &mut Query<(&Chunk, &mut ChunkCellMatrix), With<ChunkDependency>>,
) {
    if affected_chunks.is_empty() {
        return;
    }
    let cw = cells_per_chunk.x.max(1);
    let ch = cells_per_chunk.y.max(1);
    for (tf, h, m, temp, terr) in tiles.iter() {
        let x = tf.translation.x.round() as i32;
        let z = tf.translation.z.round() as i32;
        if x < 0 || z < 0 {
            continue;
        }
        let tx = x as u32;
        let tz = z as u32;
        if tx >= params.width || tz >= params.height {
            continue;
        }
        let cc = IVec2::new((tx / cw) as i32, (tz / ch) as i32);
        if !affected_chunks.contains(&cc) {
            continue;
        }
        let base_tx = (cc.x.max(0) as u32).saturating_mul(cw);
        let base_tz = (cc.y.max(0) as u32).saturating_mul(ch);
        let lx = tx.saturating_sub(base_tx);
        let lz = tz.saturating_sub(base_tz);
        for (chunk, mut matrix) in chunks.iter_mut() {
            if chunk.coord != cc {
                continue;
            }
            if lx >= matrix.size.x || lz >= matrix.size.y {
                continue;
            }
            let idx = matrix.idx(lx, lz);
            matrix.elevation[idx] = h.0;
            matrix.moisture[idx] = m.0;
            matrix.temperature[idx] = temp.0;
            matrix.family[idx] = terr.0;
            break;
        }
    }
}
