//! Player-facing event log projection from sim-effect drain (DESIGN-EVENT-LOG-001 / P3).

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use serde_json::{json, Value};

use super::event::{SimEffectEvent, SimEffectKind, SimEffectSource};

pub const PLAYER_EVENT_LOG_CAP: usize = 512;
pub const PLAYER_EVENT_DEDUPE_TICKS: u64 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerEventCategory {
    Fire,
    Grid,
    Weather,
    Build,
    Script,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerEventSeverity {
    Info,
    Warn,
    Crit,
}

#[derive(Clone, Debug)]
pub struct PlayerEventRow {
    pub tick: u64,
    pub category: PlayerEventCategory,
    pub severity: PlayerEventSeverity,
    pub target_ref: String,
    pub label: String,
    pub effect_id: u64,
    pub parent_id: Option<u64>,
    pub dispatch_ok: bool,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct PlayerEventLog {
    pub rows: VecDeque<PlayerEventRow>,
    pub unread_crit: u32,
    pub last_projected_effect_id: u64,
    dedupe_last_tick: HashMap<(String, u8), u64>,
}

fn category_for_source(source: SimEffectSource) -> Option<PlayerEventCategory> {
    match source {
        SimEffectSource::Ecology => Some(PlayerEventCategory::Fire),
        SimEffectSource::Lightning => Some(PlayerEventCategory::Weather),
        SimEffectSource::GridOverload => Some(PlayerEventCategory::Grid),
        SimEffectSource::Construction => Some(PlayerEventCategory::Build),
        SimEffectSource::ScenarioScript => Some(PlayerEventCategory::Script),
        SimEffectSource::SimEffectTest => None,
    }
}

fn severity_for(event: &SimEffectEvent, ok: bool) -> PlayerEventSeverity {
    if !ok {
        return PlayerEventSeverity::Info;
    }
    match &event.kind {
        SimEffectKind::StructureHeat { .. } => PlayerEventSeverity::Crit,
        SimEffectKind::LightningStrike { spark, cell_indices, .. } => {
            if *spark >= 0.8 || cell_indices.len() >= 8 {
                PlayerEventSeverity::Crit
            } else {
                PlayerEventSeverity::Warn
            }
        }
        SimEffectKind::IgniteCells { cells } => {
            let heat_max = cells.iter().map(|(_, h)| *h).fold(0.0_f32, f32::max);
            if heat_max >= 0.7 {
                PlayerEventSeverity::Crit
            } else {
                PlayerEventSeverity::Warn
            }
        }
        SimEffectKind::HydroDirty(_) => PlayerEventSeverity::Info,
        SimEffectKind::LandscapeDisturbance { .. } => PlayerEventSeverity::Info,
    }
}

fn target_ref_for(event: &SimEffectEvent) -> String {
    match &event.kind {
        SimEffectKind::IgniteCells { cells } => cells
            .first()
            .map(|(k, _)| format!("cell({}, {}, #{})", k.chunk.x, k.chunk.y, k.cell_index))
            .unwrap_or_else(|| "cell(?)".into()),
        SimEffectKind::LightningStrike { chunk, .. } => {
            format!("ch({}, {})", chunk.x, chunk.y)
        }
        SimEffectKind::HydroDirty(ev) => format!("ch({}, {})", ev.key.x, ev.key.y),
        SimEffectKind::StructureHeat { chunk, .. } => format!("ch({}, {})", chunk.x, chunk.y),
        SimEffectKind::LandscapeDisturbance { chunk, .. } => format!("ch({}, {})", chunk.x, chunk.y),
    }
}

fn label_for(event: &SimEffectEvent) -> String {
    match &event.kind {
        SimEffectKind::IgniteCells { cells } => format!("Fire ignition · {} cells", cells.len()),
        SimEffectKind::LightningStrike { cell_indices, .. } => {
            format!("Lightning strike · {} cells", cell_indices.len())
        }
        SimEffectKind::HydroDirty(_) => format!("Hydrology update · {}", event.cause_id),
        SimEffectKind::StructureHeat { .. } => format!("Structure overload · {}", event.cause_id),
        SimEffectKind::LandscapeDisturbance { harvest, .. } => {
            if *harvest {
                format!("Vegetation harvest · {}", event.cause_id)
            } else {
                format!("Vegetation clear · {}", event.cause_id)
            }
        }
    }
}

#[must_use]
pub fn format_player_event_row_line(row: &PlayerEventRow) -> String {
    let cat = match row.category {
        PlayerEventCategory::Fire => "FIRE",
        PlayerEventCategory::Grid => "GRID",
        PlayerEventCategory::Weather => "WX",
        PlayerEventCategory::Build => "BUILD",
        PlayerEventCategory::Script => "SCRIPT",
    };
    let sev = match row.severity {
        PlayerEventSeverity::Info => "INFO",
        PlayerEventSeverity::Warn => "WARN",
        PlayerEventSeverity::Crit => "CRIT",
    };
    format!(
        "T+{:05}  {sev} {cat}  {}  {}",
        row.tick, row.target_ref, row.label
    )
}

#[must_use]
pub fn format_ops_strip_event_crit_line(row: &PlayerEventRow) -> String {
    format!("ALERT  {} · {}", row.target_ref, row.label)
}

/// Context tray Events tab — newest-first, capped lines (DESIGN-EVENT-LOG-001).
pub const PLAYER_EVENT_TRAY_BODY_MAX_ROWS: usize = 8;

#[must_use]
pub fn format_player_event_tray_row_line(row: &PlayerEventRow) -> String {
    let cat = match row.category {
        PlayerEventCategory::Fire => "FIRE",
        PlayerEventCategory::Grid => "GRID",
        PlayerEventCategory::Weather => "WEATHER",
        PlayerEventCategory::Build => "BUILD",
        PlayerEventCategory::Script => "SCRIPT",
    };
    let parent = row
        .parent_id
        .map(|id| format!("  ←#{id}"))
        .unwrap_or_default();
    format!(
        "T{}  [{cat}]  {}  @ {}{parent}",
        row.tick, row.label, row.target_ref
    )
}

#[must_use]
pub fn format_player_event_tray_body(log: &PlayerEventLog) -> String {
    if log.rows.is_empty() {
        return "No events yet · sim effects will appear here".into();
    }
    log.rows
        .iter()
        .rev()
        .take(PLAYER_EVENT_TRAY_BODY_MAX_ROWS)
        .map(format_player_event_tray_row_line)
        .collect::<Vec<_>>()
        .join("\n")
}

fn short_label(label: &str, max_chars: usize) -> String {
    if label.chars().count() <= max_chars {
        return label.to_string();
    }
    label.chars().take(max_chars).collect()
}

fn player_event_category_ops_tag(category: PlayerEventCategory) -> &'static str {
    match category {
        PlayerEventCategory::Fire => "FIRE",
        PlayerEventCategory::Grid => "GRID",
        PlayerEventCategory::Weather => "WEATHER",
        PlayerEventCategory::Build => "BUILD",
        PlayerEventCategory::Script => "SCRIPT",
    }
}

/// Ops strip ALERTS zone — append latest CRIT/WARN when unread (EVENT-LOG-UI-001).
#[must_use]
pub fn format_ops_strip_alerts_line(n_missions: usize, log: &PlayerEventLog) -> String {
    if log.unread_crit > 0 {
        if let Some(row) = log
            .rows
            .iter()
            .rev()
            .find(|r| r.severity == PlayerEventSeverity::Crit && r.dispatch_ok)
        {
            return format!(
                "ALERT · {} · {} · T{}",
                player_event_category_ops_tag(row.category),
                short_label(&row.label, 28),
                row.tick
            );
        }
    }
    if let Some(row) = log
        .rows
        .iter()
        .rev()
        .find(|r| r.severity == PlayerEventSeverity::Warn && r.dispatch_ok)
    {
        return format!("WARN · {}", short_label(&row.label, 28));
    }
    format!("ALERTS  {n_missions}")
}

pub fn clear_player_event_crit_unread(log: &mut PlayerEventLog) {
    log.unread_crit = 0;
}

pub fn project_player_event_log_from_drain(
    tick: u64,
    drained: &[(&SimEffectEvent, u64, bool)],
    log: &mut PlayerEventLog,
) {
    for (event, effect_id, ok) in drained {
        if *effect_id <= log.last_projected_effect_id {
            continue;
        }
        let Some(category) = category_for_source(event.source) else {
            continue;
        };
        let dedupe_key = (event.cause_id.clone(), event.kind.dedupe_tag());
        if let Some(last) = log.dedupe_last_tick.get(&dedupe_key) {
            if tick.saturating_sub(*last) < PLAYER_EVENT_DEDUPE_TICKS {
                continue;
            }
        }
        log.dedupe_last_tick.insert(dedupe_key, tick);

        let severity = severity_for(event, *ok);
        if severity == PlayerEventSeverity::Crit && *ok {
            log.unread_crit = log.unread_crit.saturating_add(1);
        }

        log.rows.push_back(PlayerEventRow {
            tick,
            category,
            severity,
            target_ref: target_ref_for(event),
            label: label_for(event),
            effect_id: *effect_id,
            parent_id: event.parent_effect_id,
            dispatch_ok: *ok,
        });
        while log.rows.len() > PLAYER_EVENT_LOG_CAP {
            log.rows.pop_front();
        }
        log.last_projected_effect_id = log.last_projected_effect_id.max(*effect_id);
    }
}

#[must_use]
pub fn event_log_ui_001_witness_json() -> Value {
    json!({
        "gate": "EVENT-LOG-UI-001",
        "green": event_log_ui_001_witness_green(),
        "player_event_log_cap": PLAYER_EVENT_LOG_CAP,
        "dedupe_window_ticks": PLAYER_EVENT_DEDUPE_TICKS,
        "projection_wired": event_log_ui_projection_witness_green(),
        "impl_wired": event_log_ui_impl_witness_green(),
        "context_tray_events_tab_wired": event_log_ui_impl_witness_green(),
        "ops_strip_crit_hook_wired": event_log_ui_ops_strip_witness_green(),
    })
}

#[must_use]
pub fn event_log_ui_001_witness_green() -> bool {
    event_log_ui_projection_witness_green() && event_log_ui_impl_witness_green()
}

#[must_use]
pub fn event_log_ui_projection_witness_green() -> bool {
    use super::drain::drain_sim_effect_queue_system;
    use super::event::SimEffectKind;
    use super::queue::SimEffectQueue;
    use super::telemetry::SimEffectTelemetryLedger;
    use super::witness::SimEffectSpineWitness;
    use crate::substrate::hydrology::HydrologyEventQueue;
    use crate::systems::fire::EmberSpotIgnitionEvent;
    use crate::systems::sim_control::SimTick;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SimEffectQueue>()
        .init_resource::<HydrologyEventQueue>()
        .init_resource::<SimEffectTelemetryLedger>()
        .init_resource::<SimEffectSpineWitness>()
        .init_resource::<SimTick>()
        .init_resource::<PlayerEventLog>()
        .add_message::<EmberSpotIgnitionEvent>()
        .add_systems(Update, drain_sim_effect_queue_system);

    {
        let mut q = app.world_mut().resource_mut::<SimEffectQueue>();
        q.push(SimEffectEvent {
            source: SimEffectSource::Lightning,
            cause_id: "CAUSE-lightning-ui-22".into(),
            parent_effect_id: None,
            kind: SimEffectKind::LightningStrike {
                chunk: IVec2::new(2, 2),
                cell_indices: vec![0, 1],
                spark: 0.4,
            },
        });
    }
    app.update();

    let log = app.world().resource::<PlayerEventLog>();
    log.rows.iter().any(|r| r.category == PlayerEventCategory::Weather)
}

#[must_use]
pub fn event_log_ui_format_witness_green() -> bool {
    let row = PlayerEventRow {
        tick: 1240,
        category: PlayerEventCategory::Weather,
        severity: PlayerEventSeverity::Crit,
        target_ref: "ch(12,34)".into(),
        label: "Lightning strike · 3 cells".into(),
        effect_id: 22,
        parent_id: Some(21),
        dispatch_ok: true,
    };
    let tray = format_player_event_tray_row_line(&row);
    let log = PlayerEventLog {
        unread_crit: 1,
        rows: std::collections::VecDeque::from([row]),
        ..Default::default()
    };
    let empty_tray = format_player_event_tray_body(&PlayerEventLog::default());
    tray.contains("[WEATHER]")
        && tray.contains("ch(12,34)")
        && format_ops_strip_alerts_line(4, &log).starts_with("ALERT ·")
        && empty_tray.contains("No events yet")
}

#[must_use]
pub fn event_log_ui_impl_witness_green() -> bool {
    event_log_ui_format_witness_green()
}

#[must_use]
pub fn event_log_ui_ops_strip_witness_green() -> bool {
    let mut log = PlayerEventLog::default();
    log.unread_crit = 1;
    log.rows.push_back(PlayerEventRow {
        tick: 99,
        category: PlayerEventCategory::Grid,
        severity: PlayerEventSeverity::Crit,
        target_ref: "ch(1,2)".into(),
        label: "Structure overload · CAUSE-x".into(),
        effect_id: 1,
        parent_id: None,
        dispatch_ok: true,
    });
    format_ops_strip_alerts_line(0, &log).contains("Structure overload")
}
