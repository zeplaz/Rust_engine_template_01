//! **HUD-NAT-008/009 + HUD-WIT-001** — `debug_runs/production_sim_hud_native_live.json`.

use serde_json::{json, Value};

use crate::dev::runtime_witness::write_enveloped_witness_unchecked;
use crate::gui::hud::hud_side_status_panel::hud_nat_008_side_status_sim_gated;
use crate::gui::hud::plant_focus_card::hud_nat_009_plant_focus_map_clamped;
use crate::gui::hud::power_node_hover_egui::power_node_hover_card_wired;
use crate::gui::hud::sim_hud_egui_theme::hud_nat_009_single_themed_satellite_wired;

pub const PRODUCTION_SIM_HUD_NATIVE_LIVE_JSON: &str =
    "debug_runs/production_sim_hud_native_live.json";

#[must_use]
pub fn production_sim_hud_native_exit_predicates() -> Value {
    let nat008 = hud_nat_008_side_status_sim_gated();
    let nat009_plant = hud_nat_009_plant_focus_map_clamped();
    let nat009_power = power_node_hover_card_wired();
    let nat009_one = hud_nat_009_single_themed_satellite_wired();
    json!({
        "hud_nat_001_bevy_picker": crate::gui::hud::sim_build_picker_bevy::SIM_BUILD_PICKER_USE_BEVY,
        "hud_nat_002_road_power_bevy":
            crate::gui::hud::sim_road_power_sheets_bevy::SIM_ROAD_POWER_SHEETS_USE_BEVY,
        "hud_nat_003_tray_build_bevy":
            crate::gui::hud::context_tray_build_egui::CONTEXT_TRAY_BUILD_USE_BEVY,
        "hud_nat_004_staged_editor_only": !crate::construction::STAGED_PANEL_FLOATING_SIM,
        "hud_nat_005_tool_hints_off_sim": !crate::construction::TOOL_HINTS_DRAW_IN_SIM,
        "hud_nat_007_road_popup_off_sim": !crate::gui::hud::sim_road_tool_sheet::ROAD_POPUP_FLOATING_IN_SIM,
        "hud_nat_008_side_status_sim_gated": nat008,
        "hud_nat_008_no_sim_overflow": nat008,
        "hud_nat_009_plant_map_clamped": nat009_plant,
        "hud_nat_009_power_map_clamped": nat009_power,
        "hud_nat_009_single_themed_satellite": nat009_one,
        "hud_a_003_window_ban":
            crate::dev::hud_a_sim_window_ban_proof::hud_a_003_sim_place_path_window_ban_green(),
    })
}

#[must_use]
pub fn production_sim_hud_native_lib_green() -> bool {
    let p = production_sim_hud_native_exit_predicates();
    p.as_object()
        .map(|o| o.values().all(|v| v.as_bool() == Some(true)))
        .unwrap_or(false)
}

#[must_use]
pub fn build_production_sim_hud_native_witness_body() -> Value {
    let predicates = production_sim_hud_native_exit_predicates();
    let green = production_sim_hud_native_lib_green();
    json!({
        "schema": "production_sim_hud_native_live_v1",
        "slice_id": "HUD-WIT-001",
        "status": if green { "green" } else { "pending" },
        "green": green,
        "updated": "2026-09-24",
        "exit_predicates": predicates,
        "human_gates_open": ["HUD-OPS-001", "HUD-GPLAY-002"],
        "designer_open": ["VSS-T4-005"],
        "notes": "Machine native HUD drain closed through NAT-009; side status retired in sim; plant/power share map-attached chip + map-hole clamp (plant yields to power hover)."
    })
}

#[must_use]
pub fn refresh_production_sim_hud_native_live_witness() -> bool {
    if !production_sim_hud_native_lib_green() {
        return false;
    }
    write_enveloped_witness_unchecked(
        "PRODUCTION-SIM-HUD-NATIVE",
        "coder_hud_nat_008_009",
        PRODUCTION_SIM_HUD_NATIVE_LIVE_JSON,
        build_production_sim_hud_native_witness_body(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_sim_hud_native_live_refresh_green() {
        assert!(production_sim_hud_native_lib_green());
        assert!(refresh_production_sim_hud_native_live_witness());
    }
}
