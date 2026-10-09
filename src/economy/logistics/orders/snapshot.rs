//! Read-only snapshot of the order sim for UI, map overlay and AI (plan §2, §6).
//!
//! Built by one system per frame from [`LogisticsOrderSim`]; consumers never read the sim.
//! Pure Rust — site placement arrives as [`SiteMeta`] from the adapter.

use std::collections::BTreeMap;

use super::model::*;
use super::sim::{LogisticsOrderSim, OrderEventKind, TravelModel};

/// What a site is for — lets the UI group pickers and the AI find depots vs fronts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SiteRole {
    #[default]
    Depot,
    Front,
    Construction,
    Plant,
    Unit,
}

impl SiteRole {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Depot => "depot",
            Self::Front => "front",
            Self::Construction => "construction",
            Self::Plant => "plant",
            Self::Unit => "unit",
        }
    }
}

/// Adapter-supplied placement for one site (world XZ in metres; tile = 16 m).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SiteMeta {
    pub name: String,
    pub owner: OwnerId,
    pub role: SiteRole,
    pub world_xz: [f32; 2],
}

/// World metres per designer tile (matches `economy::site_placement`).
pub const METRES_PER_TILE: f32 = 16.0;

#[derive(Clone, Debug, PartialEq)]
pub struct StockRow {
    pub cargo: String,
    pub unit: &'static str,
    pub on_hand: u32,
    pub reserved_out: u32,
    pub inbound: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SiteRow {
    pub id: SiteId,
    pub name: String,
    pub owner: OwnerId,
    pub role: SiteRole,
    pub world_xz: [f32; 2],
    pub tile: [i32; 2],
    pub stock: Vec<StockRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderRow {
    pub id: OrderId,
    pub owner: OwnerId,
    pub standing: bool,
    pub source: SiteId,
    pub sink: SiteId,
    pub source_name: String,
    pub sink_name: String,
    pub cargo: String,
    pub unit: &'static str,
    pub priority: Priority,
    pub status: OrderStatus,
    pub status_label: String,
    /// One-off quantity; `None` for standing lines.
    pub quantity: Option<u32>,
    pub delivered: u32,
    pub in_flight: u32,
    /// One-off: not yet delivered (includes in flight). Line: current deficit to buffer target.
    pub remaining: u32,
    pub trips_done: u32,
    pub trips_planned: u32,
    pub vehicles: u32,
    pub vehicle_cap: Option<u32>,
    /// Ticks until the next truck of this order unloads.
    pub next_eta_ticks: Option<u32>,
    /// Standing line: sink on-hand / buffer target.
    pub fill_ratio: Option<f32>,
    /// Average delivered units per sim minute since creation.
    pub throughput_per_min: f32,
    pub lost: u32,
    pub created_tick: Tick,
    pub finished_tick: Option<Tick>,
}

impl OrderRow {
    /// 0..1 progress for the run bar (lines: fill ratio).
    #[must_use]
    pub fn progress(&self) -> f32 {
        match self.quantity {
            Some(q) if q > 0 => (self.delivered as f32 / q as f32).clamp(0.0, 1.0),
            Some(_) => 1.0,
            None => self.fill_ratio.unwrap_or(0.0).clamp(0.0, 1.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DemandRow {
    pub site: SiteId,
    pub site_name: String,
    pub owner: OwnerId,
    pub cargo: String,
    pub unit: &'static str,
    pub consumer: ConsumerKind,
    pub on_hand: u32,
    pub inbound: u32,
    pub buffer_target: u32,
    pub reorder_point: u32,
    pub draw_rate_per_min: u32,
    pub priority: Priority,
    pub health: DemandHealth,
    /// Live standing lines feeding this `(site, cargo)`.
    pub served_by: u32,
    pub starved_ticks: u64,
    pub shortfall_total: u64,
}

impl DemandRow {
    #[must_use]
    pub fn fill_ratio(&self) -> f32 {
        if self.buffer_target == 0 {
            1.0
        } else {
            (self.on_hand as f32 / self.buffer_target as f32).clamp(0.0, 1.0)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VehicleRow {
    pub id: VehicleId,
    pub owner: OwnerId,
    pub class_mask: CargoClassMask,
    pub capacity_units: u32,
    pub phase: &'static str,
    pub order: Option<OrderId>,
    pub cargo: Option<String>,
    pub load: u32,
    pub at: SiteId,
    /// Map interpolation: the vehicle sits `fraction` of the way `from → to`.
    pub from: SiteId,
    pub to: SiteId,
    pub fraction: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FleetRow {
    pub owner: OwnerId,
    pub total: u32,
    pub free: u32,
    pub busy: u32,
    pub capacity_total: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventRow {
    pub tick: Tick,
    pub order: Option<OrderId>,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LogisticsOrdersSnapshot {
    pub tick: Tick,
    pub sites: Vec<SiteRow>,
    pub orders: Vec<OrderRow>,
    pub demands: Vec<DemandRow>,
    pub vehicles: Vec<VehicleRow>,
    pub fleets: Vec<FleetRow>,
    pub events: Vec<EventRow>,
    pub rejected: Vec<String>,
}

impl LogisticsOrdersSnapshot {
    #[must_use]
    pub fn site(&self, id: SiteId) -> Option<&SiteRow> {
        self.sites.iter().find(|s| s.id == id)
    }

    /// Sites of one owner with a role (AI: "my depots", "my front supply points").
    pub fn sites_of(&self, owner: OwnerId, role: SiteRole) -> impl Iterator<Item = &SiteRow> {
        self.sites
            .iter()
            .filter(move |s| s.owner == owner && s.role == role)
    }

    /// Demands of `owner`'s sites that no standing line serves (UI call-to-action, AI auto-lines).
    pub fn unserved_demands(&self, owner: OwnerId) -> impl Iterator<Item = &DemandRow> {
        self.demands
            .iter()
            .filter(move |d| d.owner == owner && d.served_by == 0)
    }

    pub fn standing_lines(&self) -> impl Iterator<Item = &OrderRow> {
        self.orders.iter().filter(|o| o.standing)
    }

    pub fn one_off_runs(&self) -> impl Iterator<Item = &OrderRow> {
        self.orders.iter().filter(|o| !o.standing)
    }

    #[must_use]
    pub fn fleet(&self, owner: OwnerId) -> FleetRow {
        self.fleets
            .iter()
            .find(|f| f.owner == owner)
            .cloned()
            .unwrap_or(FleetRow {
                owner,
                ..Default::default()
            })
    }
}

fn site_name(meta: &BTreeMap<SiteId, SiteMeta>, sim: &LogisticsOrderSim, id: SiteId) -> String {
    meta.get(&id)
        .map(|m| m.name.clone())
        .or_else(|| sim.sites.get(&id).cloned())
        .unwrap_or_else(|| format!("site {}", id.0))
}

/// Remaining ticks until this busy vehicle finishes unloading.
fn vehicle_eta(sim: &LogisticsOrderSim, v: &Vehicle, travel: &dyn TravelModel) -> Option<u32> {
    let a = v.assignment.as_ref()?;
    let (load, unload) = (sim.config.load_ticks, sim.config.unload_ticks);
    let haul = || travel.travel_ticks(a.source, a.sink, v.speed_mps).unwrap_or(0);
    Some(match v.phase {
        VehiclePhase::Idle => return None,
        VehiclePhase::ToPickup(l) => l.remaining + load + haul() + unload,
        VehiclePhase::Loading(l) => l.remaining + haul() + unload,
        VehiclePhase::Hauling(l) => l.remaining + unload,
        VehiclePhase::Unloading(l) => l.remaining,
    })
}

/// Build the full snapshot. Deterministic: rows follow id order.
#[must_use]
pub fn build_snapshot(
    sim: &LogisticsOrderSim,
    meta: &BTreeMap<SiteId, SiteMeta>,
    travel: &dyn TravelModel,
) -> LogisticsOrdersSnapshot {
    let mut snap = LogisticsOrdersSnapshot {
        tick: sim.tick,
        ..Default::default()
    };

    for (&id, name) in &sim.sites {
        let m = meta.get(&id).cloned().unwrap_or(SiteMeta {
            name: name.clone(),
            ..Default::default()
        });
        let stock = sim
            .stock
            .entries
            .range((id, CargoKind(String::new()))..)
            .take_while(|((s, _), _)| *s == id)
            .filter(|(_, e)| e.on_hand > 0 || e.inbound > 0 || e.reserved_out > 0)
            .map(|((_, c), e)| StockRow {
                cargo: c.0.clone(),
                unit: c.unit_label(),
                on_hand: e.on_hand,
                reserved_out: e.reserved_out,
                inbound: e.inbound,
            })
            .collect();
        snap.sites.push(SiteRow {
            id,
            tile: [
                (m.world_xz[0] / METRES_PER_TILE).floor() as i32,
                (m.world_xz[1] / METRES_PER_TILE).floor() as i32,
            ],
            name: m.name,
            owner: m.owner,
            role: m.role,
            world_xz: m.world_xz,
            stock,
        });
    }

    let mut next_eta: BTreeMap<OrderId, u32> = BTreeMap::new();
    let mut fleets: BTreeMap<OwnerId, FleetRow> = BTreeMap::new();
    for v in sim.vehicles.values() {
        let f = fleets.entry(v.owner).or_insert(FleetRow {
            owner: v.owner,
            ..Default::default()
        });
        f.total += 1;
        f.capacity_total += v.capacity_units;
        if v.is_free() {
            f.free += 1;
        } else {
            f.busy += 1;
        }
        if let (Some(a), Some(eta)) = (&v.assignment, vehicle_eta(sim, v, travel)) {
            let e = next_eta.entry(a.order).or_insert(eta);
            *e = (*e).min(eta);
        }
        let (from, to, fraction) = match (&v.phase, &v.assignment) {
            (VehiclePhase::ToPickup(l), Some(a)) => (v.at, a.source, l.fraction()),
            (VehiclePhase::Loading(_), Some(a)) => (a.source, a.source, 0.0),
            (VehiclePhase::Hauling(l), Some(a)) => (a.source, a.sink, l.fraction()),
            (VehiclePhase::Unloading(_), Some(a)) => (a.sink, a.sink, 0.0),
            _ => (v.at, v.at, 0.0),
        };
        snap.vehicles.push(VehicleRow {
            id: v.id,
            owner: v.owner,
            class_mask: v.class_mask,
            capacity_units: v.capacity_units,
            phase: v.phase.label(),
            order: v.assignment.as_ref().map(|a| a.order),
            cargo: v.assignment.as_ref().map(|a| a.cargo.0.clone()),
            load: v.assignment.as_ref().map_or(0, |a| a.load),
            at: v.at,
            from,
            to,
            fraction,
        });
    }
    snap.fleets = fleets.into_values().collect();

    let mut lines_per_demand: BTreeMap<(SiteId, CargoKind), u32> = BTreeMap::new();
    let tpm = sim.config.ticks_per_minute.max(1) as f32;
    for o in sim.orders.values() {
        if o.kind.is_standing() && !o.status.is_terminal() {
            *lines_per_demand
                .entry((o.sink, o.cargo.clone()))
                .or_default() += 1;
        }
        let sink_stock = sim.stock.get(o.sink, &o.cargo);
        let demand = sim.demands.get(&(o.sink, o.cargo.clone()));
        let (quantity, remaining, fill_ratio) = match o.kind {
            OrderKind::OneOffRun { quantity } => (
                Some(quantity),
                quantity.saturating_sub(o.stats.delivered),
                None,
            ),
            OrderKind::StandingLine => (
                None,
                demand.map_or(0, |d| d.buffer_target.saturating_sub(sink_stock.projected())),
                demand.map(|d| {
                    if d.buffer_target == 0 {
                        1.0
                    } else {
                        sink_stock.on_hand as f32 / d.buffer_target as f32
                    }
                }),
            ),
        };
        let age = o
            .finished_tick
            .unwrap_or(sim.tick)
            .saturating_sub(o.created_tick)
            .max(1) as f32;
        snap.orders.push(OrderRow {
            id: o.id,
            owner: o.owner,
            standing: o.kind.is_standing(),
            source: o.source,
            sink: o.sink,
            source_name: site_name(meta, sim, o.source),
            sink_name: site_name(meta, sim, o.sink),
            cargo: o.cargo.0.clone(),
            unit: o.cargo.unit_label(),
            priority: o.priority,
            status: o.status,
            status_label: o.status.label(),
            quantity,
            delivered: o.stats.delivered,
            in_flight: o.stats.in_flight,
            remaining,
            trips_done: o.stats.trips_done,
            trips_planned: sim.planned_trips(o),
            vehicles: sim.vehicles_on(o.id),
            vehicle_cap: o.vehicle_cap,
            next_eta_ticks: next_eta.get(&o.id).copied(),
            fill_ratio,
            throughput_per_min: o.stats.delivered as f32 * tpm / age,
            lost: o.stats.lost,
            created_tick: o.created_tick,
            finished_tick: o.finished_tick,
        });
    }

    for d in sim.demands.values() {
        let e = sim.stock.get(d.site, &d.cargo);
        snap.demands.push(DemandRow {
            site: d.site,
            site_name: site_name(meta, sim, d.site),
            owner: meta.get(&d.site).map(|m| m.owner).unwrap_or_default(),
            cargo: d.cargo.0.clone(),
            unit: d.cargo.unit_label(),
            consumer: d.consumer,
            on_hand: e.on_hand,
            inbound: e.inbound,
            buffer_target: d.buffer_target,
            reorder_point: d.reorder_point,
            draw_rate_per_min: d.draw_rate_per_min,
            priority: d.priority,
            health: d.health,
            served_by: lines_per_demand
                .get(&(d.site, d.cargo.clone()))
                .copied()
                .unwrap_or(0),
            starved_ticks: d.starved_ticks,
            shortfall_total: d.shortfall_total,
        });
    }

    for e in &sim.events {
        let describe = |id: OrderId| -> String {
            sim.orders.get(&id).map_or_else(
                || format!("order #{}", id.0),
                |o| {
                    let kind = if o.kind.is_standing() { "Line" } else { "Run" };
                    format!(
                        "{kind} #{} {} {} {} → {}",
                        id.0,
                        o.stats.delivered,
                        o.cargo.unit_label(),
                        o.cargo.0,
                        site_name(meta, sim, o.sink)
                    )
                },
            )
        };
        let text = match (&e.kind, e.order) {
            (OrderEventKind::Completed, Some(id)) => format!("{} — completed", describe(id)),
            (OrderEventKind::Cancelled, Some(id)) => format!("{} — cancelled", describe(id)),
            (OrderEventKind::Failed(_), Some(id)) => format!("{} — failed: site removed", describe(id)),
            (OrderEventKind::DemandStarving { site, cargo }, _) => {
                format!("{} is out of {}", site_name(meta, sim, *site), cargo.0)
            }
            (_, None) => "order event".into(),
        };
        snap.events.push(EventRow {
            tick: e.tick,
            order: e.order,
            text,
        });
    }

    snap.rejected = sim
        .rejected
        .iter()
        .map(|(t, why)| format!("t{t}: {why}"))
        .collect();
    snap
}
