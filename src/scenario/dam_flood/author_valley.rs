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
const BANK_ABOVE_WATER_LINE: f32 = 0.10;
/// Authored channel depth (metres) — matrix wet / inundation threshold uses [`WET_DEPTH_M`].
const CORRIDOR_DEPTH_M: f32 = 3.5;

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
    let river_bed = (water_line * 0.55).clamp(0.02, 0.45);
    let land_h = (water_line + BANK_ABOVE_WATER_LINE).clamp(0.08, 0.95);
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
                let along = (z as f32) / (h.max(1) as f32);
                dense.elevation[i] = (river_bed + along * 0.04).clamp(0.02, water_line * 0.9);
                dense.moisture[i] = 0.95;
                if let Some(st) = static_depth.as_mut() {
                    st.depth_m[i] = CORRIDOR_DEPTH_M;
                }
            } else {
                // Dry banks: raise bed; depth = 0 (no class wipe as lake hack).
                let bank = land_h + (dx.saturating_sub(RIVER_HALF_WIDTH) as f32) * 0.012;
                dense.elevation[i] = bank.clamp(land_h, 0.98);
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
