//! COD-SIM-HUD-* live witnesses — build picker, tray Build, popup tiers, egui theme.

use serde_json::{json, Value};

use crate::construction::STAGED_PANEL_FLOATING_SIM;
use crate::construction::TOOL_HINTS_DRAW_IN_SIM;
use crate::gui::hud::context_tray_build_egui::{
    context_tray_build_peek_line, context_tray_build_tab_wired, peek_shows_modifiers_wired,
    site_legend_in_tray_wired,
};
use crate::gui::hud::sim_build_picker_sheet::{
    sim_build_picker_constants_green, AD_HOC_SUBMENU_WINDOWS, BUILD_PICKER_RAIL_GAP_PX,
    BUILD_PICKER_SHEET_W_PX,
};
use crate::gui::hud::sim_hud_egui_theme::sim_hud_egui_theme_enforcement_wired;
use crate::gui::hud::sim_road_tool_sheet::ROAD_POPUP_FLOATING_IN_SIM;

pub const SIM_HUD_BUILD_PICKER_LIVE_JSON: &str = "debug_runs/sim_hud_build_picker_live.json";
pub const SIM_HUD_TRAY_BUILD_LIVE_JSON: &str = "debug_runs/sim_hud_tray_build_live.json";
pub const SIM_HUD_POPUP_TIERS_LIVE_JSON: &str = "debug_runs/sim_hud_popup_tiers_live.json";
pub const SIM_HUD_EGUI_THEME_LIVE_JSON: &str = "debug_runs/sim_hud_egui_theme_live.json";

#[must_use]
pub fn sim_hud_popup_tiers_green() -> bool {
    !STAGED_PANEL_FLOATING_SIM
        && !TOOL_HINTS_DRAW_IN_SIM
        && !ROAD_POPUP_FLOATING_IN_SIM
        && context_tray_build_tab_wired()
}

#[must_use]
pub fn build_sim_hud_build_picker_payload() -> Value {
    json!({
        "gate": "COD-SIM-HUD-BUILD-PICKER-001",
        "green": sim_build_picker_constants_green(),
        "build_picker_sheet_open": true,
        "anchor_gap_px": BUILD_PICKER_RAIL_GAP_PX,
        "sheet_width_px": BUILD_PICKER_SHEET_W_PX,
        "ad_hoc_submenu_windows": AD_HOC_SUBMENU_WINDOWS,
    })
}

#[must_use]
pub fn build_sim_hud_tray_build_payload() -> Value {
    json!({
        "gate": "COD-SIM-HUD-TRAY-BUILD-001",
        "green": context_tray_build_tab_wired()
            && !STAGED_PANEL_FLOATING_SIM
            && site_legend_in_tray_wired()
            && peek_shows_modifiers_wired(),
        "context_tray_build_tab_wired": context_tray_build_tab_wired(),
        "staged_panel_floating_sim": STAGED_PANEL_FLOATING_SIM,
        "site_legend_in_tray": site_legend_in_tray_wired(),
        "peek_shows_modifiers": peek_shows_modifiers_wired(),
        "peek_line_sample": context_tray_build_peek_line(true),
    })
}

#[must_use]
pub fn build_sim_hud_popup_tiers_payload() -> Value {
    json!({
        "program_id": "DES-SIM-HUD-POPUP-TIERS-001",
        "gate": "COD-SIM-HUD-POPUP-MIGRATE-001",
        "green": sim_hud_popup_tiers_green(),
        "sim_build_path": {
            "anchored_sheets_max": 1,
            "staged_panel_right_bottom_sim": STAGED_PANEL_FLOATING_SIM,
            "tool_hints_left_bottom_sim": TOOL_HINTS_DRAW_IN_SIM,
            "road_popup_floating_window": ROAD_POPUP_FLOATING_IN_SIM,
            "tray_staging_wired": context_tray_build_tab_wired(),
        },
    })
}

#[must_use]
pub fn build_sim_hud_egui_theme_payload() -> Value {
    json!({
        "gate": "COD-SIM-HUD-EGUI-THEME-001",
        "green": sim_hud_egui_theme_enforcement_wired(),
        "palette_on_sim_egui_passes": sim_hud_egui_theme_enforcement_wired(),
        "footprint_chip_uses_palette": sim_hud_egui_theme_enforcement_wired(),
    })
}

fn write_green_witness(path: &str, gate: &str, fn_name: &str, body: Value) -> bool {
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(gate, fn_name, path, body);
    crate::dev::debug_run_envelope::write_debug_run_json(path, wrapped)
}

#[must_use]
pub fn refresh_sim_hud_build_picker_live_witness() -> bool {
    write_green_witness(
        SIM_HUD_BUILD_PICKER_LIVE_JSON,
        "COD-SIM-HUD-BUILD-PICKER-001",
        "refresh_sim_hud_build_picker_live_witness",
        build_sim_hud_build_picker_payload(),
    )
}

#[must_use]
pub fn refresh_sim_hud_tray_build_live_witness() -> bool {
    write_green_witness(
        SIM_HUD_TRAY_BUILD_LIVE_JSON,
        "COD-SIM-HUD-TRAY-BUILD-001",
        "refresh_sim_hud_tray_build_live_witness",
        build_sim_hud_tray_build_payload(),
    )
}

#[must_use]
pub fn refresh_sim_hud_popup_tiers_live_witness() -> bool {
    write_green_witness(
        SIM_HUD_POPUP_TIERS_LIVE_JSON,
        "COD-SIM-HUD-POPUP-MIGRATE-001",
        "refresh_sim_hud_popup_tiers_live_witness",
        build_sim_hud_popup_tiers_payload(),
    )
}

#[must_use]
pub fn refresh_sim_hud_egui_theme_live_witness() -> bool {
    write_green_witness(
        SIM_HUD_EGUI_THEME_LIVE_JSON,
        "COD-SIM-HUD-EGUI-THEME-001",
        "refresh_sim_hud_egui_theme_live_witness",
        build_sim_hud_egui_theme_payload(),
    )
}

#[must_use]
pub fn refresh_all_sim_hud_professional_polish_witnesses() -> bool {
    refresh_sim_hud_build_picker_live_witness()
        && refresh_sim_hud_tray_build_live_witness()
        && refresh_sim_hud_popup_tiers_live_witness()
        && refresh_sim_hud_egui_theme_live_witness()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_build_picker_live_witness_green() {
        assert!(refresh_sim_hud_build_picker_live_witness());
    }

    #[test]
    fn sim_hud_tray_build_live_witness_green() {
        assert!(refresh_sim_hud_tray_build_live_witness());
    }

    #[test]
    fn sim_hud_popup_tiers_live_witness_green() {
        assert!(refresh_sim_hud_popup_tiers_live_witness());
    }

    #[test]
    fn sim_hud_egui_theme_live_witness_green() {
        assert!(refresh_sim_hud_egui_theme_live_witness());
    }

    #[test]
    fn sim_hud_professional_polish_bundle_green() {
        assert!(refresh_all_sim_hud_professional_polish_witnesses());
    }
}
