//! RD-2 off-road speed field (PLAN-ROADS-v1): how fast a wheeled vehicle crosses each tile off the
//! road network, as a multiplier on the off-road speed (`road_tiers.ron` `offroad_speed_kmh`).
//! `0` is a wall (too steep, deep water, a `blocked` mobility rule).
//!
//! Inputs, each from its owner (read only):
//! - land: the cell's tags through the `wheeled_logistics` mobility profile
//!   ([`crate::terrain::mobility::evaluate_tile`]; cost multiplier → speed, `blocked` → wall);
//! - slope: the dense terrain height (ground deformation included) against the profile's `max_grade`;
//! - water: [`WaterTileProbe`] depth — a shallow ford is slow, deeper than `ford_max_m` is a wall;
//! - rubble / craters: [`DebrisRouteBlockBook`] tiles;
//! - wetness: chunk weather soil moisture (mud) and snow cover.
//!
//! Sole writer: [`publish_offroad_speed_field_system`] (rebuilt when terrain changes and every
//! [`REBUILD_EVERY_STEPS`] sim steps for water and weather). Readers: the cross-country truck path
//! and its trip time ([`super::road_vehicles_motion`]), road on/off ramps
//! ([`crate::economy::logistics::routes::truck_road_route`]) and formations
//! ([`crate::combat::formation::advance_formations_system`]).

use std::collections::HashSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::logistics_floodfill::LogisticsTile;
use crate::economy::logistics::debris_block::DebrisRouteBlockBook;
use crate::infrastructure::road_tier::OffroadTuning;
use crate::systems::sim_control::SimStepClock;
use crate::systems::terrain::material_plugin::TerrainRegistriesHandles;
use crate::systems::water::WaterTileProbe;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix, WorldGenDenseTerrainCache};
use crate::terrain::material::{TagRegistry, TagSet};
use crate::terrain::mobility::{evaluate_tile, MobilityProfile, MobilityProfileRegistry};
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

/// Water and weather move; the field is rebuilt this often (sim steps) even when terrain did not,
/// but at most once per [`MIN_REBUILD_REAL_S`] of real time (time skip runs many steps a frame).
pub const REBUILD_EVERY_STEPS: u64 = 10;
/// Real seconds between rebuilds that terrain did not force.
pub const MIN_REBUILD_REAL_S: f32 = 5.0;
/// Mobility profile trucks use.
pub const WHEELED_PROFILE: &str = "wheeled_logistics";
/// Slowest a passable tile gets (keeps times finite).
pub const MIN_PASSABLE_MUL: f32 = 0.05;

/// Speed multiplier per tile (row-major `width × height`); empty until terrain is ready.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct OffroadSpeedField {
    pub width: u32,
    pub height: u32,
    pub mul: Vec<f32>,
    pub revision: u64,
    /// Wall tiles (dump).
    pub walls: u32,
    /// Times the field was recomputed (dump: rebuild cost watch).
    pub rebuilds: u32,
}

impl OffroadSpeedField {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mul.is_empty()
    }

    /// Multiplier at a tile: `1` while the field is empty, `0` off the map.
    #[must_use]
    pub fn mul_at(&self, t: IVec2) -> f32 {
        if self.is_empty() {
            return 1.0;
        }
        if t.x < 0 || t.y < 0 || t.x >= self.width as i32 || t.y >= self.height as i32 {
            return 0.0;
        }
        self.mul[(t.y * self.width as i32 + t.x) as usize]
    }

    /// Path-grid cell for the cross-country planner (time cost = 1 / multiplier).
    #[must_use]
    pub fn logistics_tile(&self, t: IVec2) -> LogisticsTile {
        let m = self.mul_at(t);
        LogisticsTile { blocked: m <= 0.0, cost_mul: 1.0 / m.max(MIN_PASSABLE_MUL), env_risk: 0.0 }
    }

    /// Mean multiplier over the straight stretch `a → b` (tile units), by time (harmonic mean of
    /// tile samples); `0` if it crosses a wall.
    #[must_use]
    pub fn line_mul(&self, a: Vec2, b: Vec2) -> f32 {
        if self.is_empty() {
            return 1.0;
        }
        let len = a.distance(b);
        let n = (len.ceil() as usize).max(1);
        let mut inv = 0.0;
        for i in 0..n {
            let p = a.lerp(b, (i as f32 + 0.5) / n as f32);
            let m = self.mul_at(p.floor().as_ivec2());
            if m <= 0.0 {
                return 0.0;
            }
            inv += 1.0 / m;
        }
        n as f32 / inv
    }
}

/// One tile's multiplier from its land rule result, water depth, rubble and wetness.
#[must_use]
pub fn offroad_cell_mul(
    land_cost_mul: f32,
    land_blocked: bool,
    depth_m: f32,
    rubble: bool,
    soil_moisture: f32,
    snow: f32,
    t: &OffroadTuning,
) -> f32 {
    if land_blocked || depth_m > t.ford_max_m {
        return 0.0;
    }
    let mut m = 1.0 / land_cost_mul.max(1e-3);
    if depth_m > t.wet_m {
        m *= t.ford_mul;
    }
    if rubble {
        m *= t.rubble_mul;
    }
    let mud = ((soil_moisture - t.mud_from_moisture) / (1.0 - t.mud_from_moisture).max(1e-3)).clamp(0.0, 1.0);
    m *= 1.0 - t.mud_slow * mud;
    m *= 1.0 - t.snow_slow * snow.clamp(0.0, 1.0);
    m.clamp(MIN_PASSABLE_MUL, 1.0)
}

/// Steepest grade to a 4-neighbour (rise / run), from normalised heights × `range_m`.
#[must_use]
pub fn tile_grade(elev: &[f32], w: i32, h: i32, x: i32, y: i32, range_m: f32) -> f32 {
    let at = |x: i32, y: i32| elev[(y * w + x) as usize];
    let h0 = at(x, y);
    let mut g: f32 = 0.0;
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let (nx, ny) = (x + dx, y + dy);
        if nx >= 0 && ny >= 0 && nx < w && ny < h {
            g = g.max((at(nx, ny) - h0).abs() * range_m / TILE_METRES_GAMEPLAY);
        }
    }
    g
}

/// Terrain inputs of the field (one domain: ground shape and cover; read only).
#[derive(SystemParam)]
pub struct OffroadTerrainReads<'w, 's> {
    dense: Option<Res<'w, WorldGenDenseTerrainCache>>,
    params: Option<Res<'w, WorldGenParams>>,
    chunks: Query<'w, 's, (Ref<'static, ChunkCellMatrix>, &'static Chunk, Option<&'static ChunkWeather>)>,
    handles: Option<Res<'w, TerrainRegistriesHandles>>,
    tag_assets: Option<Res<'w, Assets<TagRegistry>>>,
    mobility_assets: Option<Res<'w, Assets<MobilityProfileRegistry>>>,
}

/// Default truck profile when the registries are not loaded (slope limit only).
fn fallback_profile() -> MobilityProfile {
    MobilityProfile {
        id: WHEELED_PROFILE.into(),
        traction_requirement: 0.0,
        max_grade: 0.4,
        amphibious: false,
        rules: Vec::new(),
    }
}

/// Sole writer of [`OffroadSpeedField`].
pub fn publish_offroad_speed_field_system(
    terrain: OffroadTerrainReads,
    water: WaterTileProbe,
    debris: Option<Res<DebrisRouteBlockBook>>,
    clock: Option<Res<SimStepClock>>,
    time: Res<Time<Real>>,
    mut field: ResMut<OffroadSpeedField>,
    mut last: Local<Option<(u64, f32)>>,
) {
    let (Some(dense), Some(params)) = (terrain.dense.as_deref(), terrain.params.as_deref()) else {
        return;
    };
    if !dense.is_live_height_grid() {
        return;
    }
    let step = clock.as_ref().map_or(0, |c| c.steps);
    let terrain_changed = terrain.dense.as_ref().is_some_and(|d| d.is_changed())
        || debris.as_ref().is_some_and(|d| d.is_changed());
    let now = time.elapsed_secs();
    let due = last.is_none_or(|(s, t)| step >= s + REBUILD_EVERY_STEPS && now >= t + MIN_REBUILD_REAL_S);
    if !field.is_empty() && !terrain_changed && !due {
        return;
    }
    *last = Some((step, now));
    field.rebuilds = field.rebuilds.wrapping_add(1);

    let (w, h) = (dense.width as i32, dense.height as i32);
    let tuning = &crate::infrastructure::road_tier_table().offroad;
    let tag_reg = terrain
        .handles
        .as_deref()
        .zip(terrain.tag_assets.as_deref())
        .and_then(|(hd, a)| a.get(&hd.tag_registry));
    let profile = terrain
        .handles
        .as_deref()
        .zip(terrain.mobility_assets.as_deref())
        .and_then(|(hd, a)| a.get(&hd.mobility_profiles))
        .and_then(|reg| reg.profiles.iter().find(|p| p.id == WHEELED_PROFILE).cloned())
        .unwrap_or_else(fallback_profile);
    let rubble: HashSet<IVec2> = debris
        .as_deref()
        .map(|b| {
            b.blocked
                .iter()
                .filter_map(|k| {
                    let (x, z) = k.strip_prefix('t')?.split_once('_')?;
                    Some(IVec2::new(x.parse().ok()?, z.parse().ok()?))
                })
                .collect()
        })
        .unwrap_or_default();

    // Per tile: land rule result (tags) and wetness, from the chunk cells.
    let n = (w * h) as usize;
    let mut tags: Vec<Option<TagSet>> = vec![None; n];
    let mut wet: Vec<(f32, f32)> = vec![(0.45, 0.0); n];
    for (matrix, chunk, weather) in &terrain.chunks {
        let origin = chunk.coord * CHUNK_TILES_SIM as i32;
        let (soil, snow) = weather.map_or((0.45, 0.0), |c| (c.soil_moisture, c.snow_depth));
        for ly in 0..matrix.size.y as i32 {
            for lx in 0..matrix.size.x as i32 {
                let t = origin + IVec2::new(lx, ly);
                if t.x < 0 || t.y < 0 || t.x >= w || t.y >= h {
                    continue;
                }
                let i = (t.y * w + t.x) as usize;
                tags[i] = matrix.tags.get((ly * matrix.size.x as i32 + lx) as usize).copied();
                wet[i] = (soil, snow);
            }
        }
    }

    let mut mul = Vec::with_capacity(n);
    let mut walls = 0;
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let grade = tile_grade(&dense.elevation, w, h, x, y, params.elevation_range_m);
            let (cost, blocked) = match (tag_reg, tags[i].as_ref()) {
                (Some(reg), Some(ts)) => {
                    let hint = evaluate_tile(&profile, ts, grade, 1.0, reg, 1.0);
                    (hint.cost_mul, hint.blocked)
                }
                _ => (1.0, grade > profile.max_grade),
            };
            let t = IVec2::new(x, y);
            let m = offroad_cell_mul(cost, blocked, water.depth_m(t), rubble.contains(&t), wet[i].0, wet[i].1, tuning);
            walls += u32::from(m <= 0.0);
            mul.push(m);
        }
    }
    if field.width != dense.width || field.height != dense.height || field.mul != mul {
        field.width = dense.width;
        field.height = dense.height;
        field.mul = mul;
        field.walls = walls;
        field.revision = field.revision.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning() -> OffroadTuning {
        OffroadTuning::default()
    }

    #[test]
    fn deep_water_and_blocked_land_are_walls_a_ford_is_slow() {
        let t = tuning();
        assert_eq!(offroad_cell_mul(1.0, false, t.ford_max_m + 0.1, false, 0.45, 0.0, &t), 0.0);
        assert_eq!(offroad_cell_mul(1.0, true, 0.0, false, 0.45, 0.0, &t), 0.0);
        let dry = offroad_cell_mul(1.0, false, 0.0, false, 0.45, 0.0, &t);
        let ford = offroad_cell_mul(1.0, false, t.ford_max_m * 0.5, false, 0.45, 0.0, &t);
        assert!((dry - 1.0).abs() < 1e-5 && ford < dry * 0.5, "dry {dry} ford {ford}");
    }

    #[test]
    fn forest_mud_snow_and_rubble_slow_a_tile() {
        let t = tuning();
        let open = offroad_cell_mul(1.0, false, 0.0, false, 0.45, 0.0, &t);
        let forest = offroad_cell_mul(1.5, false, 0.0, false, 0.45, 0.0, &t);
        let mud = offroad_cell_mul(1.0, false, 0.0, false, 1.0, 0.0, &t);
        let snow = offroad_cell_mul(1.0, false, 0.0, false, 0.45, 1.0, &t);
        let rubble = offroad_cell_mul(1.0, false, 0.0, true, 0.45, 0.0, &t);
        for (name, m) in [("forest", forest), ("mud", mud), ("snow", snow), ("rubble", rubble)] {
            assert!(m < open && m >= MIN_PASSABLE_MUL, "{name} {m}");
        }
    }

    #[test]
    fn a_line_is_timed_by_its_slow_tiles_and_stopped_by_a_wall() {
        let mut f = OffroadSpeedField { width: 4, height: 1, mul: vec![1.0, 0.5, 1.0, 1.0], ..Default::default() };
        let m = f.line_mul(Vec2::new(0.5, 0.5), Vec2::new(3.5, 0.5));
        assert!(m < 1.0 && m > 0.5, "harmonic mean of 1, 0.5, 1: {m}");
        f.mul[2] = 0.0;
        assert_eq!(f.line_mul(Vec2::new(0.5, 0.5), Vec2::new(3.5, 0.5)), 0.0);
        assert!(f.logistics_tile(IVec2::new(2, 0)).blocked);
        assert_eq!(OffroadSpeedField::default().line_mul(Vec2::ZERO, Vec2::ONE), 1.0, "empty field = plain off-road");
    }

    #[test]
    fn steep_ground_reads_as_a_steep_grade() {
        // 1 normalised unit × 100 m range over one 16 m tile.
        let elev = vec![0.0, 1.0, 0.0, 0.0];
        let g = tile_grade(&elev, 2, 2, 0, 0, 100.0);
        assert!((g - 100.0 / TILE_METRES_GAMEPLAY).abs() < 1e-4, "{g}");
    }

    /// A 20 × 12 map with a river (wall) down x = 10, fordable only at y = 10.
    fn river_map() -> OffroadSpeedField {
        let (w, h) = (20u32, 12u32);
        let mut mul = vec![1.0; (w * h) as usize];
        for y in 0..h {
            if y != 10 {
                mul[(y * w + 10) as usize] = 0.0;
            }
        }
        mul[(10 * w + 10) as usize] = 0.35;
        OffroadSpeedField { width: w, height: h, mul, ..Default::default() }
    }

    #[test]
    fn the_cross_country_path_goes_round_a_river_through_its_ford() {
        use crate::systems::navigation::{plan_truck_tile_waypoints, LogisticsNodeArray, LogisticsPathConcern};
        let field = river_map();
        let mut scratch = LogisticsNodeArray::default();
        let (a, b) = (Vec2::new(3.5, 2.5), Vec2::new(16.5, 2.5));
        let pts = plan_truck_tile_waypoints(&mut scratch, a, b, LogisticsPathConcern::DEFAULT, None, 0, Some(&field));
        let crosses_at_ford = pts.windows(2).any(|w| {
            let (lo, hi) = (w[0].x.min(w[1].x), w[0].x.max(w[1].x));
            w[0].y.floor() == 10.0 && w[1].y.floor() == 10.0 && lo < 10.0 && hi > 11.0
        });
        assert!(crosses_at_ford, "crosses the river at the ford row: {pts:?}");
        for w in pts.windows(2) {
            assert!(field.line_mul(w[0], w[1]) > 0.0, "no stretch through the river: {w:?}");
        }
        // Timed over the field: slower than the straight line at the plain off-road speed.
        let route = crate::systems::navigation::offroad_route(pts, 5.0, Some(&field));
        let straight = crate::economy::logistics::routes::TruckRoute::uniform(vec![a, b], 5.0);
        assert!(route.time_s(12.0) > straight.time_s(12.0) * 1.5, "the detour and ford cost time");
    }

    #[test]
    fn a_formation_steps_round_a_wall_and_holds_when_boxed_in() {
        use crate::combat::formation::next_formation_step;
        let field = river_map();
        // Heading east into the river: sidesteps along y toward a target that is also south.
        assert_eq!(next_formation_step(IVec2::new(9, 3), IVec2::new(15, 5), Some(&field)), Some(IVec2::new(9, 4)));
        // Straight east into the wall with nothing to the side: holds.
        assert_eq!(next_formation_step(IVec2::new(9, 3), IVec2::new(15, 3), Some(&field)), None);
        // At the ford it crosses.
        assert_eq!(next_formation_step(IVec2::new(9, 10), IVec2::new(15, 10), Some(&field)), Some(IVec2::new(10, 10)));
    }
}
