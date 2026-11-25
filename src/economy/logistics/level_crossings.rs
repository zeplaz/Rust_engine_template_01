//! RD-7 level crossings (PLAN-ROADS-v1): where a road crosses a railway at grade (roads and rail
//! never share nodes, so a crossing is where a road stretch cuts a rail stretch), trucks wait for
//! the barrier: a base delay per crossing plus more for every train running on that line.
//!
//! Sole writer of [`LevelCrossingBook`]. The road traffic publisher folds each road edge's delay
//! into its time multiplier ([`super::road_flow::RoadFlowBook::time_mul`]), so route choice and
//! trip times both see it.

use std::collections::HashMap;

use bevy::prelude::*;

use super::orders::model::VehiclePhase;
use super::orders::{LogisticsOrderSimRes, SiteRailLegBook};
use crate::construction::corridor_network::CorridorNetwork;
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId};

/// Crossings on one road edge and the delay a truck takes over them now.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelCrossings {
    /// Crossing points (tile units).
    pub points: Vec<Vec2>,
    /// Seconds a truck waits over this edge's crossings.
    pub delay_s: f32,
}

/// Level crossings by road edge. Sole writer: [`publish_level_crossings_system`].
#[derive(Resource, Debug, Default, Clone)]
pub struct LevelCrossingBook {
    pub by_edge: HashMap<TransportEdgeId, LevelCrossings>,
    pub revision: u64,
}

/// Where segment `a→b` meets `c→d`, ends included (a road node may sit on the track), tile units.
#[must_use]
pub fn segments_cross(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<Vec2> {
    let r = b - a;
    let s = d - c;
    let den = r.perp_dot(s);
    if den.abs() < 1e-6 {
        return None;
    }
    let t = (c - a).perp_dot(s) / den;
    let u = (c - a).perp_dot(r) / den;
    let inside = |v: f32| (-1e-4..=1.0 + 1e-4).contains(&v);
    (inside(t) && inside(u)).then(|| a + r * t)
}

/// Distance from `p` to a polyline.
fn dist_to_polyline(points: &[Vec2], p: Vec2) -> f32 {
    points
        .windows(2)
        .map(|w| {
            let d = w[1] - w[0];
            let t = ((p - w[0]).dot(d) / d.length_squared().max(1e-6)).clamp(0.0, 1.0);
            (w[0] + d * t).distance(p)
        })
        .fold(f32::INFINITY, f32::min)
}

fn centre(t: crate::strategic::BuildSiteTile) -> Vec2 {
    Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5)
}

/// Find crossings when the network changes; price them by the trains running on their lines.
pub fn publish_level_crossings_system(
    network: Option<Res<CorridorNetwork>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    sim: Option<Res<LogisticsOrderSimRes>>,
    rail: Option<Res<SiteRailLegBook>>,
    mut book: ResMut<LevelCrossingBook>,
    mut found: Local<(u64, usize, HashMap<TransportEdgeId, Vec<Vec2>>)>,
) {
    let (Some(network), Some(directory)) = (network, directory) else {
        return;
    };
    if found.0 != network.revision || found.1 != directory.by_edge.len() {
        let mut by_edge: HashMap<TransportEdgeId, Vec<Vec2>> = HashMap::new();
        let rail_spans: Vec<(Vec2, Vec2)> = network
            .segments
            .iter()
            .filter(|s| s.kind.is_rail())
            .flat_map(|s| s.spans().map(|(a, b)| (centre(a), centre(b))).collect::<Vec<_>>())
            .collect();
        if !rail_spans.is_empty() {
            // One entry per crossing point (a road node on the track touches two road stretches).
            let mut seen: Vec<Vec2> = Vec::new();
            for seg in network.segments.iter().filter(|s| !s.kind.is_rail()) {
                for (a, b) in seg.spans() {
                    let (pa, pb) = (centre(a), centre(b));
                    let hits: Vec<Vec2> = rail_spans
                        .iter()
                        .filter_map(|(c, d)| segments_cross(pa, pb, *c, *d))
                        .filter(|p| !seen.iter().any(|q| q.distance(*p) < 0.1))
                        .collect();
                    seen.extend(hits.iter().copied());
                    if hits.is_empty() {
                        continue;
                    }
                    if let Some(id) = crate::construction::find_edge_id_for_segment(&directory, a, b) {
                        by_edge.entry(id).or_default().extend(hits);
                    }
                }
            }
        }
        *found = (network.revision, directory.by_edge.len(), by_edge);
    }
    let tuning = &crate::infrastructure::road_tier_table().rail;
    // Trains on the move and the routes they run.
    let running: Vec<&[Vec2]> = match (sim.as_deref(), rail.as_deref()) {
        (Some(sim), Some(rail)) => sim
            .sim
            .vehicles
            .values()
            .filter(|v| v.mode.is_rail())
            .filter_map(|v| match (&v.phase, &v.assignment) {
                (VehiclePhase::Hauling(_), Some(a)) => rail.legs.get(&(a.source, a.sink)),
                (VehiclePhase::ToPickup(_), Some(a)) => rail.legs.get(&(v.at, a.source)),
                _ => None,
            })
            .map(|l| l.route.points.as_slice())
            .collect(),
        _ => Vec::new(),
    };
    let by_edge: HashMap<TransportEdgeId, LevelCrossings> = found
        .2
        .iter()
        .map(|(id, points)| {
            let delay_s = points
                .iter()
                .map(|p| {
                    let trains = running.iter().filter(|r| dist_to_polyline(r, *p) <= 1.5).count() as f32;
                    tuning.crossing_base_s + tuning.crossing_per_train_s * trains
                })
                .sum();
            (*id, LevelCrossings { points: points.clone(), delay_s })
        })
        .collect();
    if book.by_edge != by_edge {
        book.by_edge = by_edge;
        book.revision = book.revision.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_road_across_a_railway_crosses_once_ends_do_not_count() {
        let p = segments_cross(Vec2::new(0.0, 5.0), Vec2::new(10.0, 5.0), Vec2::new(5.0, 0.0), Vec2::new(5.0, 10.0));
        assert_eq!(p, Some(Vec2::new(5.0, 5.0)));
        assert_eq!(segments_cross(Vec2::ZERO, Vec2::X, Vec2::new(0.0, 1.0), Vec2::new(1.0, 1.0)), None, "parallel");
        assert_eq!(
            segments_cross(Vec2::ZERO, Vec2::new(5.0, 0.0), Vec2::new(5.0, -5.0), Vec2::new(5.0, 5.0)),
            Some(Vec2::new(5.0, 0.0)),
            "a road node on the track is a crossing"
        );
    }

    #[test]
    fn a_road_over_a_railway_gets_a_level_crossing_delay() {
        use crate::construction::corridor_network::CorridorKind;
        use crate::infrastructure::RoadTier;
        use crate::strategic::BuildSiteTile;
        use crate::systems::transport::{hydrate_transport_from_snapshot, TransportFieldStore, TransportTopology};
        let mut net = CorridorNetwork::default();
        net.add_segment(CorridorKind::Rail { profile: "default_rail".into() }, BuildSiteTile { x: 10, z: 0 }, BuildSiteTile { x: 10, z: 20 })
            .expect("rail");
        net.add_segment(CorridorKind::Road { tier: RoadTier::Gravel, lanes: 2 }, BuildSiteTile { x: 3, z: 9 }, BuildSiteTile { x: 17, z: 9 })
            .expect("road");
        let snap = net.bake_snapshot(10.25);
        let (mut top, mut field, mut dir) = (TransportTopology::default(), TransportFieldStore::default(), TransportEdgeDirectory::default());
        hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &snap).unwrap();
        let mut app = App::new();
        app.insert_resource(net).insert_resource(dir).init_resource::<LevelCrossingBook>();
        app.add_systems(Update, publish_level_crossings_system);
        app.update();
        let book = app.world().resource::<LevelCrossingBook>();
        let total: usize = book.by_edge.values().map(|c| c.points.len()).sum();
        assert_eq!(total, 1, "one crossing: {:?}", book.by_edge);
        let base = crate::infrastructure::road_tier_table().rail.crossing_base_s;
        assert!(book.by_edge.values().all(|c| (c.delay_s - base).abs() < 1e-3), "no trains: base delay");
    }
}
