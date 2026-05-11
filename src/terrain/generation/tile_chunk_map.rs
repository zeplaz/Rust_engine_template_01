//! Deterministic **world tile** ↔ **chunk coord** mapping for U7 / previews / editor bridge.
//!
//! World tile `(tx, tz)` matches map editor / [`crate::terrain::generation::world_generator_enhanced`] spawn:
//! column → `x`, row → `z`. Chunk `(cx, cz)` contains tiles where
//! `tx ∈ [cx * cw, (cx+1)*cw)` and `tz ∈ [cz * ch, (cz+1)*ch)`.

use bevy::prelude::{IVec2, UVec2};
use std::collections::HashSet;

#[inline]
pub fn tile_to_chunk_coord(tile_x: u32, tile_z: u32, cells_per_chunk: UVec2) -> IVec2 {
    let cw = cells_per_chunk.x.max(1);
    let ch = cells_per_chunk.y.max(1);
    IVec2::new(
        (tile_x / cw) as i32,
        (tile_z / ch) as i32,
    )
}

/// Inclusive tile AABB for a round brush in tile space (disk bounds square).
#[inline]
pub fn brush_tile_inclusive_bounds(center_tx: u32, center_tz: u32, radius_tiles: f32) -> (UVec2, UVec2) {
    let r = radius_tiles.ceil().max(1.0) as i64;
    let ix = center_tx as i64;
    let iz = center_tz as i64;
    let min_tx = (ix - r).max(0) as u32;
    let min_tz = (iz - r).max(0) as u32;
    let max_tx = (ix + r).max(0) as u32;
    let max_tz = (iz + r).max(0) as u32;
    (UVec2::new(min_tx, min_tz), UVec2::new(max_tx, max_tz))
}
pub fn tile_rect_to_chunk_coords(
    min_tx: u32,
    min_tz: u32,
    max_tx: u32,
    max_tz: u32,
    cells_per_chunk: UVec2,
) -> Vec<IVec2> {
    let cw = cells_per_chunk.x.max(1);
    let ch = cells_per_chunk.y.max(1);
    let min_cx = min_tx / cw;
    let min_cz = min_tz / ch;
    let max_cx = max_tx / cw;
    let max_cz = max_tz / ch;
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for cz in min_cz..=max_cz {
        for cx in min_cx..=max_cx {
            let v = IVec2::new(cx as i32, cz as i32);
            if seen.insert(v) {
                out.push(v);
            }
        }
    }
    out.sort_by(|a, b| (a.y, a.x).cmp(&(b.y, b.x)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_rect_covers_multiple_chunks() {
        let cells = UVec2::new(8, 8);
        let v = tile_rect_to_chunk_coords(0, 0, 9, 9, cells);
        assert!(v.contains(&IVec2::ZERO));
        assert!(v.contains(&IVec2::new(1, 0)));
        assert!(v.contains(&IVec2::new(0, 1)));
        assert!(v.contains(&IVec2::new(1, 1)));
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn single_tile_maps_to_chunk() {
        let cells = UVec2::new(32, 32);
        assert_eq!(
            tile_to_chunk_coord(31, 31, cells),
            IVec2::ZERO
        );
        assert_eq!(
            tile_to_chunk_coord(32, 0, cells),
            IVec2::new(1, 0)
        );
    }
}
