//! Standing **surface water** vs **atmospheric moisture** for fire.
//!
//! - [`ChunkCellMatrix::moisture`] â€” humidity / rainfall / dryness proxy on **burnable** land.
//! - Standing water â€” a cell under at least [`FIRE_WATER_DEPTH_M`] of live water cannot burn and
//!   blocks spread; a dry climate does not burn water, and a flooded forest does not burn. A cell
//!   the water left (dried river, drained flood) can burn again.
//!
//! The depth is the live map water field read through [`WaterTileProbe`] (WM-4). Only before the
//! field is seeded (world still loading) does the gate fall back to the world-gen water class on
//! [`ChunkCellMatrix::water`].

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::systems::water::WaterTileProbe;
use crate::terrain::generation::ChunkCellMatrix;

/// Live water depth (m) over a cell at which it cannot burn.
pub const FIRE_WATER_DEPTH_M: f32 = 0.1;

/// Per-cell standing-water test shared by overlay, ember, and scalar chunk fire.
#[derive(SystemParam)]
pub struct SurfaceWaterFireGate<'w> {
    water: WaterTileProbe<'w>,
}

impl SurfaceWaterFireGate<'_> {
    /// Cell `i` of chunk `chunk` (row-major in `matrix`) is under standing water.
    #[inline]
    pub fn cell_has_standing_water(&self, chunk: IVec2, matrix: &ChunkCellMatrix, i: usize) -> bool {
        let (sx, sy) = (matrix.size.x as i32, matrix.size.y as i32);
        if sx > 0 && sy > 0 {
            let tile = IVec2::new(chunk.x * sx + i as i32 % sx, chunk.y * sy + i as i32 / sx);
            if let Some(depth) = self.water.live_depth_m(tile) {
                return depth >= FIRE_WATER_DEPTH_M;
            }
        }
        // Field not seeded (or the cell lies off the map): the world-gen class stands in.
        !self.water.seeded() && matrix.water.get(i).is_some_and(|c| c.is_open_water())
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
    use crate::systems::water::SurfaceWaterField;
    use crate::terrain::generation::WaterClass;
    use bevy::ecs::system::RunSystemOnce;

    fn matrix() -> ChunkCellMatrix {
        let mut m = ChunkCellMatrix::new(UVec2::new(4, 1));
        for i in 0..4 {
            m.moisture[i] = 0.05;
        }
        m
    }

    #[test]
    fn before_the_field_is_seeded_the_world_gen_class_stands_in() {
        let mut m = matrix();
        m.water[0] = WaterClass::Lake;
        m.water[1] = WaterClass::River;
        m.water[2] = WaterClass::Ocean;
        m.water[3] = WaterClass::Wetland;
        let mut world = World::new();
        world
            .run_system_once(move |gate: SurfaceWaterFireGate| {
                assert!(gate.cell_has_standing_water(IVec2::ZERO, &m, 0));
                assert!(gate.cell_has_standing_water(IVec2::ZERO, &m, 1));
                assert!(gate.cell_has_standing_water(IVec2::ZERO, &m, 2));
                assert!(!gate.cell_has_standing_water(IVec2::ZERO, &m, 3), "wetland is wet ground");
            })
            .unwrap();
    }

    /// Live depth decides once the field exists: flooded dry land cannot burn, a dried river can.
    #[test]
    fn live_depth_decides_flooded_cells_cannot_burn_and_dried_ones_can() {
        let mut m = matrix(); // chunk (1, 0) covers world tiles x 4..8
        m.water[1] = WaterClass::River; // world class says river, the water has left
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, 16, 4, vec![0.0; 64], 0.0);
        let flooded = field.idx(4, 0);
        field.depth_m[flooded] = 0.5; // land cell 0 of chunk (1,0), flooded
        let shallow = field.idx(6, 0);
        field.depth_m[shallow] = 0.05; // film below the burn gate
        let mut world = World::new();
        world.insert_resource(field);
        world
            .run_system_once(move |gate: SurfaceWaterFireGate| {
                let c = IVec2::new(1, 0);
                assert!(gate.cell_has_standing_water(c, &m, 0), "flooded forest does not burn");
                assert!(!gate.cell_has_standing_water(c, &m, 1), "dried river burns");
                assert!(!gate.cell_has_standing_water(c, &m, 2), "a film is not standing water");
                assert!(!gate.cell_has_standing_water(IVec2::new(9, 9), &m, 0), "off-map cell is land");
            })
            .unwrap();
    }
}
