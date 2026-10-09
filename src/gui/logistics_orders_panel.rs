//! Logistics orders panel — issue and monitor standing supply lines (⟳) and one-off supply
//! runs (➜), review draw demands and the vehicle fleet (PLAN-LOGISTICS-ORDERS-v1 §6).
//!
//! Presentation only: reads [`LogisticsOrdersSnapshotRes`], writes [`LogisticsOrderRequest`].
//! Toggle: **Shift + logistics-targets key** (default Shift+F6) or the button in the targets panel.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::economy::logistics::orders::{
    suggest_lines_for_unserved, AdvisorGap, CargoKind, ConsumerKind, DemandHealth, DemandRow, LogisticsOrderCommand,
    LogisticsOrderRequest, LogisticsOrdersSnapshot, LogisticsOrdersSnapshotRes,
    OrderRow, OrderStatus, OwnerId, Priority, SiteId, SiteRole, CargoClassMask,
};
use crate::gui::input_bindings::InputBindings;
use crate::gui::style::{muted_label, primary_label, section_heading, CmdHeadingStyle, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;

/// Cargo offered in the picker besides whatever the source currently stocks.
pub const COMMON_CARGO: &[&str] = &["ammo", "fuel", "personnel", "Cement", "Steel", "food"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OrdersTab {
    #[default]
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
    pub last_error: Option<String>,
    pub show_finished: bool,
    /// Keep order routes on the map while the panel is closed.
    pub always_show_routes: bool,
    /// Dev seeding (Sites tab): amount added per click.
    pub seed_amount: u32,
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
///
/// A line to a destination without a draw demand for that cargo also publishes one from the
/// draft's buffer fields, so a new line is never stuck on "no demand at sink".
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

/// `m:ss` from order-sim ticks (1 tick = 1 simulated second).
#[must_use]
pub fn format_eta(ticks: u32) -> String {
    format!("{}:{:02}", ticks / 60, ticks % 60)
}

/// Headline for a one-off run row, e.g. `24/40 t · trip 3/5`.
///
/// The current trip counts the one under way, if any trucks are out.
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

fn status_color(status: OrderStatus, p: &UiPalette) -> egui::Color32 {
    match status {
        OrderStatus::Waiting(_) | OrderStatus::Paused | OrderStatus::Cancelling => p.warn,
        OrderStatus::Failed(_) | OrderStatus::Cancelled => p.danger,
        OrderStatus::Completed | OrderStatus::Idle => p.fg_muted,
        OrderStatus::Pending | OrderStatus::Active => p.fg_data,
    }
}

fn health_color(h: DemandHealth, p: &UiPalette) -> egui::Color32 {
    match h {
        DemandHealth::Satisfied => p.fg_data,
        DemandHealth::Low => p.warn,
        DemandHealth::Starving => p.danger,
    }
}

pub struct LogisticsOrdersPanelPlugin;

impl Plugin for LogisticsOrdersPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LogisticsOrdersPanelState>()
            .add_systems(
                Update,
                toggle_logistics_orders_panel.run_if(in_simulation_or_editor),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    super::logistics_orders_overlay::draw_logistics_orders_overlay_egui,
                    logistics_orders_panel_ui,
                )
                    .chain()
                    .run_if(crate::gui::ui_gates::product_egui_shell_active),
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

fn site_combo(
    ui: &mut egui::Ui,
    salt: &str,
    snap: &LogisticsOrdersSnapshot,
    owner: OwnerId,
    value: &mut Option<SiteId>,
) {
    egui::ComboBox::from_id_salt(salt)
        .selected_text(site_label(snap, *value))
        .width(200.0)
        .show_ui(ui, |ui| {
            for s in snap.sites.iter().filter(|s| s.owner == owner) {
                let stock: u32 = s.stock.iter().map(|r| r.on_hand).sum();
                ui.selectable_value(
                    value,
                    Some(s.id),
                    format!("{} ({}) · {stock} on hand", s.name, s.role.label()),
                );
            }
        });
}

fn priority_combo(ui: &mut egui::Ui, salt: String, value: &mut Priority) -> bool {
    let before = *value;
    egui::ComboBox::from_id_salt(salt)
        .selected_text(value.label())
        .width(80.0)
        .show_ui(ui, |ui| {
            for p in Priority::ALL {
                ui.selectable_value(value, p, p.label());
            }
        });
    *value != before
}

fn new_order_form(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    state: &mut LogisticsOrdersPanelState,
    snap: &LogisticsOrdersSnapshot,
    out: &mut Vec<LogisticsOrderCommand>,
) {
    let draft = &mut state.draft;
    ui.horizontal(|ui| {
        ui.selectable_value(&mut draft.kind, DraftKind::Line, "⟳ Supply line");
        ui.selectable_value(&mut draft.kind, DraftKind::Run, "➜ Supply run");
    });
    muted_label(
        ui,
        palette,
        match draft.kind {
            DraftKind::Line => {
                "Continuous: trucks keep the destination topped up whenever its stock falls below the reorder point."
            }
            DraftKind::Run => {
                "One-off: deliver a fixed amount using free trucks, repeating trips until done, then release them."
            }
        },
    );
    egui::Grid::new("logistics_order_form")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            ui.label("From");
            site_combo(ui, "lo_src", snap, draft.owner, &mut draft.source);
            ui.end_row();
            ui.label("To");
            site_combo(ui, "lo_dst", snap, draft.owner, &mut draft.sink);
            ui.end_row();
            ui.label("Cargo");
            ui.horizontal(|ui| {
                let mut offered: Vec<String> = draft
                    .source
                    .and_then(|s| snap.site(s))
                    .map(|s| s.stock.iter().map(|r| r.cargo.clone()).collect())
                    .unwrap_or_default();
                for c in COMMON_CARGO {
                    if !offered.iter().any(|o| o == c) {
                        offered.push((*c).to_string());
                    }
                }
                egui::ComboBox::from_id_salt("lo_cargo")
                    .selected_text(draft.cargo.clone())
                    .width(110.0)
                    .show_ui(ui, |ui| {
                        for c in offered {
                            ui.selectable_value(&mut draft.cargo, c.clone(), c);
                        }
                    });
                ui.add(egui::TextEdit::singleline(&mut draft.cargo).desired_width(80.0));
            });
            ui.end_row();
            let unit = CargoKind::new(draft.cargo.clone()).unit_label();
            match draft.kind {
                DraftKind::Run => {
                    ui.label("Quantity");
                    ui.add(egui::DragValue::new(&mut draft.quantity).range(1..=100_000).suffix(format!(" {unit}")));
                    ui.end_row();
                }
                DraftKind::Line => {
                    let existing = draft.sink.and_then(|s| {
                        snap.demands
                            .iter()
                            .find(|d| d.site == s && d.cargo == draft.cargo.trim())
                    });
                    if let Some(d) = existing {
                        ui.label("Demand");
                        ui.label(format!(
                            "keep {} {unit}, reorder below {} (published by {})",
                            d.buffer_target,
                            d.reorder_point,
                            d.consumer.label()
                        ));
                        ui.end_row();
                    } else {
                        ui.label("Keep on hand");
                        ui.add(egui::DragValue::new(&mut draft.buffer_target).range(1..=100_000).suffix(format!(" {unit}")));
                        ui.end_row();
                        ui.label("Reorder below");
                        ui.add(egui::DragValue::new(&mut draft.reorder_point).range(0..=draft.buffer_target).suffix(format!(" {unit}")));
                        ui.end_row();
                        ui.label("Consumption");
                        ui.add(egui::DragValue::new(&mut draft.draw_rate_per_min).range(0..=10_000).suffix(format!(" {unit}/min")));
                        ui.end_row();
                    }
                }
            }
            ui.label("Priority");
            priority_combo(ui, "lo_prio".into(), &mut draft.priority);
            ui.end_row();
            ui.label("Vehicles");
            ui.horizontal(|ui| {
                ui.checkbox(&mut draft.cap_vehicles, "cap at");
                ui.add_enabled(
                    draft.cap_vehicles,
                    egui::DragValue::new(&mut draft.vehicle_cap).range(1..=64),
                );
                let fleet = snap.fleet(draft.owner);
                ui.label(format!("· {} free / {} total", fleet.free, fleet.total));
            });
            ui.end_row();
        });
    let label = match draft.kind {
        DraftKind::Line => "Start supply line",
        DraftKind::Run => "Dispatch supply run",
    };
    if ui.button(label).clicked() {
        match draft_commands(draft, snap) {
            Ok(cmds) => {
                out.extend(cmds);
                state.last_error = None;
                state.tab = match state.draft.kind {
                    DraftKind::Line => OrdersTab::Lines,
                    DraftKind::Run => OrdersTab::Runs,
                };
            }
            Err(e) => state.last_error = Some(e.to_string()),
        }
    }
    if let Some(e) = &state.last_error {
        crate::gui::style::warning_text(ui, palette, e);
    }
}

fn order_controls(ui: &mut egui::Ui, row: &OrderRow, out: &mut Vec<LogisticsOrderCommand>) {
    if row.status.is_terminal() {
        return;
    }
    let mut prio = row.priority;
    if priority_combo(ui, format!("lo_row_prio_{}", row.id.0), &mut prio) {
        out.push(LogisticsOrderCommand::SetPriority {
            order: row.id,
            priority: prio,
        });
    }
    if row.status == OrderStatus::Paused {
        if ui.small_button("Resume").clicked() {
            out.push(LogisticsOrderCommand::Resume { order: row.id });
        }
    } else if row.status != OrderStatus::Cancelling && ui.small_button("Pause").clicked() {
        out.push(LogisticsOrderCommand::Pause { order: row.id });
    }
    if row.status != OrderStatus::Cancelling && ui.small_button("Cancel").clicked() {
        out.push(LogisticsOrderCommand::Cancel { order: row.id });
    }
}

fn order_row_ui(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    row: &OrderRow,
    out: &mut Vec<LogisticsOrderCommand>,
) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            let glyph = if row.standing { "⟳" } else { "➜" };
            ui.label(egui::RichText::new(format!("{glyph} #{}", row.id.0)).strong());
            ui.label(format!(
                "{} · {} → {}",
                row.cargo, row.source_name, row.sink_name
            ));
        });
        let text = if row.standing {
            line_progress_text(row)
        } else {
            run_progress_text(row)
        };
        ui.add(egui::ProgressBar::new(row.progress()).text(text));
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&row.status_label).color(status_color(row.status, palette)),
            );
            ui.label(format!("⛟×{}", row.vehicles));
            if let Some(cap) = row.vehicle_cap {
                ui.label(format!("(cap {cap})"));
            }
            if let Some(eta) = row.next_eta_ticks {
                ui.label(format!("next drop {}", format_eta(eta)));
            }
            if row.lost > 0 {
                ui.label(
                    egui::RichText::new(format!("lost {} {}", row.lost, row.unit))
                        .color(palette.danger),
                );
            }
        });
        ui.horizontal(|ui| order_controls(ui, row, out));
    });
}

fn demand_row_ui(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    d: &DemandRow,
    state: &mut LogisticsOrdersPanelState,
) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("●").color(health_color(d.health, palette)));
            ui.label(egui::RichText::new(&d.site_name).strong());
            ui.label(format!(
                "{} · {} · {}",
                d.cargo,
                d.consumer.label(),
                d.priority.label()
            ));
        });
        ui.add(egui::ProgressBar::new(d.fill_ratio()).text(format!(
            "{}/{} {} (+{} inbound) · reorder < {}",
            d.on_hand, d.buffer_target, d.unit, d.inbound, d.reorder_point
        )));
        ui.horizontal(|ui| {
            ui.label(format!("draw {} {}/min", d.draw_rate_per_min, d.unit));
            if let Some(t) = d.time_to_empty_ticks {
                let color = if t < 120 { palette.warn } else { palette.fg_muted };
                ui.label(egui::RichText::new(format!("empty in {}", format_eta(t))).color(color));
            }
            if let (Some(n), Some(rt)) = (d.trucks_needed, d.round_trip_ticks) {
                ui.label(format!("needs ⛟×{n} (round trip {})", format_eta(rt)));
            }
            if d.shortfall_total > 0 {
                ui.label(
                    egui::RichText::new(format!("short {} {}", d.shortfall_total, d.unit))
                        .color(palette.danger),
                );
            }
            if d.served_by == 0 {
                ui.label(egui::RichText::new("no supply line").color(palette.warn));
                if ui.small_button("+ Create line").clicked() {
                    state.draft.kind = DraftKind::Line;
                    state.draft.sink = Some(d.site);
                    state.draft.cargo = d.cargo.clone();
                    state.draft.priority = d.priority;
                    state.draft.owner = d.owner;
                }
            } else {
                ui.label(format!("served by {} line(s)", d.served_by));
            }
        });
    });
}

#[allow(clippy::too_many_arguments)]
fn logistics_orders_panel_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<LogisticsOrdersPanelState>,
    snapshot: Res<LogisticsOrdersSnapshotRes>,
    bindings: Res<InputBindings>,
    palette: Res<UiPalette>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) -> Result {
    if !state.visible {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let snap = &snapshot.0;
    let palette = palette.as_ref();
    let mut out: Vec<LogisticsOrderCommand> = Vec::new();
    let mut open = true;
    let state = &mut *state;

    crate::gui::std_floating(egui::Window::new(format!(
        "Logistics orders (Shift+{})",
        InputBindings::format_key(bindings.toggle_logistics_targets_panel)
    )))
    .open(&mut open)
    .default_size(egui::vec2(420.0, 560.0))
    .show(ctx, |ui| {
        egui::CollapsingHeader::new("New order")
            .default_open(true)
            .show(ui, |ui| new_order_form(ui, palette, state, snap, &mut out));
        ui.separator();
        let lines = snap.standing_lines().filter(|o| !o.status.is_terminal()).count();
        let runs = snap.one_off_runs().filter(|o| !o.status.is_terminal()).count();
        let starving = snap
            .demands
            .iter()
            .filter(|d| d.health == DemandHealth::Starving)
            .count();
        ui.horizontal(|ui| {
            ui.selectable_value(&mut state.tab, OrdersTab::Runs, format!("➜ Runs ({runs})"));
            ui.selectable_value(&mut state.tab, OrdersTab::Lines, format!("⟳ Lines ({lines})"));
            let demands_label = if starving > 0 {
                format!("Demands ({starving}!)")
            } else {
                format!("Demands ({})", snap.demands.len())
            };
            ui.selectable_value(&mut state.tab, OrdersTab::Demands, demands_label);
            ui.selectable_value(&mut state.tab, OrdersTab::Fleet, "Fleet");
            ui.selectable_value(&mut state.tab, OrdersTab::Sites, "Sites");
        });
        egui::ScrollArea::vertical()
            .id_salt("logistics_orders_scroll")
            .max_height(320.0)
            .show(ui, |ui| match state.tab {
                OrdersTab::Runs => {
                    ui.checkbox(&mut state.show_finished, "show finished");
                    let mut any = false;
                    for row in snap
                        .one_off_runs()
                        .filter(|o| state.show_finished || !o.status.is_terminal())
                    {
                        any = true;
                        order_row_ui(ui, palette, row, &mut out);
                    }
                    if !any {
                        muted_label(ui, palette, "No supply runs. Use ➜ Supply run above.");
                    }
                }
                OrdersTab::Lines => {
                    let mut any = false;
                    for row in snap.standing_lines().filter(|o| !o.status.is_terminal()) {
                        any = true;
                        order_row_ui(ui, palette, row, &mut out);
                    }
                    if !any {
                        muted_label(ui, palette, "No supply lines. Use ⟳ Supply line above.");
                    }
                }
                OrdersTab::Demands => {
                    if snap.demands.is_empty() {
                        muted_label(ui, palette, "No site publishes a draw demand yet.");
                    }
                    let advice = suggest_lines_for_unserved(snap, state.draft.owner);
                    if !advice.commands.is_empty()
                        && ui
                            .button(format!(
                                "Auto-supply: start {} line(s) from nearest stocked depots",
                                advice.commands.len()
                            ))
                            .clicked()
                    {
                        out.extend(advice.commands.iter().cloned());
                    }
                    for gap in &advice.gaps {
                        let AdvisorGap::NoSourceWithStock { site, cargo } = gap;
                        let name = snap.site(*site).map_or("?", |s| s.name.as_str());
                        crate::gui::style::warning_text(
                            ui,
                            palette,
                            format!("{name}: no depot holds {cargo}"),
                        );
                    }
                    for d in &snap.demands {
                        demand_row_ui(ui, palette, d, state);
                    }
                }
                OrdersTab::Fleet => fleet_tab(ui, palette, snap, state, &mut out),
                OrdersTab::Sites => sites_tab(ui, palette, snap, state, &mut out),
            });
        ui.separator();
        ui.checkbox(&mut state.always_show_routes, "keep routes on map when closed");
        section_heading(ui, palette, CmdHeadingStyle::Gt, "Recent");
        for e in snap.events.iter().rev().take(5) {
            muted_label(ui, palette, format!("t{} {}", e.tick, e.text));
        }
        if let Some(r) = snap.rejected.last() {
            crate::gui::style::warning_text(ui, palette, format!("rejected {r}"));
        }
    });
    if !open {
        state.visible = false;
    }
    for cmd in out {
        requests.write(LogisticsOrderRequest(cmd));
    }
    Ok(())
}

fn fleet_tab(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    snap: &LogisticsOrdersSnapshot,
    state: &LogisticsOrdersPanelState,
    out: &mut Vec<LogisticsOrderCommand>,
) {
    for f in &snap.fleets {
        primary_label(
            ui,
            palette,
            format!(
                "Faction {}: {} vehicles · {} free · {} busy · {} units capacity",
                f.owner.0, f.total, f.free, f.busy, f.capacity_total
            ),
        );
    }
    if let Some(home) = snap
        .sites_of(state.draft.owner, SiteRole::Depot)
        .next()
        .map(|s| s.id)
    {
        ui.horizontal(|ui| {
            muted_label(ui, palette, "Dev:");
            for (label, mask, cap, speed) in [
                ("+ truck", CargoClassMask::FREIGHT, 10, 12.0),
                ("+ tanker", CargoClassMask::TANKER, 20, 11.0),
                ("+ troop carrier", CargoClassMask::TROOP, 12, 15.0),
            ] {
                if ui.small_button(label).clicked() {
                    out.push(LogisticsOrderCommand::AddVehicle {
                        owner: state.draft.owner,
                        home,
                        class_mask: mask,
                        capacity_units: cap,
                        speed_mps: speed,
                    });
                }
            }
        });
    }
    ui.separator();
    for v in &snap.vehicles {
        let at = snap.site(v.at).map_or("?", |s| s.name.as_str());
        let job = match (v.order, &v.cargo) {
            (Some(o), Some(c)) => format!("#{} {} {}", o.0, v.load, c),
            _ => String::new(),
        };
        muted_label(
            ui,
            palette,
            format!(
                "⛟{} cap {} · {} · near {} {}",
                v.id.0, v.capacity_units, v.phase, at, job
            ),
        );
    }
}

fn sites_tab(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    snap: &LogisticsOrdersSnapshot,
    state: &mut LogisticsOrdersPanelState,
    out: &mut Vec<LogisticsOrderCommand>,
) {
    if state.seed_amount == 0 {
        state.seed_amount = 100;
    }
    ui.horizontal(|ui| {
        muted_label(ui, palette, "Dev seed:");
        ui.add(egui::DragValue::new(&mut state.seed_amount).range(1..=100_000));
        muted_label(ui, palette, format!("of {}", state.draft.cargo));
    });
    for s in &snap.sites {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&s.name).strong());
                ui.label(format!(
                    "{} · faction {} · tile {},{}",
                    s.role.label(),
                    s.owner.0,
                    s.tile[0],
                    s.tile[1]
                ));
                if ui.small_button("+ stock").clicked() {
                    out.push(LogisticsOrderCommand::AddStock {
                        site: s.id,
                        cargo: CargoKind::new(state.draft.cargo.trim()),
                        amount: state.seed_amount,
                    });
                }
                if ui.small_button("from").clicked() {
                    state.draft.source = Some(s.id);
                }
                if ui.small_button("to").clicked() {
                    state.draft.sink = Some(s.id);
                }
            });
            for r in &s.stock {
                muted_label(
                    ui,
                    palette,
                    format!(
                        "  {}: {} {} (reserved {}, inbound {})",
                        r.cargo, r.on_hand, r.unit, r.reserved_out, r.inbound
                    ),
                );
            }
        });
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::{OrderId, SiteRow, StockRow};

    fn snap_with_sites() -> LogisticsOrdersSnapshot {
        let site = |id: u32, role| SiteRow {
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
        };
        LogisticsOrdersSnapshot {
            sites: vec![site(1, SiteRole::Depot), site(2, SiteRole::Front)],
            ..Default::default()
        }
    }

    #[test]
    fn line_draft_publishes_demand_when_destination_has_none() {
        let snap = snap_with_sites();
        let draft = OrderDraft {
            source: Some(SiteId(1)),
            sink: Some(SiteId(2)),
            ..Default::default()
        };
        let cmds = draft_commands(&draft, &snap).unwrap();
        assert_eq!(cmds.len(), 2);
        assert!(matches!(
            cmds[0],
            LogisticsOrderCommand::PublishDemand {
                consumer: ConsumerKind::Front,
                buffer_target: 40,
                reorder_point: 15,
                ..
            }
        ));
        assert!(matches!(
            cmds[1],
            LogisticsOrderCommand::CreateStandingLine { .. }
        ));
    }

    #[test]
    fn run_draft_validates_and_builds_one_command() {
        let snap = snap_with_sites();
        let mut draft = OrderDraft {
            kind: DraftKind::Run,
            source: Some(SiteId(1)),
            sink: Some(SiteId(1)),
            ..Default::default()
        };
        assert!(draft_commands(&draft, &snap).is_err());
        draft.sink = Some(SiteId(2));
        draft.cap_vehicles = true;
        draft.vehicle_cap = 3;
        let cmds = draft_commands(&draft, &snap).unwrap();
        assert_eq!(
            cmds,
            vec![LogisticsOrderCommand::CreateOneOffRun {
                owner: OwnerId::PLAYER,
                source: SiteId(1),
                sink: SiteId(2),
                cargo: CargoKind::new("ammo"),
                quantity: 40,
                priority: Priority::Normal,
                vehicle_cap: Some(3),
            }]
        );
    }

    #[test]
    fn formats_eta_and_progress() {
        assert_eq!(format_eta(75), "1:15");
        let row = OrderRow {
            id: OrderId(3),
            owner: OwnerId::PLAYER,
            standing: false,
            source: SiteId(1),
            sink: SiteId(2),
            source_name: "A".into(),
            sink_name: "B".into(),
            cargo: "Cement".into(),
            unit: "t",
            priority: Priority::Normal,
            status: OrderStatus::Active,
            status_label: "active".into(),
            quantity: Some(40),
            delivered: 20,
            in_flight: 10,
            remaining: 20,
            trips_done: 2,
            trips_planned: 4,
            vehicles: 1,
            vehicle_cap: None,
            next_eta_ticks: Some(30),
            fill_ratio: None,
            throughput_per_min: 0.0,
            lost: 0,
            created_tick: 0,
            finished_tick: None,
        };
        assert_eq!(run_progress_text(&row), "20/40 t · trip 3/4");
        assert!((row.progress() - 0.5).abs() < 1e-6);
    }
}
