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
