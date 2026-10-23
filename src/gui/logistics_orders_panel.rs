//! Logistics orders panel — Bevy UI for standing lines / one-off runs
//! (PLAN-LOGISTICS-ORDERS-v1 §6). Presentation only: reads [`LogisticsOrdersSnapshotRes`],
//! writes [`LogisticsOrderRequest`]. Toggle: **Shift + logistics-targets key** (default Shift+F6).
//!
//! Every button carries a [`PanelAction`]; [`apply_panel_action`] is the single pure funnel from
//! a click to draft edits / [`LogisticsOrderCommand`]s, so the whole command path is headless-testable.

use std::hash::{Hash, Hasher};

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::economy::logistics::orders::{
    best_source_for, suggest_lines_for_unserved, AdvisorGap, CargoClassMask, CargoKind,
    ConsumerKind, DemandHealth, DemandRow, LogisticsOrderCommand, LogisticsOrderRequest,
    LogisticsOrdersSnapshot, LogisticsOrdersSnapshotRes, OrderId, OrderRow, OrderStatus, OwnerId,
    Priority, SiteId, SiteRole, SiteRow,
};
use crate::gui::input_bindings::InputBindings;
use crate::gui::panel_presence::PanelPresence;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;

/// Cargo offered in the picker besides whatever the source currently stocks.
pub const COMMON_CARGO: &[&str] = &["ammo", "fuel", "personnel", "Cement", "Steel", "food"];

/// Site buttons shown per picker before the rest is folded into "+N more" (Sites tab has all).
const PICKER_CAP: usize = 12;
/// Rows shown in the long lists (fleet, sites) before folding.
const LIST_CAP: usize = 40;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrdersTab {
    /// Order form (the default: players come here to issue orders).
    #[default]
    New,
    Runs,
    Lines,
    Demands,
    Fleet,
    Sites,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DraftKind {
    /// Continuous: keep the destination supplied (driven by its draw demand).
    #[default]
    Line,
    /// One-off: deliver a quantity, repeat trips as needed, then finish.
    Run,
}

/// Unsent order form.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderDraft {
    pub kind: DraftKind,
    pub owner: OwnerId,
    pub source: Option<SiteId>,
    pub sink: Option<SiteId>,
    pub cargo: String,
    pub quantity: u32,
    pub priority: Priority,
    pub cap_vehicles: bool,
    pub vehicle_cap: u32,
    /// Line only: demand published at the destination when it has none.
    pub buffer_target: u32,
    pub reorder_point: u32,
    pub draw_rate_per_min: u32,
}

impl Default for OrderDraft {
    fn default() -> Self {
        Self {
            kind: DraftKind::Line,
            owner: OwnerId::PLAYER,
            source: None,
            sink: None,
            cargo: "ammo".into(),
            quantity: 40,
            priority: Priority::Normal,
            cap_vehicles: false,
            vehicle_cap: 2,
            buffer_target: 40,
            reorder_point: 15,
            draw_rate_per_min: 0,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct LogisticsOrdersPanelState {
    pub visible: bool,
    pub tab: OrdersTab,
    pub draft: OrderDraft,
    /// Why the last submit / action was refused (shown in the danger colour).
    pub last_error: Option<String>,
    /// What the last submit / action did (shown in the data colour).
    pub last_status: Option<String>,
    pub show_finished: bool,
    /// Keep order routes on the map while the panel is closed.
    pub always_show_routes: bool,
    /// Dev seeding (Sites tab): amount added per click.
    pub seed_amount: u32,
    /// Mirror of `AutoSupplyPolicy::enabled(draft.owner)` (the policy resource is the authority).
    pub auto_supply: bool,
}

impl LogisticsOrdersPanelState {
    fn seed(&self) -> u32 {
        if self.seed_amount == 0 {
            100
        } else {
            self.seed_amount
        }
    }

    fn fail(&mut self, msg: impl Into<String>) {
        self.last_error = Some(msg.into());
        self.last_status = None;
    }

    fn ok(&mut self, msg: impl Into<String>) {
        self.last_error = None;
        self.last_status = Some(msg.into());
    }
}

/// Consumer kind implied by a destination's role.
#[must_use]
pub fn consumer_for_role(role: SiteRole) -> ConsumerKind {
    match role {
        SiteRole::Front => ConsumerKind::Front,
        SiteRole::Construction => ConsumerKind::Construction,
        SiteRole::Plant => ConsumerKind::Plant,
        SiteRole::Unit => ConsumerKind::Unit,
        SiteRole::Depot => ConsumerKind::Depot,
    }
}

/// Validate the draft against the snapshot and produce the commands to send.
pub fn draft_commands(
    draft: &OrderDraft,
    snap: &LogisticsOrdersSnapshot,
) -> Result<Vec<LogisticsOrderCommand>, &'static str> {
    let source = draft.source.ok_or("pick a source")?;
    let sink = draft.sink.ok_or("pick a destination")?;
    if source == sink {
        return Err("source and destination must differ");
    }
    let cargo = draft.cargo.trim();
    if cargo.is_empty() {
        return Err("pick a cargo");
    }
    let cargo = CargoKind::new(cargo);
    let vehicle_cap = draft.cap_vehicles.then_some(draft.vehicle_cap.max(1));
    let mut out = Vec::new();
    match draft.kind {
        DraftKind::Run => {
            if draft.quantity == 0 {
                return Err("quantity must be above zero");
            }
            out.push(LogisticsOrderCommand::CreateOneOffRun {
                owner: draft.owner,
                source,
                sink,
                cargo,
                quantity: draft.quantity,
                priority: draft.priority,
                vehicle_cap,
            });
        }
        DraftKind::Line => {
            let has_demand = snap
                .demands
                .iter()
                .any(|d| d.site == sink && d.cargo == cargo.0);
            if !has_demand {
                if draft.buffer_target == 0 {
                    return Err("set a buffer target for the destination");
                }
                let role = snap.site(sink).map_or(SiteRole::Depot, |s| s.role);
                out.push(LogisticsOrderCommand::PublishDemand {
                    site: sink,
                    cargo: cargo.clone(),
                    consumer: consumer_for_role(role),
                    buffer_target: draft.buffer_target,
                    reorder_point: draft.reorder_point.min(draft.buffer_target),
                    draw_rate_per_min: draft.draw_rate_per_min,
                    priority: draft.priority,
                });
            }
            out.push(LogisticsOrderCommand::CreateStandingLine {
                owner: draft.owner,
                source,
                sink,
                cargo,
                priority: draft.priority,
                vehicle_cap,
            });
        }
    }
    Ok(out)
}

/// Units a one-off run must bring to fill the demand to its buffer target.
#[must_use]
pub fn demand_deficit(d: &DemandRow) -> u32 {
    d.buffer_target.saturating_sub(d.on_hand.saturating_add(d.inbound))
}

/// Fill the order form from a demand row.
pub fn prefill_from_demand(
    state: &mut LogisticsOrdersPanelState,
    snap: &LogisticsOrdersSnapshot,
    d: &DemandRow,
    kind: DraftKind,
) {
    state.draft.kind = kind;
    state.draft.owner = d.owner;
    state.draft.sink = Some(d.site);
    state.draft.cargo = d.cargo.clone();
    state.draft.priority = d.priority;
    if let Some(src) = best_source_for(snap, d) {
        state.draft.source = Some(src);
    }
    if kind == DraftKind::Run {
        state.draft.quantity = demand_deficit(d).max(1);
    }
}

/// `m:ss` from order-sim ticks (1 tick = 1 simulated second).
#[must_use]
pub fn format_eta(ticks: u32) -> String {
    format!("{}:{:02}", ticks / 60, ticks % 60)
}

/// Headline for a one-off run row, e.g. `24/40 t · trip 3/5`.
#[must_use]
pub fn run_progress_text(row: &OrderRow) -> String {
    let planned = row.trips_planned.max(row.trips_done);
    let current = (row.trips_done + u32::from(row.vehicles > 0)).min(planned.max(1));
    format!(
        "{}/{} {} · trip {}/{}",
        row.delivered,
        row.quantity.unwrap_or(0),
        row.unit,
        current,
        planned
    )
}

/// Headline for a standing line row, e.g. `fill 62% · 12.0 t/min`.
#[must_use]
pub fn line_progress_text(row: &OrderRow) -> String {
    format!(
        "fill {:.0}% · {:.1} {}/min",
        row.fill_ratio.unwrap_or(0.0) * 100.0,
        row.throughput_per_min,
        row.unit
    )
}

/// Standing line detail with the sink's demand: stock vs buffer target and the draw rate.
#[must_use]
pub fn line_detail_text(row: &OrderRow, demand: Option<&DemandRow>) -> String {
    match demand {
        Some(d) => format!(
            "stock {}/{} {} (+{} inbound) · draw {} {}/min · delivering {:.1} {}/min",
            d.on_hand,
            d.buffer_target,
            d.unit,
            d.inbound,
            d.draw_rate_per_min,
            d.unit,
            row.throughput_per_min,
            row.unit
        ),
        None => line_progress_text(row),
    }
}

/// Demand a standing line feeds (`(sink, cargo)`).
#[must_use]
pub fn demand_for_line<'a>(
    snap: &'a LogisticsOrdersSnapshot,
    row: &OrderRow,
) -> Option<&'a DemandRow> {
    snap.demands
        .iter()
        .find(|d| d.site == row.sink && d.cargo == row.cargo)
}

/// One-line order title: `RUN #3  ammo  Depot → Front` / `LINE #4 …`.
#[must_use]
pub fn order_title(row: &OrderRow) -> String {
    format!(
        "{} #{}  {}  {} → {}",
        if row.standing { "LINE" } else { "RUN" },
        row.id.0,
        row.cargo,
        row.source_name,
        row.sink_name
    )
}

/// Which row controls make sense for an order in its current state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OrderControls {
    pub pause: bool,
    pub resume: bool,
    pub cancel: bool,
    pub priority: bool,
}

#[must_use]
pub fn order_controls(row: &OrderRow) -> OrderControls {
    if row.status.is_terminal() {
        return OrderControls::default();
    }
    let cancelling = row.status == OrderStatus::Cancelling;
    OrderControls {
        pause: !cancelling && row.status != OrderStatus::Paused,
        resume: row.status == OrderStatus::Paused,
        cancel: !cancelling,
        priority: !cancelling,
    }
}

#[must_use]
pub fn priority_up(p: Priority) -> Priority {
    match p {
        Priority::Low => Priority::Normal,
        Priority::Normal => Priority::High,
        Priority::High | Priority::Critical => Priority::Critical,
    }
}

#[must_use]
pub fn priority_down(p: Priority) -> Priority {
    match p {
        Priority::Critical => Priority::High,
        Priority::High => Priority::Normal,
        Priority::Normal | Priority::Low => Priority::Low,
    }
}

fn priority_cycle(p: Priority) -> Priority {
    match p {
        Priority::Low => Priority::Normal,
        Priority::Normal => Priority::High,
        Priority::High => Priority::Critical,
        Priority::Critical => Priority::Low,
    }
}

/// Order rows of one kind that still need the player's attention (waiting / starving).
#[must_use]
pub fn attention_count(snap: &LogisticsOrdersSnapshot) -> usize {
    snap.orders
        .iter()
        .filter(|o| matches!(o.status, OrderStatus::Waiting(_)))
        .count()
        + snap
            .demands
            .iter()
            .filter(|d| d.health == DemandHealth::Starving)
            .count()
}

/// Rows of one order kind, live ones first.
#[must_use]
pub fn visible_orders(
    snap: &LogisticsOrdersSnapshot,
    standing: bool,
    show_finished: bool,
) -> Vec<&OrderRow> {
    let mut rows: Vec<&OrderRow> = snap
        .orders
        .iter()
        .filter(|o| o.standing == standing && (show_finished || !o.status.is_terminal()))
        .collect();
    rows.sort_by_key(|o| (o.status.is_terminal(), o.id));
    rows
}

/// Source picker: the owner's depots / plants that hold anything.
#[must_use]
pub fn source_sites(snap: &LogisticsOrdersSnapshot, owner: OwnerId) -> Vec<&SiteRow> {
    snap.sites
        .iter()
        .filter(|s| {
            s.owner == owner
                && matches!(s.role, SiteRole::Depot | SiteRole::Plant)
                && s.stock.iter().any(|r| r.on_hand > 0)
        })
        .collect()
}

/// Destination picker: any site of the owner (depots, plants, fronts, construction, units).
#[must_use]
pub fn sink_sites(
    snap: &LogisticsOrdersSnapshot,
    owner: OwnerId,
    source: Option<SiteId>,
) -> Vec<&SiteRow> {
    snap.sites
        .iter()
        .filter(|s| s.owner == owner && Some(s.id) != source)
        .collect()
}

/// Cargo picker: what the source stocks first, then the common tags.
#[must_use]
pub fn cargo_options(snap: &LogisticsOrdersSnapshot, source: Option<SiteId>) -> Vec<String> {
    let mut out: Vec<String> = source
        .and_then(|id| snap.site(id))
        .map(|s| {
            s.stock
                .iter()
                .filter(|r| r.on_hand > 0)
                .map(|r| r.cargo.clone())
                .collect()
        })
        .unwrap_or_default();
    for c in COMMON_CARGO {
        if !out.iter().any(|o| o == c) {
            out.push((*c).to_string());
        }
    }
    out
}

fn stock_of<'a>(site: &'a SiteRow, cargo: &str) -> Option<&'a crate::economy::logistics::orders::StockRow> {
    site.stock.iter().find(|r| r.cargo == cargo)
}

/// Vehicles the Fleet tab can add for development.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevVehicle {
    Truck,
    Tanker,
    TroopCarrier,
}

impl DevVehicle {
    const fn label(self) -> &'static str {
        match self {
            Self::Truck => "+ truck",
            Self::Tanker => "+ tanker",
            Self::TroopCarrier => "+ troop carrier",
        }
    }

    const fn spec(self) -> (CargoClassMask, u32, f32) {
        match self {
            Self::Truck => (CargoClassMask::FREIGHT, 10, 12.0),
            Self::Tanker => (CargoClassMask::TANKER, 20, 11.0),
            Self::TroopCarrier => (CargoClassMask::TROOP, 12, 15.0),
        }
    }
}

/// Everything a panel button can do.
#[derive(Clone, Debug, PartialEq)]
pub enum PanelAction {
    Close,
    Tab(OrdersTab),
    Kind(DraftKind),
    Source(SiteId),
    Sink(SiteId),
    Cargo(String),
    Quantity(i32),
    Buffer(i32),
    DrawRate(i32),
    CyclePriority,
    ToggleCap,
    CapDelta(i32),
    Submit,
    Pause(OrderId),
    Resume(OrderId),
    Cancel(OrderId),
    SetOrderPriority(OrderId, Priority),
    ToggleShowFinished,
    ToggleAlwaysRoutes,
    AutoSupply,
    /// Flip the faction's `AutoSupplyPolicy` (applied by the click system, not the pure funnel).
    ToggleAutoSupply,
    PrefillDemand {
        site: SiteId,
        cargo: String,
        kind: DraftKind,
    },
    SeedDelta(i32),
    AddStock(SiteId),
    AddVehicle(DevVehicle),
}

fn step(value: u32, delta: i32, min: u32, max: u32) -> u32 {
    (i64::from(value) + i64::from(delta)).clamp(i64::from(min), i64::from(max)) as u32
}

fn describe_draft(draft: &OrderDraft, snap: &LogisticsOrdersSnapshot) -> String {
    let name = |id: Option<SiteId>| {
        id.and_then(|i| snap.site(i))
            .map_or("?".to_string(), |s| s.name.clone())
    };
    let unit = CargoKind::new(draft.cargo.trim()).unit_label();
    match draft.kind {
        DraftKind::Run => format!(
            "Dispatched run: {} {unit} {} {} → {}",
            draft.quantity,
            draft.cargo.trim(),
            name(draft.source),
            name(draft.sink)
        ),
        DraftKind::Line => format!(
            "Started supply line: {} {} → {}",
            draft.cargo.trim(),
            name(draft.source),
            name(draft.sink)
        ),
    }
}

/// The single click funnel: edits the draft / panel state and returns the commands to send.
pub fn apply_panel_action(
    state: &mut LogisticsOrdersPanelState,
    snap: &LogisticsOrdersSnapshot,
    action: PanelAction,
) -> Vec<LogisticsOrderCommand> {
    let mut out = Vec::new();
    match action {
        PanelAction::Close => state.visible = false,
        PanelAction::Tab(t) => state.tab = t,
        PanelAction::Kind(k) => state.draft.kind = k,
        PanelAction::Source(id) => {
            state.draft.source = Some(id);
            if state.draft.sink == Some(id) {
                state.draft.sink = None;
            }
            // Offer something the source can actually ship.
            if let Some(site) = snap.site(id) {
                let stocked: Vec<&str> = site
                    .stock
                    .iter()
                    .filter(|r| r.on_hand > 0)
                    .map(|r| r.cargo.as_str())
                    .collect();
                if !stocked.is_empty() && !stocked.contains(&state.draft.cargo.trim()) {
                    state.draft.cargo = stocked[0].to_string();
                }
            }
            state.last_error = None;
        }
        PanelAction::Sink(id) => {
            state.draft.sink = Some(id);
            if state.draft.source == Some(id) {
                state.draft.source = None;
            }
            state.last_error = None;
        }
        PanelAction::Cargo(c) => state.draft.cargo = c,
        PanelAction::Quantity(d) => state.draft.quantity = step(state.draft.quantity, d, 1, 100_000),
        PanelAction::Buffer(d) => {
            state.draft.buffer_target = step(state.draft.buffer_target, d, 1, 100_000);
            state.draft.reorder_point = (state.draft.buffer_target * 3 / 8).max(1);
        }
        PanelAction::DrawRate(d) => {
            state.draft.draw_rate_per_min = step(state.draft.draw_rate_per_min, d, 0, 10_000);
        }
        PanelAction::CyclePriority => state.draft.priority = priority_cycle(state.draft.priority),
        PanelAction::ToggleCap => state.draft.cap_vehicles = !state.draft.cap_vehicles,
        PanelAction::CapDelta(d) => state.draft.vehicle_cap = step(state.draft.vehicle_cap, d, 1, 64),
        PanelAction::Submit => match draft_commands(&state.draft, snap) {
            Ok(cmds) => {
                let msg = describe_draft(&state.draft, snap);
                state.tab = match state.draft.kind {
                    DraftKind::Line => OrdersTab::Lines,
                    DraftKind::Run => OrdersTab::Runs,
                };
                state.ok(msg);
                out = cmds;
            }
            Err(e) => state.fail(e),
        },
        PanelAction::Pause(order) => out.push(LogisticsOrderCommand::Pause { order }),
        PanelAction::Resume(order) => out.push(LogisticsOrderCommand::Resume { order }),
        PanelAction::Cancel(order) => out.push(LogisticsOrderCommand::Cancel { order }),
        PanelAction::SetOrderPriority(order, priority) => {
            out.push(LogisticsOrderCommand::SetPriority { order, priority });
        }
        PanelAction::ToggleShowFinished => state.show_finished = !state.show_finished,
        PanelAction::ToggleAlwaysRoutes => state.always_show_routes = !state.always_show_routes,
        PanelAction::AutoSupply => {
            let advice = suggest_lines_for_unserved(snap, state.draft.owner);
            if advice.commands.is_empty() {
                state.ok("nothing to supply: every demand has a line or no depot holds the cargo");
            } else {
                state.ok(format!("Started {} supply line(s)", advice.commands.len()));
                out = advice.commands;
            }
        }
        PanelAction::ToggleAutoSupply => {}
        PanelAction::PrefillDemand { site, cargo, kind } => {
            match snap.demands.iter().find(|d| d.site == site && d.cargo == cargo) {
                Some(d) => {
                    prefill_from_demand(state, snap, d, kind);
                    state.tab = OrdersTab::New;
                    if state.draft.source.is_none() {
                        state.fail("no depot holds this cargo — pick a source");
                    } else {
                        state.ok("form filled from demand — review and send");
                    }
                }
                None => state.fail("demand no longer exists"),
            }
        }
        PanelAction::SeedDelta(d) => state.seed_amount = step(state.seed(), d, 1, 100_000),
        PanelAction::AddStock(site) => {
            let cargo = state.draft.cargo.trim().to_string();
            if cargo.is_empty() {
                state.fail("pick a cargo to seed");
            } else {
                let amount = state.seed();
                state.ok(format!("seeded {amount} {cargo}"));
                out.push(LogisticsOrderCommand::AddStock {
                    site,
                    cargo: CargoKind::new(cargo),
                    amount,
                });
            }
        }
        PanelAction::AddVehicle(kind) => {
            match snap.sites_of(state.draft.owner, SiteRole::Depot).next() {
                Some(home) => {
                    let (class_mask, capacity_units, speed_mps) = kind.spec();
                    state.ok(format!("added {} at {}", kind.label().trim_start_matches("+ "), home.name));
                    out.push(LogisticsOrderCommand::AddVehicle {
                        owner: state.draft.owner,
                        home: home.id,
                        class_mask,
                        capacity_units,
                        speed_mps,
                    });
                }
                None => state.fail("no depot to host a vehicle"),
            }
        }
    }
    out
}

#[derive(Component)]
pub struct LogisticsOrdersBevyRoot;

/// A panel button; clicking sends its [`PanelAction`] through [`apply_panel_action`].
#[derive(Component, Clone, Debug, PartialEq)]
pub struct LogisticsOrdersAction(pub PanelAction);

#[derive(Default)]
struct OrdersPanelStamp {
    key: u64,
    tab: Option<OrdersTab>,
    scroll: Vec2,
}

pub struct LogisticsOrdersPanelPlugin;

impl Plugin for LogisticsOrdersPanelPlugin {
    fn build(&self, app: &mut App) {
        // A windowed game leaves the player's supply lines to the player; the panel toggles it.
        app.init_resource::<crate::economy::logistics::orders::AutoSupplyPolicy>();
        app.world_mut()
            .resource_mut::<crate::economy::logistics::orders::AutoSupplyPolicy>()
            .player_default = false;
        app.init_resource::<LogisticsOrdersPanelState>()
            .add_systems(
                Update,
                // Clicks before the rebuild: a rebuild respawns the buttons, which would eat a
                // press that landed this frame.
                (
                    toggle_logistics_orders_panel,
                    logistics_orders_panel_clicks,
                    sync_auto_supply_mirror,
                    sync_logistics_orders_bevy_panel,
                    logistics_orders_panel_scroll,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            );
    }
}

fn toggle_logistics_orders_panel(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut state: ResMut<LogisticsOrdersPanelState>,
) {
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if shift && keys.just_pressed(bindings.toggle_logistics_targets_panel) {
        state.visible = !state.visible;
    }
}

fn site_label(snap: &LogisticsOrdersSnapshot, id: Option<SiteId>) -> String {
    id.and_then(|id| snap.site(id))
        .map_or_else(|| "—".into(), |s| format!("{} ({})", s.name, s.role.label()))
}

/// Cheap change key: panel state + the snapshot parts the panel shows. The snapshot is rebuilt
/// every frame, so comparing `tick` + per-row state avoids respawning the panel needlessly.
fn panel_stamp(snap: &LogisticsOrdersSnapshot, state: &LogisticsOrdersPanelState) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    format!("{state:?}").hash(&mut h);
    snap.tick.hash(&mut h);
    snap.orders.len().hash(&mut h);
    snap.demands.len().hash(&mut h);
    snap.vehicles.len().hash(&mut h);
    snap.sites.len().hash(&mut h);
    snap.events.len().hash(&mut h);
    snap.rejected.len().hash(&mut h);
    for o in &snap.orders {
        (o.id.0, &o.status_label, o.priority, o.delivered, o.in_flight, o.trips_done, o.vehicles)
            .hash(&mut h);
    }
    for d in &snap.demands {
        (d.on_hand, d.inbound, d.served_by, d.health.severity()).hash(&mut h);
    }
    for s in &snap.sites {
        for r in &s.stock {
            (r.on_hand, r.reserved_out, r.inbound).hash(&mut h);
        }
    }
    h.finish()
}

// ───────────────────────────── Bevy UI building ─────────────────────────────

struct Look<'a> {
    tf: TextFont,
    p: &'a UiPalette,
}

fn rgba(c: crate::gui::UiRgba) -> Color {
    Color::srgba_u8(c.r, c.g, c.b, c.a)
}

impl Look<'_> {
    fn text_color(&self) -> Color {
        self.p.bevy_primary_text()
    }
    fn muted(&self) -> Color {
        self.p.bevy_text_muted()
    }
    fn warn(&self) -> Color {
        rgba(self.p.warn)
    }
    fn danger(&self) -> Color {
        rgba(self.p.danger)
    }
    fn data(&self) -> Color {
        rgba(self.p.fg_data)
    }
    fn status_color(&self, status: OrderStatus) -> Color {
        match status {
            OrderStatus::Waiting(_) | OrderStatus::Paused | OrderStatus::Cancelling => self.warn(),
            OrderStatus::Failed(_) | OrderStatus::Cancelled => self.danger(),
            OrderStatus::Completed | OrderStatus::Idle => self.muted(),
            OrderStatus::Pending | OrderStatus::Active => self.data(),
        }
    }
    fn health_color(&self, h: DemandHealth) -> Color {
        match h {
            DemandHealth::Satisfied => self.data(),
            DemandHealth::Low => self.warn(),
            DemandHealth::Starving => self.danger(),
        }
    }
}

fn label(parent: &mut ChildSpawnerCommands<'_>, look: &Look, text: impl Into<String>, color: Color) {
    parent.spawn((Text::new(text), look.tf.clone(), TextColor(color)));
}

fn heading(parent: &mut ChildSpawnerCommands<'_>, look: &Look, text: &str) {
    label(parent, look, format!("── {text} ──"), look.p.bevy_accent_terminal());
}

#[derive(Clone, Copy)]
enum BtnStyle {
    /// Toggle / picker; `true` = currently selected.
    Pick(bool),
    /// One-shot command.
    Act,
}

fn button(
    parent: &mut ChildSpawnerCommands<'_>,
    look: &Look,
    text: impl Into<String>,
    action: PanelAction,
    style: BtnStyle,
) {
    let (bg, border) = match style {
        BtnStyle::Pick(true) => (look.p.bevy_button_idle(), look.p.bevy_accent_terminal()),
        BtnStyle::Pick(false) => (look.p.bevy_hud_panel_fill(), look.p.bevy_wire_magenta()),
        BtnStyle::Act => (look.p.bevy_button_idle(), look.p.bevy_accent_action()),
    };
    parent
        .spawn((
            LogisticsOrdersAction(action),
            Button,
            Node {
                padding: UiRect::axes(Val::Px(5.0), Val::Px(2.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::all(border),
        ))
        .with_children(|b| {
            b.spawn((Text::new(text), look.tf.clone(), TextColor(look.text_color())));
        });
}

fn wrap_row(parent: &mut ChildSpawnerCommands<'_>, f: impl FnOnce(&mut ChildSpawnerCommands<'_>)) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(4.0),
            row_gap: Val::Px(3.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(f);
}

fn card(parent: &mut ChildSpawnerCommands<'_>, look: &Look, f: impl FnOnce(&mut ChildSpawnerCommands<'_>)) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(look.p.bevy_bg_deep()),
            BorderColor::all(look.p.bevy_border_subtle()),
        ))
        .with_children(f);
}

fn progress_bar(parent: &mut ChildSpawnerCommands<'_>, look: &Look, fraction: f32, color: Color) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(look.p.bevy_bg_elevated()),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(fraction.clamp(0.0, 1.0) * 100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(color),
            ));
        });
}

fn stepper(
    parent: &mut ChildSpawnerCommands<'_>,
    look: &Look,
    caption: &str,
    value: String,
    make: fn(i32) -> PanelAction,
    steps: [i32; 2],
) {
    wrap_row(parent, |r| {
        label(r, look, format!("{caption:<14}"), look.muted());
        button(r, look, format!("-{}", steps[1]), make(-steps[1]), BtnStyle::Act);
        button(r, look, format!("-{}", steps[0]), make(-steps[0]), BtnStyle::Act);
        label(r, look, value, look.text_color());
        button(r, look, format!("+{}", steps[0]), make(steps[0]), BtnStyle::Act);
        button(r, look, format!("+{}", steps[1]), make(steps[1]), BtnStyle::Act);
    });
}

fn site_pick_row(
    parent: &mut ChildSpawnerCommands<'_>,
    look: &Look,
    caption: &str,
    sites: &[&SiteRow],
    selected: Option<SiteId>,
    make: fn(SiteId) -> PanelAction,
    text: impl Fn(&SiteRow) -> String,
    empty_hint: &str,
) {
    label(parent, look, caption, look.muted());
    if sites.is_empty() {
        label(parent, look, empty_hint, look.warn());
        return;
    }
    let mut shown: Vec<&SiteRow> = sites.iter().copied().take(PICKER_CAP).collect();
    if let Some(sel) = selected.and_then(|id| sites.iter().find(|s| s.id == id)) {
        if !shown.iter().any(|s| s.id == sel.id) {
            shown.push(sel);
        }
    }
    let hidden = sites.len().saturating_sub(shown.len());
    wrap_row(parent, |r| {
        for s in &shown {
            button(r, look, text(s), make(s.id), BtnStyle::Pick(selected == Some(s.id)));
        }
        if hidden > 0 {
            label(r, look, format!("+{hidden} more (Sites tab)"), look.muted());
        }
    });
}

fn build_header(root: &mut ChildSpawnerCommands<'_>, look: &Look, state: &LogisticsOrdersPanelState, snap: &LogisticsOrdersSnapshot) {
    let fleet = snap.fleet(state.draft.owner);
    wrap_row(root, |r| {
        label(
            r,
            look,
            format!(
                "Logistics orders · attention {} · fleet {} free / {}",
                attention_count(snap),
                fleet.free,
                fleet.total
            ),
            look.text_color(),
        );
        button(r, look, "x", PanelAction::Close, BtnStyle::Act);
    });
    label(
        root,
        look,
        format!(
            "Draft: {:?} · {} → {} · {}",
            state.draft.kind,
            site_label(snap, state.draft.source),
            site_label(snap, state.draft.sink),
            state.draft.cargo
        ),
        look.muted(),
    );
    if let Some(err) = &state.last_error {
        label(root, look, format!("! {err}"), look.danger());
    }
    if let Some(msg) = &state.last_status {
        label(root, look, msg.clone(), look.data());
    }
    if let Some(r) = snap.rejected.last() {
        label(root, look, format!("sim rejected {r}"), look.warn());
    }
}

fn build_tabs(root: &mut ChildSpawnerCommands<'_>, look: &Look, state: &LogisticsOrdersPanelState, snap: &LogisticsOrdersSnapshot) {
    let live = |standing: bool| {
        snap.orders
            .iter()
            .filter(|o| o.standing == standing && !o.status.is_terminal())
            .count()
    };
    let starving = snap
        .demands
        .iter()
        .filter(|d| d.health == DemandHealth::Starving)
        .count();
    let demands = if starving > 0 {
        format!("Demands ({starving}!)")
    } else {
        format!("Demands ({})", snap.demands.len())
    };
    let tabs = [
        (OrdersTab::New, "New order".to_string()),
        (OrdersTab::Runs, format!("Runs ({})", live(false))),
        (OrdersTab::Lines, format!("Lines ({})", live(true))),
        (OrdersTab::Demands, demands),
        (OrdersTab::Fleet, "Fleet".to_string()),
        (OrdersTab::Sites, "Sites".to_string()),
    ];
    wrap_row(root, |r| {
        for (tab, text) in tabs {
            button(r, look, text, PanelAction::Tab(tab), BtnStyle::Pick(state.tab == tab));
        }
    });
}

fn build_new_order(root: &mut ChildSpawnerCommands<'_>, look: &Look, state: &LogisticsOrdersPanelState, snap: &LogisticsOrdersSnapshot) {
    let draft = &state.draft;
    wrap_row(root, |r| {
        button(r, look, "Supply line", PanelAction::Kind(DraftKind::Line), BtnStyle::Pick(draft.kind == DraftKind::Line));
        button(r, look, "Supply run", PanelAction::Kind(DraftKind::Run), BtnStyle::Pick(draft.kind == DraftKind::Run));
    });
    label(
        root,
        look,
        match draft.kind {
            DraftKind::Line => "Continuous: trucks keep the destination topped up whenever its stock falls below the reorder point.",
            DraftKind::Run => "One-off: deliver a fixed amount using free trucks, repeating trips until done, then release them.",
        },
        look.muted(),
    );

    let cargo = draft.cargo.trim().to_string();
    let sources = source_sites(snap, draft.owner);
    site_pick_row(
        root,
        look,
        "From (depots / plants with stock)",
        &sources,
        draft.source,
        PanelAction::Source,
        |s| match stock_of(s, &cargo) {
            Some(r) => format!("{} · {} {}", s.name, r.on_hand, r.unit),
            None => format!("{} · no {cargo}", s.name),
        },
        "no depot or plant holds stock — seed one on the Sites tab",
    );
    let sinks = sink_sites(snap, draft.owner, draft.source);
    site_pick_row(
        root,
        look,
        "To (any site)",
        &sinks,
        draft.sink,
        PanelAction::Sink,
        |s| format!("{} ({})", s.name, s.role.label()),
        "no other site registered",
    );

    label(root, look, "Cargo", look.muted());
    wrap_row(root, |r| {
        for c in cargo_options(snap, draft.source) {
            let selected = c == cargo;
            button(r, look, c.clone(), PanelAction::Cargo(c), BtnStyle::Pick(selected));
        }
    });

    let unit = CargoKind::new(cargo.clone()).unit_label();
    match draft.kind {
        DraftKind::Run => stepper(
            root,
            look,
            "Quantity",
            format!("{} {unit}", draft.quantity),
            PanelAction::Quantity,
            [1, 10],
        ),
        DraftKind::Line => {
            let existing = draft.sink.and_then(|sink| {
                snap.demands
                    .iter()
                    .find(|d| d.site == sink && d.cargo == cargo)
            });
            if let Some(d) = existing {
                label(
                    root,
                    look,
                    format!(
                        "Demand: keep {} {unit}, reorder below {} (published by {})",
                        d.buffer_target,
                        d.reorder_point,
                        d.consumer.label()
                    ),
                    look.text_color(),
                );
            } else {
                stepper(root, look, "Keep on hand", format!("{} {unit}", draft.buffer_target), PanelAction::Buffer, [10, 100]);
                label(root, look, format!("Reorder below {} {unit}", draft.reorder_point.min(draft.buffer_target)), look.muted());
                stepper(root, look, "Consumption", format!("{} {unit}/min", draft.draw_rate_per_min), PanelAction::DrawRate, [1, 10]);
            }
        }
    }

    wrap_row(root, |r| {
        label(r, look, "Priority", look.muted());
        button(r, look, draft.priority.label(), PanelAction::CyclePriority, BtnStyle::Act);
        label(r, look, "Vehicles", look.muted());
        button(
            r,
            look,
            if draft.cap_vehicles { "cap on" } else { "no cap" },
            PanelAction::ToggleCap,
            BtnStyle::Pick(draft.cap_vehicles),
        );
        if draft.cap_vehicles {
            button(r, look, "-", PanelAction::CapDelta(-1), BtnStyle::Act);
            label(r, look, format!("{}", draft.vehicle_cap), look.text_color());
            button(r, look, "+", PanelAction::CapDelta(1), BtnStyle::Act);
        }
    });
    button(
        root,
        look,
        match draft.kind {
            DraftKind::Line => "Start supply line",
            DraftKind::Run => "Dispatch supply run",
        },
        PanelAction::Submit,
        BtnStyle::Act,
    );
}

fn build_order_row(root: &mut ChildSpawnerCommands<'_>, look: &Look, snap: &LogisticsOrdersSnapshot, row: &OrderRow) {
    card(root, look, |c| {
        label(c, look, order_title(row), look.text_color());
        progress_bar(c, look, row.progress(), look.status_color(row.status));
        label(
            c,
            look,
            if row.standing {
                line_detail_text(row, demand_for_line(snap, row))
            } else {
                run_progress_text(row)
            },
            look.text_color(),
        );
        let mut info = format!("{} · {}", row.status_label, row.priority.label());
        if row.vehicles > 0 {
            info.push_str(&format!(" · trucks x{}", row.vehicles));
        }
        if let Some(cap) = row.vehicle_cap {
            info.push_str(&format!(" (cap {cap})"));
        }
        if let Some(eta) = row.next_eta_ticks {
            info.push_str(&format!(" · next drop {}", format_eta(eta)));
        }
        if row.lost > 0 {
            info.push_str(&format!(" · lost {} {}", row.lost, row.unit));
        }
        label(c, look, info, look.status_color(row.status));
        let ctl = order_controls(row);
        if ctl.pause || ctl.resume || ctl.cancel || ctl.priority {
            wrap_row(c, |r| {
                if ctl.pause {
                    button(r, look, "Pause", PanelAction::Pause(row.id), BtnStyle::Act);
                }
                if ctl.resume {
                    button(r, look, "Resume", PanelAction::Resume(row.id), BtnStyle::Act);
                }
                if ctl.cancel {
                    button(r, look, "Cancel", PanelAction::Cancel(row.id), BtnStyle::Act);
                }
                if ctl.priority {
                    button(r, look, "Prio +", PanelAction::SetOrderPriority(row.id, priority_up(row.priority)), BtnStyle::Act);
                    button(r, look, "Prio -", PanelAction::SetOrderPriority(row.id, priority_down(row.priority)), BtnStyle::Act);
                }
            });
        }
    });
}

fn build_orders_tab(root: &mut ChildSpawnerCommands<'_>, look: &Look, state: &LogisticsOrdersPanelState, snap: &LogisticsOrdersSnapshot, standing: bool) {
    heading(root, look, if standing { "Standing lines" } else { "One-off runs" });
    button(
        root,
        look,
        if state.show_finished { "hide finished" } else { "show finished" },
        PanelAction::ToggleShowFinished,
        BtnStyle::Pick(state.show_finished),
    );
    let rows = visible_orders(snap, standing, state.show_finished);
    if rows.is_empty() {
        label(
            root,
            look,
            if standing {
                "No supply lines. Use the New order tab."
            } else {
                "No supply runs. Use the New order tab."
            },
            look.muted(),
        );
    }
    for row in rows {
        build_order_row(root, look, snap, row);
    }
}

fn build_demands_tab(root: &mut ChildSpawnerCommands<'_>, look: &Look, state: &LogisticsOrdersPanelState, snap: &LogisticsOrdersSnapshot) {
    heading(root, look, "Demands");
    if snap.demands.is_empty() {
        label(root, look, "No site publishes a draw demand yet.", look.muted());
    }
    button(
        root,
        look,
        if state.auto_supply {
            "Auto-supply ON: new demands get lines from the nearest stocked depot"
        } else {
            "Auto-supply OFF: click to start lines for new demands automatically"
        },
        PanelAction::ToggleAutoSupply,
        BtnStyle::Pick(state.auto_supply),
    );
    let advice = suggest_lines_for_unserved(snap, state.draft.owner);
    if !advice.commands.is_empty() {
        button(
            root,
            look,
            format!("Auto-supply: start {} line(s) from nearest stocked depots", advice.commands.len()),
            PanelAction::AutoSupply,
            BtnStyle::Act,
        );
    }
    for gap in &advice.gaps {
        let AdvisorGap::NoSourceWithStock { site, cargo } = gap;
        let name = snap.site(*site).map_or("?", |s| s.name.as_str());
        label(root, look, format!("{name}: no depot holds {cargo}"), look.warn());
    }
    for d in &snap.demands {
        card(root, look, |c| {
            label(
                c,
                look,
                format!("{} · {} · {} · {}", d.site_name, d.cargo, d.consumer.label(), d.priority.label()),
                look.health_color(d.health),
            );
            progress_bar(c, look, d.fill_ratio(), look.health_color(d.health));
            label(
                c,
                look,
                format!(
                    "{}/{} {} (+{} inbound) · reorder < {} · draw {} {}/min",
                    d.on_hand, d.buffer_target, d.unit, d.inbound, d.reorder_point, d.draw_rate_per_min, d.unit
                ),
                look.text_color(),
            );
            let mut extra = Vec::new();
            if let Some(t) = d.time_to_empty_ticks {
                extra.push(format!("empty in {}", format_eta(t)));
            }
            if let (Some(n), Some(rt)) = (d.trucks_needed, d.round_trip_ticks) {
                extra.push(format!("needs {n} trucks (round trip {})", format_eta(rt)));
            }
            if d.shortfall_total > 0 {
                extra.push(format!("short {} {}", d.shortfall_total, d.unit));
            }
            if !extra.is_empty() {
                label(c, look, extra.join(" · "), look.muted());
            }
            if d.served_by == 0 {
                wrap_row(c, |r| {
                    label(r, look, "no supply line", look.warn());
                    button(
                        r,
                        look,
                        "+ Create line",
                        PanelAction::PrefillDemand { site: d.site, cargo: d.cargo.clone(), kind: DraftKind::Line },
                        BtnStyle::Act,
                    );
                    let deficit = demand_deficit(d);
                    if deficit > 0 {
                        button(
                            r,
                            look,
                            format!("+ Run {deficit} {}", d.unit),
                            PanelAction::PrefillDemand { site: d.site, cargo: d.cargo.clone(), kind: DraftKind::Run },
                            BtnStyle::Act,
                        );
                    }
                });
            } else {
                label(c, look, format!("served by {} line(s)", d.served_by), look.muted());
            }
        });
    }
}

fn build_fleet_tab(root: &mut ChildSpawnerCommands<'_>, look: &Look, snap: &LogisticsOrdersSnapshot) {
    heading(root, look, "Fleet");
    for f in &snap.fleets {
        label(
            root,
            look,
            format!(
                "Faction {}: {} vehicles · {} free · {} busy · {} units capacity",
                f.owner.0, f.total, f.free, f.busy, f.capacity_total
            ),
            look.text_color(),
        );
    }
    wrap_row(root, |r| {
        label(r, look, "Dev:", look.muted());
        for kind in [DevVehicle::Truck, DevVehicle::Tanker, DevVehicle::TroopCarrier] {
            button(r, look, kind.label(), PanelAction::AddVehicle(kind), BtnStyle::Act);
        }
    });
    for v in snap.vehicles.iter().take(LIST_CAP) {
        let at = snap.site(v.at).map_or("?", |s| s.name.as_str());
        let job = match (v.order, &v.cargo) {
            (Some(o), Some(c)) => format!("#{} {} {}", o.0, v.load, c),
            _ => String::new(),
        };
        label(root, look, format!("veh {} cap {} · {} · near {at} {job}", v.id.0, v.capacity_units, v.phase), look.muted());
    }
    if snap.vehicles.len() > LIST_CAP {
        label(root, look, format!("+{} more", snap.vehicles.len() - LIST_CAP), look.muted());
    }
}

fn build_sites_tab(root: &mut ChildSpawnerCommands<'_>, look: &Look, state: &LogisticsOrdersPanelState, snap: &LogisticsOrdersSnapshot) {
    heading(root, look, "Sites");
    stepper(
        root,
        look,
        "Dev seed",
        format!("{} {}", state.seed(), state.draft.cargo.trim()),
        PanelAction::SeedDelta,
        [10, 100],
    );
    for s in snap.sites.iter().take(LIST_CAP) {
        card(root, look, |c| {
            label(c, look, format!("{} · {} · faction {} · tile {},{}", s.name, s.role.label(), s.owner.0, s.tile[0], s.tile[1]), look.text_color());
            wrap_row(c, |r| {
                button(r, look, "+ stock", PanelAction::AddStock(s.id), BtnStyle::Act);
                button(r, look, "from", PanelAction::Source(s.id), BtnStyle::Pick(state.draft.source == Some(s.id)));
                button(r, look, "to", PanelAction::Sink(s.id), BtnStyle::Pick(state.draft.sink == Some(s.id)));
            });
            for r in &s.stock {
                label(
                    c,
                    look,
                    format!("  {}: {} {} (reserved {}, inbound {})", r.cargo, r.on_hand, r.unit, r.reserved_out, r.inbound),
                    look.muted(),
                );
            }
        });
    }
    if snap.sites.len() > LIST_CAP {
        label(root, look, format!("+{} more sites", snap.sites.len() - LIST_CAP), look.muted());
    }
}

fn build_footer(root: &mut ChildSpawnerCommands<'_>, look: &Look, state: &LogisticsOrdersPanelState, snap: &LogisticsOrdersSnapshot) {
    button(
        root,
        look,
        "keep routes on map when closed",
        PanelAction::ToggleAlwaysRoutes,
        BtnStyle::Pick(state.always_show_routes),
    );
    if !snap.events.is_empty() {
        heading(root, look, "Recent");
        for e in snap.events.iter().rev().take(5) {
            label(root, look, format!("t{} {}", e.tick, e.text), look.muted());
        }
    }
}

fn spawn_orders_panel(
    commands: &mut Commands,
    state: &LogisticsOrdersPanelState,
    snap: &LogisticsOrdersSnapshot,
    palette: &UiPalette,
    font: &Handle<Font>,
    scroll: Vec2,
) {
    let look = Look {
        tf: TextFont::from_font_size(11.5).with_font(font.clone()),
        p: palette,
    };
    commands
        .spawn((
            LogisticsOrdersBevyRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(64.0),
                left: Val::Px(380.0),
                width: Val::Px(470.0),
                max_height: Val::Vh(86.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollPosition(scroll),
            RelativeCursorPosition::default(),
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
            ZIndex(955),
        ))
        .with_children(|root| {
            build_header(root, &look, state, snap);
            build_tabs(root, &look, state, snap);
            match state.tab {
                OrdersTab::New => build_new_order(root, &look, state, snap),
                OrdersTab::Runs => build_orders_tab(root, &look, state, snap, false),
                OrdersTab::Lines => build_orders_tab(root, &look, state, snap, true),
                OrdersTab::Demands => build_demands_tab(root, &look, state, snap),
                OrdersTab::Fleet => build_fleet_tab(root, &look, snap),
                OrdersTab::Sites => build_sites_tab(root, &look, state, snap),
            }
            build_footer(root, &look, state, snap);
        });
}

#[allow(clippy::too_many_arguments)]
fn sync_logistics_orders_bevy_panel(
    mut commands: Commands,
    state: Res<LogisticsOrdersPanelState>,
    snap: Option<Res<LogisticsOrdersSnapshotRes>>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<(Entity, &ScrollPosition), With<LogisticsOrdersBevyRoot>>,
    mut stamp: Local<OrdersPanelStamp>,
) {
    presence.logistics_orders_root = !roots.is_empty();
    let Some(snap) = snap else {
        for (e, _) in &roots {
            commands.entity(e).despawn();
        }
        presence.logistics_orders_root = false;
        stamp.key = 0;
        return;
    };

    if !state.visible {
        for (e, _) in &roots {
            commands.entity(e).despawn();
        }
        presence.logistics_orders_root = false;
        stamp.key = 0;
        return;
    }

    let next = panel_stamp(&snap.0, state.as_ref());
    if !roots.is_empty() && stamp.key == next {
        return;
    }
    // Keep the scroll offset across rebuilds unless the player changed tab.
    if let Some((_, pos)) = roots.iter().next() {
        stamp.scroll = pos.0;
    }
    if stamp.tab != Some(state.tab) {
        stamp.scroll = Vec2::ZERO;
        stamp.tab = Some(state.tab);
    }
    for (e, _) in &roots {
        commands.entity(e).despawn();
    }
    spawn_orders_panel(&mut commands, state.as_ref(), &snap.0, &palette, &mono.0, stamp.scroll);
    stamp.key = next;
    presence.logistics_orders_root = true;
}

fn logistics_orders_panel_scroll(
    mut wheel: MessageReader<MouseWheel>,
    mut roots: Query<(&RelativeCursorPosition, &mut ScrollPosition), With<LogisticsOrdersBevyRoot>>,
) {
    let mut dy = 0.0;
    for ev in wheel.read() {
        dy += match ev.unit {
            MouseScrollUnit::Line => ev.y * 28.0,
            MouseScrollUnit::Pixel => ev.y,
        };
    }
    if dy == 0.0 {
        return;
    }
    for (cursor, mut pos) in &mut roots {
        if cursor.cursor_over() {
            pos.0.y = (pos.0.y - dy).max(0.0);
        }
    }
}

fn sync_auto_supply_mirror(
    policy: Res<crate::economy::logistics::orders::AutoSupplyPolicy>,
    mut state: ResMut<LogisticsOrdersPanelState>,
) {
    let on = policy.enabled(state.draft.owner);
    if state.auto_supply != on {
        state.auto_supply = on;
    }
}

fn logistics_orders_panel_clicks(
    mut state: ResMut<LogisticsOrdersPanelState>,
    mut policy: ResMut<crate::economy::logistics::orders::AutoSupplyPolicy>,
    snap: Option<Res<LogisticsOrdersSnapshotRes>>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
    buttons: Query<(&Interaction, &LogisticsOrdersAction), Changed<Interaction>>,
) {
    let empty = LogisticsOrdersSnapshot::default();
    let snap = snap.as_deref().map_or(&empty, |s| &s.0);
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if action.0 == PanelAction::ToggleAutoSupply {
            let owner = state.draft.owner;
            let on = !policy.enabled(owner);
            policy.set(owner, on);
            state.ok(if on { "Auto-supply on" } else { "Auto-supply off" });
            continue;
        }
        for cmd in apply_panel_action(&mut state, snap, action.0.clone()) {
            requests.write(LogisticsOrderRequest(cmd));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::{SiteRow, StockRow};

    fn site(id: u32, role: SiteRole) -> SiteRow {
        SiteRow {
            id: SiteId(id),
            name: format!("S{id}"),
            owner: OwnerId::PLAYER,
            role,
            world_xz: [0.0, 0.0],
            tile: [0, 0],
            stock: vec![StockRow {
                cargo: "ammo".into(),
                unit: "t",
                on_hand: 10,
                reserved_out: 0,
                inbound: 0,
            }],
        }
    }

    fn snap_with_sites() -> LogisticsOrdersSnapshot {
        LogisticsOrdersSnapshot {
            sites: vec![site(1, SiteRole::Depot), site(2, SiteRole::Front)],
            ..Default::default()
        }
    }

    #[test]
    fn submit_without_picks_reports_error_then_picked_form_emits_run_and_line() {
        let snap = snap_with_sites();
        let mut st = LogisticsOrdersPanelState::default();
        assert!(apply_panel_action(&mut st, &snap, PanelAction::Submit).is_empty());
        assert_eq!(st.last_error.as_deref(), Some("pick a source"));

        apply_panel_action(&mut st, &snap, PanelAction::Kind(DraftKind::Run));
        apply_panel_action(&mut st, &snap, PanelAction::Source(SiteId(1)));
        apply_panel_action(&mut st, &snap, PanelAction::Sink(SiteId(2)));
        apply_panel_action(&mut st, &snap, PanelAction::Quantity(10));
        let cmds = apply_panel_action(&mut st, &snap, PanelAction::Submit);
        assert_eq!(
            cmds,
            vec![LogisticsOrderCommand::CreateOneOffRun {
                owner: OwnerId::PLAYER,
                source: SiteId(1),
                sink: SiteId(2),
                cargo: CargoKind::new("ammo"),
                quantity: 50,
                priority: Priority::Normal,
                vehicle_cap: None,
            }]
        );
        assert!(st.last_error.is_none());

        apply_panel_action(&mut st, &snap, PanelAction::Kind(DraftKind::Line));
        let cmds = apply_panel_action(&mut st, &snap, PanelAction::Submit);
        assert!(matches!(
            cmds[..],
            [
                LogisticsOrderCommand::PublishDemand { .. },
                LogisticsOrderCommand::CreateStandingLine { source: SiteId(1), sink: SiteId(2), .. }
            ]
        ));
        assert_eq!(
            apply_panel_action(&mut st, &snap, PanelAction::Pause(OrderId(7))),
            vec![LogisticsOrderCommand::Pause { order: OrderId(7) }]
        );
    }
}
