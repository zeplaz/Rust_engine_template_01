//! Road congestion (PLAN-ROADS-v1 RD-4). Every truck driving a planned leg adds its rate
//! (vehicles per hour: one pass each way per round trip) to each road edge of the leg's route;
//! an edge's capacity is its tier's `capacity_veh_h_lane` × lanes. Travel time over the edge is
//! the free time × the BPR curve `1 + 0.15 (volume / capacity)^4` (the standard traffic-planning
//! form), capped so a jammed road still crawls. Route choice (`routes::route_edge_cost_with_nav`)
//! and trip time (leg route caps) both use it, so traffic spreads onto parallel roads.
//!
//! Sole writer of [`RoadFlowBook`]: [`publish_road_flow_system`].

use std::collections::HashMap;

use bevy::math::IVec2;
use bevy::prelude::*;

use super::orders::plugin::LogisticsOrderSimRes;
use super::orders::travel::SiteLegRouteBook;
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId};

/// BPR coefficients (Bureau of Public Roads curve).
const BPR_ALPHA: f32 = 0.15;
const BPR_BETA: i32 = 4;
/// A jammed road still moves at this share of its free speed.
const MIN_SPEED_SHARE: f32 = 0.15;
/// Shortest round trip (s) a truck's rate is computed from (keeps very short legs finite).
const MIN_ROUND_TRIP_S: f32 = 60.0;
/// Frames between recomputes when nothing else changed.
const REFRESH_FRAMES: u32 = 30;

/// Travel-time multiplier at `volume / capacity`.
#[must_use]
pub fn bpr_time_mul(vc: f32) -> f32 {
    (1.0 + BPR_ALPHA * vc.max(0.0).powi(BPR_BETA)).min(1.0 / MIN_SPEED_SHARE)
}

/// Traffic on one road edge.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeFlow {
    pub veh_h: f32,
    pub cap_veh_h: f32,
}

impl EdgeFlow {
    #[must_use]
    pub fn vc(&self) -> f32 {
        if self.cap_veh_h > 0.0 { self.veh_h / self.cap_veh_h } else { 0.0 }
    }
}

/// Per road edge volume / capacity. Revision bumps only when some edge's slow-down moves by a
/// quarter step, so legs re-plan on real changes, not on every truck.
#[derive(Resource, Debug, Default, Clone)]
pub struct RoadFlowBook {
    pub by_edge: HashMap<TransportEdgeId, EdgeFlow>,
    pub revision: u64,
    /// Highest volume / capacity seen on any edge this run, and the most edges loaded at once
    /// (dump: trucks are often idle when the dump fires).
    pub peak_vc: f32,
    pub peak_edges: usize,
    buckets: HashMap<TransportEdgeId, u16>,
    /// Level-crossing delay as a time multiplier per road edge (RD-7).
    pub crossing_mul: HashMap<TransportEdgeId, f32>,
}

impl RoadFlowBook {
    /// Travel-time multiplier on `id` (1 = free flow).
    #[must_use]
    pub fn time_mul(&self, id: TransportEdgeId) -> f32 {
        self.by_edge.get(&id).map_or(1.0, |f| bpr_time_mul(f.vc())) * self.crossing_mul.get(&id).copied().unwrap_or(1.0)
    }

    /// Highest volume / capacity on any edge (dump).
    #[must_use]
    pub fn max_vc(&self) -> f32 {
        self.by_edge.values().map(EdgeFlow::vc).fold(0.0, f32::max)
    }
}

/// Edge between two node tiles, built from the directory (the leg routes run node to node).
#[derive(Default)]
pub struct EdgeIndex {
    edges: usize,
    by_pair: HashMap<(IVec2, IVec2), (TransportEdgeId, f32)>,
}

fn parse_tile(key: &str) -> Option<IVec2> {
    let (x, z) = key.strip_prefix('t')?.split_once('_')?;
    Some(IVec2::new(x.parse().ok()?, z.parse().ok()?))
}

fn pair(a: IVec2, b: IVec2) -> (IVec2, IVec2) {
    if (a.x, a.y) <= (b.x, b.y) { (a, b) } else { (b, a) }
}

/// Recomputes the flow on every road edge from the trucks on their legs.
pub fn publish_road_flow_system(
    sim: Option<Res<LogisticsOrderSimRes>>,
    legs: Option<Res<SiteLegRouteBook>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    crossings: Option<Res<super::level_crossings::LevelCrossingBook>>,
    mut book: ResMut<RoadFlowBook>,
    mut index: Local<EdgeIndex>,
    mut frames: Local<u32>,
) {
    let (Some(sim), Some(legs), Some(directory)) = (sim, legs, directory) else {
        return;
    };
    *frames += 1;
    let crossings_changed = crossings.as_ref().is_some_and(|c| c.is_changed());
    if *frames < REFRESH_FRAMES && !legs.is_changed() && !crossings_changed {
        return;
    }
    *frames = 0;
    if directory.is_changed() || index.edges != directory.by_edge.len() {
        index.edges = directory.by_edge.len();
        index.by_pair.clear();
        for (id, m) in &directory.by_edge {
            let (Some(tier), Some(a), Some(b)) = (
                crate::infrastructure::RoadTier::from_profile_id(&m.profile),
                parse_tile(&m.head_key),
                parse_tile(&m.tail_key),
            ) else {
                continue;
            };
            let spec = tier.spec();
            let cap = spec.capacity_veh_h_lane * f32::from(spec.lanes_default.max(1));
            index.by_pair.insert(pair(a, b), (*id, cap));
        }
    }
    let mut by_edge: HashMap<TransportEdgeId, EdgeFlow> = HashMap::new();
    for v in sim.sim.vehicles.values() {
        let Some(key) = crate::systems::navigation::vehicle_leg_pair(v) else {
            continue;
        };
        let Some(leg) = legs.legs.get(&key) else {
            continue;
        };
        let round_trip = (2.0 * leg.route.time_s(12.0)).max(MIN_ROUND_TRIP_S);
        let rate = 3600.0 / round_trip * 2.0; // a pass each way per round trip
        for w in leg.route.points.windows(2) {
            let (a, b) = (w[0].floor().as_ivec2(), w[1].floor().as_ivec2());
            if let Some(&(id, cap)) = index.by_pair.get(&pair(a, b)) {
                let f = by_edge.entry(id).or_insert(EdgeFlow { veh_h: 0.0, cap_veh_h: cap });
                f.veh_h += rate;
            }
        }
    }
    // RD-7 level crossings: delay over the edge's free-flow time, as a multiplier.
    let crossing_mul: HashMap<TransportEdgeId, f32> = crossings
        .as_deref()
        .map(|c| {
            c.by_edge
                .iter()
                .filter_map(|(id, x)| {
                    let m = directory.by_edge.get(id)?;
                    let (a, b) = (parse_tile(&m.head_key)?, parse_tile(&m.tail_key)?);
                    let speed = crate::infrastructure::road_profile_speed_mps(&m.profile)?.max(0.1);
                    let free_s = (a.as_vec2().distance(b.as_vec2()) * crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY / speed).max(1.0);
                    Some((*id, 1.0 + x.delay_s / free_s))
                })
                .collect()
        })
        .unwrap_or_default();
    let mut buckets: HashMap<TransportEdgeId, u16> = by_edge
        .iter()
        .map(|(id, f)| (*id, (bpr_time_mul(f.vc()) * 4.0).round() as u16))
        .filter(|(_, b)| *b > 4)
        .collect();
    for (id, m) in &crossing_mul {
        let b = buckets.entry(*id).or_insert(4);
        *b = b.saturating_add(((m - 1.0) * 4.0).round() as u16);
    }
    book.crossing_mul = crossing_mul;
    if buckets != book.buckets {
        book.buckets = buckets;
        book.revision = book.revision.wrapping_add(1);
    }
    book.peak_vc = book.peak_vc.max(by_edge.values().map(EdgeFlow::vc).fold(0.0, f32::max));
    book.peak_edges = book.peak_edges.max(by_edge.len());
    book.by_edge = by_edge;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_road_slows_as_it_fills_and_never_stops() {
        assert!((bpr_time_mul(0.0) - 1.0).abs() < 1e-6);
        assert!(bpr_time_mul(0.5) < 1.02, "half full: barely slower");
        assert!((bpr_time_mul(1.0) - 1.15).abs() < 1e-4, "at capacity: +15%");
        assert!(bpr_time_mul(2.0) > 3.0, "twice over: a crawl");
        assert!(bpr_time_mul(10.0) <= 1.0 / MIN_SPEED_SHARE + 1e-4, "jammed still moves");
    }
}
