//! Stitched chunk/slab lookups for world-tile coordinates.

use crate::terrain::ChunkCellKey;
use crate::terrain::material::TagSet;
use bevy::math::{IVec2, UVec2};

/// Row-major cell layer at world tile `(tx, ty)` from overlapping chunk slabs `(coord, size, slice)`.
pub fn chunk_cell_layer_at_world_tile<T: Copy>(
    tx: u32,
    ty: u32,
    chunks: &[(IVec2, UVec2, &[T])],
) -> Option<T> {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size, layer) in chunks {
        let sx = size.x as i32;
        let sy = size.y as i32;
        let wx0 = coord.x * sx;
        let wy0 = coord.y * sy;
        if tx_i < wx0 || ty_i < wy0 {
            continue;
        }
        let lx = tx_i - wx0;
        let ly = ty_i - wy0;
        if lx < 0 || ly < 0 || lx >= sx || ly >= sy {
            continue;
        }
        let idx = (ly * sx + lx) as usize;
        if idx < layer.len() {
            return Some(layer[idx]);
        }
    }
    None
}

/// [`ChunkCellKey`] for world tile `(tx, ty)` when it falls on a listed chunk slab.
pub fn chunk_cell_key_for_world_tile(
    tx: u32,
    ty: u32,
    chunks: &[(IVec2, UVec2)],
) -> Option<ChunkCellKey> {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size) in chunks {
        let sx = size.x as i32;
        let sy = size.y as i32;
        let wx0 = coord.x * sx;
        let wy0 = coord.y * sy;
        if tx_i < wx0 || ty_i < wy0 {
            continue;
        }
        let lx = tx_i - wx0;
        let ly = ty_i - wy0;
        if lx < 0 || ly < 0 || lx >= sx || ly >= sy {
            continue;
        }
        let idx = (ly * sx + lx) as u32;
        return Some(ChunkCellKey::new(*coord, idx));
    }
    None
}

pub fn cell_tags_for_world_tile(
    tx: u32,
    ty: u32,
    chunks: &[(IVec2, UVec2, &[TagSet])],
) -> Option<TagSet> {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size, tags_vec) in chunks {
        let sx = size.x as i32;
        let sy = size.y as i32;
        let wx0 = coord.x * sx;
        let wy0 = coord.y * sy;
        if tx_i < wx0 || ty_i < wy0 {
            continue;
        }
        let lx = tx_i - wx0;
        let ly = ty_i - wy0;
        if lx < 0 || ly < 0 || lx >= sx || ly >= sy {
            continue;
        }
        let idx = (ly * sx + lx) as usize;
        if idx < tags_vec.len() {
            return Some(tags_vec[idx]);
        }
    }
    None
}

/// Chunk-derived stitched `slope_grade` at world tile `(tx, ty)` when present on a materialized chunk.
#[inline]
pub fn slope_grade_for_world_tile(
    tx: u32,
    ty: u32,
    chunks: &[(IVec2, UVec2, &[f32])],
) -> Option<f32> {
    chunk_cell_layer_at_world_tile(tx, ty, chunks)
}

/// Cardinal-neighbor max \|Δelevation\| in **world tile** space (stitched from chunk slabs + tile fallback).
pub fn slope_grade_from_world_elevation_neighbors(
    tx: u32,
    ty: u32,
    world_w: u32,
    world_h: u32,
    center_h: f32,
    elev_chunks: &[(IVec2, UVec2, &[f32])],
    tile_height_lut: Option<&std::collections::HashMap<(u32, u32), f32>>,
) -> f32 {
    let mut max_d = 0.0f32;
    for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1)] {
        let nx = tx as i32 + dx;
        let ny = ty as i32 + dy;
        let nh = if nx < 0 || ny < 0 || nx >= world_w as i32 || ny >= world_h as i32 {
            center_h
        } else {
            let nxu = nx as u32;
            let nyu = ny as u32;
            chunk_cell_layer_at_world_tile(nxu, nyu, elev_chunks)
                .or_else(|| tile_height_lut.and_then(|m| m.get(&(nxu, nyu)).copied()))
                .unwrap_or(center_h)
        };
        max_d = max_d.max((center_h - nh).abs());
    }
    max_d
}
