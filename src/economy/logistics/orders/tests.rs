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

// ── LO-3: snapshot + Bevy adapter ───────────────────────────────────────────────

#[test]
fn snapshot_rows_show_trip_progress_and_demand_service() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, cement(), 100);
    stock(&mut sim, DEPOT, ammo(), 100);
    let id = run(&mut sim, cement(), SITE, 25);
    demand(&mut sim, FRONT, ammo(), 20, 10, 0, Priority::High);
    demand(&mut sim, SITE, ammo(), 5, 1, 0, Priority::Low);
    line(&mut sim, ammo(), FRONT, Priority::Normal);
    step(&mut sim, &d, 3);
    let mut meta = BTreeMap::new();
    meta.insert(
        FRONT,
        SiteMeta {
            name: "Front Alpha".into(),
            owner: OwnerId::PLAYER,
            role: SiteRole::Front,
            world_xz: [160.0, 32.0],
        },
    );
    let snap = build_snapshot(&sim, &meta, &d);
    let row = snap.orders.iter().find(|o| o.id == id).unwrap();
    assert!(!row.standing);
    assert_eq!(row.quantity, Some(25));
    assert_eq!(row.unit, "t");
    assert_eq!(row.sink_name, "Site");
    assert!(row.trips_planned >= 3);
    let front = snap.site(FRONT).unwrap();
    assert_eq!(front.tile, [10, 2]);
    assert_eq!(front.role, SiteRole::Front);
    assert_eq!(snap.sites_of(OwnerId::PLAYER, SiteRole::Front).count(), 1);
    let served: Vec<_> = snap.unserved_demands(OwnerId::PLAYER).map(|d| d.site).collect();
    assert_eq!(served, vec![SITE], "front demand is served by the line; site demand is not");
    assert_eq!(snap.fleet(OwnerId::PLAYER).total, 1);
    assert_eq!(snap.fleet(OwnerId::PLAYER).busy, 1);
    assert!(row.next_eta_ticks.is_some());
}

#[test]
fn plant_reconcile_applies_order_deltas_to_the_node_buffer() {
    use super::plugin::reconcile_plant_stock;
    // First contact: node 50.7 t, sim empty → sim sees 50 whole units.
    assert_eq!(reconcile_plant_stock(50.7, 0, None), (50.7, 50, 50));
    // A truck picked up 10 since the mirror (sim 40, mirror 50) → node loses 10.
    let (node, sim, mirror) = reconcile_plant_stock(50.7, 40, Some(50));
    assert!((node - 40.7).abs() < 1e-4);
    assert_eq!((sim, mirror), (40, 40));
    // A delivery of 12 arrived (sim 52, mirror 40) while the plant consumed 5 (node 35.7).
    let (node, sim, _) = reconcile_plant_stock(35.7, 52, Some(40));
    assert!((node - 47.7).abs() < 1e-4);
    assert_eq!(sim, 47);
    // Dev-seeded sim stock before first contact is pushed into the node.
    assert_eq!(reconcile_plant_stock(0.0, 8, None), (8.0, 8, 8));
}

#[test]
fn due_steps_counts_whole_seconds_and_caps_backlog() {
    use super::plugin::due_steps;
    assert_eq!(due_steps(None, 5_000_000, 1_000_000, 30), (0, 5_000_000));
    assert_eq!(due_steps(Some(0), 2_500_000, 1_000_000, 30), (2, 2_000_000));
    assert_eq!(due_steps(Some(2_000_000), 2_400_000, 1_000_000, 30), (0, 2_000_000));
    assert_eq!(due_steps(Some(0), 100_000_000, 1_000_000, 30), (30, 100_000_000));
}

#[test]
fn site_distance_travel_uses_metres_and_speed() {
    let mut t = SiteDistanceTravel::default();
    t.detour = 1.0;
    t.positions.insert(SiteId(1), [0.0, 0.0]);
    t.positions.insert(SiteId(2), [300.0, 400.0]); // 500 m
    assert_eq!(t.travel_ticks(SiteId(1), SiteId(2), 10.0), Some(50));
    assert_eq!(t.travel_ticks(SiteId(1), SiteId(1), 10.0), Some(0));
    assert_eq!(t.travel_ticks(SiteId(1), SiteId(9), 10.0), None);
}

// ── LO-6: forecasts + advisor ───────────────────────────────────────────────────

fn meta_for(sim: &LogisticsOrderSim, roles: &[(SiteId, SiteRole, f32)]) -> BTreeMap<SiteId, SiteMeta> {
    roles
        .iter()
        .map(|&(id, role, x)| {
            (
                id,
                SiteMeta {
                    name: sim.sites[&id].clone(),
                    owner: OwnerId::PLAYER,
                    role,
                    world_xz: [x, 0.0],
                },
            )
        })
        .collect()
}

#[test]
fn demand_rows_forecast_time_to_empty_and_trucks_needed() {
    let (mut sim, d) = world();
    truck(&mut sim, OwnerId::PLAYER, 10);
    stock(&mut sim, DEPOT, ammo(), 1_000);
    stock(&mut sim, FRONT, ammo(), 60);
    // 30 t/min; route 200 m at 10 m/s = 20 ticks/leg → round trip 20+20+2+2 = 44 ticks.
    demand(&mut sim, FRONT, ammo(), 60, 10, 30, Priority::High);
    line(&mut sim, ammo(), FRONT, Priority::High);
    let snap = build_snapshot(&sim, &BTreeMap::new(), &d);
    let row = &snap.demands[0];
    assert_eq!(row.time_to_empty_ticks, Some(120), "60 t at 30 t/min = 2 min");
    assert_eq!(row.round_trip_ticks, Some(44));
    // 30 t/min × 44 s ÷ (10 t × 60 s/min) = 2.2 → 3 trucks.
    assert_eq!(row.trucks_needed, Some(3));
}

#[test]
fn advisor_picks_stocked_nearest_source_and_reports_gaps() {
    let (mut sim, d) = world();
    // SITE (100 m from depot, 150 m from front) acts as a second depot.
    stock(&mut sim, DEPOT, ammo(), 500);
    stock(&mut sim, SITE, ammo(), 500);
    demand(&mut sim, FRONT, ammo(), 40, 10, 10, Priority::Critical);
    demand(&mut sim, FRONT, CargoKind::new("fuel"), 20, 5, 0, Priority::Low);
    let meta = meta_for(
        &sim,
        &[
            (DEPOT, SiteRole::Depot, 0.0),
            (SITE, SiteRole::Depot, 150.0),
            (FRONT, SiteRole::Front, 200.0),
        ],
    );
    let snap = build_snapshot(&sim, &meta, &d);
    let advice = suggest_lines_for_unserved(&snap, OwnerId::PLAYER);
    assert_eq!(
        advice.commands,
        vec![LogisticsOrderCommand::CreateStandingLine {
            owner: OwnerId::PLAYER,
            source: SITE,
            sink: FRONT,
            cargo: ammo(),
            priority: Priority::Critical,
            vehicle_cap: None,
        }],
        "nearer stocked depot wins"
    );
    assert_eq!(
        advice.gaps,
        vec![AdvisorGap::NoSourceWithStock {
            site: FRONT,
            cargo: "fuel".into()
        }]
    );
    // Applying the advice serves the demand.
    for c in advice.commands {
        ok(sim.apply(c));
    }
    let snap = build_snapshot(&sim, &meta, &d);
    assert!(snap
        .unserved_demands(OwnerId::PLAYER)
        .all(|r| r.cargo == "fuel"));
}

#[test]
fn advisor_prefers_source_that_covers_the_whole_target() {
    let (mut sim, d) = world();
    stock(&mut sim, SITE, ammo(), 5); // near but nearly empty
    stock(&mut sim, DEPOT, ammo(), 100); // farther, full
    demand(&mut sim, FRONT, ammo(), 40, 10, 0, Priority::Normal);
    let meta = meta_for(
        &sim,
        &[
            (DEPOT, SiteRole::Depot, 0.0),
            (SITE, SiteRole::Depot, 190.0),
            (FRONT, SiteRole::Front, 200.0),
        ],
    );
    let snap = build_snapshot(&sim, &meta, &d);
    let demand_row = &snap.demands[0];
    assert_eq!(best_source_for(&snap, demand_row), Some(DEPOT));
}

// ── LO-7: persistence ───────────────────────────────────────────────────────────

#[test]
fn ron_round_trip_mid_run_continues_identically() {
    use super::persistence::{
        logistics_orders_save_from_ron, logistics_orders_save_to_ron, LogisticsOrdersSave,
        LOGISTICS_ORDERS_SAVE_SCHEMA,
    };
    let (mut sim, d) = world();
    for cap in [10, 12] {
        truck(&mut sim, OwnerId::PLAYER, cap);
    }
    stock(&mut sim, DEPOT, cement(), 300);
    stock(&mut sim, DEPOT, ammo(), 300);
    demand(&mut sim, FRONT, ammo(), 40, 15, 25, Priority::High);
    line(&mut sim, ammo(), FRONT, Priority::High);
    let id = run(&mut sim, cement(), SITE, 77);
    step(&mut sim, &d, 37); // trucks mid-leg, run partly delivered

    let save = LogisticsOrdersSave {
        schema_version: LOGISTICS_ORDERS_SAVE_SCHEMA,
        sim: sim.clone(),
        sites: vec![],
    };
    let text = logistics_orders_save_to_ron(&save).unwrap();
    let mut loaded = logistics_orders_save_from_ron(&text).unwrap().sim;
    assert_eq!(loaded.orders, sim.orders);
    assert_eq!(loaded.vehicles, sim.vehicles);

    step(&mut sim, &d, 400);
    step(&mut loaded, &d, 400);
    assert_eq!(loaded.orders, sim.orders, "reloaded sim diverged");
    assert_eq!(loaded.stock, sim.stock);
    assert_eq!(loaded.events, sim.events);
    // Ids keep allocating past the saved ones.
    let next = run(&mut loaded, cement(), SITE, 5);
    assert!(next > id);
}

#[test]
fn saved_sites_rebind_by_name_owner_role_within_a_tile() {
    use super::persistence::{saved_site_matches, SavedSite};
    let saved = SavedSite {
        id: SiteId(4),
        name: "Front Alpha".into(),
        owner: OwnerId(2),
        role: SiteRole::Front,
        world_xz: [100.0, 100.0],
    };
    let mut meta = SiteMeta {
        name: "Front Alpha".into(),
        owner: OwnerId(2),
        role: SiteRole::Front,
        world_xz: [110.0, 105.0],
    };
    assert!(saved_site_matches(&saved, &meta));
    meta.world_xz = [130.0, 100.0];
    assert!(!saved_site_matches(&saved, &meta), "two tiles away is another site");
    meta.world_xz = [100.0, 100.0];
    meta.owner = OwnerId(3);
    assert!(!saved_site_matches(&saved, &meta));
}

mod app {
    use bevy::prelude::*;

    use super::super::*;
    use crate::systems::sim_control::SimTimeMicros;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SimTimeMicros>();
        app.add_plugins(LogisticsOrdersPlugin);
        app
    }

    fn advance_secs(app: &mut App, secs: u64) {
        for _ in 0..secs {
            app.world_mut().resource_mut::<SimTimeMicros>().0 += 1_000_000;
            app.update();
        }
    }

    fn spawn_site(app: &mut App, name: &str, role: SiteRole, x: f32) -> Entity {
        app.world_mut()
            .spawn((
                LogisticsOrderSite {
                    name: name.into(),
                    owner: OwnerId::PLAYER,
                    role,
                    position_m: None,
                },
                Transform::from_xyz(x, 0.0, 0.0),
            ))
            .id()
    }

    fn send(app: &mut App, cmd: LogisticsOrderCommand) {
        app.world_mut().write_message(LogisticsOrderRequest(cmd));
    }

    #[test]
    fn requests_drive_a_multi_trip_run_to_completion_in_app() {
        let mut app = app();
        let depot = spawn_site(&mut app, "Depot North", SiteRole::Depot, 0.0);
        let site = spawn_site(&mut app, "Bridge site", SiteRole::Construction, 240.0);
        app.update();
        app.update();
        let reg = app.world().resource::<LogisticsSiteRegistry>();
        let (src, dst) = (reg.site_of(depot).unwrap(), reg.site_of(site).unwrap());
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert_eq!(snap.fleet(OwnerId::PLAYER).total, 6, "depot seeded with default fleet");

        send(
            &mut app,
            LogisticsOrderCommand::SetStock {
                site: src,
                cargo: CargoKind::new("Cement"),
                on_hand: 500,
            },
        );
        send(
            &mut app,
            LogisticsOrderCommand::CreateOneOffRun {
                owner: OwnerId::PLAYER,
                source: src,
                sink: dst,
                cargo: CargoKind::new("Cement"),
                quantity: 120,
                priority: Priority::High,
                vehicle_cap: None,
            },
        );
        advance_secs(&mut app, 2);
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        let row = snap.one_off_runs().next().expect("run row");
        assert_eq!(row.vehicles, 4, "all four freight trucks dispatched");
        assert_eq!(row.trips_planned, 12);
        advance_secs(&mut app, 600);
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        let row = snap.one_off_runs().next().expect("run row kept for review");
        assert_eq!(row.status, OrderStatus::Completed);
        assert_eq!(row.delivered, 120);
        assert_eq!(row.trips_done, 12);
        assert_eq!(snap.fleet(OwnerId::PLAYER).free, 6, "vehicles released");
        let dst_row = snap.site(dst).unwrap();
        assert_eq!(dst_row.stock[0].on_hand, 120);
    }

    #[test]
    fn plant_buffers_feed_runs_and_receive_deliveries() {
        use crate::economy::resource_flow::ResourceFlowNode;
        use std::collections::HashMap;

        let mut app = app();
        let depot = spawn_site(&mut app, "Depot", SiteRole::Depot, 0.0);
        let plant = app
            .world_mut()
            .spawn((
                LogisticsOrderSite {
                    name: "Cement works".into(),
                    owner: OwnerId::PLAYER,
                    role: SiteRole::Plant,
                    position_m: Some([320.0, 0.0]),
                },
                ResourceFlowNode {
                    catalog_id: "cement_works".into(),
                    inventory: HashMap::new(),
                    buffer_by_tag: HashMap::from([("Cement".to_string(), 50.5)]),
                    throughput_limit: 10.0,
                    production: vec![],
                    consumption: vec![],
                },
            ))
            .id();
        app.update();
        app.update();
        let reg = app.world().resource::<LogisticsSiteRegistry>();
        let (d, p) = (reg.site_of(depot).unwrap(), reg.site_of(plant).unwrap());
        send(
            &mut app,
            LogisticsOrderCommand::SetStock {
                site: d,
                cargo: CargoKind::new("Limestone"),
                on_hand: 100,
            },
        );
        for (source, sink, cargo, quantity) in [(p, d, "Cement", 30), (d, p, "Limestone", 15)] {
            send(
                &mut app,
                LogisticsOrderCommand::CreateOneOffRun {
                    owner: OwnerId::PLAYER,
                    source,
                    sink,
                    cargo: CargoKind::new(cargo),
                    quantity,
                    priority: Priority::Normal,
                    vehicle_cap: None,
                },
            );
        }
        advance_secs(&mut app, 600);
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert!(snap.one_off_runs().all(|o| o.status == OrderStatus::Completed), "{:?}", snap.orders);
        let node = app.world().get::<ResourceFlowNode>(plant).unwrap();
        assert!((node.buffer_by_tag["Cement"] - 20.5).abs() < 1e-4, "30 t picked up from the plant buffer");
        assert!((node.buffer_by_tag["Limestone"] - 15.0).abs() < 1e-4, "delivery landed in the plant buffer");
        let depot_row = snap.site(d).unwrap();
        let cement = depot_row.stock.iter().find(|r| r.cargo == "Cement").unwrap();
        assert_eq!(cement.on_hand, 30);
    }

    #[test]
    fn save_then_load_into_fresh_app_rebinds_sites_and_keeps_orders() {
        let mut app = app();
        let depot = spawn_site(&mut app, "Depot", SiteRole::Depot, 0.0);
        let front = spawn_site(&mut app, "Front", SiteRole::Front, 600.0);
        app.update();
        app.update();
        let reg = app.world().resource::<LogisticsSiteRegistry>();
        let (src, dst) = (reg.site_of(depot).unwrap(), reg.site_of(front).unwrap());
        send(
            &mut app,
            LogisticsOrderCommand::SetStock {
                site: src,
                cargo: CargoKind::new("ammo"),
                on_hand: 400,
            },
        );
        send(
            &mut app,
            LogisticsOrderCommand::CreateOneOffRun {
                owner: OwnerId::PLAYER,
                source: src,
                sink: dst,
                cargo: CargoKind::new("ammo"),
                quantity: 200,
                priority: Priority::Normal,
                vehicle_cap: None,
            },
        );
        advance_secs(&mut app, 90);
        let save = capture_logistics_orders_save(
            app.world().resource::<LogisticsOrderSimRes>(),
            app.world().resource::<LogisticsSiteRegistry>(),
        );
        let delivered_at_save = app.world().resource::<LogisticsOrdersSnapshotRes>().0.orders[0].delivered;

        // Fresh world: the lanes spawn their endpoints again (new entities, different order).
        let mut app2 = self::app();
        let front2 = spawn_site(&mut app2, "Front", SiteRole::Front, 600.0);
        let depot2 = spawn_site(&mut app2, "Depot", SiteRole::Depot, 0.0);
        {
            let world = app2.world_mut();
            world.resource_scope(|world, mut res: Mut<LogisticsOrderSimRes>| {
                world.resource_scope(|world, mut registry: Mut<LogisticsSiteRegistry>| {
                    let mut mirror = world.resource_mut::<super::super::plugin::PlantStockMirror>();
                    install_logistics_orders_save(&mut res, &mut registry, &mut mirror, save);
                });
            });
        }
        app2.update();
        app2.update();
        let reg2 = app2.world().resource::<LogisticsSiteRegistry>();
        assert_eq!(reg2.site_of(depot2), Some(src), "depot rebound to its saved id");
        assert_eq!(reg2.site_of(front2), Some(dst));
        assert!(reg2.pending_rebind.is_empty());
        let snap = &app2.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert_eq!(snap.fleet(OwnerId::PLAYER).total, 6, "no second seed fleet");
        assert_eq!(snap.orders[0].delivered, delivered_at_save);
        advance_secs(&mut app2, 900);
        let snap = &app2.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert_eq!(snap.orders[0].status, OrderStatus::Completed);
        assert_eq!(snap.orders[0].delivered, 200);
    }

    #[test]
    fn construction_site_publishes_material_demands_until_built() {
        use crate::strategic::{
            ConstructionSite, SiteArchetype, SiteConstructionPhase, SiteResourceManifest,
        };
        let mut app = app();
        let depot = spawn_site(&mut app, "Depot", SiteRole::Depot, 0.0);
        let site = app
            .world_mut()
            .spawn((
                ConstructionSite {
                    site_id: 7,
                    owner: Entity::PLACEHOLDER,
                    archetype: SiteArchetype::Factory,
                    phase: SiteConstructionPhase::UnderConstruction,
                    operational_readiness: 0.0,
                },
                SiteResourceManifest {
                    concrete_required: 100.0,
                    steel_required: 40.0,
                    fuel_required: 20.0,
                    machinery_required: 10.0,
                    ..Default::default()
                },
                Transform::from_xyz(400.0, 0.0, 0.0),
            ))
            .id();
        for _ in 0..3 {
            app.update();
        }
        let src = app.world().resource::<LogisticsSiteRegistry>().site_of(depot).unwrap();
        for (tag, n) in [("concrete", 300), ("steel", 100), ("fuel", 50), ("machinery", 30)] {
            send(
                &mut app,
                LogisticsOrderCommand::SetStock {
                    site: src,
                    cargo: CargoKind::new(tag),
                    on_hand: n,
                },
            );
        }
        app.update();
        app.update();
        let snap = app.world().resource::<LogisticsOrdersSnapshotRes>().0.clone();
        let construction: Vec<_> = snap
            .demands
            .iter()
            .filter(|d| d.consumer == ConsumerKind::Construction)
            .collect();
        assert_eq!(construction.len(), 4, "one demand per material");
        let advice = suggest_lines_for_unserved(&snap, OwnerId::PLAYER);
        assert_eq!(advice.commands.len(), 4);
        for c in advice.commands {
            send(&mut app, c);
        }
        advance_secs(&mut app, 900);
        let m = app.world().get::<SiteResourceManifest>(site).unwrap();
        assert_eq!(
            (m.delivered_concrete, m.delivered_steel, m.delivered_fuel, m.delivered_machinery),
            (100.0, 40.0, 20.0, 10.0),
            "deliveries fill the manifest"
        );
        assert!((m.delivered_ratio() - 1.0).abs() < 1e-6);
        let sid = app.world().resource::<LogisticsSiteRegistry>().site_of(site).unwrap();

        app.world_mut().get_mut::<ConstructionSite>(site).unwrap().phase =
            SiteConstructionPhase::Provisioning;
        advance_secs(&mut app, 3);
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert!(snap.demands.iter().all(|d| d.site != sid), "demands retracted");
        assert!(snap.site(sid).unwrap().stock.is_empty(), "materials consumed");
        assert!(
            snap.orders
                .iter()
                .filter(|o| o.sink == sid)
                .all(|o| o.status.is_terminal()),
            "lines into the finished site closed"
        );
    }

    #[test]
    fn despawned_site_fails_its_orders() {
        let mut app = app();
        let depot = spawn_site(&mut app, "Depot", SiteRole::Depot, 0.0);
        let front = spawn_site(&mut app, "Front", SiteRole::Front, 800.0);
        app.update();
        app.update();
        let reg = app.world().resource::<LogisticsSiteRegistry>();
        let (src, dst) = (reg.site_of(depot).unwrap(), reg.site_of(front).unwrap());
        send(
            &mut app,
            LogisticsOrderCommand::CreateStandingLine {
                owner: OwnerId::PLAYER,
                source: src,
                sink: dst,
                cargo: CargoKind::new("ammo"),
                priority: Priority::Normal,
                vehicle_cap: None,
            },
        );
        advance_secs(&mut app, 1);
        app.world_mut().despawn(front);
        advance_secs(&mut app, 1);
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        assert!(snap.site(dst).is_none());
        assert!(matches!(
            snap.orders[0].status,
            OrderStatus::Failed(FailReason::SiteRemoved)
        ));
    }
}

/// Scale probe: run with `--ignored --nocapture` to print timings.
#[test]
#[ignore]
fn scale_probe_many_orders_and_vehicles() {
    let mut sim = LogisticsOrderSim::new(cfg());
    let mut d = Distances::default();
    let n_sites = 60u32;
    for i in 1..=n_sites {
        ok(sim.apply(LogisticsOrderCommand::RegisterSite {
            site: SiteId(i),
            name: format!("S{i}"),
        }));
        for j in 1..i {
            d = d.with(i, j, 50.0 + f32::from(((i * 7 + j * 13) % 40) as u16) * 25.0);
        }
    }
    for i in 0..600u32 {
        ok(sim.apply(LogisticsOrderCommand::AddVehicle {
            owner: OwnerId::PLAYER,
            home: SiteId(1 + i % 10),
            class_mask: CargoClassMask::FREIGHT,
            capacity_units: 10,
            speed_mps: 12.0,
        }));
    }
    for s in 1..=10 {
        stock(&mut sim, SiteId(s), ammo(), 1_000_000);
        stock(&mut sim, SiteId(s), cement(), 1_000_000);
    }
    for k in 0..300u32 {
        let src = SiteId(1 + k % 10);
        let dst = SiteId(11 + k % (n_sites - 10));
        if k % 2 == 0 {
            ok(sim.apply(LogisticsOrderCommand::CreateOneOffRun {
                owner: OwnerId::PLAYER,
                source: src,
                sink: dst,
                cargo: cement(),
                quantity: 500,
                priority: Priority::ALL[(k % 4) as usize],
                vehicle_cap: None,
            }));
        } else {
            demand(&mut sim, dst, ammo(), 200, 80, 30, Priority::High);
            ok(sim.apply(LogisticsOrderCommand::CreateStandingLine {
                owner: OwnerId::PLAYER,
                source: src,
                sink: dst,
                cargo: ammo(),
                priority: Priority::Normal,
                vehicle_cap: Some(4),
            }));
        }
    }
    let t0 = std::time::Instant::now();
    sim.tick(&d);
    let first = t0.elapsed();
    let t1 = std::time::Instant::now();
    step(&mut sim, &d, 300);
    let rest = t1.elapsed();
    println!(
        "first tick {:?}; 300 ticks {:?} ({:?}/tick); busy {}",
        first,
        rest,
        rest / 300,
        sim.vehicles.values().filter(|v| !v.is_free()).count()
    );
}
