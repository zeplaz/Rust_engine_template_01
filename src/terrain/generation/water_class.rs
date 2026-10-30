//! Per-cell **water class** — the one sim-side answer to "is this cell water, and what kind".
//!
//! World generation decides water exactly once, from the hydrology solve
//! ([`HydrologyResult`]): [`WorldWaterGrid::from_hydrology`]. The result is
//!
//! - stamped into every chunk's [`ChunkCellMatrix::water`](super::ChunkCellMatrix) by pass 4 (the only writer),
//! - used to carve the dense terrain cache so elevation and terrain family agree with it,
//! - projected by the render catalog (`WaterSurfaceVisualCatalog`) into presentation tiles
//!   (render reads sim, never the reverse).
//!
//! Vegetation suitability, fire (surface-water gate), the tactical tree layer and the substrate slab all
//! read the class. Nobody re-derives water from terrain family names, elevation lines or biome weights.

use bevy::prelude::*;

use super::hydrology::HydrologyResult;

/// What a cell is, water-wise.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WaterClass {
    /// Ordinary land.
    #[default]
    Dry = 0,
    /// Wet ground that is not open water: unlisted depressions and swamp. Reeds / riparian cover only.
    Wetland = 1,
    River = 2,
    /// Inland standing water (depression fill or an inland basin below sea level).
    Lake = 3,
    /// Standing water connected to the map edge.
    Ocean = 4,
}

impl WaterClass {
    /// Rivers, lakes and the ocean: nothing grows and nothing burns here.
    #[inline]
    #[must_use]
    pub const fn is_open_water(self) -> bool {
        matches!(self, Self::River | Self::Lake | Self::Ocean)
    }

    /// Lake or ocean (no flow).
    #[inline]
    #[must_use]
    pub const fn is_standing(self) -> bool {
        matches!(self, Self::Lake | Self::Ocean)
    }
}

/// World-resolution water classes, row-major `width x height`. Written once per world generation.
#[derive(Resource, Clone, Debug, Default)]
pub struct WorldWaterGrid {
    pub width: u32,
    pub height: u32,
    pub classes: Vec<WaterClass>,
    /// Changes with every new generation, so consumers refresh only when the world changed.
    pub stamp: u64,
}

impl WorldWaterGrid {
    #[must_use]
    pub fn from_hydrology(hydro: &HydrologyResult, width: u32, height: u32, stamp: u64) -> Self {
        Self {
            width,
            height,
            classes: classify_world_water(hydro, width, height),
            stamp,
        }
    }

    /// Like [`Self::from_hydrology`], then reconciled with the terrain the raster draws: a cell whose height is
    /// at or under `water_line` is drawn as water, so it is never `Dry` / `Wetland` here (see
    /// [`classify_world_water_on_terrain`]).
    #[must_use]
    pub fn from_hydrology_on_terrain(
        hydro: &HydrologyResult,
        width: u32,
        height: u32,
        stamp: u64,
        dem: &[f32],
        water_line: f32,
    ) -> Self {
        Self {
            width,
            height,
            classes: classify_world_water_on_terrain(hydro, width, height, dem, water_line),
            stamp,
        }
    }

    #[inline]
    #[must_use]
    pub fn class_at(&self, x: i32, y: i32) -> Option<WaterClass> {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return None;
        }
        self.classes
            .get(y as usize * self.width as usize + x as usize)
            .copied()
    }

    #[must_use]
    pub fn count(&self, class: WaterClass) -> usize {
        self.classes.iter().filter(|c| **c == class).count()
    }
}

/// Classify every world cell from the hydrology solve.
///
/// - **Ocean**: a standing-water region (`lake_mask` component) that reaches the map edge. Priority-flood
///   makes edge cells outlets, so an edge-connected region can only be sea-level water.
/// - **Lake**: a hydrology lake region ([`HydrologyResult::lakes`], already limited to the designed lake
///   count) that does not reach the edge.
/// - **Wetland**: remaining depression cells (pits too small or too many to be a designed lake).
/// - **River**: the channel mask, wherever it is not already ocean.
#[must_use]
pub fn classify_world_water(hydro: &HydrologyResult, width: u32, height: u32) -> Vec<WaterClass> {
    let w = width as usize;
    let h = height as usize;
    let n = w * h;
    let mut out = vec![WaterClass::Dry; n];
    if n == 0 {
        return out;
    }

    if hydro.lake_mask.len() == n {
        // Depression components (8-connected, as the lake regions are labelled).
        let mut seen = vec![false; n];
        let mut stack: Vec<usize> = Vec::new();
        let mut cells: Vec<usize> = Vec::new();
        for start in 0..n {
            if seen[start] || !hydro.lake_mask[start] {
                continue;
            }
            seen[start] = true;
            stack.push(start);
            cells.clear();
            let mut touches_edge = false;
            while let Some(i) = stack.pop() {
                cells.push(i);
                let (x, y) = (i % w, i / w);
                if x == 0 || y == 0 || x + 1 == w || y + 1 == h {
                    touches_edge = true;
                }
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let j = ny as usize * w + nx as usize;
                        if !seen[j] && hydro.lake_mask[j] {
                            seen[j] = true;
                            stack.push(j);
                        }
                    }
                }
            }
            let class = if touches_edge {
                WaterClass::Ocean
            } else {
                WaterClass::Wetland
            };
            for &i in &cells {
                out[i] = class;
            }
        }
        // Designed lakes: the listed regions that are not ocean.
        for lake in &hydro.lakes {
            for &(x, y) in &lake.cells {
                let i = y as usize * w + x as usize;
                if i < n && out[i] == WaterClass::Wetland {
                    out[i] = WaterClass::Lake;
                }
            }
        }
    }

    if hydro.river_mask.len() == n {
        for i in 0..n {
            if hydro.river_mask[i] && out[i] != WaterClass::Ocean {
                out[i] = WaterClass::River;
            }
        }
    }
    out
}

/// [`classify_world_water`] plus the terrain rule that keeps the sim and the drawn map on one truth: a land
/// or wetland cell that sits under the water line is standing water (the terrain family there is water), so
/// it is a `Lake`. Inland low ground the hydrology did not list as a designed lake would otherwise draw as
/// water while vegetation and fire treated it as ground.
#[must_use]
pub fn classify_world_water_on_terrain(
    hydro: &HydrologyResult,
    width: u32,
    height: u32,
    dem: &[f32],
    water_line: f32,
) -> Vec<WaterClass> {
    let mut out = classify_world_water(hydro, width, height);
    if dem.len() == out.len() {
        for (class, &h) in out.iter_mut().zip(dem) {
            if matches!(*class, WaterClass::Dry | WaterClass::Wetland) && h <= water_line {
                *class = WaterClass::Lake;
            }
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::terrain::generation::hydrology::{
        compute_hydrology_world, HydrologyParams, LakeRegion,
    };
    use crate::terrain::generation::world_generator_enhanced::{
        build_world_noise_kernels, sample_fields_at_world_tile, WorldGenParams,
    };
    use crate::terrain::world_map_scale::TileExtentPreset;

    /// Real world generation up to the hydrology solve: the same noise path as the world-gen job.
    pub(crate) fn generate_hydrology(
        seed: u64,
    ) -> (WorldGenParams, Vec<f32>, Vec<f32>, HydrologyResult) {
        let mut p = WorldGenParams::default();
        p.apply_tile_extent_preset(TileExtentPreset::TacticalSmall);
        p.seed = seed;
        let (w, h) = (p.width as usize, p.height as usize);
        let k = build_world_noise_kernels(&p, &p.noise_sampling);
        let mut dem = vec![0.0f32; w * h];
        let mut moist = vec![0.0f32; w * h];
        for y in 0..h {
            for x in 0..w {
                let (hv, mv, _) =
                    sample_fields_at_world_tile(x as i32, y as i32, &p, &k, &p.noise_sampling);
                dem[y * w + x] = hv;
                moist[y * w + x] = mv;
            }
        }
        let hp = HydrologyParams::from_biome_tuning(&p.biome_tuning);
        let hydro = compute_hydrology_world(
            p.width,
            p.height,
            &dem,
            Some(&moist),
            &hp,
            p.river_count,
            p.lake_count,
        );
        (p, dem, moist, hydro)
    }

    #[test]
    fn classes_follow_hydrology_masks() {
        let (p, _, _, hydro) = generate_hydrology(7);
        let grid = WorldWaterGrid::from_hydrology(&hydro, p.width, p.height, 1);
        assert_eq!(grid.classes.len(), (p.width * p.height) as usize);
        for (i, class) in grid.classes.iter().enumerate() {
            if hydro.river_mask[i] && *class != WaterClass::Ocean {
                assert_eq!(*class, WaterClass::River);
            }
            if *class == WaterClass::Lake {
                assert!(hydro.lake_mask[i], "lake cell {i} outside the depression mask");
            }
        }
        assert!(grid.count(WaterClass::River) > 0);
        for lake in &hydro.lakes {
            let (x, y) = lake.cells[0];
            let c = grid.class_at(x as i32, y as i32).unwrap();
            assert!(matches!(c, WaterClass::Lake | WaterClass::Ocean | WaterClass::River));
        }
    }

    #[test]
    fn edge_connected_water_is_ocean_and_inland_water_is_lake() {
        // 8x8: a basin in the middle (inland) and a sea-level strip on the left edge.
        let (w, h) = (8usize, 8usize);
        let mut lake_mask = vec![false; w * h];
        for y in 2..5 {
            for x in 3..6 {
                lake_mask[y * w + x] = true;
            }
        }
        for y in 0..8 {
            lake_mask[y * w] = true;
        }
        let lakes = vec![LakeRegion {
            cells: (2..5u32)
                .flat_map(|y| (3..6u32).map(move |x| (x, y)))
                .collect(),
        }];
        let hydro = HydrologyResult {
            rivers: Vec::new(),
            lakes,
            accumulation: vec![0.0; w * h],
            river_mask: vec![false; w * h],
            lake_mask,
            filled_dem: vec![0.5; w * h],
        };
        let c = classify_world_water(&hydro, w as u32, h as u32);
        assert_eq!(c[3 * w + 4], WaterClass::Lake);
        assert_eq!(c[3 * w], WaterClass::Ocean);
        assert_eq!(c[3 * w + 7], WaterClass::Dry);
    }
}
