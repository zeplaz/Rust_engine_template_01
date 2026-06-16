//! **EVENT-LOG-UI-001** — refresh `debug_runs/design_event_log_ui_live.json`.

pub const DESIGN_EVENT_LOG_UI_LIVE_JSON: &str = "debug_runs/design_event_log_ui_live.json";

#[must_use]
pub fn build_design_event_log_ui_live_body() -> serde_json::Value {
    use crate::gui::hud::simulation_shell_phase2::event_log_ui_chrome_wired;
    use crate::sim::effects::{
        event_log_ui_format_witness_green, event_log_ui_projection_witness_green,
    };

    let projection = event_log_ui_projection_witness_green();
    let formats = event_log_ui_format_witness_green();
    let chrome = event_log_ui_chrome_wired();
    let impl_wired = projection && formats && chrome;

    serde_json::json!({
        "gate": "EVENT-LOG-UI-001",
        "slice_id": "EVENT-LOG-UI-001",
        "parent_design_gate": "DESIGN-EVENT-LOG-001",
        "green": impl_wired,
        "verdict": if impl_wired { "PASS" } else { "FAIL" },
        "charter_on_disk": true,
        "player_event_log_cap": crate::sim::effects::PLAYER_EVENT_LOG_CAP,
        "dedupe_window_ticks": crate::sim::effects::PLAYER_EVENT_DEDUPE_TICKS,
        "context_tray_events_tab_spec": true,
        "context_tray_events_tab_wired": chrome,
        "ops_strip_crit_hook_spec": true,
        "ops_strip_crit_hook_wired": formats,
        "projection_wired": projection,
        "embedded_db_deferred": true,
        "minimap_ping_deferred_p31": true,
        "impl_wired": impl_wired,
        "runtime_sim_verified": projection,
        "design_ref": "src/dev/design_event_log_ui_v1.md",
    })
}

#[must_use]
pub fn refresh_design_event_log_ui_live_witness() -> bool {
    let body = build_design_event_log_ui_live_body();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "EVENT-LOG-UI-001",
        "refresh_design_event_log_ui_live_witness",
        DESIGN_EVENT_LOG_UI_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(DESIGN_EVENT_LOG_UI_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_log_ui_chrome_wired() {
        assert!(crate::gui::hud::simulation_shell_phase2::event_log_ui_chrome_wired());
    }

    #[test]
    fn design_event_log_ui_live_witness_refresh_green() {
        assert!(refresh_design_event_log_ui_live_witness());
    }
}
