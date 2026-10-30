//! Standing **surface water** vs **atmospheric moisture** for fire.
//!
//! - [`ChunkCellMatrix::moisture`] — humidity / rainfall / dryness proxy on **burnable** land.
//! - Open water (river, lake, ocean) — no combustion regardless of moisture; a dry climate does not burn
//!   water. Whether a cell is water is the per-cell water class ([`ChunkCellMatrix::water`]), decided once by
//!   world generation; this gate only reads it.

use bevy::prelude::*;

use crate::terrain::generation::ChunkCellMatrix;

/// Per-cell open-water test shared by overlay, ember, and scalar chunk fire.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SurfaceWaterFireGate;

impl SurfaceWaterFireGate {
    #[inline]
    pub fn cell_has_standing_water(&self, matrix: &ChunkCellMatrix, i: usize) -> bool {
        matrix.water.get(i).is_some_and(|c| c.is_open_water())
    }

    /// Dryness from humidity proxy only (land cells); not used on standing water.
    #[inline]
    pub fn atmospheric_dryness(moisture: f32) -> f32 {
        (0.42 - moisture).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::generation::WaterClass;
    use bevy::prelude::UVec2;

    #[test]
    fn standing_water_is_the_water_class_not_elevation_or_moisture() {
        let gate = SurfaceWaterFireGate;
        let mut matrix = ChunkCellMatrix::new(UVec2::new(4, 1));
        for i in 0..4 {
            matrix.elevation[i] = 0.0;
            matrix.moisture[i] = 0.05;
        }
        matrix.water[0] = WaterClass::Lake;
        matrix.water[1] = WaterClass::River;
        matrix.water[2] = WaterClass::Ocean;
        matrix.water[3] = WaterClass::Wetland;
        assert!(gate.cell_has_standing_water(&matrix, 0));
        assert!(gate.cell_has_standing_water(&matrix, 1));
        assert!(gate.cell_has_standing_water(&matrix, 2));
        assert!(!gate.cell_has_standing_water(&matrix, 3), "wetland is wet ground, not water");
        matrix.water[0] = WaterClass::Dry;
        matrix.elevation[0] = 0.0;
        assert!(!gate.cell_has_standing_water(&matrix, 0));
    }
}
