//! Load-aware speed modifiers + NAV-06 path smoothing + MOVE-08 output filtering.
//!
//! Dual-cite: NEW **NAV-06** (aig3 4.4.6–4.4.7) · **MOVE-08** (aig3 output filtering /
//! capability-sensitive steering) · LEGACY `pathfinding` / `steering` (aig_e3).
//! Tile search stays on [`super::logistics_floodfill`]; motion consumers call
//! [`smooth_vehicle_tile_path`] then [`filter_vehicle_steering_output`] so trucks/buses
//! respect turn rate and accel limits (no jitter spikes).

use bevy::prelude::*;

use crate::economy::logistics::orders::{
    LogisticsOrderSimRes, LogisticsSiteRegistry, SiteId, SiteRole, VehicleId, VehiclePhase,
};
use crate::entities::vehicles::runtime::{Bus, Truck};
use crate::entities::components::Waypoints;
use crate::systems::sim_control::{SimControlState, SimStepClock};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;
use crate::traits::LoadBasedSpeedModifier;

use super::logistics_floodfill::{logistics_grid_path, smooth_logistics_tile_path, LogisticsNodeArray, LogisticsTile};
use super::path_concerns::{front_haul_path_concern, LogisticsPathConcern, LogisticsPathConcernRegistry};
use super::cost_forecast::LogisticsCostForecastBook;
use crate::economy::logistics::orders::{PlannedLeg, SiteLegRouteBook};
use crate::economy::logistics::routes::{TruckRoadNetwork, TruckRoute};

/// Marks an entity whose waypoint motion is filtered by [`filter_vehicle_steering_output`].
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct RoadVehicleKinematics {
    pub velocity: Vec2,
    pub heading: Vec2,
}

/// Tracks which order-sim leg produced the current waypoint polyline (presentation motion only).
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct TruckMotionLeg {
    pub vehicle_id: VehicleId,
    pub leg_key: u64,
    /// [`PlannedLeg::version`] the waypoints were copied from (0 = straight-line placeholder).
    pub route_version: u64,
}

/// Reused NAV-04 node array for truck tile path queries (sole writer: [`plan_site_leg_routes_system`]).
#[derive(Resource, Default)]
pub struct LogisticsGridPathScratch {
    pub nodes: LogisticsNodeArray,
}

impl LoadBasedSpeedModifier for Truck {
    fn speed_modifier(&self) -> f32 {
        let cap = self.vehicle.capacity.max(1) as f32;
        let load_ratio = self.current_load / cap;
        0.7 + (1.0 - load_ratio) * 0.7
    }
}

impl LoadBasedSpeedModifier for Bus {
    fn speed_modifier(&self) -> f32 {
        let cap = self.capacity.max(1) as f32;
        let passenger_ratio = self.passengers as f32 / cap;
        0.8 + (1.0 - passenger_ratio) * 0.8
    }
}

/// NAV-06: string-pull a logistics tile path for vehicle motion (open corridors → endpoints).
#[must_use]
pub fn smooth_vehicle_tile_path(
    path: &[(i32, i32)],
    width: i32,
    height: i32,
    tile_at: &mut impl FnMut(i32, i32) -> LogisticsTile,
) -> Vec<(i32, i32)> {
    smooth_logistics_tile_path(path, width, height, tile_at)
}

/// MOVE-08 capability limits for heavy vehicles (world units / sec, rad / sec).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleMotionLimits {
    pub max_speed: f32,
    pub max_accel: f32,
    pub max_turn_rate: f32,
}

impl VehicleMotionLimits {
    pub const TRUCK: Self = Self {
        max_speed: 8.0,
        max_accel: 3.0,
        max_turn_rate: 1.2,
    };
    pub const BUS: Self = Self {
        max_speed: 6.5,
        max_accel: 2.2,
        max_turn_rate: 0.9,
    };
}

/// MOVE-08: filter desired velocity by accel + turn-rate caps (capability-sensitive).
///
/// `heading` is the current facing unit vector (or zero → snap to desired direction).
/// Returns (filtered_velocity, new_heading_unit).
#[must_use]
pub fn filter_vehicle_steering_output(
    current_vel: Vec2,
    desired_vel: Vec2,
    heading: Vec2,
    limits: VehicleMotionLimits,
    dt: f32,
) -> (Vec2, Vec2) {
    let dt = dt.max(1e-4);
    let mut desired = desired_vel;
    let des_speed = desired.length();
    if des_speed > limits.max_speed {
        desired *= limits.max_speed / des_speed;
    }

    // Accel clamp toward desired.
    let delta = desired - current_vel;
    let max_delta = limits.max_accel * dt;
    let filtered = if delta.length_squared() > max_delta * max_delta {
        current_vel + delta.normalize_or_zero() * max_delta
    } else {
        desired
    };

    // Turn-rate clamp on heading.
    let want = if filtered.length_squared() > 1e-8 {
        filtered.normalize()
    } else if desired.length_squared() > 1e-8 {
        desired.normalize()
    } else {
        heading
    };
    let h = if heading.length_squared() > 1e-8 {
        heading.normalize()
    } else {
        want
    };
    let cross = h.x * want.y - h.y * want.x;
    let dot = h.dot(want).clamp(-1.0, 1.0);
    let ang = cross.atan2(dot);
    let max_ang = limits.max_turn_rate * dt;
    let applied = ang.clamp(-max_ang, max_ang);
    let (s, c) = applied.sin_cos();
    let new_heading = Vec2::new(h.x * c - h.y * s, h.x * s + h.y * c).normalize_or_zero();

    // Project filtered speed along new heading (no sideways jitter).
    let speed = filtered.length().min(limits.max_speed);
    (new_heading * speed, new_heading)
}

fn site_xz(registry: &LogisticsSiteRegistry, site: SiteId) -> Option<Vec2> {
    registry
        .travel
        .positions
        .get(&site)
        .map(|p| Vec2::new(p[0], p[1]))
        .or_else(|| registry.meta.get(&site).map(|m| Vec2::new(m.world_xz[0], m.world_xz[1])))
}

/// Grid cell of a tile-space position (cell = floor).
fn xz_to_grid(xz: Vec2) -> (i32, i32) {
    (xz.x.floor() as i32, xz.y.floor() as i32)
}

/// Identifies the order-sim leg a truck's waypoints were planned for.
#[must_use]
pub fn truck_leg_key(v: &crate::economy::logistics::orders::Vehicle) -> u64 {
    let Some(a) = v.assignment.as_ref() else {
        return 0;
    };
    let phase = match v.phase {
        VehiclePhase::Idle => 0u64,
        VehiclePhase::ToPickup(_) => 1,
        VehiclePhase::Loading(_) => 2,
        VehiclePhase::Hauling(_) => 3,
        VehiclePhase::Unloading(_) => 4,
    };
    (u64::from(a.order.0) << 16) | (u64::from(a.sink.0) << 8) | phase
}

fn concern_for_sink(
    concerns: &LogisticsPathConcernRegistry,
    registry: &LogisticsSiteRegistry,
    sink: SiteId,
) -> LogisticsPathConcern {
    match registry.meta.get(&sink).map(|m| m.role) {
        Some(SiteRole::Front) => front_haul_path_concern(concerns),
        _ => concerns.default,
    }
}

/// Cross-country waypoint polyline for a truck leg; positions in **tile units**, cell centres out.
#[must_use]
pub fn plan_truck_tile_waypoints(
    scratch: &mut LogisticsNodeArray,
    from_xz: Vec2,
    to_xz: Vec2,
    concern: LogisticsPathConcern,
    forecast: Option<&LogisticsCostForecastBook>,
    depart_tick: u64,
) -> Vec<Vec2> {
    let (sx, sy) = xz_to_grid(from_xz);
    let (gx, gy) = xz_to_grid(to_xz);
    let min_x = sx.min(gx);
    let max_x = sx.max(gx);
    let min_y = sy.min(gy);
    let max_y = sy.max(gy);
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    if width <= 0 || height <= 0 {
        return vec![from_xz, to_xz];
    }
    let start = (sx - min_x, sy - min_y);
    let goal = (gx - min_x, gy - min_y);
    let mut tile = |_lx: i32, _ly: i32| LogisticsTile::default();
    let raw_path = logistics_grid_path(
        width,
        height,
        start,
        goal,
        &mut tile,
        concern,
        forecast,
        depart_tick,
        scratch,
    )
    .map(|r| r.path)
    .unwrap_or_else(|| vec![start, goal]);
    let smooth = smooth_vehicle_tile_path(&raw_path, width, height, &mut tile);
    smooth
        .into_iter()
        .map(|(lx, ly)| Vec2::new((lx + min_x) as f32 + 0.5, (ly + min_y) as f32 + 0.5))
        .collect()
}

/// Position and heading `fraction` (0..1) of the way along `points` by arc length.
#[must_use]
pub fn point_along_polyline(points: &[Vec2], fraction: f32) -> Option<(Vec2, Vec2)> {
    let total: f32 = points.windows(2).map(|w| w[0].distance(w[1])).sum();
    let first = *points.first()?;
    if points.len() < 2 || total < 1e-4 {
        return Some((first, Vec2::ZERO));
    }
    let mut left = total * fraction.clamp(0.0, 1.0);
    let last = points.len() - 2;
    for (i, w) in points.windows(2).enumerate() {
        let len = w[0].distance(w[1]);
        if left <= len || i == last {
            let dir = (w[1] - w[0]).normalize_or_zero();
            return Some((w[0] + dir * left.min(len), dir));
        }
        left -= len;
    }
    None
}

/// Truck leg planning inputs: the cross-country grid scratch, the cost forecast and the built
/// road network (one domain: how a truck gets from A to B).
#[derive(bevy::ecs::system::SystemParam)]
pub struct TruckLegPlanner<'w> {
    scratch: ResMut<'w, LogisticsGridPathScratch>,
    forecast: Option<Res<'w, LogisticsCostForecastBook>>,
    roads: TruckRoadNetwork<'w>,
}

/// Site pairs (re)planned per frame at most; the rest wait (unplanned pairs use the off-road estimate).
const LEG_PLANS_PER_FRAME: usize = 16;
/// Truck speed (m/s) the planner compares road and off-road routes at.
const PLAN_REF_TRUCK_MPS: f32 = 12.0;
/// Pairs no vehicle needed for this many order-sim ticks are dropped.
const LEG_STALE_TICKS: u64 = 3_600;

/// The route a truck drives between two sites (tile units): the faster of the built-road route
/// and the cross-country path, by time at [`PLAN_REF_TRUCK_MPS`].
fn plan_leg_route(
    planner: &mut TruckLegPlanner,
    from_t: Vec2,
    to_t: Vec2,
    concern: LogisticsPathConcern,
    depart_tick: u64,
) -> TruckRoute {
    let forecast = planner.forecast.as_deref();
    let offroad = crate::infrastructure::road_tier_table().offroad_speed_mps();
    let mut pts = plan_truck_tile_waypoints(&mut planner.scratch.nodes, from_t, to_t, concern, forecast, depart_tick);
    if pts.len() < 2 {
        pts = vec![from_t, to_t];
    }
    let last = pts.len() - 1;
    pts[0] = from_t;
    pts[last] = to_t;
    let cross_country = TruckRoute::uniform(pts, offroad);
    match planner.roads.route(forecast, depart_tick, from_t, to_t, PLAN_REF_TRUCK_MPS) {
        Some(road) if road.time_s(PLAN_REF_TRUCK_MPS) < cross_country.time_s(PLAN_REF_TRUCK_MPS) => road,
        _ => cross_country,
    }
}

/// Site pair a vehicle's current or next leg drives, if it is moving or about to.
fn vehicle_leg_pair(v: &crate::economy::logistics::orders::Vehicle) -> Option<(SiteId, SiteId)> {
    let a = v.assignment.as_ref()?;
    match v.phase {
        VehiclePhase::ToPickup(_) => Some((v.at, a.source)),
        // Plan the haul while loading so the trip time is ready when it departs.
        VehiclePhase::Loading(_) | VehiclePhase::Hauling(_) => Some((a.source, a.sink)),
        _ => None,
    }
}

/// RD-1/RD-3 truck route planner. Sole writer of [`SiteLegRouteBook`]: plans every site pair a
/// vehicle drives (re-plans all when a road is built, flooded, buried or cleared). The order sim
/// times trips from it, presentation draws trucks along it.
pub fn plan_site_leg_routes_system(
    sim_ctrl: Res<SimControlState>,
    sim: Option<Res<LogisticsOrderSimRes>>,
    registry: Option<Res<LogisticsSiteRegistry>>,
    concerns: Option<Res<LogisticsPathConcernRegistry>>,
    clock: Option<Res<SimStepClock>>,
    mut planner: TruckLegPlanner,
    mut book: ResMut<SiteLegRouteBook>,
) {
    if !sim_ctrl.should_tick() {
        return;
    }
    let (Some(sim), Some(registry), Some(concerns)) = (sim, registry, concerns) else {
        return;
    };
    let tick = clock.as_ref().map(|c| c.steps).unwrap_or(0);
    let stamp = planner.roads.stamp();
    let mut wanted: Vec<(SiteId, SiteId)> = sim.sim.vehicles.values().filter_map(vehicle_leg_pair).collect();
    wanted.sort_unstable_by_key(|(a, b)| (a.0, b.0));
    wanted.dedup();
    let mut budget = LEG_PLANS_PER_FRAME;
    for pair in wanted {
        let fresh = book.legs.get(&pair).is_some_and(|l| l.network_stamp == stamp);
        if let Some(l) = book.legs.get_mut(&pair) {
            l.last_used_tick = tick;
        }
        if fresh || budget == 0 {
            continue;
        }
        let (Some(from_xz), Some(to_xz)) = (site_xz(&registry, pair.0), site_xz(&registry, pair.1)) else {
            continue;
        };
        budget -= 1;
        let concern = concern_for_sink(&concerns, &registry, pair.1);
        let route = plan_leg_route(&mut planner, from_xz / TILE_METRES_GAMEPLAY, to_xz / TILE_METRES_GAMEPLAY, concern, tick);
        book.revision = book.revision.wrapping_add(1);
        let version = book.revision;
        book.legs.insert(pair, PlannedLeg { route, version, network_stamp: stamp, last_used_tick: tick });
    }
    let before = book.legs.len();
    book.legs.retain(|_, l| tick.saturating_sub(l.last_used_tick) <= LEG_STALE_TICKS);
    if book.legs.len() != before {
        book.revision = book.revision.wrapping_add(1);
    }
}

/// Truck proxies take their leg's planned route as waypoints (metres). Reads the book only.
pub fn sync_truck_motion_waypoints_system(
    sim_ctrl: Res<SimControlState>,
    sim: Option<Res<LogisticsOrderSimRes>>,
    registry: Option<Res<LogisticsSiteRegistry>>,
    book: Res<SiteLegRouteBook>,
    mut q: Query<(
        &mut TruckMotionLeg,
        &mut Waypoints,
        Option<&mut RoadVehicleKinematics>,
    )>,
) {
    if !sim_ctrl.should_tick() {
        return;
    }
    let (Some(sim), Some(registry)) = (sim, registry) else {
        return;
    };
    for (mut leg, mut wp, kin) in q.iter_mut() {
        let Some(v) = sim.sim.vehicles.get(&leg.vehicle_id) else {
            continue;
        };
        let key = truck_leg_key(v);
        let pair = match v.phase {
            VehiclePhase::ToPickup(_) | VehiclePhase::Hauling(_) => vehicle_leg_pair(v),
            _ => None,
        };
        let Some(pair) = pair else {
            continue;
        };
        let planned = book.legs.get(&pair);
        let version = planned.map_or(0, |p| p.version);
        if leg.leg_key == key && leg.route_version == version && !wp.points.is_empty() {
            continue;
        }
        let tiles = match planned {
            Some(p) if p.route.points.len() >= 2 => p.route.points.clone(),
            // Not planned yet: straight line until the planner gets to it (next frames).
            _ => {
                let (Some(a), Some(b)) = (site_xz(&registry, pair.0), site_xz(&registry, pair.1)) else {
                    continue;
                };
                vec![a / TILE_METRES_GAMEPLAY, b / TILE_METRES_GAMEPLAY]
            }
        };
        leg.leg_key = key;
        leg.route_version = version;
        wp.points = tiles.into_iter().map(|p| p * TILE_METRES_GAMEPLAY).collect();
        wp.current_waypoint_index = 0;
        if let Some(mut k) = kin {
            k.velocity = Vec2::ZERO;
            if wp.points.len() >= 2 {
                let d = wp.points[1] - wp.points[0];
                k.heading = d.normalize_or_zero();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav06_vehicle_smooth_matches_floodfill_pull() {
        let raw = [(0, 0), (1, 0), (2, 0), (3, 0)];
        let mut tile = |_x: i32, _y: i32| LogisticsTile::default();
        let s = smooth_vehicle_tile_path(&raw, 4, 1, &mut tile);
        assert_eq!(s, vec![(0, 0), (3, 0)]);
    }

    #[test]
    fn move08_accel_limits_speed_ramp() {
        let (v, _) = filter_vehicle_steering_output(
            Vec2::ZERO,
            Vec2::new(100.0, 0.0),
            Vec2::X,
            VehicleMotionLimits::TRUCK,
            0.1,
        );
        assert!(v.x <= VehicleMotionLimits::TRUCK.max_accel * 0.1 + 1e-3);
        assert!(v.x > 0.0);
    }

    #[test]
    fn move08_turn_rate_limits_heading_change() {
        let (_, h) = filter_vehicle_steering_output(
            Vec2::X,
            Vec2::Y * 4.0,
            Vec2::X,
            VehicleMotionLimits::BUS,
            0.05,
        );
        // Cannot instantly face +Y in 0.05s at max_turn_rate 0.9.
        assert!(h.x > 0.5, "heading must still lean forward: {h:?}");
        assert!(h.y > 0.0);
    }
}