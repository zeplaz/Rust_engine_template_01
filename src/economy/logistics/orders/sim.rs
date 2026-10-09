//! Deterministic order simulation (PLAN-LOGISTICS-ORDERS-v1 §3.5, §4, §5).
//!
//! [`LogisticsOrderSim`] is the sole owner of order, vehicle, stock and demand state.
//! Writes arrive only through [`LogisticsOrderSim::apply`]; time advances only through
//! [`LogisticsOrderSim::tick`]. No Bevy types — tested headless in `tests.rs`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::model::*;

/// Travel time source. `None` = unreachable. Same-site travel should return `Some(0)`.
pub trait TravelModel {
    fn travel_ticks(&self, from: SiteId, to: SiteId, speed_mps: f32) -> Option<u32>;
}

/// Tunables. Defaults assume one sim tick per simulated second.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimConfig {
    pub load_ticks: u32,
    pub unload_ticks: u32,
    /// Ticks per simulated minute (converts `draw_rate_per_min`).
    pub ticks_per_minute: u32,
    /// Ticks of unmet waiting per +1 aging weight.
    pub aging_interval_ticks: u32,
    pub aging_cap: u32,
    /// Extra weight for orders feeding a starving demand.
    pub starving_bonus: u32,
    /// How long finished orders stay listed before pruning.
    pub keep_finished_ticks: u64,
    pub max_events: usize,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            load_ticks: 10,
            unload_ticks: 10,
            ticks_per_minute: 60,
            aging_interval_ticks: 30,
            aging_cap: 150,
            starving_bonus: 120,
            keep_finished_ticks: 600,
            max_events: 64,
        }
    }
}

/// Shared write API for player UI, AI and fronts (§5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LogisticsOrderCommand {
    RegisterSite {
        site: SiteId,
        name: String,
    },
    RemoveSite {
        site: SiteId,
    },
    AddVehicle {
        owner: OwnerId,
        home: SiteId,
        class_mask: CargoClassMask,
        capacity_units: u32,
        speed_mps: f32,
    },
    SetStock {
        site: SiteId,
        cargo: CargoKind,
        on_hand: u32,
    },
    /// Adds to on-hand stock (production output, bridge import).
    AddStock {
        site: SiteId,
        cargo: CargoKind,
        amount: u32,
    },
    CreateStandingLine {
        owner: OwnerId,
        source: SiteId,
        sink: SiteId,
        cargo: CargoKind,
        priority: Priority,
        vehicle_cap: Option<u32>,
    },
    CreateOneOffRun {
        owner: OwnerId,
        source: SiteId,
        sink: SiteId,
        cargo: CargoKind,
        quantity: u32,
        priority: Priority,
        vehicle_cap: Option<u32>,
    },
    Cancel {
        order: OrderId,
    },
    Pause {
        order: OrderId,
    },
    Resume {
        order: OrderId,
    },
    SetPriority {
        order: OrderId,
        priority: Priority,
    },
    /// Upsert keyed by `(site, cargo)` — fronts may republish every tick.
    PublishDemand {
        site: SiteId,
        cargo: CargoKind,
        consumer: ConsumerKind,
        buffer_target: u32,
        reorder_point: u32,
        draw_rate_per_min: u32,
        priority: Priority,
    },
    RetractDemand {
        site: SiteId,
        cargo: CargoKind,
    },
}

/// Result of [`LogisticsOrderSim::apply`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandOutcome {
    Ok,
    CreatedOrder(OrderId),
    CreatedVehicle(VehicleId),
    Rejected(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderEventKind {
    Completed,
    Cancelled,
    Failed(FailReason),
    /// A demand entered `Starving`.
    DemandStarving { site: SiteId, cargo: CargoKind },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderEvent {
    pub tick: Tick,
    pub order: Option<OrderId>,
    pub kind: OrderEventKind,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LogisticsOrderSim {
    pub config: SimConfig,
    pub tick: Tick,
    pub sites: BTreeMap<SiteId, String>,
    pub stock: StockLedger,
    pub vehicles: BTreeMap<VehicleId, Vehicle>,
    pub orders: BTreeMap<OrderId, Order>,
    pub demands: BTreeMap<(SiteId, CargoKind), DrawDemand>,
    pub events: Vec<OrderEvent>,
    pub rejected: Vec<(Tick, String)>,
    next_order: u32,
    next_vehicle: u32,
}

impl LogisticsOrderSim {
    #[must_use]
    pub fn new(config: SimConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    // ── commands ────────────────────────────────────────────────────────────────

    pub fn apply(&mut self, cmd: LogisticsOrderCommand) -> CommandOutcome {
        let out = self.apply_inner(cmd);
        if let CommandOutcome::Rejected(why) = &out {
            self.rejected.push((self.tick, why.clone()));
            if self.rejected.len() > 32 {
                self.rejected.drain(0..16);
            }
        }
        out
    }

    fn apply_inner(&mut self, cmd: LogisticsOrderCommand) -> CommandOutcome {
        use LogisticsOrderCommand as C;
        match cmd {
            C::RegisterSite { site, name } => {
                self.sites.insert(site, name);
                CommandOutcome::Ok
            }
            C::RemoveSite { site } => {
                if self.sites.remove(&site).is_none() {
                    return CommandOutcome::Rejected(format!("unknown site {}", site.0));
                }
                self.fail_orders_touching(site);
                self.demands.retain(|(s, _), _| *s != site);
                self.stock.entries.retain(|(s, _), _| *s != site);
                CommandOutcome::Ok
            }
            C::AddVehicle {
                owner,
                home,
                class_mask,
                capacity_units,
                speed_mps,
            } => {
                if !self.sites.contains_key(&home) {
                    return CommandOutcome::Rejected(format!("vehicle home {} unknown", home.0));
                }
                if capacity_units == 0 || !(speed_mps > 0.0) {
                    return CommandOutcome::Rejected("vehicle needs capacity and speed".into());
                }
                self.next_vehicle += 1;
                let id = VehicleId(self.next_vehicle);
                self.vehicles.insert(
                    id,
                    Vehicle {
                        id,
                        owner,
                        class_mask,
                        capacity_units,
                        speed_mps,
                        home,
                        at: home,
                        phase: VehiclePhase::Idle,
                        assignment: None,
                    },
                );
                CommandOutcome::CreatedVehicle(id)
            }
            C::SetStock {
                site,
                cargo,
                on_hand,
            } => {
                if !self.sites.contains_key(&site) {
                    return CommandOutcome::Rejected(format!("unknown site {}", site.0));
                }
                self.stock.entry_mut(site, &cargo).on_hand = on_hand;
                CommandOutcome::Ok
            }
            C::AddStock {
                site,
                cargo,
                amount,
            } => {
                if !self.sites.contains_key(&site) {
                    return CommandOutcome::Rejected(format!("unknown site {}", site.0));
                }
                let e = self.stock.entry_mut(site, &cargo);
                e.on_hand = e.on_hand.saturating_add(amount);
                CommandOutcome::Ok
            }
            C::CreateStandingLine {
                owner,
                source,
                sink,
                cargo,
                priority,
                vehicle_cap,
            } => self.create_order(
                owner,
                OrderKind::StandingLine,
                source,
                sink,
                cargo,
                priority,
                vehicle_cap,
            ),
            C::CreateOneOffRun {
                owner,
                source,
                sink,
                cargo,
                quantity,
                priority,
                vehicle_cap,
            } => {
                if quantity == 0 {
                    return CommandOutcome::Rejected("run quantity must be > 0".into());
                }
                self.create_order(
                    owner,
                    OrderKind::OneOffRun { quantity },
                    source,
                    sink,
                    cargo,
                    priority,
                    vehicle_cap,
                )
            }
            C::Cancel { order } => self.cancel(order),
            C::Pause { order } => self.with_live_order(order, |o| {
                if o.status != OrderStatus::Cancelling {
                    o.status = OrderStatus::Paused;
                }
            }),
            C::Resume { order } => self.with_live_order(order, |o| {
                if o.status == OrderStatus::Paused {
                    o.status = OrderStatus::Pending;
                    o.waiting_ticks = 0;
                }
            }),
            C::SetPriority { order, priority } => {
                self.with_live_order(order, |o| o.priority = priority)
            }
            C::PublishDemand {
                site,
                cargo,
                consumer,
                buffer_target,
                reorder_point,
                draw_rate_per_min,
                priority,
            } => {
                if !self.sites.contains_key(&site) {
                    return CommandOutcome::Rejected(format!("demand site {} unknown", site.0));
                }
                let reorder_point = reorder_point.min(buffer_target);
                let d = self
                    .demands
                    .entry((site, cargo.clone()))
                    .or_insert_with(|| DrawDemand {
                        site,
                        cargo,
                        consumer,
                        buffer_target,
                        reorder_point,
                        draw_rate_per_min,
                        priority,
                        health: DemandHealth::Satisfied,
                        starved_ticks: 0,
                        consumed_total: 0,
                        shortfall_total: 0,
                        draw_accum: 0,
                    });
                d.consumer = consumer;
                d.buffer_target = buffer_target;
                d.reorder_point = reorder_point;
                d.draw_rate_per_min = draw_rate_per_min;
                d.priority = priority;
                CommandOutcome::Ok
            }
            C::RetractDemand { site, cargo } => {
                if self.demands.remove(&(site, cargo)).is_none() {
                    return CommandOutcome::Rejected("no such demand".into());
                }
                CommandOutcome::Ok
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_order(
        &mut self,
        owner: OwnerId,
        kind: OrderKind,
        source: SiteId,
        sink: SiteId,
        cargo: CargoKind,
        priority: Priority,
        vehicle_cap: Option<u32>,
    ) -> CommandOutcome {
        if !self.sites.contains_key(&source) || !self.sites.contains_key(&sink) {
            return CommandOutcome::Rejected("order source/sink site unknown".into());
        }
        if source == sink {
            return CommandOutcome::Rejected("order source and sink are the same site".into());
        }
        if vehicle_cap == Some(0) {
            return CommandOutcome::Rejected("vehicle cap must be > 0".into());
        }
        self.next_order += 1;
        let id = OrderId(self.next_order);
        self.orders.insert(
            id,
            Order {
                id,
                owner,
                kind,
                source,
                sink,
                cargo,
                priority,
                vehicle_cap,
                status: OrderStatus::Pending,
                created_tick: self.tick,
                finished_tick: None,
                stats: OrderStats::default(),
                waiting_ticks: 0,
                refilling: false,
            },
        );
        CommandOutcome::CreatedOrder(id)
    }

    fn with_live_order(&mut self, id: OrderId, f: impl FnOnce(&mut Order)) -> CommandOutcome {
        match self.orders.get_mut(&id) {
            Some(o) if !o.status.is_terminal() => {
                f(o);
                CommandOutcome::Ok
            }
            Some(_) => CommandOutcome::Rejected(format!("order {} already finished", id.0)),
            None => CommandOutcome::Rejected(format!("unknown order {}", id.0)),
        }
    }

    /// Cancel: abort unloaded pickups (release reservations); loaded trips finish delivering.
    fn cancel(&mut self, id: OrderId) -> CommandOutcome {
        match self.orders.get(&id) {
            None => return CommandOutcome::Rejected(format!("unknown order {}", id.0)),
            Some(o) if o.status.is_terminal() => {
                return CommandOutcome::Rejected(format!("order {} already finished", id.0))
            }
            Some(_) => {}
        }
        let vids: Vec<VehicleId> = self
            .vehicles
            .values()
            .filter(|v| {
                v.assignment.as_ref().is_some_and(|a| a.order == id) && !v.phase.is_loaded()
            })
            .map(|v| v.id)
            .collect();
        for vid in vids {
            self.abort_unloaded_trip(vid);
        }
        let tick = self.tick;
        let o = self.orders.get_mut(&id).expect("checked above");
        o.status = OrderStatus::Cancelling;
        if o.stats.trips_in_flight == 0 {
            o.status = OrderStatus::Cancelled;
            o.finished_tick = Some(tick);
            self.push_event(Some(id), OrderEventKind::Cancelled);
        }
        CommandOutcome::Ok
    }

    /// Release a vehicle whose cargo is not yet on board; undo reservations.
    fn abort_unloaded_trip(&mut self, vid: VehicleId) {
        let Some(v) = self.vehicles.get_mut(&vid) else {
            return;
        };
        let Some(a) = v.assignment.take() else {
            return;
        };
        let was_loading = matches!(v.phase, VehiclePhase::Loading(_));
        v.phase = VehiclePhase::Idle;
        if was_loading {
            // Loading is the pickup site; the deadhead leg already finished.
            v.at = a.source;
        }
        let src = self.stock.entry_mut(a.source, &a.cargo);
        src.reserved_out = src.reserved_out.saturating_sub(a.load);
        let dst = self.stock.entry_mut(a.sink, &a.cargo);
        dst.inbound = dst.inbound.saturating_sub(a.load);
        if let Some(o) = self.orders.get_mut(&a.order) {
            o.stats.in_flight = o.stats.in_flight.saturating_sub(a.load);
            o.stats.trips_in_flight = o.stats.trips_in_flight.saturating_sub(1);
        }
    }

    fn fail_orders_touching(&mut self, site: SiteId) {
        let ids: Vec<OrderId> = self
            .orders
            .values()
            .filter(|o| !o.status.is_terminal() && (o.source == site || o.sink == site))
            .map(|o| o.id)
            .collect();
        for id in &ids {
            let vids: Vec<VehicleId> = self
                .vehicles
                .values()
                .filter(|v| v.assignment.as_ref().is_some_and(|a| a.order == *id))
                .map(|v| v.id)
                .collect();
            for vid in vids {
                let loaded = self.vehicles[&vid].phase.is_loaded();
                if loaded {
                    // Cargo on board with nowhere to go: written off.
                    let v = self.vehicles.get_mut(&vid).expect("listed");
                    let a = v.assignment.take().expect("filtered");
                    v.phase = VehiclePhase::Idle;
                    v.at = a.source;
                    let dst = self.stock.entry_mut(a.sink, &a.cargo);
                    dst.inbound = dst.inbound.saturating_sub(a.load);
                    let o = self.orders.get_mut(id).expect("listed");
                    o.stats.in_flight = o.stats.in_flight.saturating_sub(a.load);
                    o.stats.trips_in_flight = o.stats.trips_in_flight.saturating_sub(1);
                    o.stats.lost += a.load;
                } else {
                    self.abort_unloaded_trip(vid);
                }
            }
            let tick = self.tick;
            let o = self.orders.get_mut(id).expect("listed");
            o.status = OrderStatus::Failed(FailReason::SiteRemoved);
            o.finished_tick = Some(tick);
            self.push_event(Some(*id), OrderEventKind::Failed(FailReason::SiteRemoved));
        }
        for v in self.vehicles.values_mut() {
            if v.at == site {
                v.at = v.home;
            }
        }
    }

    fn push_event(&mut self, order: Option<OrderId>, kind: OrderEventKind) {
        self.events.push(OrderEvent {
            tick: self.tick,
            order,
            kind,
        });
        let max = self.config.max_events.max(1);
        if self.events.len() > max {
            let drop = self.events.len() - max;
            self.events.drain(0..drop);
        }
    }

    // ── tick ────────────────────────────────────────────────────────────────────

    /// Advance one sim tick: trips → consumption → order closure → allocation → prune.
    pub fn tick(&mut self, travel: &dyn TravelModel) {
        self.advance_vehicles(travel);
        self.consume_demands();
        self.close_finished_orders();
        self.allocate(travel);
        self.prune_finished();
        self.tick += 1;
    }

    fn advance_vehicles(&mut self, travel: &dyn TravelModel) {
        let ids: Vec<VehicleId> = self.vehicles.keys().copied().collect();
        for vid in ids {
            // Zero-length legs chain within the same tick (bounded: four phases).
            let mut steps = 0;
            let mut counted = false;
            loop {
                steps += 1;
                if steps > 5 {
                    break;
                }
                let v = self.vehicles.get_mut(&vid).expect("listed");
                let leg = match &mut v.phase {
                    VehiclePhase::Idle => break,
                    VehiclePhase::ToPickup(l)
                    | VehiclePhase::Loading(l)
                    | VehiclePhase::Hauling(l)
                    | VehiclePhase::Unloading(l) => l,
                };
                if leg.remaining > 0 {
                    if counted {
                        break;
                    }
                    leg.remaining -= 1;
                    counted = true;
                    if leg.remaining > 0 {
                        break;
                    }
                }
                self.finish_leg(vid, travel);
            }
        }
    }

    /// Transition a vehicle whose current leg reached zero.
    fn finish_leg(&mut self, vid: VehicleId, travel: &dyn TravelModel) {
        let (load_ticks, unload_ticks) = (self.config.load_ticks, self.config.unload_ticks);
        let v = self.vehicles.get_mut(&vid).expect("caller checked");
        let Some(a) = v.assignment.clone() else {
            v.phase = VehiclePhase::Idle;
            return;
        };
        match v.phase {
            VehiclePhase::ToPickup(_) => {
                v.at = a.source;
                v.phase = VehiclePhase::Loading(Leg::new(load_ticks));
            }
            VehiclePhase::Loading(_) => {
                let speed = v.speed_mps;
                let src = self.stock.entry_mut(a.source, &a.cargo);
                let actual = a.load.min(src.on_hand);
                src.on_hand -= actual;
                src.reserved_out = src.reserved_out.saturating_sub(a.load);
                let short = a.load - actual;
                if short > 0 {
                    let dst = self.stock.entry_mut(a.sink, &a.cargo);
                    dst.inbound = dst.inbound.saturating_sub(short);
                    if let Some(o) = self.orders.get_mut(&a.order) {
                        o.stats.in_flight = o.stats.in_flight.saturating_sub(short);
                    }
                }
                let v = self.vehicles.get_mut(&vid).expect("listed");
                if actual == 0 {
                    v.assignment = None;
                    v.phase = VehiclePhase::Idle;
                    if let Some(o) = self.orders.get_mut(&a.order) {
                        o.stats.trips_in_flight = o.stats.trips_in_flight.saturating_sub(1);
                    }
                    return;
                }
                if let Some(asg) = v.assignment.as_mut() {
                    asg.load = actual;
                }
                // Route checked at allocation; a vanished route falls back to one tick.
                let t = travel.travel_ticks(a.source, a.sink, speed).unwrap_or(1);
                v.phase = VehiclePhase::Hauling(Leg::new(t));
            }
            VehiclePhase::Hauling(_) => {
                v.at = a.sink;
                v.phase = VehiclePhase::Unloading(Leg::new(unload_ticks));
            }
            VehiclePhase::Unloading(_) => {
                v.phase = VehiclePhase::Idle;
                v.assignment = None;
                v.at = a.sink;
                let dst = self.stock.entry_mut(a.sink, &a.cargo);
                dst.on_hand = dst.on_hand.saturating_add(a.load);
                dst.inbound = dst.inbound.saturating_sub(a.load);
                if let Some(o) = self.orders.get_mut(&a.order) {
                    o.stats.delivered += a.load;
                    o.stats.in_flight = o.stats.in_flight.saturating_sub(a.load);
                    o.stats.trips_done += 1;
                    o.stats.trips_in_flight = o.stats.trips_in_flight.saturating_sub(1);
                }
            }
            VehiclePhase::Idle => {}
        }
    }

    fn consume_demands(&mut self) {
        let tpm = u64::from(self.config.ticks_per_minute.max(1));
        let mut newly_starving = Vec::new();
        for d in self.demands.values_mut() {
            let e = self.stock.entries.entry((d.site, d.cargo.clone())).or_default();
            if d.draw_rate_per_min > 0 {
                d.draw_accum += u64::from(d.draw_rate_per_min);
                let want = d.draw_accum / tpm;
                d.draw_accum %= tpm;
                if want > 0 {
                    let avail = u64::from(e.available());
                    let take = want.min(avail);
                    e.on_hand -= take as u32;
                    d.consumed_total += take;
                    d.shortfall_total += want - take;
                }
            }
            let prev = d.health;
            d.health = if d.draw_rate_per_min > 0 && e.on_hand == 0 {
                DemandHealth::Starving
            } else if e.projected() < d.reorder_point {
                DemandHealth::Low
            } else {
                DemandHealth::Satisfied
            };
            if d.health == DemandHealth::Starving {
                d.starved_ticks += 1;
                if prev != DemandHealth::Starving {
                    newly_starving.push((d.site, d.cargo.clone()));
                }
            }
        }
        for (site, cargo) in newly_starving {
            self.push_event(None, OrderEventKind::DemandStarving { site, cargo });
        }
    }

    fn close_finished_orders(&mut self) {
        let tick = self.tick;
        let mut done = Vec::new();
        for o in self.orders.values_mut() {
            if o.status.is_terminal() || o.stats.trips_in_flight > 0 {
                continue;
            }
            if o.status == OrderStatus::Cancelling {
                o.status = OrderStatus::Cancelled;
                o.finished_tick = Some(tick);
                done.push((o.id, OrderEventKind::Cancelled));
            } else if let OrderKind::OneOffRun { quantity } = o.kind {
                if o.stats.delivered >= quantity {
                    o.status = OrderStatus::Completed;
                    o.finished_tick = Some(tick);
                    done.push((o.id, OrderEventKind::Completed));
                }
            }
        }
        for (id, kind) in done {
            self.push_event(Some(id), kind);
        }
    }

    /// Units this order still wants assigned right now (0 = nothing to do).
    fn need(&mut self, id: OrderId) -> Result<u32, WaitReason> {
        let o = &self.orders[&id];
        match o.kind {
            OrderKind::OneOffRun { .. } => Ok(o.run_remaining()),
            OrderKind::StandingLine => {
                let Some(d) = self.demands.get(&(o.sink, o.cargo.clone())) else {
                    return Err(WaitReason::NoDemand);
                };
                let projected = self.stock.get(o.sink, &o.cargo).projected();
                let (target, reorder) = (d.buffer_target, d.reorder_point);
                let o = self.orders.get_mut(&id).expect("listed");
                if !o.refilling && projected < reorder.max(1) {
                    o.refilling = true;
                }
                if o.refilling {
                    let deficit = target.saturating_sub(projected);
                    if deficit == 0 {
                        o.refilling = false;
                    }
                    Ok(deficit)
                } else {
                    Ok(0)
                }
            }
        }
    }

    fn effective_weight(&self, o: &Order) -> u32 {
        let aging = (o.waiting_ticks / self.config.aging_interval_ticks.max(1))
            .min(self.config.aging_cap);
        let starving = self
            .demands
            .get(&(o.sink, o.cargo.clone()))
            .is_some_and(|d| d.health == DemandHealth::Starving);
        o.priority.weight() + aging + if starving { self.config.starving_bonus } else { 0 }
    }

    /// Free vehicle with the shortest deadhead to `source`, ties by id. Also returns
    /// whether the owner has any compatible vehicle at all.
    fn pick_vehicle(
        &self,
        o: &Order,
        travel: &dyn TravelModel,
    ) -> (Option<(VehicleId, u32)>, bool) {
        let class = o.cargo.class();
        let mut any_compatible = false;
        let mut best: Option<(u32, VehicleId)> = None;
        for v in self.vehicles.values() {
            if v.owner != o.owner || !v.class_mask.accepts(class) {
                continue;
            }
            any_compatible = true;
            if !v.is_free() {
                continue;
            }
            let Some(dead) = travel.travel_ticks(v.at, o.source, v.speed_mps) else {
                continue;
            };
            if best.is_none_or(|(bd, bid)| (dead, v.id) < (bd, bid)) {
                best = Some((dead, v.id));
            }
        }
        (best.map(|(d, id)| (id, d)), any_compatible)
    }

    fn allocate(&mut self, travel: &dyn TravelModel) {
        // Deterministic service order: weight desc, then id asc.
        let mut order_ids: Vec<(u32, OrderId)> = self
            .orders
            .values()
            .filter(|o| o.status.accepts_trips())
            .map(|o| (self.effective_weight(o), o.id))
            .collect();
        order_ids.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let order_ids: Vec<OrderId> = order_ids.into_iter().map(|(_, id)| id).collect();

        let mut served: BTreeSet<OrderId> = BTreeSet::new();
        let mut reason: BTreeMap<OrderId, Option<WaitReason>> = BTreeMap::new();
        let mut held: BTreeMap<OrderId, u32> = BTreeMap::new();
        for v in self.vehicles.values() {
            if let Some(a) = &v.assignment {
                *held.entry(a.order).or_default() += 1;
            }
        }

        // Round-robin: one vehicle per order per round until nothing changes.
        loop {
            let mut assigned_any = false;
            for &id in &order_ids {
                let need = match self.need(id) {
                    Ok(n) => n,
                    Err(r) => {
                        reason.insert(id, Some(r));
                        continue;
                    }
                };
                if need == 0 {
                    reason.entry(id).or_insert(None);
                    continue;
                }
                let o = self.orders[&id].clone();
                if let Some(cap) = o.vehicle_cap {
                    if held.get(&id).copied().unwrap_or(0) >= cap {
                        reason.insert(id, Some(WaitReason::VehicleCap));
                        continue;
                    }
                }
                if travel.travel_ticks(o.source, o.sink, 1.0).is_none() {
                    reason.insert(id, Some(WaitReason::Unreachable));
                    continue;
                }
                let available = self.stock.get(o.source, &o.cargo).available();
                if available == 0 {
                    reason.insert(id, Some(WaitReason::SourceEmpty));
                    continue;
                }
                let (pick, any_compatible) = self.pick_vehicle(&o, travel);
                let Some((vid, deadhead)) = pick else {
                    reason.insert(
                        id,
                        Some(if any_compatible {
                            WaitReason::NoVehicles
                        } else {
                            WaitReason::NoCompatibleVehicle
                        }),
                    );
                    continue;
                };
                let cap = self.vehicles[&vid].capacity_units;
                let load = cap.min(need).min(available);
                self.stock.entry_mut(o.source, &o.cargo).reserved_out += load;
                self.stock.entry_mut(o.sink, &o.cargo).inbound += load;
                let v = self.vehicles.get_mut(&vid).expect("picked");
                v.assignment = Some(Assignment {
                    order: id,
                    source: o.source,
                    sink: o.sink,
                    cargo: o.cargo.clone(),
                    load,
                });
                v.phase = VehiclePhase::ToPickup(Leg::new(deadhead));
                let om = self.orders.get_mut(&id).expect("listed");
                om.stats.in_flight += load;
                om.stats.trips_in_flight += 1;
                *held.entry(id).or_default() += 1;
                served.insert(id);
                reason.insert(id, None);
                assigned_any = true;
            }
            if !assigned_any {
                break;
            }
        }

        // Status + aging.
        for &id in &order_ids {
            let o = self.orders.get_mut(&id).expect("listed");
            let r = reason.get(&id).copied().flatten();
            if served.contains(&id) {
                o.waiting_ticks = 0;
            }
            o.status = match r {
                Some(r) if o.stats.trips_in_flight == 0 => {
                    if r != WaitReason::NoDemand {
                        o.waiting_ticks = o.waiting_ticks.saturating_add(1);
                    }
                    OrderStatus::Waiting(r)
                }
                Some(_) => {
                    o.waiting_ticks = o.waiting_ticks.saturating_add(1);
                    OrderStatus::Active
                }
                None if o.stats.trips_in_flight > 0 => OrderStatus::Active,
                None => match o.kind {
                    OrderKind::StandingLine => OrderStatus::Idle,
                    // Run with zero need and nothing in flight closes next tick.
                    OrderKind::OneOffRun { .. } => OrderStatus::Active,
                },
            };
        }
    }

    fn prune_finished(&mut self) {
        let keep = self.config.keep_finished_ticks;
        let now = self.tick;
        self.orders.retain(|_, o| match o.finished_tick {
            Some(t) => now.saturating_sub(t) < keep,
            None => true,
        });
    }

    // ── queries ─────────────────────────────────────────────────────────────────

    /// Display estimate of total trips for a one-off run (done + in flight + still needed).
    #[must_use]
    pub fn planned_trips(&self, o: &Order) -> u32 {
        let base = o.stats.trips_done + o.stats.trips_in_flight;
        let remaining = o.run_remaining();
        if remaining == 0 {
            return base;
        }
        let class = o.cargo.class();
        let typical = self
            .vehicles
            .values()
            .filter(|v| v.owner == o.owner && v.class_mask.accepts(class))
            .map(|v| v.capacity_units)
            .max()
            .unwrap_or(0);
        if typical == 0 {
            return base;
        }
        base + remaining.div_ceil(typical)
    }

    #[must_use]
    pub fn vehicles_on(&self, id: OrderId) -> u32 {
        self.vehicles
            .values()
            .filter(|v| v.assignment.as_ref().is_some_and(|a| a.order == id))
            .count() as u32
    }
}
