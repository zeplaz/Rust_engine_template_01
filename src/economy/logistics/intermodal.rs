//! RD-7 intermodal hauls (PLAN-ROADS-v1): a long truck order whose ends both have a freight yard
//! nearby, with rail between those yards and a quicker trip that way, is replaced by three legs of
//! the same cargo — truck source → yard A, train yard A → yard B, truck yard B → sink — and goes
//! back to one direct truck order when the rail route is lost (flood, rubble, a cut line).
//!
//! A haul that already starts (or ends) at a yard skips that truck leg. One-off runs split into
//! runs of the remaining quantity (a leg waits until goods reach its yard). A standing line becomes three standing lines; the two yards publish a transit demand the
//! size of the sink's so the lines have something to fill.
//!
//! Sole writer of [`IntermodalBook`]; every change to orders goes out as ordinary
//! [`LogisticsOrderRequest`]s (the order sim stays the only order writer).

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::orders::model::{CargoKind, ConsumerKind, OrderId, OwnerId, Priority, SiteId};
use super::orders::plugin::{LogisticsOrderRequest, LogisticsOrdersSnapshotRes};
use super::orders::sim::LogisticsOrderCommand;
use super::orders::snapshot::{LogisticsOrdersSnapshot, OrderRow, SiteRole};
use super::orders::SiteRailLegBook;
use crate::systems::sim_control::SimStepClock;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Planner pass cadence (sim steps).
const PLAN_EVERY_STEPS: u64 = 10;
/// Truck speed (m/s) the planner estimates road legs at, and the road detour over the straight line.
const TRUCK_REF_MPS: f32 = 12.0;
const ROAD_DETOUR: f32 = 1.25;

/// One order split into rail legs.
#[derive(Clone, Debug, PartialEq)]
pub struct IntermodalChain {
    pub owner: OwnerId,
    pub source: SiteId,
    pub sink: SiteId,
    pub cargo: String,
    pub yards: (SiteId, SiteId),
    /// `Some(quantity)` for a one-off run, `None` for a standing line.
    pub quantity: Option<u32>,
    /// Transit demand published at the yards (standing lines).
    pub transit_target: u32,
}

/// Orders sent by rail (sole writer: [`plan_intermodal_system`]).
#[derive(Resource, Debug, Default, Clone)]
pub struct IntermodalBook {
    /// By the original order's id.
    pub chains: BTreeMap<OrderId, IntermodalChain>,
    /// Chains reverted to road because their rail route went (dump).
    pub reverted: u32,
}

/// Where a haul should change to rail: `(yard near source, yard near sink)` when both exist,
/// differ, have a rail leg, the haul is long enough and the rail way is quicker.
#[must_use]
pub fn pick_rail_yards(
    snap: &LogisticsOrdersSnapshot,
    rail: &SiteRailLegBook,
    owner: OwnerId,
    source: SiteId,
    sink: SiteId,
    rail_mps: f32,
    min_tiles: f32,
) -> Option<(SiteId, SiteId)> {
    let pos = |id: SiteId| snap.site(id).map(|s| Vec2::from(s.world_xz));
    let (ps, pk) = (pos(source)?, pos(sink)?);
    if ps.distance(pk) / TILE_METRES_GAMEPLAY < min_tiles {
        return None;
    }
    let road_s = |a: Vec2, b: Vec2| a.distance(b) * ROAD_DETOUR / TRUCK_REF_MPS;
    let direct = road_s(ps, pk);
    let yards: Vec<(SiteId, Vec2)> = snap
        .sites
        .iter()
        .filter(|s| s.owner == owner && s.role == SiteRole::Yard)
        .map(|s| (s.id, Vec2::from(s.world_xz)))
        .collect();
    // An end that is a yard loads / unloads the train itself (no truck leg on that side).
    let from_yards: Vec<(SiteId, Vec2)> =
        if is_yard(snap, source) { vec![(source, ps)] } else { yards.clone() };
    let to_yards: Vec<(SiteId, Vec2)> = if is_yard(snap, sink) { vec![(sink, pk)] } else { yards.clone() };
    let mut best: Option<(f32, SiteId, SiteId)> = None;
    for &(ya, pa) in &from_yards {
        for &(yb, pb) in &to_yards {
            if ya == yb {
                continue;
            }
            let Some(leg) = rail.legs.get(&(ya, yb)) else {
                continue;
            };
            let t = road_s(ps, pa) + leg.route.time_s(rail_mps) + road_s(pb, pk);
            if t < direct && best.is_none_or(|(bt, _, _)| t < bt) {
                best = Some((t, ya, yb));
            }
        }
    }
    best.map(|(_, a, b)| (a, b))
}

fn is_yard(snap: &LogisticsOrdersSnapshot, id: SiteId) -> bool {
    snap.site(id).is_some_and(|s| s.role == SiteRole::Yard)
}

fn create(owner: OwnerId, source: SiteId, sink: SiteId, cargo: &str, quantity: Option<u32>) -> LogisticsOrderCommand {
    let cargo = CargoKind::new(cargo);
    match quantity {
        Some(quantity) => LogisticsOrderCommand::CreateOneOffRun {
            owner,
            source,
            sink,
            cargo,
            quantity,
            priority: Priority::Normal,
            vehicle_cap: None,
        },
        None => LogisticsOrderCommand::CreateStandingLine {
            owner,
            source,
            sink,
            cargo,
            priority: Priority::Normal,
            vehicle_cap: None,
        },
    }
}

/// Open orders of a chain's legs (matched by owner, ends and cargo).
fn chain_legs<'a>(snap: &'a LogisticsOrdersSnapshot, c: &IntermodalChain) -> Vec<&'a OrderRow> {
    let (ya, yb) = c.yards;
    let ends = [(c.source, ya), (ya, yb), (yb, c.sink)];
    snap.orders
        .iter()
        .filter(|o| !o.status.is_terminal() && o.owner == c.owner && o.cargo == c.cargo)
        .filter(|o| ends.contains(&(o.source, o.sink)))
        .collect()
}

/// One planner pass: the order requests that split long hauls onto rail and revert chains whose
/// rail route is gone (book updated in place).
pub fn plan_intermodal(
    snap: &LogisticsOrdersSnapshot,
    rail: &SiteRailLegBook,
    book: &mut IntermodalBook,
    rail_mps: f32,
    min_tiles: f32,
) -> Vec<LogisticsOrderCommand> {
    let mut out = Vec::new();
    // Revert: the rail leg between the chain's yards is gone.
    let lost: Vec<OrderId> = book.chains.iter().filter(|(_, c)| !rail.legs.contains_key(&c.yards)).map(|(id, _)| *id).collect();
    for id in lost {
        let c = book.chains.remove(&id).expect("listed");
        let legs = chain_legs(snap, &c);
        // What has not reached the sink yet goes by road again.
        let left = c.quantity.map(|_| legs.iter().filter(|o| o.sink == c.sink).map(|o| o.remaining).max().unwrap_or(0));
        for o in &legs {
            out.push(LogisticsOrderCommand::Cancel { order: o.id });
        }
        if c.quantity.is_none() {
            for y in [c.yards.0, c.yards.1] {
                if y != c.source && y != c.sink {
                    out.push(LogisticsOrderCommand::RetractDemand { site: y, cargo: CargoKind::new(c.cargo.clone()) });
                }
            }
        }
        if left != Some(0) {
            out.push(create(c.owner, c.source, c.sink, &c.cargo, left));
        }
        book.reverted += 1;
        bevy::log::info!(target: "logistics::rail", order = id.0, "rail route lost: haul back on the road");
    }
    // Split: long open orders that are not already a leg of a chain.
    let leg_ends: Vec<(SiteId, SiteId, OwnerId, String)> = book
        .chains
        .values()
        .flat_map(|c| {
            let (ya, yb) = c.yards;
            [(c.source, ya), (ya, yb), (yb, c.sink)].map(|(a, b)| (a, b, c.owner, c.cargo.clone()))
        })
        .collect();
    for o in &snap.orders {
        if o.status.is_terminal()
            || book.chains.contains_key(&o.id)
            || leg_ends.iter().any(|(a, b, w, c)| *a == o.source && *b == o.sink && *w == o.owner && *c == o.cargo)
        {
            continue;
        }
        // Yard to yard is a train's own work already.
        if is_yard(snap, o.source) && is_yard(snap, o.sink) {
            continue;
        }
        let quantity = if o.standing { None } else { Some(o.remaining) };
        if quantity == Some(0) {
            continue;
        }
        let Some(yards) = pick_rail_yards(snap, rail, o.owner, o.source, o.sink, rail_mps, min_tiles) else {
            continue;
        };
        let transit_target = snap.demands.iter().find(|d| d.site == o.sink && d.cargo == o.cargo).map_or(0, |d| d.buffer_target);
        out.push(LogisticsOrderCommand::Cancel { order: o.id });
        for (a, b) in [(o.source, yards.0), (yards.0, yards.1), (yards.1, o.sink)] {
            if a != b {
                out.push(create(o.owner, a, b, &o.cargo, quantity));
            }
        }
        if quantity.is_none() && transit_target > 0 {
            for y in [yards.0, yards.1] {
                if y != o.source && y != o.sink {
                    out.push(LogisticsOrderCommand::PublishDemand {
                        site: y,
                        cargo: CargoKind::new(o.cargo.clone()),
                        consumer: ConsumerKind::Depot,
                        buffer_target: transit_target,
                        reorder_point: transit_target / 2,
                        draw_rate_per_min: 0,
                        priority: Priority::Normal,
                    });
                }
            }
        }
        bevy::log::info!(target: "logistics::rail", order = o.id.0, from = o.source.0, to = o.sink.0, "haul sent by rail");
        book.chains.insert(
            o.id,
            IntermodalChain {
                owner: o.owner,
                source: o.source,
                sink: o.sink,
                cargo: o.cargo.clone(),
                yards,
                quantity,
                transit_target,
            },
        );
    }
    out
}

/// Split long truck orders onto rail; revert chains whose rail route is gone.
pub fn plan_intermodal_system(
    clock: Option<Res<SimStepClock>>,
    snapshot: Option<Res<LogisticsOrdersSnapshotRes>>,
    rail: Option<Res<SiteRailLegBook>>,
    mut book: ResMut<IntermodalBook>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
    mut last: Local<u64>,
) {
    let (Some(clock), Some(snapshot), Some(rail)) = (clock, snapshot, rail) else {
        return;
    };
    if clock.steps < *last + PLAN_EVERY_STEPS {
        return;
    }
    *last = clock.steps;
    let tuning = &crate::infrastructure::road_tier_table().rail;
    for cmd in plan_intermodal(&snapshot.0, &rail, &mut book, tuning.speed_mps(), tuning.intermodal_min_tiles) {
        requests.write(LogisticsOrderRequest(cmd));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::snapshot::SiteRow;
    use crate::economy::logistics::orders::PlannedLeg;
    use crate::economy::logistics::routes::TruckRoute;

    fn site(id: u32, role: SiteRole, x_tiles: f32) -> SiteRow {
        SiteRow {
            id: SiteId(id),
            name: format!("s{id}"),
            owner: OwnerId::PLAYER,
            role,
            world_xz: [x_tiles * TILE_METRES_GAMEPLAY, 0.0],
            tile: [x_tiles as i32, 0],
            stock: Vec::new(),
        }
    }

    fn rail_between(a: u32, b: u32, from_x: f32, to_x: f32) -> SiteRailLegBook {
        let route = TruckRoute::uniform(vec![Vec2::new(from_x, 0.0), Vec2::new(to_x, 0.0)], 22.0);
        let mut book = SiteRailLegBook::default();
        for (x, y) in [(a, b), (b, a)] {
            book.legs.insert((SiteId(x), SiteId(y)), PlannedLeg { route: route.clone(), ..Default::default() });
        }
        book
    }

    #[test]
    fn a_long_haul_with_yards_at_both_ends_goes_by_rail() {
        // Quarry at 0, yard A at 5, yard B at 295, site at 300 (tiles).
        let snap = LogisticsOrdersSnapshot {
            sites: vec![
                site(1, SiteRole::Plant, 0.0),
                site(2, SiteRole::Yard, 5.0),
                site(3, SiteRole::Yard, 295.0),
                site(4, SiteRole::Construction, 300.0),
            ],
            ..Default::default()
        };
        let rail = rail_between(2, 3, 5.0, 295.0);
        let pick = pick_rail_yards(&snap, &rail, OwnerId::PLAYER, SiteId(1), SiteId(4), 22.0, 60.0);
        assert_eq!(pick, Some((SiteId(2), SiteId(3))));
    }

    #[test]
    fn short_hauls_and_missing_rail_stay_on_the_road() {
        let snap = LogisticsOrdersSnapshot {
            sites: vec![
                site(1, SiteRole::Plant, 0.0),
                site(2, SiteRole::Yard, 5.0),
                site(3, SiteRole::Yard, 45.0),
                site(4, SiteRole::Construction, 50.0),
            ],
            ..Default::default()
        };
        let rail = rail_between(2, 3, 5.0, 45.0);
        assert_eq!(pick_rail_yards(&snap, &rail, OwnerId::PLAYER, SiteId(1), SiteId(4), 22.0, 60.0), None, "50 tiles: too short");
        assert_eq!(
            pick_rail_yards(&snap, &SiteRailLegBook::default(), OwnerId::PLAYER, SiteId(1), SiteId(4), 22.0, 10.0),
            None,
            "no rail leg"
        );
    }

    fn order(id: u32, source: u32, sink: u32, remaining: u32) -> OrderRow {
        use crate::economy::logistics::orders::model::OrderStatus;
        OrderRow {
            id: OrderId(id),
            owner: OwnerId::PLAYER,
            standing: false,
            source: SiteId(source),
            sink: SiteId(sink),
            source_name: String::new(),
            sink_name: String::new(),
            cargo: "Gravel".into(),
            unit: "t",
            priority: Priority::Normal,
            status: OrderStatus::Active,
            status_label: String::new(),
            quantity: Some(remaining),
            delivered: 0,
            in_flight: 0,
            remaining,
            trips_done: 0,
            trips_planned: 0,
            vehicles: 0,
            vehicle_cap: None,
            next_eta_ticks: None,
            fill_ratio: None,
            throughput_per_min: 0.0,
            lost: 0,
            created_tick: 0,
            finished_tick: None,
        }
    }

    fn long_haul_world() -> LogisticsOrdersSnapshot {
        LogisticsOrdersSnapshot {
            sites: vec![
                site(1, SiteRole::Plant, 0.0),
                site(2, SiteRole::Yard, 5.0),
                site(3, SiteRole::Yard, 295.0),
                site(4, SiteRole::Construction, 300.0),
            ],
            orders: vec![order(10, 1, 4, 900)],
            ..Default::default()
        }
    }

    #[test]
    fn a_long_run_splits_into_three_legs_and_goes_back_on_the_road_when_the_line_is_cut() {
        let mut snap = long_haul_world();
        let rail = rail_between(2, 3, 5.0, 295.0);
        let mut book = IntermodalBook::default();
        let cmds = plan_intermodal(&snap, &rail, &mut book, 22.0, 60.0);
        let runs: Vec<(u32, u32, u32)> = cmds
            .iter()
            .filter_map(|c| match c {
                LogisticsOrderCommand::CreateOneOffRun { source, sink, quantity, .. } => Some((source.0, sink.0, *quantity)),
                _ => None,
            })
            .collect();
        assert!(matches!(cmds[0], LogisticsOrderCommand::Cancel { order: OrderId(10) }), "{cmds:?}");
        assert_eq!(runs, vec![(1, 2, 900), (2, 3, 900), (3, 4, 900)]);
        // Next pass: the legs exist (ids 11..13), nothing is split again.
        snap.orders = vec![order(11, 1, 2, 900), order(12, 2, 3, 900), order(13, 3, 4, 600)];
        assert!(plan_intermodal(&snap, &rail, &mut book, 22.0, 60.0).is_empty(), "legs are left alone");
        // The line is cut: cancel the legs, send what has not arrived (600) by road.
        let cmds = plan_intermodal(&snap, &SiteRailLegBook::default(), &mut book, 22.0, 60.0);
        let cancelled = cmds.iter().filter(|c| matches!(c, LogisticsOrderCommand::Cancel { .. })).count();
        assert_eq!(cancelled, 3);
        assert!(cmds.iter().any(|c| matches!(c, LogisticsOrderCommand::CreateOneOffRun { source: SiteId(1), sink: SiteId(4), quantity: 600, .. })), "{cmds:?}");
        assert_eq!(book.reverted, 1);
        assert!(book.chains.is_empty());
    }

    #[test]
    fn a_haul_from_a_yard_skips_the_first_truck_leg() {
        let mut snap = long_haul_world();
        snap.orders = vec![order(20, 2, 4, 500)];
        let rail = rail_between(2, 3, 5.0, 295.0);
        let cmds = plan_intermodal(&snap, &rail, &mut IntermodalBook::default(), 22.0, 60.0);
        let runs: Vec<(u32, u32)> = cmds
            .iter()
            .filter_map(|c| match c {
                LogisticsOrderCommand::CreateOneOffRun { source, sink, .. } => Some((source.0, sink.0)),
                _ => None,
            })
            .collect();
        assert_eq!(runs, vec![(2, 3), (3, 4)]);
    }

    #[test]
    fn yards_far_from_the_ends_do_not_pay_off() {
        // Yards sit off to the side: the road detour to them is slower than driving direct.
        let snap = LogisticsOrdersSnapshot {
            sites: vec![
                site(1, SiteRole::Plant, 0.0),
                site(2, SiteRole::Yard, -200.0),
                site(3, SiteRole::Yard, 500.0),
                site(4, SiteRole::Construction, 300.0),
            ],
            ..Default::default()
        };
        let rail = rail_between(2, 3, -200.0, 500.0);
        assert_eq!(pick_rail_yards(&snap, &rail, OwnerId::PLAYER, SiteId(1), SiteId(4), 22.0, 60.0), None);
    }
}
