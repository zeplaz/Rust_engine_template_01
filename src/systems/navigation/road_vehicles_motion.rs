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
use crate::economy::logistics::routes::TruckRoadNetwork;

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
}

/// Reused NAV-04 node array for truck tile path queries (sole writer: [`sync_truck_motion_waypoints_system`]).
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

/// Refresh waypoint polylines for order-sim truck proxies (read sim + site registry only).
pub fn sync_truck_motion_waypoints_system(
    sim_ctrl: Res<SimControlState>,
    sim: Option<Res<LogisticsOrderSimRes>>,
    registry: Option<Res<LogisticsSiteRegistry>>,
    concerns: Option<Res<LogisticsPathConcernRegistry>>,
    clock: Option<Res<SimStepClock>>,
    mut planner: TruckLegPlanner,
    mut q: Query<(
        &mut TruckMotionLeg,
        &mut Waypoints,
        Option<&mut RoadVehicleKinematics>,
    )>,
    mut road_stamp: Local<u64>,
) {
    if !sim_ctrl.should_tick() {
        return;
    }
    let (Some(sim), Some(registry), Some(concerns)) = (sim, registry, concerns) else {
        return;
    };
    let depart_tick = clock.as_ref().map(|c| c.steps).unwrap_or(0);
    // A road built, flooded, buried or cleared re-plans every active leg.
    let stamp = planner.roads.stamp();
    let network_changed = std::mem::replace(&mut *road_stamp, stamp) != stamp;
    for (mut leg, mut wp, kin) in q.iter_mut() {
        let Some(v) = sim.sim.vehicles.get(&leg.vehicle_id) else {
            continue;
        };
        let key = truck_leg_key(v);
        if leg.leg_key == key && !wp.points.is_empty() && !network_changed {
            continue;
        }
        leg.leg_key = key;
        let (from, to) = match &v.phase {
            VehiclePhase::ToPickup(_) => {
                let Some(a) = v.assignment.as_ref() else {
                    continue;
                };
                (v.at, a.source)
            }
            VehiclePhase::Hauling(_) => {
                let Some(a) = v.assignment.as_ref() else {
                    continue;
                };
                (a.source, a.sink)
            }
            _ => continue,
        };
        let (Some(from_xz), Some(to_xz)) = (site_xz(&registry, from), site_xz(&registry, to)) else {
            continue;
        };
        let concern = concern_for_sink(&concerns, &registry, to);
        // Sites are in metres; routing is in tiles. Follow the built road when one connects the
        // ends (flooded / rubble edges are walls); cross-country only where none does.
        let (from_t, to_t) = (from_xz / TILE_METRES_GAMEPLAY, to_xz / TILE_METRES_GAMEPLAY);
        let forecast = planner.forecast.as_deref();
        let tiles = planner.roads.polyline(forecast, depart_tick, from_t, to_t).unwrap_or_else(|| {
            let mut pts = plan_truck_tile_waypoints(&mut planner.scratch.nodes, from_t, to_t, concern, forecast, depart_tick);
            if pts.len() < 2 {
                pts = vec![from_t, to_t];
            }
            let last = pts.len() - 1;
            pts[0] = from_t;
            pts[last] = to_t;
            pts
        });
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