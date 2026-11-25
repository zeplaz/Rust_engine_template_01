//! Deterministic order simulation (PLAN-LOGISTICS-ORDERS-v1 §3.5, §4, §5).
//!
//! [`LogisticsOrderSim`] is the sole owner of order, vehicle, stock and demand state.
//! Writes arrive only through [`LogisticsOrderSim::apply`]; time advances only through
//! [`LogisticsOrderSim::tick`]. No Bevy types — tested headless in `tests.rs`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::model::*;

/// Free vehicles of one tick's allocation: owner → current site → ids ascending.
type FreePool = BTreeMap<OwnerId, BTreeMap<SiteId, Vec<VehicleId>>>;

/// Travel time source. `None` = unreachable. Same-site travel should return `Some(0)`.
pub trait TravelModel {
    fn travel_ticks(&self, from: SiteId, to: SiteId, speed_mps: f32) -> Option<u32>;

    /// Travel by rail between two yards (RD-7); `None` = no rail route (default: no rail).
    fn rail_travel_ticks(&self, from: SiteId, to: SiteId, speed_mps: f32) -> Option<u32> {
        let _ = (from, to, speed_mps);
        None
    }

    /// Do two rail trips share track? `Some(true)` same direction, `Some(false)` head-on, `None`
    /// no shared track. Default: the same pair, or the same pair reversed.
    fn rail_conflict(&self, a: (SiteId, SiteId), b: (SiteId, SiteId)) -> Option<bool> {
        if a == b {
            Some(true)
        } else if a == (b.1, b.0) {
            Some(false)
        } else {
            None
        }
    }
}

/// Travel ticks for a vehicle of `mode` (rail vehicles only move on rail).
fn mode_travel_ticks(travel: &dyn TravelModel, mode: VehicleMode, from: SiteId, to: SiteId, speed_mps: f32) -> Option<u32> {
    if mode.is_rail() {
        travel.rail_travel_ticks(from, to, speed_mps)
    } else {
        travel.travel_ticks(from, to, speed_mps)
    }
}

/// Tunables. Defaults assume one sim tick per simulated second.
fn default_rail_headway_ticks() -> u32 {
    60
}

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
    /// RD-7 signals: ticks a train must be ahead before the next one may follow it onto the
    /// same track (block headway).
    #[serde(default = "default_rail_headway_ticks")]
    pub rail_headway_ticks: u32,
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
            rail_headway_ticks: default_rail_headway_ticks(),
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
    /// A depot builds a vehicle from its own stock: `cost` is debited atomically (all or nothing)
    /// and the vehicle joins the owner's fleet at `home`.
    BuildVehicle {
        owner: OwnerId,
        home: SiteId,
        class_mask: CargoClassMask,
        capacity_units: u32,
        speed_mps: f32,
        cost: Vec<(CargoKind, u32)>,
    },
    /// A yard builds a train from its own stock (RD-7): `cost` is debited atomically (all or
    /// nothing), then the train joins the owner's fleet at `home` like [`Self::AddTrain`].
    BuildTrain {
        owner: OwnerId,
        home: SiteId,
        wagons: u8,
        wagon_capacity: u32,
        speed_mps: f32,
        cost: Vec<(CargoKind, u32)>,
    },
    /// A freight train homed at a yard (RD-7): `wagons` × `wagon_capacity` units, bulk and liquid
    /// wagons, rail only.
    AddTrain {
        owner: OwnerId,
        home: SiteId,
        wagons: u8,
        wagon_capacity: u32,
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
    /// Removes from on-hand stock, saturating at zero (a site consuming what it holds).
    ConsumeStock {
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
    /// Bumped by every [`Self::apply`] — with `tick` it fingerprints state for snapshot rebuilds.
    #[serde(default)]
    pub command_seq: u64,
    /// W4: haul speed scale from weather (1.0 = dry). Not persisted — reset each session.
    #[serde(skip)]
    pub weather_haul_mul: f32,
    /// FIRE-ENTITY-1: cumulative heat×suscept per vehicle (fleet loss ladder).
    #[serde(skip)]
    pub vehicle_fire_accum: BTreeMap<VehicleId, f32>,
    #[serde(skip)]
    pub fire_delayed_trips: u64,
    #[serde(skip)]
    pub vehicles_lost_to_fire: u64,
    #[serde(skip)]
    pub cargo_lost_to_fire: u64,
    /// Vehicles (and, in `cargo_lost_to_blast`, their loads) destroyed by detonations. Not persisted.
    #[serde(skip)]
    pub vehicles_lost_to_blast: u64,
    #[serde(skip)]
    pub cargo_lost_to_blast: u64,
    /// HAUL-AMPLIFY-1: per-owner capacity scale from formations (1.0 default). Not persisted.
    #[serde(skip)]
    pub haul_capacity_mul_by_owner: BTreeMap<OwnerId, f32>,
    next_order: u32,
    next_vehicle: u32,
}

impl LogisticsOrderSim {
    #[must_use]
    pub fn new(config: SimConfig) -> Self {
        Self {
            config,
            weather_haul_mul: 1.0,
            ..Default::default()
        }
    }

    /// Effective trip capacity after HAUL-AMPLIFY-1 owner mul.
    #[must_use]
    pub fn effective_capacity_units(&self, owner: OwnerId, base: u32) -> u32 {
        let mul = self
            .haul_capacity_mul_by_owner
            .get(&owner)
            .copied()
            .unwrap_or(1.0)
            .clamp(1.0, 1.5);
        ((base as f32) * mul).round().max(1.0) as u32
    }

    /// Apply mid-leg fire hits: delay → cargo dump → fleet remove (deterministic thresholds).
    /// `hits` are `(vehicle, heat01)` sampled at interpolated trip xz.
    pub fn apply_vehicle_fire_hits(&mut self, hits: &[(VehicleId, f32)], suscept: f32) {
        let suscept = suscept.clamp(0.05, 1.0);
        let mut lost: Vec<VehicleId> = Vec::new();
        for &(vid, heat) in hits {
            let exposure = (heat * suscept).clamp(0.0, 1.0);
            if exposure < 0.08 {
                continue;
            }
            let Some(v) = self.vehicles.get_mut(&vid) else {
                continue;
            };
            let hauling = matches!(v.phase, VehiclePhase::Hauling(_));
            let travel = matches!(
                v.phase,
                VehiclePhase::ToPickup(_) | VehiclePhase::Hauling(_)
            );
            if !travel {
                continue;
            }
            if let VehiclePhase::ToPickup(leg) | VehiclePhase::Hauling(leg) = &mut v.phase {
                let add = 1u32.saturating_add((exposure * 3.0) as u32);
                leg.remaining = leg.remaining.saturating_add(add);
                self.fire_delayed_trips = self.fire_delayed_trips.saturating_add(1);
            }
            let dump_asg = if exposure >= 0.28 && hauling {
                v.assignment.clone()
            } else {
                None
            };
            if let Some(a) = dump_asg {
                let dump = a.load;
                if dump > 0 {
                    if let Some(asg) = self.vehicles.get_mut(&vid).and_then(|v| v.assignment.as_mut())
                    {
                        asg.load = 0;
                    }
                    let dst = self.stock.entry_mut(a.sink, &a.cargo);
                    dst.inbound = dst.inbound.saturating_sub(dump);
                    if let Some(o) = self.orders.get_mut(&a.order) {
                        o.stats.in_flight = o.stats.in_flight.saturating_sub(dump);
                        o.stats.lost = o.stats.lost.saturating_add(dump);
                    }
                    self.cargo_lost_to_fire =
                        self.cargo_lost_to_fire.saturating_add(u64::from(dump));
                }
            }
            let accum = self.vehicle_fire_accum.entry(vid).or_insert(0.0);
            *accum = (*accum + exposure * 0.35).min(2.0);
            if *accum >= 0.9 {
                lost.push(vid);
            }
        }
        for vid in lost {
            if let Some(cargo) = self.remove_destroyed_vehicle(vid) {
                self.cargo_lost_to_fire = self.cargo_lost_to_fire.saturating_add(u64::from(cargo));
                self.vehicles_lost_to_fire = self.vehicles_lost_to_fire.saturating_add(1);
            }
        }
    }

    /// Detonation losses: every listed vehicle is destroyed, with the same cargo / reservation
    /// bookkeeping as a fire loss. The caller decides who is inside the blast.
    pub fn apply_vehicle_blast_losses(&mut self, victims: &[VehicleId]) {
        for &vid in victims {
            if let Some(cargo) = self.remove_destroyed_vehicle(vid) {
                self.cargo_lost_to_blast = self.cargo_lost_to_blast.saturating_add(u64::from(cargo));
                self.vehicles_lost_to_blast = self.vehicles_lost_to_blast.saturating_add(1);
            }
        }
    }

    /// Remove a destroyed vehicle and release what it held. Returns the cargo units lost with it
    /// (0 when it was empty), or `None` when it is already gone.
    fn remove_destroyed_vehicle(&mut self, vid: VehicleId) -> Option<u32> {
        let v = self.vehicles.remove(&vid)?;
        self.vehicle_fire_accum.remove(&vid);
        let mut cargo_lost = 0u32;
        if let Some(a) = v.assignment {
            if v.phase.is_loaded() {
                let dst = self.stock.entry_mut(a.sink, &a.cargo);
                dst.inbound = dst.inbound.saturating_sub(a.load);
                if let Some(o) = self.orders.get_mut(&a.order) {
                    o.stats.in_flight = o.stats.in_flight.saturating_sub(a.load);
                    o.stats.lost = o.stats.lost.saturating_add(a.load);
                    o.stats.trips_in_flight = o.stats.trips_in_flight.saturating_sub(1);
                }
                cargo_lost = a.load;
            } else {
                let src = self.stock.entry_mut(a.source, &a.cargo);
                src.reserved_out = src.reserved_out.saturating_sub(a.load);
                let dst = self.stock.entry_mut(a.sink, &a.cargo);
                dst.inbound = dst.inbound.saturating_sub(a.load);
                if let Some(o) = self.orders.get_mut(&a.order) {
                    o.stats.in_flight = o.stats.in_flight.saturating_sub(a.load);
                    o.stats.trips_in_flight = o.stats.trips_in_flight.saturating_sub(1);
                }
            }
        }
        Some(cargo_lost)
    }

    // ── commands ────────────────────────────────────────────────────────────────

    pub fn apply(&mut self, cmd: LogisticsOrderCommand) -> CommandOutcome {
        self.command_seq = self.command_seq.wrapping_add(1);
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
            C::BuildVehicle {
                owner,
                home,
                class_mask,
                capacity_units,
                speed_mps,
                cost,
            } => {
                if !self.sites.contains_key(&home) {
                    return CommandOutcome::Rejected(format!("vehicle home {} unknown", home.0));
                }
                if capacity_units == 0 || !(speed_mps > 0.0) {
                    return CommandOutcome::Rejected("vehicle needs capacity and speed".into());
                }
                if let Some((cargo, need)) = cost
                    .iter()
                    .find(|(cargo, need)| self.stock.get(home, cargo).available() < *need)
                {
                    return CommandOutcome::Rejected(format!(
                        "site {} lacks {} {} to build a vehicle",
                        home.0, need, cargo.0
                    ));
                }
                for (cargo, need) in &cost {
                    let e = self.stock.entry_mut(home, cargo);
                    e.on_hand -= *need;
                }
                self.apply_inner(C::AddVehicle {
                    owner,
                    home,
                    class_mask,
                    capacity_units,
                    speed_mps,
                })
            }
            C::BuildTrain { owner, home, wagons, wagon_capacity, speed_mps, cost } => {
                if !self.sites.contains_key(&home) {
                    return CommandOutcome::Rejected(format!("train home {} unknown", home.0));
                }
                if let Some((cargo, need)) = cost.iter().find(|(cargo, need)| self.stock.get(home, cargo).available() < *need) {
                    return CommandOutcome::Rejected(format!("yard {} lacks {} {} to build a train", home.0, need, cargo.0));
                }
                for (cargo, need) in &cost {
                    self.stock.entry_mut(home, cargo).on_hand -= *need;
                }
                self.apply_inner(C::AddTrain { owner, home, wagons, wagon_capacity, speed_mps })
            }
            C::AddTrain { owner, home, wagons, wagon_capacity, speed_mps } => {
                if wagons == 0 || wagon_capacity == 0 {
                    return CommandOutcome::Rejected("train needs wagons".into());
                }
                let out = self.apply_inner(C::AddVehicle {
                    owner,
                    home,
                    class_mask: CargoClassMask(CargoClassMask::FREIGHT.0 | CargoClassMask::TANKER.0),
                    capacity_units: u32::from(wagons) * wagon_capacity,
                    speed_mps,
                });
                if let CommandOutcome::CreatedVehicle(id) = out {
                    if let Some(v) = self.vehicles.get_mut(&id) {
                        v.mode = VehicleMode::Rail { wagons };
                    }
                }
                out
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
                        mode: VehicleMode::Road,
                        signal_hold: false,
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
            C::ConsumeStock {
                site,
                cargo,
                amount,
            } => {
                if !self.sites.contains_key(&site) {
                    return CommandOutcome::Rejected(format!("unknown site {}", site.0));
                }
                let e = self.stock.entry_mut(site, &cargo);
                let take = amount.min(e.on_hand);
                e.on_hand -= take;
                // Self-metered consumers (plants, sites, fronts) report draw through here.
                if let Some(d) = self.demands.get_mut(&(site, cargo)) {
                    d.consumed_total += u64::from(take);
                    d.shortfall_total += u64::from(amount - take);
                }
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
                        unmet_ticks: 0,
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
        self.advance_demand_adverts();
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
        let mode = v.mode;
        let handling = mode.handling_mul();
        let Some(a) = v.assignment.clone() else {
            v.phase = VehiclePhase::Idle;
            return;
        };
        match v.phase {
            VehiclePhase::ToPickup(_) => {
                v.at = a.source;
                v.phase = VehiclePhase::Loading(Leg::new(load_ticks * handling));
            }
            VehiclePhase::Loading(_) => {
                // RD-7 signals: a loaded train leaves only onto clear track.
                if mode.is_rail() && self.rail_blocked(vid, (a.source, a.sink), travel) {
                    let v = self.vehicles.get_mut(&vid).expect("listed");
                    v.phase = VehiclePhase::Loading(Leg::new(1));
                    v.signal_hold = true;
                    return;
                }
                let v = self.vehicles.get_mut(&vid).expect("listed");
                v.signal_hold = false;
                let mul = if self.weather_haul_mul <= 0.0 {
                    1.0
                } else {
                    self.weather_haul_mul.clamp(0.25, 1.0)
                };
                let speed = v.speed_mps * mul;
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
                let t = mode_travel_ticks(travel, mode, a.source, a.sink, speed).unwrap_or(1);
                v.phase = VehiclePhase::Hauling(Leg::new(t));
            }
            VehiclePhase::Hauling(_) => {
                v.at = a.sink;
                v.phase = VehiclePhase::Unloading(Leg::new(unload_ticks * handling));
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

    fn advance_demand_adverts(&mut self) {
        let keys: Vec<(SiteId, CargoKind)> = self
            .demands
            .values()
            .map(|d| (d.site, d.cargo.clone()))
            .collect();
        for (site, cargo) in keys {
            let served = self
                .orders
                .values()
                .filter(|o| {
                    o.kind.is_standing()
                        && o.sink == site
                        && o.cargo == cargo
                        && o.status.accepts_trips()
                })
                .count() as u32;
            if let Some(d) = self.demands.get_mut(&(site, cargo)) {
                if served == 0 {
                    d.unmet_ticks = d.unmet_ticks.saturating_add(1);
                } else {
                    d.unmet_ticks = 0;
                }
            }
        }
    }

    fn consume_demands(&mut self) {
        let tpm = u64::from(self.config.ticks_per_minute.max(1));
        let mut newly_starving = Vec::new();
        for d in self.demands.values_mut() {
            let e = self.stock.entries.entry((d.site, d.cargo.clone())).or_default();
            // Plants / construction sites draw their own stock (see `ConsumerKind::drawn_by_order_sim`).
            if d.draw_rate_per_min > 0 && d.consumer.drawn_by_order_sim() {
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

    /// Batching: a truck leaves with less than its capacity only when that is worth a trip.
    ///
    /// - Deficit smaller than a truck (`need < capacity`): a one-off run's last trip goes; a
    ///   standing line tops up only once the sink is short (projected below its reorder point, or
    ///   starving). Otherwise the line waits until a full truck's worth is missing, instead of
    ///   sending a truck per unit of draw.
    /// - Source holds less than the load (`available < min(capacity, need)`): wait for the source to
    ///   fill the truck, unless the sink is short and drawing (or the order is a one-off run) and no
    ///   other truck of this order is already on the way. At most one part-load per order is in
    ///   flight, so a trickling source never ties up the fleet one unit at a time.
    fn load_worth_a_trip(&self, id: OrderId, capacity: u32, need: u32, available: u32, held: u32) -> bool {
        let full = capacity.min(need);
        let need_limited = need < capacity;
        let source_limited = available < full;
        if !need_limited && !source_limited {
            return true;
        }
        let o = &self.orders[&id];
        match o.kind {
            OrderKind::OneOffRun { .. } => !source_limited || held == 0,
            OrderKind::StandingLine => {
                let Some(d) = self.demands.get(&(o.sink, o.cargo.clone())) else {
                    return false;
                };
                let projected = self.stock.get(o.sink, &o.cargo).projected();
                let short = d.health == DemandHealth::Starving || projected < d.reorder_point;
                if source_limited {
                    short && d.draw_rate_per_min > 0 && held == 0
                } else {
                    short
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

    /// Free vehicles by owner → current site → ids (ascending), plus every `(owner, class)`
    /// any vehicle of the owner can carry (free or busy) for wait reasons.
    fn free_pool(&self) -> (FreePool, BTreeSet<(OwnerId, CargoClass)>) {
        let mut pool: FreePool = BTreeMap::new();
        let mut compatible = BTreeSet::new();
        for v in self.vehicles.values() {
            for class in CargoClass::ALL {
                if v.class_mask.accepts(class) {
                    compatible.insert((v.owner, class));
                }
            }
            if v.is_free() {
                pool.entry(v.owner)
                    .or_default()
                    .entry(v.at)
                    .or_default()
                    .push(v.id);
            }
        }
        (pool, compatible)
    }

    /// Free vehicle for `owner` carrying `class` from `source` to `sink`.
    ///
    /// Trucks: per site the lowest compatible id stands in; sites compete on `(deadhead, id)`.
    /// Trains (RD-7) only run where rail joins the deadhead and the trip; the best train is taken
    /// when its time per unit (deadhead + trip, over its capacity) beats the best truck's.
    /// Cost is O(sites holding free vehicles), not O(fleet).
    fn pick_from_pool(
        &self,
        pool: &FreePool,
        owner: OwnerId,
        class: CargoClass,
        (source, sink): (SiteId, SiteId),
        travel: &dyn TravelModel,
    ) -> Option<(SiteId, VehicleId, u32)> {
        let mut best: Option<(u32, VehicleId, SiteId)> = None;
        let mut best_rail: Option<(u64, VehicleId, SiteId, u32)> = None;
        for (&at, ids) in pool.get(&owner)? {
            if let Some(&vid) = ids
                .iter()
                .find(|id| !self.vehicles[id].mode.is_rail() && self.vehicles[id].class_mask.accepts(class))
            {
                if let Some(dead) = travel.travel_ticks(at, source, self.vehicles[&vid].speed_mps) {
                    if best.is_none_or(|(bd, bid, _)| (dead, vid) < (bd, bid)) {
                        best = Some((dead, vid, at));
                    }
                }
            }
            for &vid in ids.iter().filter(|id| self.vehicles[id].mode.is_rail() && self.vehicles[id].class_mask.accepts(class)) {
                let v = &self.vehicles[&vid];
                let (Some(dead), Some(trip)) = (
                    travel.rail_travel_ticks(at, source, v.speed_mps),
                    travel.rail_travel_ticks(source, sink, v.speed_mps),
                ) else {
                    continue;
                };
                if dead > 0 && self.rail_blocked(vid, (at, source), travel) {
                    continue;
                }
                let per_unit = (u64::from(dead) + u64::from(trip)) * 1000 / u64::from(v.capacity_units.max(1));
                if best_rail.is_none_or(|(bp, bid, _, _)| (per_unit, vid) < (bp, bid)) {
                    best_rail = Some((per_unit, vid, at, dead));
                }
            }
        }
        let road_per_unit = best.and_then(|(dead, vid, _)| {
            let v = &self.vehicles[&vid];
            let trip = travel.travel_ticks(source, sink, v.speed_mps)?;
            Some((u64::from(dead) + u64::from(trip)) * 1000 / u64::from(v.capacity_units.max(1)))
        });
        match (best_rail, road_per_unit) {
            (Some((rail_pu, vid, at, dead)), road) if road.is_none_or(|r| rail_pu < r) => Some((at, vid, dead)),
            _ => best.map(|(dead, vid, at)| (at, vid, dead)),
        }
    }

    /// RD-7 signals: may train `vid` start the rail trip `pair` now? Blocked by any other train on
    /// shared track coming the other way, or going the same way less than a headway ahead.
    /// Yards are where trains pass each other.
    fn rail_blocked(&self, vid: VehicleId, pair: (SiteId, SiteId), travel: &dyn TravelModel) -> bool {
        let headway = self.config.rail_headway_ticks;
        self.vehicles.values().filter(|o| o.id != vid && o.mode.is_rail()).any(|o| {
            let (other, leg) = match (&o.phase, &o.assignment) {
                (VehiclePhase::Hauling(l), Some(a)) => ((a.source, a.sink), l),
                (VehiclePhase::ToPickup(l), Some(a)) => ((o.at, a.source), l),
                _ => return false,
            };
            match travel.rail_conflict(pair, other) {
                Some(false) => true,
                Some(true) => leg.total.saturating_sub(leg.remaining) < headway,
                None => false,
            }
        })
    }

    fn allocate(&mut self, travel: &dyn TravelModel) {
        // Deterministic service order: weight desc, then id asc.
        let mut ranked: Vec<(u32, OrderId)> = self
            .orders
            .values()
            .filter(|o| o.status.accepts_trips())
            .map(|o| (self.effective_weight(o), o.id))
            .collect();
        ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let order_ids: Vec<OrderId> = ranked.into_iter().map(|(_, id)| id).collect();

        let mut served: BTreeSet<OrderId> = BTreeSet::new();
        let mut reason: BTreeMap<OrderId, Option<WaitReason>> = BTreeMap::new();
        let mut held: BTreeMap<OrderId, u32> = BTreeMap::new();
        for v in self.vehicles.values() {
            if let Some(a) = &v.assignment {
                *held.entry(a.order).or_default() += 1;
            }
        }
        let (mut pool, compatible) = self.free_pool();

        // Round-robin: one vehicle per order per round. Within a tick vehicles and stock only
        // get consumed, so an order that could not be served drops out of later rounds.
        let mut pending = order_ids.clone();
        while !pending.is_empty() {
            let mut next = Vec::new();
            for &id in &pending {
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
                let (owner, source, sink, cap_limit) = {
                    let o = &self.orders[&id];
                    (o.owner, o.source, o.sink, o.vehicle_cap)
                };
                let cargo = self.orders[&id].cargo.clone();
                if cap_limit.is_some_and(|cap| held.get(&id).copied().unwrap_or(0) >= cap) {
                    reason.insert(id, Some(WaitReason::VehicleCap));
                    continue;
                }
                if travel.travel_ticks(source, sink, 1.0).is_none() && travel.rail_travel_ticks(source, sink, 1.0).is_none() {
                    reason.insert(id, Some(WaitReason::Unreachable));
                    continue;
                }
                let available = self.stock.get(source, &cargo).available();
                if available == 0 {
                    reason.insert(id, Some(WaitReason::SourceEmpty));
                    continue;
                }
                let class = cargo.class();
                let Some((at, vid, deadhead)) =
                    self.pick_from_pool(&pool, owner, class, (source, sink), travel)
                else {
                    reason.insert(
                        id,
                        Some(if compatible.contains(&(owner, class)) {
                            WaitReason::NoVehicles
                        } else {
                            WaitReason::NoCompatibleVehicle
                        }),
                    );
                    continue;
                };
                let base_cap = self.vehicles[&vid].capacity_units;
                let capacity = self.effective_capacity_units(owner, base_cap);
                let held_now = held.get(&id).copied().unwrap_or(0);
                if !self.load_worth_a_trip(id, capacity, need, available, held_now) {
                    reason.insert(id, Some(WaitReason::FillingLoad));
                    continue;
                }
                if let Some(sites) = pool.get_mut(&owner) {
                    if let Some(ids) = sites.get_mut(&at) {
                        ids.retain(|v| *v != vid);
                        if ids.is_empty() {
                            sites.remove(&at);
                        }
                    }
                }
                let load = capacity.min(need).min(available);
                self.stock.entry_mut(source, &cargo).reserved_out += load;
                self.stock.entry_mut(sink, &cargo).inbound += load;
                let v = self.vehicles.get_mut(&vid).expect("picked");
                v.assignment = Some(Assignment {
                    order: id,
                    source,
                    sink,
                    cargo,
                    load,
                });
                v.phase = VehiclePhase::ToPickup(Leg::new(deadhead));
                let om = self.orders.get_mut(&id).expect("listed");
                om.stats.in_flight += load;
                om.stats.trips_in_flight += 1;
                *held.entry(id).or_default() += 1;
                served.insert(id);
                reason.insert(id, None);
                next.push(id);
            }
            pending = next;
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

    /// Rewrite legacy lowercase construction cargo tags (stock, demands, orders, trips) to the
    /// catalog spelling. Idempotent; merges stock if both spellings exist.
    pub fn migrate_legacy_cargo_tags(&mut self) {
        let old = std::mem::take(&mut self.stock.entries);
        for ((site, cargo), e) in old {
            let cargo = cargo.legacy_catalog_spelling().unwrap_or(cargo);
            let slot = self.stock.entries.entry((site, cargo)).or_default();
            slot.on_hand = slot.on_hand.saturating_add(e.on_hand);
            slot.reserved_out = slot.reserved_out.saturating_add(e.reserved_out);
            slot.inbound = slot.inbound.saturating_add(e.inbound);
        }
        let old = std::mem::take(&mut self.demands);
        for ((site, cargo), mut d) in old {
            let cargo = cargo.legacy_catalog_spelling().unwrap_or(cargo);
            d.cargo = cargo.clone();
            self.demands.insert((site, cargo), d);
        }
        for o in self.orders.values_mut() {
            if let Some(c) = o.cargo.legacy_catalog_spelling() {
                o.cargo = c;
            }
        }
        for a in self.vehicles.values_mut().filter_map(|v| v.assignment.as_mut()) {
            if let Some(c) = a.cargo.legacy_catalog_spelling() {
                a.cargo = c;
            }
        }
    }

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
