//! Travel models for the order sim.

use std::collections::BTreeMap;

use super::model::SiteId;
use super::sim::TravelModel;

/// Straight-line distance between site positions × a road detour factor.
///
/// Interim model until trips follow the transport graph (plan LO-5): every registered site is
/// reachable; ticks = ⌈distance × detour / (speed × seconds_per_tick)⌉.
#[derive(Clone, Debug, PartialEq)]
pub struct SiteDistanceTravel {
    /// World XZ in metres.
    pub positions: BTreeMap<SiteId, [f32; 2]>,
    /// Road length / straight-line length.
    pub detour: f32,
    /// Simulated seconds per order-sim tick.
    pub seconds_per_tick: f32,
}

impl Default for SiteDistanceTravel {
    fn default() -> Self {
        Self {
            positions: BTreeMap::new(),
            detour: 1.3,
            seconds_per_tick: 1.0,
        }
    }
}

impl SiteDistanceTravel {
    #[must_use]
    pub fn distance_m(&self, from: SiteId, to: SiteId) -> Option<f32> {
        let a = self.positions.get(&from)?;
        let b = self.positions.get(&to)?;
        Some(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt())
    }
}

impl TravelModel for SiteDistanceTravel {
    fn travel_ticks(&self, from: SiteId, to: SiteId, speed_mps: f32) -> Option<u32> {
        if from == to {
            return Some(0);
        }
        let d = self.distance_m(from, to)? * self.detour.max(1.0);
        let metres_per_tick = (speed_mps * self.seconds_per_tick).max(1e-3);
        Some((d / metres_per_tick).ceil().max(1.0) as u32)
    }
}

/// One planned site-to-site truck leg (PLAN-ROADS-v1 RD-3): the route a truck drives (tile units)
/// with a speed cap per stretch. Written by the truck route planner only
/// (`plan_site_leg_routes_system`); read by the order sim (trip time) and truck presentation
/// (where the truck is drawn).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlannedLeg {
    pub route: crate::economy::logistics::routes::TruckRoute,
    /// Bumps each time this pair is re-planned (presentation re-syncs its waypoints).
    pub version: u64,
    /// Road network stamp the plan was made under.
    pub network_stamp: u64,
    /// Order-sim tick of the last time a vehicle needed this pair (stale pairs are dropped).
    pub last_used_tick: u64,
}

/// Planned legs by (from, to) site. Sole writer: the truck route planner.
#[derive(bevy::prelude::Resource, Clone, Debug, Default)]
pub struct SiteLegRouteBook {
    pub legs: BTreeMap<(SiteId, SiteId), PlannedLeg>,
    pub revision: u64,
}

/// Planned yard-to-yard rail legs (RD-7): the rail route between two yards (tile units, a speed
/// cap per stretch). Sole writer: the rail leg planner. Read by the order sim (train trip time)
/// and train presentation. A pair with no entry has no rail route (no line, or it is cut).
#[derive(bevy::prelude::Resource, Clone, Debug, Default)]
pub struct SiteRailLegBook {
    pub legs: BTreeMap<(SiteId, SiteId), PlannedLeg>,
    pub revision: u64,
}

/// Order-sim travel over planned legs: a planned pair takes the time of the route actually driven
/// (built roads at their tier speed, the rest off-road); an unplanned pair falls back to the
/// straight line × detour at off-road speed until the planner gets to it.
pub struct RoutedTravel<'a> {
    pub base: &'a SiteDistanceTravel,
    pub legs: &'a SiteLegRouteBook,
    /// Off-road speed (m/s) for the fallback.
    pub offroad_mps: f32,
    /// Yard-to-yard rail legs (RD-7); trains only run where one exists.
    pub rail: &'a SiteRailLegBook,
}

impl TravelModel for RoutedTravel<'_> {
    fn travel_ticks(&self, from: SiteId, to: SiteId, speed_mps: f32) -> Option<u32> {
        if from == to {
            return Some(0);
        }
        let secs = match self.legs.legs.get(&(from, to)) {
            Some(leg) if leg.route.points.len() >= 2 => leg.route.time_s(speed_mps),
            _ => self.base.distance_m(from, to)? * self.base.detour.max(1.0) / speed_mps.min(self.offroad_mps).max(0.1),
        };
        Some((secs / self.base.seconds_per_tick.max(1e-3)).ceil().max(1.0) as u32)
    }

    fn rail_travel_ticks(&self, from: SiteId, to: SiteId, speed_mps: f32) -> Option<u32> {
        if from == to {
            return Some(0);
        }
        let leg = self.rail.legs.get(&(from, to)).filter(|l| l.route.points.len() >= 2)?;
        let secs = leg.route.time_s(speed_mps);
        Some((secs / self.base.seconds_per_tick.max(1e-3)).ceil().max(1.0) as u32)
    }
}
