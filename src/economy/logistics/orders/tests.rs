//! Headless order-sim tests (LO-1 one-off runs, LO-2 draw demands + standing lines).

use std::collections::BTreeMap;

use super::*;

/// Symmetric distance table in metres; missing pair = unreachable.
#[derive(Default)]
struct Distances(BTreeMap<(SiteId, SiteId), f32>);

impl Distances {
    fn with(mut self, a: u32, b: u32, metres: f32) -> Self {
        self.0.insert((SiteId(a), SiteId(b)), metres);
        self.0.insert((SiteId(b), SiteId(a)), metres);
        self
    }
}

impl TravelModel for Distances {
    fn travel_ticks(&self, from: SiteId, to: SiteId, speed_mps: f32) -> Option<u32> {
        if from == to {
            return Some(0);
        }
        self.0
            .get(&(from, to))
            .map(|m| (m / speed_mps).ceil() as u32)
    }
}

const DEPOT: SiteId = SiteId(1);
const SITE: SiteId = SiteId(2);
const FRONT: SiteId = SiteId(3);

fn cfg() -> SimConfig {
    SimConfig {
        load_ticks: 2,
        unload_ticks: 2,
        ..SimConfig::default()
    }
}

/// Depot, site and front; 100 m depot↔site, 200 m depot↔front; truck speed 10 m/s.
fn world() -> (LogisticsOrderSim, Distances) {
    let mut sim = LogisticsOrderSim::new(cfg());
    for (id, name) in [(DEPOT, "Depot"), (SITE, "Site"), (FRONT, "Front")] {
        ok(sim.apply(LogisticsOrderCommand::RegisterSite {
            site: id,
            name: name.into(),
        }));
    }
    let d = Distances::default()
        .with(1, 2, 100.0)
        .with(1, 3, 200.0)
        .with(2, 3, 150.0);
    (sim, d)
}

fn ok(out: CommandOutcome) -> CommandOutcome {
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "command rejected: {out:?}"
    );
    out
}

fn truck(sim: &mut LogisticsOrderSim, owner: OwnerId, cap: u32) -> VehicleId {
    match ok(sim.apply(LogisticsOrderCommand::AddVehicle {
        owner,
        home: DEPOT,
        class_mask: CargoClassMask::FREIGHT,
        capacity_units: cap,
        speed_mps: 10.0,
    })) {
        CommandOutcome::CreatedVehicle(id) => id,
        other => panic!("{other:?}"),
    }
}

fn cement() -> CargoKind {
    CargoKind::new("Cement")
}

fn ammo() -> CargoKind {
    CargoKind::new("ammo")
}

fn stock(sim: &mut LogisticsOrderSim, site: SiteId, cargo: CargoKind, n: u32) {
    ok(sim.apply(LogisticsOrderCommand::SetStock {
        site,
        cargo,
        on_hand: n,
    }));
}

fn run(sim: &mut LogisticsOrderSim, cargo: CargoKind, sink: SiteId, qty: u32) -> OrderId {
    match ok(sim.apply(LogisticsOrderCommand::CreateOneOffRun {
        owner: OwnerId::PLAYER,
        source: DEPOT,
        sink,
        cargo,
        quantity: qty,
        priority: Priority::Normal,
        vehicle_cap: None,
    })) {
        CommandOutcome::CreatedOrder(id) => id,
        other => panic!("{other:?}"),
    }
}

fn line(sim: &mut LogisticsOrderSim, cargo: CargoKind, sink: SiteId, p: Priority) -> OrderId {
    match ok(sim.apply(LogisticsOrderCommand::CreateStandingLine {
        owner: OwnerId::PLAYER,
        source: DEPOT,
        sink,
        cargo,
        priority: p,
        vehicle_cap: None,
    })) {
        CommandOutcome::CreatedOrder(id) => id,
        other => panic!("{other:?}"),
    }
}

fn step(sim: &mut LogisticsOrderSim, d: &Distances, n: u32) {
    for _ in 0..n {
        sim.tick(d);
    }
}

fn run_until_done(sim: &mut LogisticsOrderSim, d: &Distances, id: OrderId, max: u32) -> u32 {
    for t in 0..max {
        sim.tick(d);
        if sim.orders.get(&id).is_some_and(|o| o.status.is_terminal()) {
            return t + 1;
        }
    }
    panic!("order {id:?} not finished in {max} ticks: {:?}", sim.orders.get(&id));
}

// ── LO-1: one-off runs ──────────────────────────────────────────────────────────

#[test]
fn single_trip_run_delivers_and_completes() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 20);
    stock(&mut sim, DEPOT, cement(), 50);
    let id = run(&mut sim, cement(), SITE, 15);
    run_until_done(&mut sim, &d, id, 200);
    let o = &sim.orders[&id];
    assert_eq!(o.status, OrderStatus::Completed);
    assert_eq!(o.stats.delivered, 15);
    assert_eq!(o.stats.trips_done, 1);
    assert_eq!(sim.stock.get(SITE, &cement()).on_hand, 15);
    assert_eq!(sim.stock.get(DEPOT, &cement()).on_hand, 35);
    assert_eq!(sim.stock.get(DEPOT, &cement()).reserved_out, 0);
    assert_eq!(sim.stock.get(SITE, &cement()).inbound, 0);
    assert!(sim.vehicles.values().all(Vehicle::is_free), "vehicle released");
}

#[test]
fn run_larger_than_pool_repeats_trips_until_fulfilled() {
    let (mut sim, d) = world();
    let v = truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 100);
    let id = run(&mut sim, cement(), SITE, 45);
    sim.tick(&d);
    assert_eq!(sim.planned_trips(&sim.orders[&id]), 5, "45 / 10 → 5 trips");
    run_until_done(&mut sim, &d, id, 2_000);
    let o = &sim.orders[&id];
    assert_eq!(o.status, OrderStatus::Completed);
    assert_eq!(o.stats.delivered, 45);
    assert_eq!(o.stats.trips_done, 5, "four full + one partial (5 t) trip");
    assert_eq!(sim.stock.get(SITE, &cement()).on_hand, 45);
    assert_eq!(sim.stock.get(DEPOT, &cement()).on_hand, 55);
    assert!(sim.vehicles[&v].is_free());
    assert!(sim
        .events
        .iter()
        .any(|e| e.order == Some(id) && e.kind == OrderEventKind::Completed));
}

#[test]
fn run_uses_whole_pool_in_parallel() {
    let (mut sim, d) = world();
    for _ in 0..3 {
        truck(&mut sim, OwnerId::PLAYER, 10);
    }
    stock(&mut sim, DEPOT, cement(), 100);
    let id = run(&mut sim, cement(), SITE, 30);
    sim.tick(&d);
    assert_eq!(sim.vehicles_on(id), 3, "all three trucks dispatched at once");
    let ticks = run_until_done(&mut sim, &d, id, 500);
    assert_eq!(sim.orders[&id].stats.trips_done, 3);
    // One round: load 2 + haul 10 + unload 2 (+1 dispatch tick, +1 close tick).
    assert!(ticks <= 18, "parallel trips finished in {ticks} ticks");
}

#[test]
fn vehicle_cap_limits_concurrent_trucks() {
    let (mut sim, d) = world();
    for _ in 0..3 {
        truck(&mut sim, OwnerId::PLAYER, 10);
    }
    stock(&mut sim, DEPOT, cement(), 100);
    let id = match ok(sim.apply(LogisticsOrderCommand::CreateOneOffRun {
        owner: OwnerId::PLAYER,
        source: DEPOT,
        sink: SITE,
        cargo: cement(),
        quantity: 30,
        priority: Priority::Normal,
        vehicle_cap: Some(1),
    })) {
        CommandOutcome::CreatedOrder(id) => id,
        other => panic!("{other:?}"),
    };
    sim.tick(&d);
    assert_eq!(sim.vehicles_on(id), 1);
    run_until_done(&mut sim, &d, id, 2_000);
    assert_eq!(sim.orders[&id].stats.trips_done, 3);
}

#[test]
fn run_waits_on_empty_source_then_resumes_with_partial_loads() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 4);
    let id = run(&mut sim, cement(), SITE, 12);
    step(&mut sim, &d, 60);
    assert_eq!(sim.orders[&id].stats.delivered, 4, "partial load moved what existed");
    assert_eq!(
        sim.orders[&id].status,
        OrderStatus::Waiting(WaitReason::SourceEmpty)
    );
    ok(sim.apply(LogisticsOrderCommand::AddStock {
        site: DEPOT,
        cargo: cement(),
        amount: 20,
    }));
    run_until_done(&mut sim, &d, id, 500);
    assert_eq!(sim.orders[&id].stats.delivered, 12);
    assert_eq!(sim.stock.get(DEPOT, &cement()).on_hand, 12);
}

#[test]
fn run_without_vehicles_reports_why() {
    let (mut sim, d) = world();
    stock(&mut sim, DEPOT, cement(), 10);
    let id = run(&mut sim, cement(), SITE, 5);
    sim.tick(&d);
    assert_eq!(
        sim.orders[&id].status,
        OrderStatus::Waiting(WaitReason::NoCompatibleVehicle)
    );
    // Tanker can't carry cement.
    ok(sim.apply(LogisticsOrderCommand::AddVehicle {
        owner: OwnerId::PLAYER,
        home: DEPOT,
        class_mask: CargoClassMask::TANKER,
        capacity_units: 10,
        speed_mps: 10.0,
    }));
    sim.tick(&d);
    assert_eq!(
        sim.orders[&id].status,
        OrderStatus::Waiting(WaitReason::NoCompatibleVehicle)
    );
    // Busy truck → NoVehicles for a second run.
    truck(&mut sim, OwnerId::PLAYER, 10);
    sim.tick(&d);
    assert_eq!(sim.orders[&id].status, OrderStatus::Active);
    stock(&mut sim, DEPOT, cement(), 50);
    let second = run(&mut sim, cement(), FRONT, 5);
    sim.tick(&d);
    assert_eq!(
        sim.orders[&second].status,
        OrderStatus::Waiting(WaitReason::NoVehicles)
    );
}

#[test]
fn unreachable_sink_waits() {
    let (mut sim, _) = world();
    let d = Distances::default().with(1, 2, 100.0); // no route to the front
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 10);
    let id = run(&mut sim, cement(), FRONT, 5);
    sim.tick(&d);
    assert_eq!(
        sim.orders[&id].status,
        OrderStatus::Waiting(WaitReason::Unreachable)
    );
}

#[test]
fn cancel_aborts_pickups_but_loaded_trucks_deliver() {
    let (mut sim, d) = world();
    let near = truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 100);
    // Park a second truck at the site so it needs a 10-tick deadhead to the depot.
    let far = truck(&mut sim, OwnerId::PLAYER, 10);
    sim.vehicles.get_mut(&far).unwrap().at = SITE;
    let id = run(&mut sim, cement(), SITE, 100);
    sim.tick(&d); // allocate both
    step(&mut sim, &d, 4); // near: loaded and hauling; far: still deadheading
    assert!(sim.vehicles[&near].phase.is_loaded());
    assert!(matches!(sim.vehicles[&far].phase, VehiclePhase::ToPickup(_)));
    ok(sim.apply(LogisticsOrderCommand::Cancel { order: id }));
    assert_eq!(sim.orders[&id].status, OrderStatus::Cancelling);
    assert!(sim.vehicles[&far].is_free(), "unloaded pickup released at once");
    assert_eq!(sim.stock.get(DEPOT, &cement()).reserved_out, 0);
    run_until_done(&mut sim, &d, id, 200);
    let o = &sim.orders[&id];
    assert_eq!(o.status, OrderStatus::Cancelled);
    assert_eq!(o.stats.delivered, 10, "the loaded truck still delivered");
    assert_eq!(sim.stock.get(SITE, &cement()).on_hand, 10);
    assert_eq!(sim.stock.get(SITE, &cement()).inbound, 0);
    assert!(sim.vehicles.values().all(Vehicle::is_free));
}

#[test]
fn paused_run_takes_no_new_trips() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 100);
    let id = run(&mut sim, cement(), SITE, 40);
    ok(sim.apply(LogisticsOrderCommand::Pause { order: id }));
    step(&mut sim, &d, 50);
    assert_eq!(sim.orders[&id].stats.delivered, 0);
    assert_eq!(sim.orders[&id].status, OrderStatus::Paused);
    ok(sim.apply(LogisticsOrderCommand::Resume { order: id }));
    run_until_done(&mut sim, &d, id, 2_000);
    assert_eq!(sim.orders[&id].stats.delivered, 40);
}

#[test]
fn removing_sink_fails_order_and_frees_vehicles() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 100);
    let id = run(&mut sim, cement(), SITE, 30);
    step(&mut sim, &d, 6); // loaded and hauling
    ok(sim.apply(LogisticsOrderCommand::RemoveSite { site: SITE }));
    let o = &sim.orders[&id];
    assert_eq!(o.status, OrderStatus::Failed(FailReason::SiteRemoved));
    assert_eq!(o.stats.lost, 10);
    assert!(sim.vehicles.values().all(Vehicle::is_free));
}

#[test]
fn owners_use_their_own_pools() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId(7), 10); // AI truck
    stock(&mut sim, DEPOT, cement(), 100);
    let id = run(&mut sim, cement(), SITE, 10);
    sim.tick(&d);
    assert_eq!(
        sim.orders[&id].status,
        OrderStatus::Waiting(WaitReason::NoCompatibleVehicle),
        "player run must not borrow the AI pool"
    );
}

#[test]
fn rejects_bad_commands() {
    let (mut sim, _) = world();
    for cmd in [
        LogisticsOrderCommand::CreateOneOffRun {
            owner: OwnerId::PLAYER,
            source: DEPOT,
            sink: SiteId(99),
            cargo: cement(),
            quantity: 5,
            priority: Priority::Normal,
            vehicle_cap: None,
        },
        LogisticsOrderCommand::CreateOneOffRun {
            owner: OwnerId::PLAYER,
            source: DEPOT,
            sink: SITE,
            cargo: cement(),
            quantity: 0,
            priority: Priority::Normal,
            vehicle_cap: None,
        },
        LogisticsOrderCommand::CreateStandingLine {
            owner: OwnerId::PLAYER,
            source: DEPOT,
            sink: DEPOT,
            cargo: cement(),
            priority: Priority::Normal,
            vehicle_cap: None,
        },
        LogisticsOrderCommand::Cancel { order: OrderId(42) },
    ] {
        assert!(matches!(sim.apply(cmd), CommandOutcome::Rejected(_)));
    }
    assert_eq!(sim.rejected.len(), 4);
}

#[test]
fn finished_orders_are_pruned_after_keep_window() {
    let (mut sim, d) = world();
    sim.config.keep_finished_ticks = 5;
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 10);
    let id = run(&mut sim, cement(), SITE, 10);
    run_until_done(&mut sim, &d, id, 200);
    step(&mut sim, &d, 6);
    assert!(!sim.orders.contains_key(&id));
}

// ── LO-2: draw demands + standing lines ─────────────────────────────────────────

fn demand(sim: &mut LogisticsOrderSim, site: SiteId, cargo: CargoKind, target: u32, reorder: u32, rate: u32, p: Priority) {
    ok(sim.apply(LogisticsOrderCommand::PublishDemand {
        site,
        cargo,
        consumer: ConsumerKind::Front,
        buffer_target: target,
        reorder_point: reorder,
        draw_rate_per_min: rate,
        priority: p,
    }));
}

#[test]
fn demand_draws_stock_and_reports_starving() {
    let (mut sim, d) = world();
    stock(&mut sim, FRONT, ammo(), 2);
    demand(&mut sim, FRONT, ammo(), 20, 10, 60, Priority::High); // 1 / tick
    step(&mut sim, &d, 1);
    assert_eq!(sim.stock.get(FRONT, &ammo()).on_hand, 1);
    assert_eq!(sim.demands[&(FRONT, ammo())].health, DemandHealth::Low);
    step(&mut sim, &d, 3);
    let dd = &sim.demands[&(FRONT, ammo())];
    assert_eq!(dd.health, DemandHealth::Starving);
    assert_eq!(dd.consumed_total, 2);
    assert_eq!(dd.shortfall_total, 2);
    assert!(sim
        .events
        .iter()
        .any(|e| matches!(&e.kind, OrderEventKind::DemandStarving { site, .. } if *site == FRONT)));
}

#[test]
fn fractional_draw_rate_is_exact() {
    let (mut sim, d) = world();
    stock(&mut sim, FRONT, ammo(), 100);
    demand(&mut sim, FRONT, ammo(), 100, 0, 30, Priority::Normal); // 0.5 / tick
    step(&mut sim, &d, 120);
    assert_eq!(sim.demands[&(FRONT, ammo())].consumed_total, 60);
}

#[test]
fn standing_line_refills_to_target_then_idles() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, ammo(), 1_000);
    demand(&mut sim, FRONT, ammo(), 30, 10, 0, Priority::Normal);
    let id = line(&mut sim, ammo(), FRONT, Priority::Normal);
    step(&mut sim, &d, 400);
    assert_eq!(sim.stock.get(FRONT, &ammo()).on_hand, 30, "filled exactly to target");
    assert_eq!(sim.orders[&id].status, OrderStatus::Idle);
    assert_eq!(sim.orders[&id].stats.trips_done, 3);
    assert!(sim.vehicles.values().all(Vehicle::is_free), "line releases trucks");
}

#[test]
fn standing_line_hysteresis_waits_for_reorder_point() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, ammo(), 1_000);
    stock(&mut sim, FRONT, ammo(), 30);
    demand(&mut sim, FRONT, ammo(), 30, 10, 60, Priority::Normal); // 1 / tick
    let id = line(&mut sim, ammo(), FRONT, Priority::Normal);
    step(&mut sim, &d, 15);
    assert_eq!(sim.orders[&id].stats.trips_in_flight, 0, "no truck for small deficits");
    assert_eq!(sim.orders[&id].status, OrderStatus::Idle);
    step(&mut sim, &d, 10); // projected < 10 → refill
    assert!(sim.orders[&id].stats.trips_in_flight > 0);
}

#[test]
fn standing_line_sustains_consuming_front() {
    let (mut sim, d) = world();
    for _ in 0..3 {
        truck(&mut sim, OwnerId::PLAYER, 10);
    }
    stock(&mut sim, DEPOT, ammo(), 10_000);
    stock(&mut sim, FRONT, ammo(), 40);
    // 20 t/min against 3 trucks × 10 t per ~45-tick cycle (~40 t/min capacity).
    demand(&mut sim, FRONT, ammo(), 40, 20, 20, Priority::High);
    line(&mut sim, ammo(), FRONT, Priority::High);
    step(&mut sim, &d, 1_800);
    let dd = &sim.demands[&(FRONT, ammo())];
    assert_eq!(dd.shortfall_total, 0, "front never ran dry");
    assert_eq!(dd.consumed_total, 600);
}

#[test]
fn standing_line_without_demand_waits_no_demand() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, ammo(), 100);
    let id = line(&mut sim, ammo(), FRONT, Priority::Normal);
    sim.tick(&d);
    assert_eq!(
        sim.orders[&id].status,
        OrderStatus::Waiting(WaitReason::NoDemand)
    );
}

#[test]
fn two_lines_share_one_deficit() {
    let (mut sim, d) = world();
    for _ in 0..4 {
        truck(&mut sim, OwnerId::PLAYER, 10);
    }
    stock(&mut sim, DEPOT, ammo(), 1_000);
    ok(sim.apply(LogisticsOrderCommand::CreateStandingLine {
        owner: OwnerId::PLAYER,
        source: SITE,
        sink: FRONT,
        cargo: ammo(),
        priority: Priority::Normal,
        vehicle_cap: None,
    }));
    stock(&mut sim, SITE, ammo(), 1_000);
    demand(&mut sim, FRONT, ammo(), 20, 5, 0, Priority::Normal);
    line(&mut sim, ammo(), FRONT, Priority::Normal);
    sim.tick(&d);
    assert_eq!(sim.stock.get(FRONT, &ammo()).inbound, 20, "no over-ordering");
    step(&mut sim, &d, 300);
    assert_eq!(sim.stock.get(FRONT, &ammo()).on_hand, 20);
}

#[test]
fn critical_front_beats_big_low_priority_run() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 1_000);
    stock(&mut sim, DEPOT, ammo(), 1_000);
    let big = match ok(sim.apply(LogisticsOrderCommand::CreateOneOffRun {
        owner: OwnerId::PLAYER,
        source: DEPOT,
        sink: SITE,
        cargo: cement(),
        quantity: 500,
        priority: Priority::Low,
        vehicle_cap: None,
    })) {
        CommandOutcome::CreatedOrder(id) => id,
        other => panic!("{other:?}"),
    };
    demand(&mut sim, FRONT, ammo(), 10, 5, 0, Priority::Critical);
    let crit = line(&mut sim, ammo(), FRONT, Priority::Critical);
    sim.tick(&d);
    assert_eq!(sim.vehicles_on(crit), 1, "single truck goes to the critical line");
    assert_eq!(sim.vehicles_on(big), 0);
}

#[test]
fn aging_lets_low_priority_overtake_normal_eventually() {
    let (mut sim, d) = world();
    sim.config.aging_interval_ticks = 1;
    sim.config.aging_cap = 150;
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 10_000);
    let low = match ok(sim.apply(LogisticsOrderCommand::CreateOneOffRun {
        owner: OwnerId::PLAYER,
        source: DEPOT,
        sink: SITE,
        cargo: cement(),
        quantity: 10,
        priority: Priority::Low,
        vehicle_cap: None,
    })) {
        CommandOutcome::CreatedOrder(id) => id,
        other => panic!("{other:?}"),
    };
    let normal = run(&mut sim, cement(), FRONT, 10_000);
    step(&mut sim, &d, 400);
    assert!(sim.orders[&normal].stats.trips_done > 0);
    assert!(
        sim.orders.get(&low).is_none_or(|o| o.stats.delivered == 10),
        "aged low-priority run got a truck: {:?}",
        sim.orders.get(&low)
    );
}

#[test]
fn round_robin_splits_pool_between_equal_orders() {
    let (mut sim, d) = world();
    for _ in 0..4 {
        truck(&mut sim, OwnerId::PLAYER, 10);
    }
    stock(&mut sim, DEPOT, cement(), 10_000);
    let a = run(&mut sim, cement(), SITE, 1_000);
    let b = run(&mut sim, cement(), FRONT, 1_000);
    sim.tick(&d);
    assert_eq!(sim.vehicles_on(a), 2);
    assert_eq!(sim.vehicles_on(b), 2);
}

#[test]
fn personnel_need_troop_transport() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, CargoKind::new("personnel"), 24);
    let id = run(&mut sim, CargoKind::new("personnel"), FRONT, 24);
    sim.tick(&d);
    assert_eq!(
        sim.orders[&id].status,
        OrderStatus::Waiting(WaitReason::NoCompatibleVehicle)
    );
    ok(sim.apply(LogisticsOrderCommand::AddVehicle {
        owner: OwnerId::PLAYER,
        home: DEPOT,
        class_mask: CargoClassMask::TROOP,
        capacity_units: 12,
        speed_mps: 15.0,
    }));
    run_until_done(&mut sim, &d, id, 1_000);
    assert_eq!(sim.orders[&id].stats.trips_done, 2);
}

#[test]
fn identical_command_streams_are_deterministic() {
    fn scenario() -> LogisticsOrderSim {
        let (mut sim, d) = world();
        for cap in [10, 12, 8] {
            truck(&mut sim, OwnerId::PLAYER, cap);
        }
        stock(&mut sim, DEPOT, cement(), 300);
        stock(&mut sim, DEPOT, ammo(), 300);
        demand(&mut sim, FRONT, ammo(), 40, 15, 25, Priority::High);
        line(&mut sim, ammo(), FRONT, Priority::High);
        run(&mut sim, cement(), SITE, 77);
        step(&mut sim, &d, 250);
        run(&mut sim, cement(), FRONT, 33);
        step(&mut sim, &d, 250);
        sim
    }
    let a = scenario();
    let b = scenario();
    assert_eq!(a.stock, b.stock);
    assert_eq!(a.orders, b.orders);
    assert_eq!(a.vehicles, b.vehicles);
    assert_eq!(a.events, b.events);
}
