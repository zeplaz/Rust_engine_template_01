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
use crate::terrain::biome::BiomeTuning;
use crate::terrain::world_map_scale::{LandFeatureRhythm, WorldMapScale};

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

/// Marsh belts only form on low ground: at most this far above the shallow-water line (normalized height).
const MARSH_BANK_HEIGHT: f32 = 0.12;
/// Moisture threshold reduction at full regional emphasis.
const MARSH_REGION_THRESHOLD_DROP: f32 = 0.15;

/// Soft marsh / wetland belt tunables (derived from [`LandFeatureRhythm`] + [`BiomeTuning`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarshBeltParams {
    /// Belt reach in tiles around river / lake cells. `0` = off.
    pub radius_tiles: u32,
    /// Minimum moisture for a belt cell (the biome `wetland_moist_threshold`).
    pub moisture_threshold: f32,
    /// `0..=1` extra reach / lower threshold inside `region`. Ignored without a region.
    pub emphasis: f32,
    /// Map fractions `[x0, y0, x1, y1]` (half-open, tile `y` up). When set, belts form only inside it.
    pub region: Option<[f32; 4]>,
}

impl MarshBeltParams {
    #[must_use]
    pub fn from_rhythm(
        rhythm: &LandFeatureRhythm,
        scale: &WorldMapScale,
        tuning: &BiomeTuning,
    ) -> Self {
        Self {
            radius_tiles: if rhythm.marsh_belt_km > 0.0 {
                scale.tiles_for_km(rhythm.marsh_belt_km)
            } else {
                0
            },
            moisture_threshold: tuning.wetland_moist_threshold,
            emphasis: rhythm.marsh_emphasis,
            region: rhythm.marsh_region,
        }
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

    /// Soft marsh belts (PLAYTEST-QUADRANT-WORLD-001 §3): wet, low `Dry` ground near a river or lake becomes
    /// [`WaterClass::Wetland`]. Never touches open water, so channel rivers keep their mask; Wetland only caps
    /// vegetation and raises dense-cache moisture — no occupation, no new authority. Returns cells converted.
    pub fn apply_marsh_belts(
        &mut self,
        moisture: &[f32],
        dem: &[f32],
        water_line: f32,
        params: &MarshBeltParams,
    ) -> usize {
        let (w, h) = (self.width as usize, self.height as usize);
        let n = w * h;
        if params.radius_tiles == 0
            || self.classes.len() != n
            || moisture.len() != n
            || dem.len() != n
        {
            return 0;
        }
        let emphasis = params.emphasis.clamp(0.0, 1.0);
        let base_r = params.radius_tiles as i32;
        // Region mode: belts only inside the rectangle, with emphasis. No region: uniform, no emphasis.
        let (reach, threshold) = if params.region.is_some() {
            (
                base_r + (emphasis * base_r as f32).ceil() as i32,
                params.moisture_threshold - emphasis * MARSH_REGION_THRESHOLD_DROP,
            )
        } else {
            (base_r, params.moisture_threshold)
        };
        // Half-open tile box, same semantics as `engine::playquad::TileAabb::contains` (tile index as f32).
        let region = params.region.map(|[x0, y0, x1, y1]| {
            (x0 * w as f32, y0 * h as f32, x1 * w as f32, y1 * h as f32)
        });
        let in_region = |x: i32, y: i32| {
            region.map_or(true, |(x0, y0, x1, y1)| {
                (x as f32) >= x0 && (x as f32) < x1 && (y as f32) >= y0 && (y as f32) < y1
            })
        };
        let max_r = reach;
        let mut marks: Vec<usize> = Vec::new();
        for i in 0..n {
            if !matches!(self.classes[i], WaterClass::River | WaterClass::Lake) {
                continue;
            }
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            for dy in -max_r..=max_r {
                for dx in -max_r..=max_r {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let j = ny as usize * w + nx as usize;
                    if self.classes[j] != WaterClass::Dry || dem[j] > water_line + MARSH_BANK_HEIGHT {
                        continue;
                    }
                    if in_region(nx, ny)
                        && dx * dx + dy * dy <= reach * reach
                        && moisture[j] >= threshold
                    {
                        marks.push(j);
                    }
                }
            }
        }
        marks.sort_unstable();
        marks.dedup();
        for &j in &marks {
            self.classes[j] = WaterClass::Wetland;
        }
        marks.len()
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
        widen_rivers_by_flow(hydro, w, h, &mut out);
    }
    out
}

/// Widest river half-width in cells beyond the centreline (a trunk is at most `2 * R + 1` cells wide).
const RIVER_MAX_HALF_WIDTH: i32 = 2;
/// A bank cell may sit at most this far above its channel cell (filled DEM) — rivers do not climb hills.
const RIVER_BANK_RISE_MAX: f32 = 0.02;

/// Channel width from flow: half-width grows with `sqrt(accumulation / channel threshold)` (hydraulic
/// geometry, width ∝ discharge^0.5), so the head of a channel stays one cell wide and trunks near the
/// mouth widen to `2 * RIVER_MAX_HALF_WIDTH + 1` cells. Banks only take dry / wetland cells that are not
/// above the channel.
fn widen_rivers_by_flow(hydro: &HydrologyResult, w: usize, h: usize, out: &mut [WaterClass]) {
    let n = w * h;
    if hydro.accumulation.len() != n {
        return;
    }
    let base = hydro
        .river_mask
        .iter()
        .zip(&hydro.accumulation)
        .filter_map(|(m, a)| m.then_some(*a))
        .fold(f32::INFINITY, f32::min);
    if !base.is_finite() || base <= 0.0 {
        return;
    }
    let filled = (hydro.filled_dem.len() == n).then_some(hydro.filled_dem.as_slice());
    let mut banks: Vec<usize> = Vec::new();
    for i in 0..n {
        if !hydro.river_mask[i] {
            continue;
        }
        let r = (((hydro.accumulation[i] / base).sqrt() - 1.0) * 0.5).floor() as i32;
        let r = r.clamp(0, RIVER_MAX_HALF_WIDTH);
        if r == 0 {
            continue;
        }
        let (x, y) = ((i % w) as i32, (i / w) as i32);
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                if !matches!(out[j], WaterClass::Dry | WaterClass::Wetland) {
                    continue;
                }
                if filled.is_some_and(|f| f[j] > f[i] + RIVER_BANK_RISE_MAX) {
                    continue;
                }
                banks.push(j);
            }
        }
    }
    for j in banks {
        out[j] = WaterClass::River;
    }
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
        generate_hydrology_with(seed, TileExtentPreset::TacticalSmall, None)
    }

    /// As [`generate_hydrology`], for any extent and an optional rhythm (PlayQuad opts in; `None` = default).
    pub(crate) fn generate_hydrology_with(
        seed: u64,
        preset: TileExtentPreset,
        rhythm: Option<LandFeatureRhythm>,
    ) -> (WorldGenParams, Vec<f32>, Vec<f32>, HydrologyResult) {
        let mut p = WorldGenParams::default();
        if let Some(r) = rhythm {
            p.feature_rhythm = r;
        }
        p.apply_tile_extent_preset(preset);
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
        let hp = HydrologyParams::from_biome_tuning(&p.biome_tuning)
            .with_rhythm(&p.feature_rhythm, &p.map_scale);
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

    fn belt_grid(w: u32, h: u32, river_x: u32) -> WorldWaterGrid {
        let mut classes = vec![WaterClass::Dry; (w * h) as usize];
        for y in 0..h {
            classes[(y * w + river_x) as usize] = WaterClass::River;
        }
        WorldWaterGrid { width: w, height: h, classes, stamp: 1 }
    }

    #[test]
    fn marsh_belt_is_soft_wetland_never_open_water() {
        let (w, h) = (16u32, 16u32);
        let n = (w * h) as usize;
        let mut grid = belt_grid(w, h, 3); // channel in the west half
        let wet = vec![0.9f32; n];
        let low = vec![0.30f32; n];
        let p = MarshBeltParams {
            radius_tiles: 2,
            moisture_threshold: 0.7,
            emphasis: 0.0,
            region: None,
        };
        let added = grid.apply_marsh_belts(&wet, &low, 0.28, &p);
        assert!(added > 0);
        assert_eq!(grid.class_at(3, 8), Some(WaterClass::River), "channel untouched");
        assert_eq!(grid.class_at(5, 8), Some(WaterClass::Wetland), "bank within reach");
        assert_eq!(grid.class_at(6, 8), Some(WaterClass::Dry), "belt is bounded");

        // Dry moisture or high ground: no belt.
        let mut dry = belt_grid(w, h, 3);
        assert_eq!(dry.apply_marsh_belts(&vec![0.2; n], &low, 0.28, &p), 0);
        assert_eq!(dry.apply_marsh_belts(&wet, &vec![0.9; n], 0.28, &p), 0);
    }

    /// SE quadrant as map fractions, from the one shared partition (`engine::playquad`).
    fn se_fractions(w: u32, h: u32) -> [f32; 4] {
        use crate::engine::playquad::{quadrant_aabb, PlayQuadrant};
        let b = quadrant_aabb(PlayQuadrant::Se, w, h);
        [b.min.x / w as f32, b.min.y / h as f32, b.max.x / w as f32, b.max.y / h as f32]
    }

    #[test]
    fn marsh_region_confines_belt_to_se_and_emphasis_widens_it() {
        use crate::engine::playquad::{quadrant_of, PlayQuadrant};
        use bevy::prelude::Vec2;
        let (w, h) = (64u32, 64u32);
        let n = (w * h) as usize;
        let wet = vec![0.75f32; n];
        let low = vec![0.30f32; n];
        // Channels: one in the west (SW/NW), one in the east (SE/NE). Full height.
        let mk = || {
            let mut g = belt_grid(w, h, 10);
            for y in 0..h {
                g.classes[(y * w + 50) as usize] = WaterClass::River;
            }
            g
        };
        let region = Some(se_fractions(w, h));
        let flat = MarshBeltParams { radius_tiles: 2, moisture_threshold: 0.7, emphasis: 0.0, region };
        let wide = MarshBeltParams { emphasis: 1.0, ..flat };
        let (mut a, mut b) = (mk(), mk());
        a.apply_marsh_belts(&wet, &low, 0.28, &flat);
        b.apply_marsh_belts(&wet, &low, 0.28, &wide);

        for y in 0..h as i32 {
            for x in 0..w as i32 {
                let in_se = quadrant_of(Vec2::new(x as f32, y as f32), w, h) == Some(PlayQuadrant::Se);
                for g in [&a, &b] {
                    if !in_se {
                        assert_ne!(
                            g.class_at(x, y),
                            Some(WaterClass::Wetland),
                            "marsh leaked outside SE at ({x},{y})"
                        );
                    }
                }
            }
        }
        // SE (x >= 32, y < 32): channel at x = 50; 3 tiles off the bank is wetland only with emphasis.
        assert_eq!(a.class_at(53, 10), Some(WaterClass::Dry));
        assert_eq!(b.class_at(53, 10), Some(WaterClass::Wetland));
        assert_eq!(b.class_at(50, 10), Some(WaterClass::River), "channel untouched");
        // NW half of the same east channel (y >= 32) gets nothing; SW channel (x = 10) gets nothing.
        assert_eq!(b.class_at(12, 10), Some(WaterClass::Dry), "SW gets no marsh");
        assert_eq!(b.class_at(53, 40), Some(WaterClass::Dry), "NE gets no marsh");
    }

    /// PQ-H1: default rhythm / params are neutral — legacy lake choice, no belts, byte-identical classes.
    #[test]
    fn default_hydrology_is_neutral() {
        let (p, dem, moist, hydro) = generate_hydrology(7);
        assert_eq!(p.feature_rhythm, LandFeatureRhythm::default());
        let hp = HydrologyParams::from_biome_tuning(&p.biome_tuning);
        assert_eq!((hp.lake_attach_radius_tiles, hp.lake_attach_weight), (0, 0.0));

        // Legacy truncation: the same solve, sorted by size only, then truncated.
        let mut legacy = crate::terrain::generation::hydrology::compute_hydrology_rect(
            p.width,
            p.height,
            &dem,
            &hp,
            p.river_count,
            Some(&moist),
        )
        .lakes;
        if legacy.len() > p.lake_count as usize {
            legacy.sort_by(|a, b| b.cells.len().cmp(&a.cells.len()));
            legacy.truncate(p.lake_count as usize);
        }
        let got: Vec<_> = hydro.lakes.iter().map(|l| l.cells.clone()).collect();
        let want: Vec<_> = legacy.iter().map(|l| l.cells.clone()).collect();
        assert_eq!(got, want, "default lake list must equal legacy size-only ranking");

        let water_line = p.biome_tuning.shallow_water_height_max;
        let mut grid =
            WorldWaterGrid::from_hydrology_on_terrain(&hydro, p.width, p.height, 1, &dem, water_line);
        let before = grid.classes.clone();
        let mp = MarshBeltParams::from_rhythm(&p.feature_rhythm, &p.map_scale, &p.biome_tuning);
        assert_eq!(mp.radius_tiles, 0);
        assert_eq!(grid.apply_marsh_belts(&moist, &dem, water_line, &mp), 0);
        assert_eq!(grid.classes, before);
    }

    #[test]
    fn real_world_marsh_keeps_open_water() {
        let (p, dem, moist, hydro) = generate_hydrology(7);
        let water_line = p.biome_tuning.shallow_water_height_max;
        let mut grid =
            WorldWaterGrid::from_hydrology_on_terrain(&hydro, p.width, p.height, 1, &dem, water_line);
        let before = grid.classes.clone();
        let rhythm = LandFeatureRhythm::playquad_hydrology(se_fractions(p.width, p.height));
        let mp = MarshBeltParams::from_rhythm(&rhythm, &p.map_scale, &p.biome_tuning);
        grid.apply_marsh_belts(&moist, &dem, water_line, &mp);
        for (b, a) in before.iter().zip(&grid.classes) {
            if b.is_open_water() {
                assert_eq!(b, a, "belt must not convert open water");
            }
        }
    }

    /// PQ-H2 / PQ-H3 + quadrant sanity for one PlayQuad world seed (512², PlayQuad rhythm). Returns
    /// `(passes, report row)`. Thresholds from the first sweep: L = 40 SW lake cells, M = 200 SE wetland
    /// cells within R = 6 tiles of river/lake, NW land ≥ 0.6, NE land ≥ 0.7, no quadrant > 40 % ocean.
    fn score_playquad_seed(seed: u64) -> (bool, String) {
        use crate::engine::playquad::{quadrant_aabb, PlayQuadrant};
        const SW_LAKE_MIN_CELLS: usize = 40; // L
        const SE_WETLAND_MIN_CELLS: usize = 200; // M
        const SE_WETLAND_REACH: i32 = 6; // R
        let preset = TileExtentPreset::Standard;
        let w = preset.tiles_per_axis();
        let frac = se_fractions(w, w);
        {
            let rhythm = LandFeatureRhythm::playquad_hydrology(frac);
            let (p, dem, moist, hydro) = generate_hydrology_with(seed, preset, Some(rhythm));
            let water_line = p.biome_tuning.shallow_water_height_max;
            let mut grid = WorldWaterGrid::from_hydrology_on_terrain(
                &hydro, p.width, p.height, seed, &dem, water_line,
            );
            let mp = MarshBeltParams::from_rhythm(&rhythm, &p.map_scale, &p.biome_tuning);
            grid.apply_marsh_belts(&moist, &dem, water_line, &mp);
            let hp = HydrologyParams::from_biome_tuning(&p.biome_tuning)
                .with_rhythm(&rhythm, &p.map_scale);

            let q = |which| quadrant_aabb(which, w, w);
            let inside = |b: &crate::engine::playquad::TileAabb, x: u32, y: u32| {
                b.contains(bevy::prelude::Vec2::new(x as f32, y as f32))
            };
            let (sw, se, nw, ne) = (
                q(PlayQuadrant::Sw),
                q(PlayQuadrant::Se),
                q(PlayQuadrant::Nw),
                q(PlayQuadrant::Ne),
            );
            // PQ-H2: best designed Lake in SW that touches river_mask.
            let sw_lake = hydro
                .lakes
                .iter()
                .filter(|l| {
                    let (x, y) = l.cells[0];
                    grid.class_at(x as i32, y as i32) == Some(WaterClass::Lake)
                        && crate::terrain::generation::hydrology::lake_touches_river(
                            l,
                            &hydro.river_mask,
                            w as usize,
                            w as usize,
                            hp.lake_attach_radius_tiles,
                        )
                })
                .map(|l| l.cells.iter().filter(|&&(x, y)| inside(&sw, x, y)).count())
                .max()
                .unwrap_or(0);
            // PQ-H3: SE Wetland cells within R of River/Lake.
            let mut se_wet = 0usize;
            for y in 0..w {
                for x in 0..w {
                    if !inside(&se, x, y)
                        || grid.class_at(x as i32, y as i32) != Some(WaterClass::Wetland)
                    {
                        continue;
                    }
                    let near = (-SE_WETLAND_REACH..=SE_WETLAND_REACH).any(|dy| {
                        (-SE_WETLAND_REACH..=SE_WETLAND_REACH).any(|dx| {
                            dx * dx + dy * dy <= SE_WETLAND_REACH * SE_WETLAND_REACH
                                && matches!(
                                    grid.class_at(x as i32 + dx, y as i32 + dy),
                                    Some(WaterClass::River | WaterClass::Lake)
                                )
                        })
                    });
                    se_wet += usize::from(near);
                }
            }
            let frac_of = |b: &crate::engine::playquad::TileAabb, f: &dyn Fn(WaterClass, f32) -> bool| {
                let (mut hit, mut tot) = (0usize, 0usize);
                for y in 0..w {
                    for x in 0..w {
                        if inside(b, x, y) {
                            tot += 1;
                            let c = grid.class_at(x as i32, y as i32).unwrap();
                            hit += usize::from(f(c, dem[(y * w + x) as usize]));
                        }
                    }
                }
                hit as f32 / tot.max(1) as f32
            };
            let land = |c: WaterClass, _: f32| !c.is_open_water() && c != WaterClass::Wetland;
            let nw_land = frac_of(&nw, &land);
            let ne_land = frac_of(&ne, &land);
            let max_ocean = [&nw, &ne, &sw, &se]
                .iter()
                .map(|b| frac_of(b, &|c, _| c == WaterClass::Ocean))
                .fold(0.0f32, f32::max);
            let ok = sw_lake >= SW_LAKE_MIN_CELLS
                && se_wet >= SE_WETLAND_MIN_CELLS
                && nw_land >= 0.6
                && ne_land >= 0.7
                && max_ocean <= 0.4;
            (
                ok,
                format!(
                    "seed {seed:>5}: sw_lake_attached={sw_lake:>5} se_wetland={se_wet:>5} nw_land={nw_land:.2} ne_land={ne_land:.2} max_ocean={max_ocean:.2} {}",
                    if ok { "PASS" } else { "-" }
                ),
            )
        }
    }

    /// Seed sweep (N ≤ 64). Run manually:
    /// `cargo test -p proc_A_dine01 --lib playquad_seed_sweep -- --ignored --nocapture`.
    #[test]
    #[ignore = "512² sweep; run manually to (re)pick PLAYQUAD_WORLD_SEED"]
    fn playquad_seed_sweep() {
        let mut winners = Vec::new();
        for seed in 1..=64u64 {
            let (ok, row) = score_playquad_seed(seed);
            println!("{row}");
            if ok {
                winners.push(seed);
            }
        }
        println!("winners: {winners:?}");
        assert!(!winners.is_empty(), "no seed in 1..=64 satisfies PQ-H2/H3 — escalate to moisture bias");
    }

    /// The committed `PLAYQUAD_WORLD_SEED` must satisfy PQ-H2/H3 (512², ~1 s).
    #[test]
    fn playquad_world_seed_satisfies_hydrology_criteria() {
        let seed = crate::engine::launch_args::TestScene::PLAYQUAD_WORLD_SEED;
        let (ok, row) = score_playquad_seed(seed);
        assert!(ok, "{row}");
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
            eroded_dem: vec![0.5; w * h],
        };
        let c = classify_world_water(&hydro, w as u32, h as u32);
        assert_eq!(c[3 * w + 4], WaterClass::Lake);
        assert_eq!(c[3 * w], WaterClass::Ocean);
        assert_eq!(c[3 * w + 7], WaterClass::Dry);
    }
}
