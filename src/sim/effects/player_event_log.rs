//! Player-facing event log projection from sim-effect drain (DESIGN-EVENT-LOG-001 / P3).

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;

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
    let cat = player_event_category_ops_tag(row.category);
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

/// Ops strip ALERTS zone — tier badge + count (DES-SIM-HUD-OPS-002 / EVENT-LOG-UI-001).
#[must_use]
pub fn ops_strip_alert_tier_counts(log: &PlayerEventLog) -> (usize, usize) {
    let p0 = log.unread_crit as usize;
    let p1 = log
        .rows
        .iter()
        .filter(|r| r.severity == PlayerEventSeverity::Warn && r.dispatch_ok)
        .count();
    (p0, p1)
}

#[must_use]
pub fn format_ops_strip_alert_badge_v2(p0: usize, p1: usize) -> String {
    if p0 > 0 {
        if p0 > 99 {
            "◆99+".to_string()
        } else {
            format!("◆{p0}")
        }
    } else if p1 > 0 {
        if p1 > 99 {
            "▲99+".to_string()
        } else {
            format!("▲{p1}")
        }
    } else {
        "◆0".to_string()
    }
}

#[must_use]
pub fn format_ops_strip_alerts_line(n_missions: usize, log: &PlayerEventLog) -> String {
    let (p0, p1) = ops_strip_alert_tier_counts(log);
    let n = p0 + p1;
    let badge = format_ops_strip_alert_badge_v2(p0, p1);
    if p0 > 0 {
        if let Some(row) = log
            .rows
            .iter()
            .rev()
            .find(|r| r.severity == PlayerEventSeverity::Crit && r.dispatch_ok)
        {
            return format!(
                "{badge}  ALERTS  {n} · {}",
                short_label(&row.label, 24)
            );
        }
    }
    if p1 > 0 && p0 == 0 {
        if let Some(row) = log
            .rows
            .iter()
            .rev()
            .find(|r| r.severity == PlayerEventSeverity::Warn && r.dispatch_ok)
        {
            return format!(
                "{badge}  ALERTS  {n} · {}",
                short_label(&row.label, 24)
            );
        }
    }
    let _ = n_missions;
    format!("{badge}  ALERTS  {n}")
}

pub fn clear_player_event_crit_unread(log: &mut PlayerEventLog) {
    log.unread_crit = 0;
}

/// Push a row from non-drain producers (e.g. nuclear SCRAM). Caps + unread_crit for Crit.
pub fn push_player_event_row(log: &mut PlayerEventLog, row: PlayerEventRow) {
    if row.severity == PlayerEventSeverity::Crit && row.dispatch_ok {
        log.unread_crit = log.unread_crit.saturating_add(1);
    }
    log.rows.push_back(row);
    while log.rows.len() > PLAYER_EVENT_LOG_CAP {
        log.rows.pop_front();
    }
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












