//! RD-7 rail leg planner (PLAN-ROADS-v1): the rail route between every pair of one owner's freight
//! yards. Sole writer of [`SiteRailLegBook`]; the order sim times train trips from it and train
//! presentation draws trains along it. Re-plans when the transport network or the yard list
//! changes (a flooded, buried or cut line drops the pair: trains stop, the intermodal planner
//! reverts to road).

use bevy::prelude::*;

use crate::economy::logistics::orders::model::SiteId;
use crate::economy::logistics::orders::snapshot::SiteRole;
use crate::economy::logistics::orders::{LogisticsSiteRegistry, PlannedLeg, SiteRailLegBook};
use crate::economy::logistics::routes::{TruckRoadNetwork, TruckRoute};
use crate::systems::sim_control::SimStepClock;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// The same route driven the other way.
#[must_use]
pub fn reversed(route: &TruckRoute) -> TruckRoute {
    let mut points = route.points.clone();
    points.reverse();
    let mut caps_mps = route.caps_mps.clone();
    caps_mps.reverse();
    TruckRoute { points, caps_mps }
}

/// Plan every yard pair (same owner) over rail; keep the book in step with the network.
pub fn plan_rail_legs_system(
    registry: Option<Res<LogisticsSiteRegistry>>,
    roads: TruckRoadNetwork,
    clock: Option<Res<SimStepClock>>,
    mut book: ResMut<SiteRailLegBook>,
    mut last: Local<Option<(u64, u64)>>,
) {
    let Some(registry) = registry else {
        return;
    };
    let stamp = (roads.stamp(), registry.revision);
    if *last == Some(stamp) {
        return;
    }
    *last = Some(stamp);
    let tick = clock.as_ref().map_or(0, |c| c.steps);
    let yards: Vec<(SiteId, crate::economy::logistics::orders::model::OwnerId, Vec2)> = registry
        .meta
        .iter()
        .filter(|(_, m)| m.role == SiteRole::Yard)
        .map(|(id, m)| (*id, m.owner, Vec2::from(m.world_xz) / TILE_METRES_GAMEPLAY))
        .collect();
    let mut legs = std::collections::BTreeMap::new();
    for (i, (a, owner_a, pa)) in yards.iter().enumerate() {
        for (b, owner_b, pb) in &yards[i + 1..] {
            if owner_a != owner_b {
                continue;
            }
            let Some(route) = roads.rail_route(tick, *pa, *pb) else {
                continue;
            };
            let back = reversed(&route);
            for ((from, to), r) in [((*a, *b), route), ((*b, *a), back)] {
                let version = book.legs.get(&(from, to)).map_or(1, |l| if l.route == r { l.version } else { l.version + 1 });
                legs.insert(
                    (from, to),
                    PlannedLeg { route: r, version, network_stamp: stamp.0, last_used_tick: tick },
                );
            }
        }
    }
    if book.legs != legs {
        book.legs = legs;
        book.revision = book.revision.wrapping_add(1);
        bevy::log::debug!(target: "logistics::rail", pairs = book.legs.len(), "rail legs planned");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reversed_route_runs_the_same_stretches_backwards() {
        let r = TruckRoute { points: vec![Vec2::ZERO, Vec2::X, Vec2::new(1.0, 5.0)], caps_mps: vec![4.0, 22.0] };
        let b = reversed(&r);
        assert_eq!(b.points, vec![Vec2::new(1.0, 5.0), Vec2::X, Vec2::ZERO]);
        assert_eq!(b.caps_mps, vec![22.0, 4.0]);
        assert!((b.time_s(30.0) - r.time_s(30.0)).abs() < 1e-3);
    }
}
