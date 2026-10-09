//! Order-layer data model (PLAN-LOGISTICS-ORDERS-v1 §3–§4).
//!
//! Plain data, no Bevy types: ids, cargo, stock, vehicles, orders, draw demands.
//! Quantities are integer **units** (1 t bulk/munitions, 1 m³ liquid, 1 person).

use std::collections::BTreeMap;

/// Sim tick counter (the order sim advances one per economy tick).
pub type Tick = u64;

/// Logistics endpoint — depot, front supply point, construction site, plant, unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SiteId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OrderId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VehicleId(pub u32);

/// Owning faction slot (overlay faction `u8`). `OwnerId::PLAYER` = slot 0; AI factions use others.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct OwnerId(pub u8);

impl OwnerId {
    pub const PLAYER: Self = Self(0);
}

/// Buffer tag of the goods moved (`"Cement"`, `"ammo"`, `"personnel"`, …).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CargoKind(pub String);

impl CargoKind {
    #[must_use]
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Vehicle-compatibility class inferred from the tag.
    #[must_use]
    pub fn class(&self) -> CargoClass {
        let t = self.0.to_ascii_lowercase();
        if matches!(t.as_str(), "personnel" | "men" | "troops" | "crew" | "workers") {
            CargoClass::Personnel
        } else if matches!(t.as_str(), "fuel" | "water" | "oil" | "diesel") {
            CargoClass::Liquid
        } else if matches!(t.as_str(), "ammo" | "munitions" | "shells" | "explosives") {
            CargoClass::Munitions
        } else if t.ends_with("_unit") {
            CargoClass::Deployable
        } else {
            CargoClass::Bulk
        }
    }

    /// Display unit for quantities of this cargo.
    #[must_use]
    pub fn unit_label(&self) -> &'static str {
        match self.class() {
            CargoClass::Personnel => "pers",
            CargoClass::Liquid => "m³",
            CargoClass::Deployable => "pcs",
            CargoClass::Bulk | CargoClass::Munitions => "t",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CargoClass {
    Bulk,
    Liquid,
    Munitions,
    Personnel,
    Deployable,
}

impl CargoClass {
    #[must_use]
    pub const fn bit(self) -> u8 {
        match self {
            Self::Bulk => 1,
            Self::Liquid => 2,
            Self::Munitions => 4,
            Self::Personnel => 8,
            Self::Deployable => 16,
        }
    }
}

/// Bitmask of [`CargoClass`]es a vehicle may carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct CargoClassMask(pub u8);

impl CargoClassMask {
    pub const ALL: Self = Self(0x1f);
    /// Flatbed / box truck: bulk, munitions, deployables.
    pub const FREIGHT: Self = Self(1 | 4 | 16);
    pub const TANKER: Self = Self(2);
    pub const TROOP: Self = Self(8);

    #[must_use]
    pub const fn of(class: CargoClass) -> Self {
        Self(class.bit())
    }

    #[must_use]
    pub const fn accepts(self, class: CargoClass) -> bool {
        self.0 & class.bit() != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

impl Priority {
    pub const ALL: [Self; 4] = [Self::Critical, Self::High, Self::Normal, Self::Low];

    /// Base allocation weight; aging adds at most [`super::SimConfig::aging_cap`].
    #[must_use]
    pub const fn weight(self) -> u32 {
        match self {
            Self::Critical => 300,
            Self::High => 200,
            Self::Normal => 100,
            Self::Low => 0,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::High => "High",
            Self::Normal => "Normal",
            Self::Low => "Low",
        }
    }
}

/// Per-site, per-cargo stock. `available` feeds pickups; `projected` feeds demand evaluation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StockEntry {
    pub on_hand: u32,
    /// Claimed by assigned pickups not yet loaded.
    pub reserved_out: u32,
    /// Loaded or assigned cargo heading here.
    pub inbound: u32,
}

impl StockEntry {
    #[must_use]
    pub fn available(&self) -> u32 {
        self.on_hand.saturating_sub(self.reserved_out)
    }

    #[must_use]
    pub fn projected(&self) -> u32 {
        self.on_hand.saturating_add(self.inbound)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StockLedger {
    pub entries: BTreeMap<(SiteId, CargoKind), StockEntry>,
}

impl StockLedger {
    #[must_use]
    pub fn get(&self, site: SiteId, cargo: &CargoKind) -> StockEntry {
        self.entries
            .get(&(site, cargo.clone()))
            .copied()
            .unwrap_or_default()
    }

    pub fn entry_mut(&mut self, site: SiteId, cargo: &CargoKind) -> &mut StockEntry {
        self.entries.entry((site, cargo.clone())).or_default()
    }
}

/// One leg of a trip with a countdown and its total (total drives progress fractions in UI).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leg {
    pub remaining: u32,
    pub total: u32,
}

impl Leg {
    #[must_use]
    pub fn new(ticks: u32) -> Self {
        Self {
            remaining: ticks,
            total: ticks,
        }
    }

    /// 0.0 at leg start → 1.0 at arrival.
    #[must_use]
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            1.0 - self.remaining as f32 / self.total as f32
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehiclePhase {
    Idle,
    ToPickup(Leg),
    Loading(Leg),
    Hauling(Leg),
    Unloading(Leg),
}

impl VehiclePhase {
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::ToPickup(_) => "to pickup",
            Self::Loading(_) => "loading",
            Self::Hauling(_) => "hauling",
            Self::Unloading(_) => "unloading",
        }
    }

    /// True once cargo is on board (cancel no longer aborts the trip).
    #[must_use]
    pub const fn is_loaded(&self) -> bool {
        matches!(self, Self::Hauling(_) | Self::Unloading(_))
    }
}

/// The trip a busy vehicle is running.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment {
    pub order: OrderId,
    pub source: SiteId,
    pub sink: SiteId,
    pub cargo: CargoKind,
    pub load: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vehicle {
    pub id: VehicleId,
    pub owner: OwnerId,
    pub class_mask: CargoClassMask,
    pub capacity_units: u32,
    pub speed_mps: f32,
    pub home: SiteId,
    /// Last site reached (pickup origin for the next deadhead leg).
    pub at: SiteId,
    pub phase: VehiclePhase,
    pub assignment: Option<Assignment>,
}

impl Vehicle {
    #[must_use]
    pub fn is_free(&self) -> bool {
        matches!(self.phase, VehiclePhase::Idle) && self.assignment.is_none()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderKind {
    /// Continuous: keep the sink's draw demand satisfied from the source.
    StandingLine,
    /// One-off: deliver `quantity`, then complete and release vehicles.
    OneOffRun { quantity: u32 },
}

impl OrderKind {
    #[must_use]
    pub const fn is_standing(&self) -> bool {
        matches!(self, Self::StandingLine)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaitReason {
    /// Every compatible vehicle of the owner is busy.
    NoVehicles,
    /// Owner has no vehicle able to carry this cargo class at all.
    NoCompatibleVehicle,
    /// Source has no unreserved stock.
    SourceEmpty,
    /// Travel model reports no route source → sink.
    Unreachable,
    /// Standing line: sink publishes no draw demand for this cargo.
    NoDemand,
    /// Order's vehicle cap is reached; more trips wait for those trucks.
    VehicleCap,
}

impl WaitReason {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NoVehicles => "no free vehicles",
            Self::NoCompatibleVehicle => "no compatible vehicle",
            Self::SourceEmpty => "source empty",
            Self::Unreachable => "no route",
            Self::NoDemand => "no demand at sink",
            Self::VehicleCap => "vehicle cap reached",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FailReason {
    SiteRemoved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderStatus {
    /// Created, not yet evaluated by an allocation pass.
    Pending,
    /// Has trips in flight or just got vehicles.
    Active,
    /// Needs trips but cannot get them right now.
    Waiting(WaitReason),
    /// Standing line with a satisfied demand — nothing to do.
    Idle,
    Paused,
    /// No new trips; loaded vehicles finish delivering.
    Cancelling,
    Cancelled,
    Completed,
    Failed(FailReason),
}

impl OrderStatus {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Completed | Self::Failed(_))
    }

    /// May the allocator hand this order new vehicles?
    #[must_use]
    pub const fn accepts_trips(self) -> bool {
        matches!(
            self,
            Self::Pending | Self::Active | Self::Waiting(_) | Self::Idle
        )
    }

    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::Pending => "pending".into(),
            Self::Active => "active".into(),
            Self::Waiting(r) => format!("waiting: {}", r.label()),
            Self::Idle => "idle (supplied)".into(),
            Self::Paused => "paused".into(),
            Self::Cancelling => "cancelling".into(),
            Self::Cancelled => "cancelled".into(),
            Self::Completed => "completed".into(),
            Self::Failed(FailReason::SiteRemoved) => "failed: site removed".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OrderStats {
    pub delivered: u32,
    /// Assigned or loaded, not yet unloaded.
    pub in_flight: u32,
    pub trips_done: u32,
    pub trips_in_flight: u32,
    /// Cargo lost to aborted trips (site removed mid-haul, source shrank under a pickup).
    pub lost: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Order {
    pub id: OrderId,
    pub owner: OwnerId,
    pub kind: OrderKind,
    pub source: SiteId,
    pub sink: SiteId,
    pub cargo: CargoKind,
    pub priority: Priority,
    /// Max vehicles this order may hold at once (`None` = whole pool).
    pub vehicle_cap: Option<u32>,
    pub status: OrderStatus,
    pub created_tick: Tick,
    pub finished_tick: Option<Tick>,
    pub stats: OrderStats,
    /// Consecutive ticks with unmet need and no new vehicle (drives aging).
    pub waiting_ticks: u32,
    /// Standing-line hysteresis: true between reorder trigger and reaching buffer target.
    pub refilling: bool,
}

impl Order {
    /// One-off: undelivered and unassigned quantity.
    #[must_use]
    pub fn run_remaining(&self) -> u32 {
        match self.kind {
            OrderKind::OneOffRun { quantity } => quantity
                .saturating_sub(self.stats.delivered)
                .saturating_sub(self.stats.in_flight),
            OrderKind::StandingLine => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ConsumerKind {
    Front,
    Depot,
    Construction,
    Unit,
    Plant,
}

impl ConsumerKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Front => "front",
            Self::Depot => "depot",
            Self::Construction => "construction",
            Self::Unit => "unit",
            Self::Plant => "plant",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DemandHealth {
    #[default]
    Satisfied,
    /// Projected stock below reorder point.
    Low,
    /// Nothing on hand while drawing.
    Starving,
}

/// A consumer's published need (fronts, depots, construction sites, units).
#[derive(Clone, Debug, PartialEq)]
pub struct DrawDemand {
    pub site: SiteId,
    pub cargo: CargoKind,
    pub consumer: ConsumerKind,
    /// High watermark — refills aim here.
    pub buffer_target: u32,
    /// Low watermark — a standing line starts refilling when projected stock drops below.
    pub reorder_point: u32,
    /// Modelled consumption drawn from on-hand stock.
    pub draw_rate_per_min: u32,
    pub priority: Priority,
    pub health: DemandHealth,
    pub starved_ticks: u64,
    pub consumed_total: u64,
    /// Units drawn but not satisfied by on-hand stock.
    pub shortfall_total: u64,
    /// Integer draw accumulator (units × ticks_per_minute).
    pub draw_accum: u64,
}
