//! Per-chunk SoA cell grid — see `prompts/matrix/terrain_biome/runbook/u4_steps_v1.md` (U4-S01).

use bevy::prelude::{Component, UVec2};

use crate::terrain::biome::BiomeWeights;
use crate::terrain::material::TagSet;
use crate::terrain::family::{TerrainFamilyId, DEFAULT_TERRAIN_FAMILY_ID};

/// SoA storage for one chunk’s worth of terrain fields + tags + biome outputs (pass pipeline fills these).
#[derive(Component, Debug, Clone)]
pub struct ChunkCellMatrix {
    pub size: UVec2,
    pub elevation: Vec<f32>,
    pub moisture: Vec<f32>,
    pub temperature: Vec<f32>,
    pub tags: Vec<TagSet>,
    /// Classified terrain **family** id — from [`TerrainFamilyRegistry`](crate::terrain::family::TerrainFamilyRegistry).
    pub family: Vec<TerrainFamilyId>,
    pub weights: Vec<BiomeWeights>,
}

impl ChunkCellMatrix {
    pub fn new(size: UVec2) -> Self {
        let n = (size.x * size.y) as usize;
        Self {
            size,
            elevation: vec![0.0; n],
            moisture: vec![0.0; n],
            temperature: vec![0.0; n],
            tags: vec![TagSet::default(); n],
            family: vec![DEFAULT_TERRAIN_FAMILY_ID; n],
            weights: vec![BiomeWeights::default(); n],
        }
    }

    /// Row-major index: `x` in `0..size.x`, `y` in `0..size.y`.
    #[inline]
    pub fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.size.x + x) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_cell_matrix_alloc() {
        let size = UVec2::new(12, 7);
        let m = ChunkCellMatrix::new(size);
        let n = (size.x * size.y) as usize;
        assert_eq!(m.elevation.len(), n);
        assert_eq!(m.moisture.len(), n);
        assert_eq!(m.temperature.len(), n);
        assert_eq!(m.tags.len(), n);
        assert_eq!(m.family.len(), n);
        assert_eq!(m.weights.len(), n);
        assert_eq!(m.idx(3, 4), (4 * size.x + 3) as usize);
    }
}
