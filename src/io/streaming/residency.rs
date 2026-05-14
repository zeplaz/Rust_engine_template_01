//! Chunk residency helpers — Moore neighborhood rings for Wave C ghost bands.

use std::collections::HashMap;

use bevy::prelude::{IVec2, Resource};

/// Chunk coords in a square window around `focus` (inclusive), row-major order.
#[must_use]
pub fn chunk_window_coords(focus: IVec2, radius: i32) -> Vec<IVec2> {
    if radius < 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for y in (focus.y - radius)..=(focus.y + radius) {
        for x in (focus.x - radius)..=(focus.x + radius) {
            out.push(IVec2::new(x, y));
        }
    }
    out
}

/// Center chunk plus its eight Moore neighbors (ghost-band seed set).
#[must_use]
pub fn ghost_band_seed_coords(center: IVec2) -> Vec<IVec2> {
    chunk_window_coords(center, 1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkResidencyRole {
    Core,
    GhostBand,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkResidencyEntry {
    pub coord: IVec2,
    pub role: ChunkResidencyRole,
    pub orb_priority: u8,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct ChunkResidencyTable {
    pub entries: HashMap<IVec2, ChunkResidencyEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ghost_band_seed_is_nine_chunk_moore_window() {
        let coords = ghost_band_seed_coords(IVec2::new(2, 3));
        assert_eq!(coords.len(), 9);
        assert!(coords.contains(&IVec2::new(2, 3)));
        assert!(coords.contains(&IVec2::new(3, 4)));
    }
}
