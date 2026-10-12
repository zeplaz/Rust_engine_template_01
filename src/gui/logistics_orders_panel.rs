//! Logistics orders panel — Bevy UI for standing lines / one-off runs
//! (PLAN-LOGISTICS-ORDERS-v1 §6). Presentation only: reads [`LogisticsOrdersSnapshotRes`],
//! writes [`LogisticsOrderRequest`]. Toggle: **Shift + logistics-targets key** (default Shift+F6).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::economy::logistics::orders::{
    best_source_for, CargoKind, ConsumerKind, DemandHealth, DemandRow, LogisticsOrderCommand,
    LogisticsOrderRequest, LogisticsOrdersSnapshot, LogisticsOrdersSnapshotRes, OrderRow,
    OrderStatus, OwnerId, Priority, SiteId, SiteRole,
};
use crate::gui::input_bindings::InputBindings;
use crate::gui::panel_presence::PanelPresence;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
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

#[derive(Component)]
pub struct LogisticsOrdersBevyRoot;

#[derive(Component)]
struct LogisticsOrdersBodyText;

#[derive(Component)]
struct LogisticsOrdersSubmitButton;

#[derive(Component)]
struct LogisticsOrdersTabButton(OrdersTab);

#[derive(Default)]
struct OrdersPanelStamp {
    key: u64,
}

pub struct LogisticsOrdersPanelPlugin;

impl Plugin for LogisticsOrdersPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LogisticsOrdersPanelState>()
            .add_systems(
                Update,
                (
                    toggle_logistics_orders_panel,
                    sync_logistics_orders_bevy_panel,
                    logistics_orders_panel_clicks,
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

fn orders_panel_text(state: &LogisticsOrdersPanelState, snap: &LogisticsOrdersSnapshot) -> String {
    let mut lines = vec![
        format!(
            "Logistics orders  attention={}  tab={:?}",
            attention_count(snap),
            state.tab
        ),
        format!(
            "Draft  {:?}  {} → {}  cargo={}  qty={}",
            state.draft.kind,
            site_label(snap, state.draft.source),
            site_label(snap, state.draft.sink),
            state.draft.cargo,
            state.draft.quantity
        ),
    ];
    if let Some(err) = &state.last_error {
        lines.push(format!("Error: {err}"));
    }
    match state.tab {
        OrdersTab::Runs => {
            lines.push("── One-off runs ──".into());
            for o in snap.orders.iter().filter(|o| !o.standing) {
                if !state.show_finished && o.status.is_terminal() {
                    continue;
                }
                lines.push(format!(
                    "#{} {}→{} {} [{}] {}",
                    o.id.0,
                    o.source_name,
                    o.sink_name,
                    o.cargo,
                    o.status_label,
                    run_progress_text(o)
                ));
            }
        }
        OrdersTab::Lines => {
            lines.push("── Standing lines ──".into());
            for o in snap.orders.iter().filter(|o| o.standing) {
                if !state.show_finished && o.status.is_terminal() {
                    continue;
                }
                lines.push(format!(
                    "#{} {}→{} {} [{}] {}",
                    o.id.0,
                    o.source_name,
                    o.sink_name,
                    o.cargo,
                    o.status_label,
                    line_progress_text(o)
                ));
            }
        }
        OrdersTab::Demands => {
            lines.push("── Demands ──".into());
            for d in &snap.demands {
                lines.push(format!(
                    "{} @ site {}  on_hand={} inbound={} buffer={} [{:?}]",
                    d.cargo, d.site.0, d.on_hand, d.inbound, d.buffer_target, d.health
                ));
            }
        }
        OrdersTab::Fleet => {
            lines.push("── Fleet ──".into());
            for v in &snap.vehicles {
                lines.push(format!(
                    "veh {}  {:?}  order={:?}  {}→{}  {:.0}%",
                    v.id.0,
                    v.phase,
                    v.order.map(|o| o.0),
                    v.from.0,
                    v.to.0,
                    v.fraction * 100.0
                ));
            }
        }
        OrdersTab::Sites => {
            lines.push("── Sites ──".into());
            for s in &snap.sites {
                lines.push(format!(
                    "site {} {} ({}) stocks={}",
                    s.id.0,
                    s.name,
                    s.role.label(),
                    s.stock.len()
                ));
            }
        }
    }
    lines.join("\n")
}

fn snap_stamp(snap: &LogisticsOrdersSnapshot, state: &LogisticsOrdersPanelState) -> u64 {
    let mut key = snap.orders.len() as u64
        + snap.demands.len() as u64 * 17
        + snap.vehicles.len() as u64 * 31
        + snap.sites.len() as u64 * 61;
    let tab_i = match state.tab {
        OrdersTab::Runs => 0u64,
        OrdersTab::Lines => 1,
        OrdersTab::Demands => 2,
        OrdersTab::Fleet => 3,
        OrdersTab::Sites => 4,
    };
    key = key
        .wrapping_add(tab_i)
        .wrapping_add(state.visible as u64)
        .wrapping_add(state.draft.quantity as u64);
    for o in snap.orders.iter().take(32) {
        key = key
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(o.id.0 as u64)
            .wrapping_add(o.delivered as u64);
    }
    key
}

fn sync_logistics_orders_bevy_panel(
    mut commands: Commands,
    state: Res<LogisticsOrdersPanelState>,
    snap: Option<Res<LogisticsOrdersSnapshotRes>>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<LogisticsOrdersBevyRoot>>,
    mut stamp: Local<OrdersPanelStamp>,
) {
    presence.logistics_orders_root = !roots.is_empty();
    let Some(snap) = snap else {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.logistics_orders_root = false;
        return;
    };

    if !state.visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.logistics_orders_root = false;
        stamp.key = 0;
        return;
    }

    let next = snap_stamp(&snap.0, state.as_ref());
    if !roots.is_empty() && stamp.key == next {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }

    let text = orders_panel_text(state.as_ref(), &snap.0);
    let tf = TextFont::from_font_size(11.5).with_font(mono.0.clone());
    commands
        .spawn((
            LogisticsOrdersBevyRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(64.0),
                left: Val::Px(380.0),
                width: Val::Px(420.0),
                max_height: Val::Px(520.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
            ZIndex(955),
        ))
        .with_children(|root| {
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|tabs| {
                for tab in [
                    OrdersTab::Runs,
                    OrdersTab::Lines,
                    OrdersTab::Demands,
                    OrdersTab::Fleet,
                    OrdersTab::Sites,
                ] {
                    let label = format!("{tab:?}");
                    tabs.spawn((
                        LogisticsOrdersTabButton(tab),
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(if state.tab == tab {
                            palette.bevy_button_idle()
                        } else {
                            palette.bevy_hud_panel_fill()
                        }),
                        BorderColor::all(palette.bevy_wire_magenta()),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(label),
                            tf.clone(),
                            TextColor(palette.bevy_primary_text()),
                        ));
                    });
                }
            });
            root.spawn((
                LogisticsOrdersBodyText,
                Text::new(text),
                tf.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
            root.spawn((
                LogisticsOrdersSubmitButton,
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(palette.bevy_button_idle()),
                BorderColor::all(palette.bevy_wire_magenta()),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Submit draft (draft_commands)"),
                    tf,
                    TextColor(palette.bevy_primary_text()),
                ));
            });
        });
    stamp.key = next;
    presence.logistics_orders_root = true;
}

fn logistics_orders_panel_clicks(
    mut state: ResMut<LogisticsOrdersPanelState>,
    snap: Option<Res<LogisticsOrdersSnapshotRes>>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
    tabs: Query<(&Interaction, &LogisticsOrdersTabButton), Changed<Interaction>>,
    submit: Query<&Interaction, (Changed<Interaction>, With<LogisticsOrdersSubmitButton>)>,
) {
    for (interaction, tab) in &tabs {
        if *interaction == Interaction::Pressed {
            state.tab = tab.0;
        }
    }
    for interaction in &submit {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(snap) = snap.as_ref() else {
            state.last_error = Some("snapshot unavailable".into());
            continue;
        };
        match draft_commands(&state.draft, &snap.0) {
            Ok(cmds) => {
                for cmd in cmds {
                    requests.write(LogisticsOrderRequest(cmd));
                }
                state.last_error = None;
            }
            Err(e) => state.last_error = Some(e.into()),
        }
    }
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
