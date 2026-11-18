//! World-static water depth bootstrap (PLAN-WATER-DEPTH-TRUTH-v1 W2).
//!
//! Outside the live [`super::SurfaceWaterField`] ROI, gameplay wet tests read this
//! depth grid — not `WaterClass` and not the render catalog.

use bevy::prelude::*;

use crate::terrain::generation::water_class::{WaterClass, WorldWaterGrid};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

use super::water_runtime::WET_DEPTH_M;

/// Full-map initial / static water depth (metres). Written at world-gen; DamFlood author may patch.
#[derive(Resource, Clone, Debug, Default)]
pub struct WorldStaticWaterDepth {
    pub width: u32,
    pub height: u32,
    pub depth_m: Vec<f32>,
    pub stamp: u64,
}

impl WorldStaticWaterDepth {
    #[inline]
    #[must_use]
    pub fn idx(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((y as usize) * (self.width as usize) + (x as usize))
    }

    #[must_use]
    pub fn depth_at_world(&self, wx: i32, wz: i32) -> Option<f32> {
        if wx < 0 || wz < 0 || self.width == 0 {
            return None;
        }
        let i = self.idx(wx as u32, wz as u32)?;
        self.depth_m.get(i).copied()
    }

    #[must_use]
    pub fn is_wet_world(&self, wx: i32, wz: i32) -> Option<bool> {
        self.depth_at_world(wx, wz).map(|d| d >= WET_DEPTH_M)
    }

    /// Bootstrap from world-gen class + DEM. Class is a **bake hint** only — output is depth.
    #[must_use]
    pub fn from_water_grid(
        grid: &WorldWaterGrid,
        elevation: &[f32],
        water_line: f32,
    ) -> Self {
        let w = grid.width;
        let h = grid.height;
        let n = (w as usize).saturating_mul(h as usize);
        let mut depth_m = vec![0.0_f32; n];
        if grid.classes.len() < n {
            return Self {
                width: w,
                height: h,
                depth_m,
                stamp: grid.stamp,
            };
        }
        // Channel / standing depths in metres (tile ≈ 16 m horizontally; depth is vertical).
        const RIVER_DEPTH_M: f32 = 2.5;
        const LAKE_MIN_M: f32 = 4.0;
        const OCEAN_MIN_M: f32 = 8.0;
        let _ = TILE_METRES_GAMEPLAY;
        for i in 0..n {
            let class = grid.classes[i];
            let elev = elevation.get(i).copied().unwrap_or(water_line);
            // Freeboard under water_line → depth when terrain is submerged.
            let from_dem = ((water_line - elev) * TILE_METRES_GAMEPLAY).max(0.0);
            depth_m[i] = match class {
                WaterClass::Dry | WaterClass::Wetland => 0.0,
                WaterClass::River => RIVER_DEPTH_M.max(from_dem * 0.35),
                WaterClass::Lake => LAKE_MIN_M.max(from_dem),
                WaterClass::Ocean => OCEAN_MIN_M.max(from_dem),
            };
        }
        Self {
            width: w,
            height: h,
            depth_m,
            stamp: grid.stamp,
        }
    }

    /// Patch a cell (DamFlood author / tools). No-op OOB.
    pub fn set_depth(&mut self, wx: i32, wz: i32, depth_m: f32) {
        if wx < 0 || wz < 0 {
            return;
        }
        if let Some(i) = self.idx(wx as u32, wz as u32) {
            if let Some(slot) = self.depth_m.get_mut(i) {
                *slot = depth_m.max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_class_is_zero_depth() {
        let mut grid = WorldWaterGrid {
            width: 2,
            height: 2,
            classes: vec![
                WaterClass::Dry,
                WaterClass::River,
                WaterClass::Lake,
                WaterClass::Ocean,
            ],
            stamp: 1,
        };
        let elev = vec![0.5_f32; 4];
        let d = WorldStaticWaterDepth::from_water_grid(&grid, &elev, 0.4);
        assert_eq!(d.depth_at_world(0, 0), Some(0.0));
        assert!(d.depth_at_world(1, 0).unwrap() >= WET_DEPTH_M);
        assert!(d.is_wet_world(0, 1).unwrap());
        assert!(d.is_wet_world(1, 1).unwrap());
        let _ = &mut grid;
    }
}
