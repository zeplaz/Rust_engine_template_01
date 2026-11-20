//! PlayQuad quadrant counts for `sim_run_dump` (read-only snapshots).
//!
//! PLAYTEST-QUADRANT-WORLD-001 P6 — uses [`crate::engine::playquad`] as the sole partition.

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use serde_json::{json, Map, Value};

use crate::engine::launch_args::TestScene;
use crate::engine::playquad::{quadrant_aabb, quadrant_tile_rect, PlayQuadrant};
use crate::strategic::{ConstructionSite, SiteFootprint};
use crate::systems::ecology::{ChunkCellStand, CELL_BURNING};
use crate::systems::fire::ChunkFireOverlay;
use crate::systems::sim_control::SimTimeMicros;
use crate::systems::weather::WeatherField;
use crate::terrain::generation::water_class::WaterClass;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::world_scale_contract::CHUNK_TILES_SIM;

/// Wetland-within-R of river/lake in SE — matches `water_class::tests::score_playquad_seed` (R = 6).
const SE_WETLAND_REACH: i32 = 6;
/// Chebyshev radius for “lake touches river” on the stamped grid (PlayQuad rhythm default).
const LAKE_RIVER_ATTACH_RADIUS: i32 = 2;

#[derive(Default)]
struct QuadrantAcc {
    land: u32,
    river: u32,
    lake: u32,
    ocean: u32,
    wetland: u32,
    lake_river_attached: u32,
    wetland_near_water: u32,
    veg_burning_cells: u32,
    fire_heat_cells: u32,
    sites: u32,
    sites_by_phase: BTreeMap<String, u32>,
}

fn q_key(q: PlayQuadrant) -> &'static str {
    match q {
        PlayQuadrant::Nw => "nw",
        PlayQuadrant::Ne => "ne",
        PlayQuadrant::Sw => "sw",
        PlayQuadrant::Se => "se",
    }
}

fn tile_in_quadrant(q: PlayQuadrant, tx: u32, ty: u32, w: u32, h: u32) -> bool {
    quadrant_tile_rect(q, w, h).contains(tx, ty)
}

fn class_at_grid(
    grid: &[(bool, WaterClass)],
    w: usize,
    x: i32,
    y: i32,
) -> Option<WaterClass> {
    if x < 0 || y < 0 || x >= w as i32 || y >= w as i32 {
        return None;
    }
    let idx = y as usize * w + x as usize;
    grid.get(idx).map(|(_, c)| *c)
}

fn near_river_or_lake(grid: &[(bool, WaterClass)], w: usize, x: u32, y: u32, reach: i32) -> bool {
    let (xi, yi) = (x as i32, y as i32);
    (-reach..=reach).any(|dy| {
        (-reach..=reach).any(|dx| {
            if dx * dx + dy * dy > reach * reach {
                return false;
            }
            matches!(
                class_at_grid(grid, w, xi + dx, yi + dy),
                Some(WaterClass::River | WaterClass::Lake)
            )
        })
    })
}

fn lake_cell_touches_river(grid: &[(bool, WaterClass)], w: usize, x: u32, y: u32, radius: i32) -> bool {
    let (xi, yi) = (x as i32, y as i32);
    (-radius..=radius).any(|dy| {
        (-radius..=radius).any(|dx| {
            class_at_grid(grid, w, xi + dx, yi + dy) == Some(WaterClass::River)
        })
    })
}

fn bump_water(acc: &mut QuadrantAcc, class: WaterClass) {
    match class {
        WaterClass::Dry => acc.land += 1,
        WaterClass::Wetland => {
            acc.land += 1;
            acc.wetland += 1;
        }
        WaterClass::River => acc.river += 1,
        WaterClass::Lake => acc.lake += 1,
        WaterClass::Ocean => acc.ocean += 1,
    }
}

/// `None` when the active scene is not PlayQuad or the world is not ready.
#[must_use]
pub fn quadrants_json(
    scene: TestScene,
    w: u32,
    h: u32,
    meters_per_tile: f32,
    matrices: &Query<'_, '_, (&Chunk, &ChunkCellMatrix)>,
    stands: &Query<'_, '_, (&Chunk, &ChunkCellStand)>,
    overlays: &Query<'_, '_, (&Chunk, &ChunkFireOverlay)>,
    sites: &Query<'_, '_, (&ConstructionSite, &SiteFootprint)>,
    weather_field: Option<&WeatherField>,
    sim_time: Option<&SimTimeMicros>,
) -> Option<Value> {
    if !scene.is_playquad() || w == 0 || h == 0 || matrices.is_empty() {
        return None;
    }

    let mut acc: HashMap<PlayQuadrant, QuadrantAcc> = HashMap::new();
    for q in PlayQuadrant::ALL {
        acc.insert(q, QuadrantAcc::default());
    }

    let mut grid: Vec<(bool, WaterClass)> = vec![(false, WaterClass::Dry); (w as usize) * (h as usize)];
    let mut global_wetland = 0u32;

    let cw = CHUNK_TILES_SIM;
    let ch = CHUNK_TILES_SIM;

    for (chunk, matrix) in matrices {
        let ox = chunk.coord.x as u32 * cw;
        let oy = chunk.coord.y as u32 * ch;
        for ly in 0..matrix.size.y {
            for lx in 0..matrix.size.x {
                let tx = ox + lx;
                let ty = oy + ly;
                if tx >= w || ty >= h {
                    continue;
                }
                let class = matrix.water[matrix.idx(lx, ly)];
                let gidx = ty as usize * w as usize + tx as usize;
                grid[gidx] = (true, class);
                if class == WaterClass::Wetland {
                    global_wetland = global_wetland.saturating_add(1);
                }
                for q in PlayQuadrant::ALL {
                    if !tile_in_quadrant(q, tx, ty, w, h) {
                        continue;
                    }
                    let a = acc.get_mut(&q).expect("quadrant acc");
                    bump_water(a, class);
                }
            }
        }
    }

    // SW lake cells that touch a river (PQ-H2 dump mirror).
    for y in 0..h {
        for x in 0..w {
            let gidx = y as usize * w as usize + x as usize;
            let (_, class) = grid[gidx];
            if class != WaterClass::Lake || !tile_in_quadrant(PlayQuadrant::Sw, x, y, w, h) {
                continue;
            }
            if lake_cell_touches_river(&grid, w as usize, x, y, LAKE_RIVER_ATTACH_RADIUS) {
                acc.get_mut(&PlayQuadrant::Sw).expect("sw").lake_river_attached += 1;
            }
        }
    }

    // SE marsh belt near river/lake (PQ-H3).
    let mut se_wetland_near_water = 0u32;
    for y in 0..h {
        for x in 0..w {
            let gidx = y as usize * w as usize + x as usize;
            if grid[gidx].1 != WaterClass::Wetland || !tile_in_quadrant(PlayQuadrant::Se, x, y, w, h) {
                continue;
            }
            if near_river_or_lake(&grid, w as usize, x, y, SE_WETLAND_REACH) {
                se_wetland_near_water += 1;
            }
        }
    }
    if let Some(se) = acc.get_mut(&PlayQuadrant::Se) {
        se.wetland_near_water = se_wetland_near_water;
    }

    for (chunk, stand) in stands {
        let ox = chunk.coord.x as u32 * cw;
        let oy = chunk.coord.y as u32 * ch;
        for (i, cell) in stand.render.iter().enumerate() {
            if *cell != CELL_BURNING {
                continue;
            }
            let lx = (i as u32) % cw;
            let ly = (i as u32) / cw;
            let tx = ox + lx;
            let ty = oy + ly;
            if tx >= w || ty >= h {
                continue;
            }
            for q in PlayQuadrant::ALL {
                if tile_in_quadrant(q, tx, ty, w, h) {
                    acc.get_mut(&q).expect("q").veg_burning_cells += 1;
                }
            }
        }
    }

    const HEAT_EPS: f32 = 0.04;
    for (chunk, overlay) in overlays {
        let ox = chunk.coord.x as u32 * cw;
        let oy = chunk.coord.y as u32 * ch;
        for (i, hcell) in overlay.heat.iter().enumerate() {
            if *hcell <= HEAT_EPS {
                continue;
            }
            let lx = (i as u32) % cw;
            let ly = (i as u32) / cw;
            let tx = ox + lx;
            let ty = oy + ly;
            if tx >= w || ty >= h {
                continue;
            }
            for q in PlayQuadrant::ALL {
                if tile_in_quadrant(q, tx, ty, w, h) {
                    acc.get_mut(&q).expect("q").fire_heat_cells += 1;
                }
            }
        }
    }

    for (site, footprint) in sites {
        let (tx, ty) = site_centroid_tile(footprint);
        let phase = format!("{:?}", site.phase);
        for q in PlayQuadrant::ALL {
            if tile_in_quadrant(q, tx, ty, w, h) {
                let a = acc.get_mut(&q).expect("q");
                a.sites += 1;
                *a.sites_by_phase.entry(phase.clone()).or_default() += 1;
            }
        }
    }

    let mut out = Map::new();
    out.insert("global_wetland_cells".into(), json!(global_wetland));
    out.insert("se_wetland_near_water".into(), json!(se_wetland_near_water));

    let sim_time = sim_time.map(|t| t.0).unwrap_or(0);
    let mpt = meters_per_tile.max(1.0);

    for q in PlayQuadrant::ALL {
        let a = acc.remove(&q).unwrap_or_default();
        let center = quadrant_aabb(q, w, h).center();
        let (rain_mm_h, rain01) = weather_field
            .filter(|f| f.is_ready())
            .map(|f| {
                let pos_m = center * mpt;
                let sample = f.sample(pos_m, crate::systems::sim_control::SimTimeMicros(sim_time));
                (
                    sample.rain_mm_h,
                    (sample.rain_mm_h / 10.0).clamp(0.0, 1.0),
                )
            })
            .unwrap_or((0.0, 0.0));

        let mut block = Map::new();
        block.insert("land_cells".into(), json!(a.land));
        block.insert("river_cells".into(), json!(a.river));
        block.insert("lake_cells".into(), json!(a.lake));
        block.insert("ocean_cells".into(), json!(a.ocean));
        block.insert("wetland_cells".into(), json!(a.wetland));
        block.insert("lake_river_attached".into(), json!(a.lake_river_attached));
        if q == PlayQuadrant::Se {
            block.insert("wetland_near_water".into(), json!(a.wetland_near_water));
        }
        block.insert("veg_burning_cells".into(), json!(a.veg_burning_cells));
        block.insert("fire_heat_cells".into(), json!(a.fire_heat_cells));
        block.insert("sites".into(), json!(a.sites));
        block.insert("sites_by_phase".into(), json!(a.sites_by_phase));
        block.insert(
            "rain_at_center".into(),
            json!({
                "tile": [center.x.floor() as i32, center.y.floor() as i32],
                "rain_mm_h": rain_mm_h,
                "rain01": rain01,
            }),
        );
        out.insert(q_key(q).into(), Value::Object(block));
    }

    Some(Value::Object(out))
}

fn site_centroid_tile(footprint: &SiteFootprint) -> (u32, u32) {
    if footprint.tiles.is_empty() {
        return (0, 0);
    }
    let (sx, sy) = footprint.tiles.iter().fold((0i64, 0i64), |(ax, ay), t| {
        (ax + i64::from(t.x), ay + i64::from(t.y))
    });
    let n = footprint.tiles.len() as i64;
    (
        (sx / n).max(0) as u32,
        (sy / n).max(0) as u32,
    )
}

use crate::dev::sim_run_dump::{SimDumpArm, SimRunDump};
use crate::engine::EngineLaunchArgs;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// CQ-68 PlayQuad reporter — no cross-domain SystemParam on the central flush.
pub fn dump_quadrants_section(
    arm: Res<SimDumpArm>,
    launch: Option<Res<EngineLaunchArgs>>,
    params: Option<Res<WorldGenParams>>,
    matrices: Query<(&Chunk, &ChunkCellMatrix)>,
    stands: Query<(&Chunk, &ChunkCellStand)>,
    overlays: Query<(&Chunk, &ChunkFireOverlay)>,
    sites: Query<(&ConstructionSite, &SiteFootprint)>,
    weather_field: Option<Res<WeatherField>>,
    sim_time: Option<Res<SimTimeMicros>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let scene = launch.as_ref().map(|l| l.test_scene).unwrap_or_default();
    let world_w = params.as_ref().map_or(0, |p| p.width);
    let world_h = params.as_ref().map_or(0, |p| p.height);
    let mpt = params
        .as_ref()
        .map_or(100.0, |p| p.map_scale.meters_per_tile);
    if let Some(block) = quadrants_json(
        scene,
        world_w,
        world_h,
        mpt,
        &matrices,
        &stands,
        &overlays,
        &sites,
        weather_field.as_deref(),
        sim_time.as_deref(),
    ) {
        dump.put_value("quadrants", block);
    }
}
