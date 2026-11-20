//! Authored valley for DamFlood — bed + corridor depth (PLAN-WATER-DEPTH-TRUTH-v1 W4).
//! Look class on `WorldWaterGrid` is a bake hint only; gameplay wet is depth.

use bevy::prelude::*;

use crate::engine::test_harness::ActiveTestScene;
use crate::render::fx_spine::water_surface_visual::WaterSurfaceVisualCatalog;
use crate::render::TileWorldFallbackRasterDirty;
use crate::systems::water::{
    SurfaceWaterField, SurfaceWaterRoiRequest, SurfaceWaterRoiSpec, WorldStaticWaterDepth,
    WET_DEPTH_M,
};
use crate::terrain::family::default_terrain_families;
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::generation::chunk::Chunk;
use crate::terrain::generation::water_class::{WaterClass, WorldWaterGrid};
use crate::terrain::generation::{DenseTerrainHydrationGate, WorldGenDenseTerrainCache};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::fixture::load_dam_valley_fixture;
use super::scene_seed::DamFloodWaterConfigLatch;

const RIVER_HALF_WIDTH: u32 = 3;
/// Authored channel depth (metres) — matrix wet / inundation threshold uses [`WET_DEPTH_M`].
const CORRIDOR_DEPTH_M: f32 = 3.5;

/// Valley profile (OP-12). Heights are metres above the channel bed at the same z; the author
/// converts to the matrix's normalised elevation with `WorldGenParams::elevation_range_m`.
///
/// The old profile put the banks at `water_line + 0.10` (~310 m above the bed) and rising, so a
/// 40 m embankment could only ever fill a 7-tile canyon and the forest floodplain stayed dry. Now
/// the floodplain is a low terrace inside the crest, hills close the valley at the narrows (dam
/// abutments) and at its head, and walls bound it on both sides, so the reservoir is real
/// topography filled by the real solver.
///
/// Channel bed fall per tile. Uniform flow of the baseflow in the 7-tile channel at the static
/// depth (`CORRIDOR_DEPTH_M`) needs a water-surface fall of about 0.02 m/tile on the solver's
/// conductance (`Q = 16*dx*sqrt(g*h) * dEta` per cell); a steeper bed (it was 0.15 m/tile) makes
/// the solver run the river as a ~7 cm film while world static water says 3.5 m (OP-10 seam).
pub const CHANNEL_FALL_M_PER_TILE: f32 = 0.02;
/// Floodplain terrace above the channel bed at the channel edge, and its cross-slope.
pub const TERRACE_ABOVE_BED_M: f32 = 6.0;
pub const TERRACE_SLOPE_M_PER_TILE: f32 = 0.15;
/// Lowest dry cell above the world water line (see [`ValleyProfile::bed0_m`]).
pub const LAND_ABOVE_WATER_LINE_M: f32 = 2.0;
/// Valley walls: start this many tiles from the river axis, rise this fast, plateau at this height.
pub const WALL_START_TILES: u32 = 50;
/// How far (tiles) each wall line wanders in or out along the valley (mean = `WALL_START_TILES`).
pub const WALL_WANDER_TILES: f32 = 8.0;
pub const WALL_RISE_M_PER_TILE: f32 = 12.0;
pub const WALL_PLATEAU_M: f32 = 100.0;
/// Dam abutment hills across the narrows (must stand above crest + freeboard).
pub const ABUTMENT_M: f32 = 90.0;
/// Rows fully abutment-high either side of the dam centre row, and where the hill fades out.
pub const ABUTMENT_FULL_ROWS: f32 = 4.0;
pub const ABUTMENT_FADE_ROWS: f32 = 9.0;
/// The valley head closes (hills across the floodplain) this many rows upstream of the dam.
pub const HEAD_START_ROWS: i32 = 44;
pub const HEAD_RISE_ROWS: i32 = 16;
pub const HEAD_RISE_M: f32 = 70.0;

/// Geometry that places the narrows in the profile.
#[derive(Clone, Copy, Debug)]
pub struct ValleyProfile {
    pub river_x: i32,
    /// Dam centre row (a 2-row footprint centred on the narrows covers `dam_z` +- 1).
    pub dam_z: f32,
    pub dam_half_span: i32,
    pub head_z: i32,
}

impl ValleyProfile {
    #[must_use]
    pub fn from_fixture(f: &super::fixture::DamValleyFixture) -> Self {
        Self {
            river_x: f.narrows_tile.0,
            dam_z: f.narrows_tile.1 as f32 - 0.5,
            dam_half_span: (f.dam_span_tiles.max(4) / 2) as i32,
            head_z: f.narrows_tile.1 + HEAD_START_ROWS,
        }
    }

    /// Ground height above the channel bed (same z) at tile `(x, z)`; 0 inside the channel.
    #[must_use]
    pub fn above_channel_m(&self, x: i32, z: i32) -> f32 {
        let dx = (x - self.river_x).unsigned_abs();
        if dx <= RIVER_HALF_WIDTH {
            return 0.0;
        }
        let (xf, zf) = (x as f32, z as f32);
        // Gentle swells on the floodplain (0..+2 m: never below the terrace, so the lowest dry
        // ground stays above the world water line and is never stamped as lake).
        let swell = 1.0 + 0.6 * (xf * 0.21 + zf * 0.13).sin() + 0.4 * (zf * 0.17 - xf * 0.07).sin();
        let mut m = TERRACE_ABOVE_BED_M
            + (dx - RIVER_HALF_WIDTH) as f32 * TERRACE_SLOPE_M_PER_TILE
            + swell.max(0.0);
        // Valley walls wander along the valley (each bank its own line), so a full reservoir
        // has a natural shoreline instead of a ruler-straight edge.
        let side = if x >= self.river_x { 1.3 } else { 0.0 };
        let wall_start = WALL_START_TILES as f32
            + WALL_WANDER_TILES * (0.6 * (zf * 0.11 + side).sin() + 0.4 * (zf * 0.27 + 2.0 * side).sin());
        let into_wall = dx as f32 - wall_start;
        if into_wall > 0.0 {
            m += (into_wall * WALL_RISE_M_PER_TILE).min(WALL_PLATEAU_M);
        }
        // Abutments: only outside the dam span, so the dam closes exactly the gap between them.
        if dx as i32 >= self.dam_half_span {
            let dz = (z as f32 - self.dam_z).abs();
            let t = ((ABUTMENT_FADE_ROWS - dz) / (ABUTMENT_FADE_ROWS - ABUTMENT_FULL_ROWS))
                .clamp(0.0, 1.0);
            m += ABUTMENT_M * t;
        }
        // Valley head, curved across the valley.
        let head = self.head_z as f32 + 6.0 * (xf * 0.09).sin() + 3.0 * (xf * 0.23 + 0.7).sin();
        let h = ((zf - head) / HEAD_RISE_ROWS as f32).clamp(0.0, 1.0);
        m += HEAD_RISE_M * h * h * (3.0 - 2.0 * h);
        m
    }

    /// Channel bed at row 0 (metres above the world datum). The datum is chosen so the lowest dry
    /// terrace cell stands [`LAND_ABOVE_WATER_LINE_M`] above the world water line: terrain stamping
    /// (`stamp_water_class`) turns any cell at or under the water line into a Lake, which made the
    /// whole forest floodplain open water to ecology (no vegetation, `CELL_WATER`) while the water
    /// grid and the voxel field still called it dry land.
    #[must_use]
    pub fn bed0_m(water_line: f32, range_m: f32) -> f32 {
        water_line * range_m + LAND_ABOVE_WATER_LINE_M - TERRACE_ABOVE_BED_M
    }

    /// Channel bed height above the world datum at row `z` given the bed at row 0.
    #[must_use]
    pub fn channel_bed_m(bed0_m: f32, z: i32) -> f32 {
        bed0_m + CHANNEL_FALL_M_PER_TILE * z as f32
    }
}

#[derive(Resource, Debug, Default)]
pub struct DamFloodValleyAuthorLatch {
    pub done: bool,
}

pub fn author_dam_flood_valley_system(
    scene: Option<Res<ActiveTestScene>>,
    mut latch: ResMut<DamFloodValleyAuthorLatch>,
    params: Res<WorldGenParams>,
    mut grid: Option<ResMut<WorldWaterGrid>>,
    mut dense: Option<ResMut<WorldGenDenseTerrainCache>>,
    mut static_depth: Option<ResMut<WorldStaticWaterDepth>>,
    mut chunks: Query<(&Chunk, &mut ChunkCellMatrix)>,
    mut gate: Option<ResMut<DenseTerrainHydrationGate>>,
) {
    if latch.done || !scene.is_some_and(|s| s.0.is_dam_flood()) {
        return;
    }
    let Some(grid) = grid.as_mut() else {
        return;
    };
    let Some(dense) = dense.as_mut() else {
        return;
    };
    if !dense.is_live_height_grid() || grid.width == 0 || grid.height == 0 || chunks.is_empty() {
        return;
    }

    let fixture = load_dam_valley_fixture();
    let cx = fixture.narrows_tile.0.max(0) as u32;
    let cz = fixture.narrows_tile.1.max(0) as u32;
    let w = grid.width.min(dense.width);
    let h = grid.height.min(dense.height);
    let n = (w as usize) * (h as usize);
    if grid.classes.len() < n || dense.elevation.len() < n {
        return;
    }

    let water_line = params.biome_tuning.shallow_water_height_max;
    let range_m = params.elevation_range_m.max(1.0);
    let bed0_m = ValleyProfile::bed0_m(water_line, range_m);
    let profile = ValleyProfile::from_fixture(&fixture);
    let families = default_terrain_families();
    let grassland = families
        .id("Grassland")
        .unwrap_or(crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID);
    // CLOSE-T3: the fixture's forest floodplain is forest ground (S0 "forest on the floodplain");
    // grassland everywhere capped every stand at shrub, so no tree stage could exist.
    let forest = families.id("Forest").unwrap_or(grassland);
    let fp = &fixture.forest_floodplain;
    let in_floodplain = |x: u32, z: u32| {
        let (x, z) = (x as i32, z as i32);
        x >= fp.min.0 && x <= fp.max.0 && z >= fp.min.1 && z <= fp.max.1
    };

    // Ensure static depth grid matches world size.
    if let Some(st) = static_depth.as_mut() {
        if st.width != w || st.height != h || st.depth_m.len() < n {
            st.width = w;
            st.height = h;
            st.depth_m = vec![0.0; n];
        } else {
            st.depth_m[..n].fill(0.0);
        }
    }

    for c in grid.classes.iter_mut().take(n) {
        *c = WaterClass::Dry;
    }
    for z in 0..h {
        for x in 0..w {
            let dx = (x as i32 - cx as i32).unsigned_abs();
            let i = (z as usize) * (w as usize) + (x as usize);
            if dx <= RIVER_HALF_WIDTH {
                // Look hint = channel; truth = bed + depth.
                grid.classes[i] = WaterClass::River;
                let bed_m = ValleyProfile::channel_bed_m(bed0_m, z as i32);
                dense.elevation[i] = (bed_m / range_m).clamp(0.02, 0.98);
                dense.moisture[i] = 0.95;
                if let Some(st) = static_depth.as_mut() {
                    st.depth_m[i] = CORRIDOR_DEPTH_M;
                }
            } else {
                // Dry terrace / walls: metres above the channel bed at this row; depth = 0.
                let ground_m = ValleyProfile::channel_bed_m(bed0_m, z as i32)
                    + profile.above_channel_m(x as i32, z as i32);
                dense.elevation[i] = (ground_m / range_m).clamp(0.02, 0.98);
                dense.moisture[i] = dense.moisture[i].min(0.55);
                dense.family[i] = if in_floodplain(x, z) { forest } else { grassland };
                if let Some(st) = static_depth.as_mut() {
                    st.depth_m[i] = 0.0;
                }
            }
        }
    }
    grid.stamp = grid.stamp.wrapping_add(0xDA_F100D1);
    if let Some(st) = static_depth.as_mut() {
        st.stamp = grid.stamp;
        st.width = w;
        st.height = h;
    }
    dense.apply_water_grid(params.as_ref(), grid);
    // `apply_water_grid` pulls River cells under the water line (it assumes a deep channel in
    // dry-above-water terrain). Here the channel is a metres-deep cut in a terrace that sits just
    // above the water line, so restore the authored bed (the solver reads it from the matrix).
    for z in 0..h {
        let bed_m = ValleyProfile::channel_bed_m(bed0_m, z as i32);
        for x in cx.saturating_sub(RIVER_HALF_WIDTH)..=(cx + RIVER_HALF_WIDTH).min(w.saturating_sub(1)) {
            let i = (z as usize) * (w as usize) + (x as usize);
            dense.elevation[i] = (bed_m / range_m).clamp(0.02, 0.98);
        }
    }

    for (chunk, mut matrix) in chunks.iter_mut() {
        let sx = matrix.size.x;
        let sy = matrix.size.y;
        if sx == 0 || sy == 0 {
            continue;
        }
        if matrix.water.len() != matrix.elevation.len() {
            matrix.water = vec![WaterClass::Dry; matrix.elevation.len()];
        }
        for y in 0..sy {
            for x in 0..sx {
                let wx = chunk.coord.x * sx as i32 + x as i32;
                let wy = chunk.coord.y * sy as i32 + y as i32;
                if wx < 0 || wy < 0 || wx as u32 >= w || wy as u32 >= h {
                    continue;
                }
                let src = (wy as usize) * (w as usize) + (wx as usize);
                let dst = matrix.idx(x, y);
                matrix.elevation[dst] = dense.elevation[src];
                matrix.moisture[dst] = dense.moisture[src];
                matrix.family[dst] = dense.family[src];
                matrix.water[dst] = grid.classes[src];
            }
        }
    }

    if let Some(g) = gate.as_mut() {
        g.reset();
    }
    latch.done = true;
    info!(
        target: "dam_flood::author_valley",
        "authored bed+depth corridor at x={cx} z≈{cz} half_width={RIVER_HALF_WIDTH} depth_m={CORRIDOR_DEPTH_M}"
    );
}

pub fn author_dam_flood_valley_rearm_water_system(
    scene: Option<Res<ActiveTestScene>>,
    latch: Res<DamFloodValleyAuthorLatch>,
    mut rear_latch: Local<bool>,
    mut raster_dirty: Option<ResMut<TileWorldFallbackRasterDirty>>,
    mut water_req: Option<ResMut<SurfaceWaterRoiRequest>>,
    mut water_field: Option<ResMut<SurfaceWaterField>>,
    mut water_cfg: Option<ResMut<DamFloodWaterConfigLatch>>,
    mut seed_latch: Option<ResMut<crate::systems::water::SurfaceWaterSeedLatch>>,
) {
    if *rear_latch || !latch.done || !scene.is_some_and(|s| s.0.is_dam_flood()) {
        return;
    }
    *rear_latch = true;
    if let Some(d) = raster_dirty.as_mut() {
        d.bump();
    }
    if let Some(field) = water_field.as_mut() {
        **field = SurfaceWaterField::default();
    }
    if let Some(sl) = seed_latch.as_mut() {
        sl.seeded = false;
    }
    if let Some(cfg) = water_cfg.as_mut() {
        cfg.enqueued = false;
    }
    let fixture = load_dam_valley_fixture();
    if let Some(req) = water_req.as_mut() {
        let fp = &fixture.forest_floodplain;
        // Match scene_seed gauge coverage — G1 upstream / G3 downstream must sit inside the ROI
        // or matrix s0/s7 gauges read zeros forever.
        let mut origin_x = fp.min.0.min(fp.max.0) - 8;
        let mut origin_z = fp.min.1.min(fp.max.1) - 16;
        let mut x1 = fp.min.0.max(fp.max.0) + 8;
        let mut z1 = fp.min.1.max(fp.max.1) + 16;
        for g in &fixture.gauges {
            origin_x = origin_x.min(g.tile.0 - 4);
            origin_z = origin_z.min(g.tile.1 - 4);
            x1 = x1.max(g.tile.0 + 4);
            z1 = z1.max(g.tile.1 + 4);
        }
        req.pending = Some(SurfaceWaterRoiSpec {
            origin_x,
            origin_z,
            width: (x1 - origin_x).max(16) as u32,
            height: (z1 - origin_z).max(16) as u32,
            inflow_m3s: fixture.river_baseflow_m3s,
            min_release_m3s: fixture.river_baseflow_m3s * 0.1,
            river_center_x: Some(fixture.narrows_tile.0),
            river_half_width: RIVER_HALF_WIDTH,
            river_seed_depth_m: CORRIDOR_DEPTH_M,
            barrier_world_z: None,
            barrier_crest_m: 0.0,
            barrier_half_span: fixture.dam_span_tiles.max(4) / 2,
            synthetic_valley_if_no_dem: true,
        });
    }
}

/// Presentation-only corridor inject after bed+depth author (look, not gameplay).
pub fn author_dam_flood_valley_catalog_look_system(
    scene: Option<Res<ActiveTestScene>>,
    latch: Res<DamFloodValleyAuthorLatch>,
    grid: Option<Res<WorldWaterGrid>>,
    mut catalog: Option<ResMut<WaterSurfaceVisualCatalog>>,
    mut done: Local<bool>,
) {
    if *done || !latch.done || !scene.is_some_and(|s| s.0.is_dam_flood()) {
        return;
    }
    let Some(grid) = grid.as_ref() else {
        return;
    };
    let Some(cat) = catalog.as_mut() else {
        return;
    };
    let fixture = load_dam_valley_fixture();
    let cx = fixture.narrows_tile.0.max(0) as u32;
    let w = grid.width;
    let h = grid.height;
    **cat = WaterSurfaceVisualCatalog::default();
    cat.grid_width = w;
    cat.grid_height = h;
    cat.stamp = grid.stamp;
    cat.inject_ns_river_corridor(cx, 4, h.saturating_sub(4), RIVER_HALF_WIDTH);
    *done = true;
}

/// Nearest dry tile: prefer static depth thresholds; fall back to look class.
#[must_use]
pub fn nearest_dry_tile(grid: &WorldWaterGrid, origin: IVec2, max_r: i32) -> IVec2 {
    nearest_dry_tile_depth(grid, None, origin, max_r)
}

#[must_use]
pub fn nearest_dry_tile_depth(
    grid: &WorldWaterGrid,
    static_depth: Option<&WorldStaticWaterDepth>,
    origin: IVec2,
    max_r: i32,
) -> IVec2 {
    let is_wet = |p: IVec2| -> bool {
        if let Some(st) = static_depth {
            if let Some(wet) = st.is_wet_world(p.x, p.y) {
                return wet;
            }
        }
        grid.class_at(p.x, p.y)
            .is_some_and(WaterClass::is_open_water)
    };
    if !is_wet(origin) {
        return origin;
    }
    for r in 1..=max_r {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                let p = IVec2::new(origin.x + dx, origin.y + dy);
                if !is_wet(p) {
                    return p;
                }
            }
        }
    }
    for dy in -max_r..=max_r {
        for dx in -max_r..=max_r {
            let p = IVec2::new(origin.x + dx, origin.y + dy);
            if !is_wet(p) {
                return p;
            }
        }
    }
    let _ = WET_DEPTH_M;
    origin
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> ValleyProfile {
        ValleyProfile {
            river_x: 160,
            dam_z: 159.5,
            dam_half_span: 6,
            head_z: 204,
        }
    }

    /// Dam crest height above the lowest bed in its footprint (catalog `dam_embankment_v1`).
    const CREST_M: f32 = 40.0;

    #[test]
    fn floodplain_sits_below_the_crest_so_the_reservoir_can_cover_it() {
        let p = profile();
        // Forest floodplain upstream of the abutment hills: x 120..=200, z 169..=200.
        for z in 169..=200 {
            for x in 120..=200 {
                let above = p.above_channel_m(x, z);
                assert!(
                    above >= 0.0 && above < CREST_M - 10.0,
                    "({x},{z}) is {above} m above the bed, a {CREST_M} m dam cannot flood it"
                );
            }
        }
    }

    /// T-5: terrain stamping makes every cell at/under the water line a Lake (open water: no
    /// vegetation, `CELL_WATER`). The dry floodplain must stamp as land, the channel stays River.
    #[test]
    fn dry_valley_ground_stamps_as_land_not_lake() {
        use crate::terrain::generation::passes::stamp_water_class;
        let tuning = crate::terrain::biome::BiomeTuning::default();
        let (wl, range) = (tuning.shallow_water_height_max, 1200.0_f32);
        let bed0 = ValleyProfile::bed0_m(wl, range);
        let p = profile();
        let mut grid = WorldWaterGrid {
            width: 320,
            height: 320,
            classes: vec![WaterClass::Dry; 320 * 320],
            stamp: 0,
        };
        let mut m = ChunkCellMatrix::new(UVec2::new(320, 1));
        for z in [0_i32, 100, 169, 200, 319] {
            for x in 0..320_i32 {
                let river = (x - p.river_x).unsigned_abs() <= RIVER_HALF_WIDTH;
                let m_above = if river { 0.0 } else { p.above_channel_m(x, z) };
                let i = m.idx(x as u32, 0);
                m.elevation[i] = ((ValleyProfile::channel_bed_m(bed0, z) + m_above) / range).clamp(0.02, 0.98);
                grid.classes[(z as usize) * 320 + x as usize] =
                    if river { WaterClass::River } else { WaterClass::Dry };
            }
            stamp_water_class(&mut m, IVec2::new(0, z), wl, Some(&grid));
            for x in 0..320_i32 {
                let river = (x - p.river_x).unsigned_abs() <= RIVER_HALF_WIDTH;
                let got = m.water[m.idx(x as u32, 0)];
                assert_eq!(
                    got,
                    if river { WaterClass::River } else { WaterClass::Dry },
                    "({x},{z}) stamped {got:?}"
                );
            }
        }
    }

    #[test]
    fn abutments_close_the_narrows_above_crest() {
        let p = profile();
        for z in [159, 160] {
            for x in (0..=153).chain(166..320) {
                assert!(
                    p.above_channel_m(x, z) > CREST_M + 20.0,
                    "gap beside the dam at ({x},{z})"
                );
            }
        }
        // The span between the abutments is the dam's: terrace height, below crest (the
        // footprint's west end x=154 is the abutment foot itself).
        for x in 155..=165 {
            assert!(p.above_channel_m(x, 159) < CREST_M);
        }
    }

    #[test]
    fn valley_is_bounded_by_walls_and_head() {
        let p = profile();
        assert!(p.above_channel_m(160 + 70, 180) > CREST_M + 20.0, "east wall");
        assert!(p.above_channel_m(160 - 70, 180) > CREST_M + 20.0, "west wall");
        assert!(p.above_channel_m(140, 225) > CREST_M + 20.0, "valley head");
        assert_eq!(p.above_channel_m(160, 180), 0.0, "channel stays the bed");
    }

    #[test]
    fn channel_fall_is_the_slope_the_static_river_depth_needs() {
        // 128 ROI rows fall only a few metres: the 3.5 m static river is equilibrium on it.
        let drop = ValleyProfile::channel_bed_m(231.0, 224) - ValleyProfile::channel_bed_m(231.0, 96);
        assert!(drop > 0.0 && drop < 4.0, "drop {drop}");
    }
}
