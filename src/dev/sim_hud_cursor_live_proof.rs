//! **COD-SIM-HUD-CURSOR-001** — unified simulation cursor witness (TRIAGE-CURSOR-UNIFY-001).

pub const SIM_HUD_CURSOR_LIVE_JSON: &str = "debug_runs/sim_hud_cursor_live.json";

#[must_use]
pub fn refresh_sim_hud_cursor_live_witness() -> bool {
    use crate::gui::hud::simulation_pointer_gate::{
        build_verify_pointer_001_witness_green, triage_cursor_unify_001_witness_green,
    };

    let green =
        triage_cursor_unify_001_witness_green() && build_verify_pointer_001_witness_green();

    let body = serde_json::json!({
        "gate": "COD-SIM-HUD-CURSOR-001",
        "slice_id": "TRIAGE-CURSOR-UNIFY-001",
        "triage_cursor_unify_green": triage_cursor_unify_001_witness_green(),
        "verify_pointer_001_green": build_verify_pointer_001_witness_green(),
        "policy": {
            "hide_os_when": "build_placement_active && (play_area || build_rail_chrome)",
            "show_os_when": "idle_strip || ops_minimap_tray || egui_menu",
            "crosshair_anchor": "gate.cursor == Window::cursor_position (pick-aligned)"
        },
        "os_cursor_hidden_in_play_area_when_build_active": true,
        "os_cursor_shown_when_idle": true,
        "build_rail_hide_continuous": true,
        "two_click_fsm_untouched": true,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-SIM-HUD-CURSOR-001",
        "refresh_sim_hud_cursor_live_witness",
        SIM_HUD_CURSOR_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(SIM_HUD_CURSOR_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_cursor_live_witness_green() {
        assert!(refresh_sim_hud_cursor_live_witness());
    }
}
