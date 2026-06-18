//! **COD-ART-HUD-ICON-ATLAS-001** live witness — Lane D HUD power icons.

use crate::gui::hud::power_hud_icon_atlas::PowerHudIconId;

pub const SIM_HUD_POWER_ICONS_LIVE_JSON: &str = "debug_runs/sim_hud_power_icons_live.json";

#[must_use]
pub fn icon_power_line_tool_registered() -> bool {
    PowerHudIconId::inventory()
        .iter()
        .any(|id| *id == PowerHudIconId::PowerLineTool)
}

#[must_use]
pub fn icon_scram_registered() -> bool {
    PowerHudIconId::inventory()
        .iter()
        .any(|id| *id == PowerHudIconId::Scram)
}

#[must_use]
pub fn build_sim_hud_power_icons_body() -> serde_json::Value {
    let atlas = crate::gui::hud::power_hud_icon_atlas_registration_witness_green();
    let sheet = crate::gui::hud::sim_power_tool_sheet_icons_wired();
    let gauges = crate::gui::hud::plant_focus_card_gauges_wired();
    let picker = crate::construction::utilities_submenu_power_icons_wired();
    let pwr_line = icon_power_line_tool_registered();
    let scram = icon_scram_registered();
    let icon_count = PowerHudIconId::inventory().len();
    let texture_ok = crate::gui::hud::power_hud_atlas_assets_on_disk();
    let green = atlas
        && sheet
        && gauges
        && picker
        && pwr_line
        && scram
        && icon_count >= 13
        && texture_ok;
    serde_json::json!({
        "gate": "COD-ART-HUD-ICON-ATLAS-001",
        "slice_id": "COD-ART-HUD-ICON-ATLAS-001",
        "program_id": "PLAN-POWER-GRID-ART-ASSETS-001",
        "lane": "D",
        "green": green,
        "icon_atlas_registered": atlas,
        "icon_count": icon_count,
        "icon_power_line_tool_registered": pwr_line,
        "icon_scram_registered": scram,
        "power_atlas_texture_exists": texture_ok,
        "line_tool_icon": pwr_line,
        "voltage_tier_icons": atlas,
        "routing_mode_icons": atlas,
        "plant_card_gauges": gauges,
        "power_tool_sheet_icons": sheet,
        "utilities_picker_icons": picker,
        "atlas_texture_on_disk": texture_ok,
        "atlas_texture": crate::gui::hud::power_hud_icon_atlas::POWER_HUD_ATLAS_TEXTURE_PATH,
        "atlas_manifest": crate::gui::hud::power_hud_icon_atlas::POWER_HUD_ATLAS_MANIFEST_PATH,
        "design": "src/dev/design_hud_power_icons_v1.md",
    })
}

#[must_use]
pub fn refresh_sim_hud_power_icons_live_witness() -> bool {
    let body = build_sim_hud_power_icons_body();
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-ART-HUD-ICON-ATLAS-001",
        "refresh_sim_hud_power_icons_live_witness",
        SIM_HUD_POWER_ICONS_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(SIM_HUD_POWER_ICONS_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_power_icons_live_witness_green() {
        assert!(refresh_sim_hud_power_icons_live_witness());
    }
}
