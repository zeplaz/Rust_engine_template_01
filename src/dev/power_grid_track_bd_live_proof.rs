//! Track B/D rollup witness — overlay · island · activation · toast · tool rail.

pub const POWER_GRID_TRACK_BD_LIVE_JSON: &str = "debug_runs/power_grid_track_bd_live.json";

#[must_use]
pub fn refresh_power_grid_track_bd_live_witness() -> bool {
    use crate::construction::utilities_submenu_power_icons_wired;
    use crate::construction::{ActiveBuildTool, BuildStripState, BuildTool, ToolContext};
    use crate::economy::activation::{
        power_island_ux_001_green, power_island_ux_toast_ui_wired, PowerIslandToastState,
    };
    use crate::gui::hud::simulation_shell_phase2::witness_build_rail_tool_authoritative_replay;
    use crate::gui::hud::sim_power_tool_sheet::SimPowerToolSheetState;
    use crate::gui::hud::UiShellMigrationWitness;
    use crate::infrastructure::utility_activation_link_witness_green;
    use crate::infrastructure::VoltageClass;
    use crate::infrastructure::{UtilityAuthoringMode, UtilityAuthoringTool};
    use crate::render::{
        power_map_overlay_draw_witness_green, power_map_overlay_green,
        InfrastructureOverlaySettings, PowerMapOverlayPresentation,
    };

    let overlay_green = crate::dev::power_map_overlay_live_proof::refresh_power_map_overlay_live_witness();

    let mut strip = BuildStripState::default();
    let mut tool = ActiveBuildTool::default();
    let mut witness = UiShellMigrationWitness::default();
    let mut power_sheet = SimPowerToolSheetState::default();
    witness_build_rail_tool_authoritative_replay(
        &mut strip,
        &mut tool,
        &mut witness,
        ToolContext::Utilities,
    );
    power_sheet.sync_from_tool(&tool);
    let tool_rail_green = strip.active == ToolContext::Utilities
        && matches!(tool.tool, BuildTool::PowerLine(VoltageClass::Medium))
        && power_sheet.open;

    let settings = InfrastructureOverlaySettings {
        enabled: true,
        power: true,
        ..Default::default()
    };
    let presentation = PowerMapOverlayPresentation {
        island_highlight_active: true,
        island_offline_buildings: 2,
        ..Default::default()
    };
    let authoring = UtilityAuthoringTool {
        mode: UtilityAuthoringMode::PlacePower,
        ..Default::default()
    };
    let island_toast = PowerIslandToastState {
        show_count: 1,
        offline_buildings: 2,
        last_message: crate::economy::activation::power_island_toast_message(2),
        ..Default::default()
    };

    let green = overlay_green
        && utility_activation_link_witness_green()
        && power_map_overlay_draw_witness_green()
        && power_map_overlay_green(&settings, &presentation, &authoring)
        && tool_rail_green
        && utilities_submenu_power_icons_wired()
        && power_island_ux_toast_ui_wired()
        && power_island_ux_001_green(&island_toast, 1);

    let body = serde_json::json!({
        "gate": "PLAN-POWER-GRID-CONSTRUCTION-UX-001",
        "track_b_d_green": green,
        "COD-POWER-OVERLAY-RENDER-001": overlay_green,
        "COD-POWER-ISLAND-HIGHLIGHT-001": presentation.island_highlight_active,
        "COD-UTILITY-ACTIVATION-LINK-001": utility_activation_link_witness_green(),
        "COD-POWER-ISLAND-TOAST-001": power_island_ux_001_green(&island_toast, 1),
        "COD-POWER-TOOL-RAIL-001": tool_rail_green,
        "utilities_submenu_icons": utilities_submenu_power_icons_wired(),
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "PLAN-POWER-GRID-CONSTRUCTION-UX-001",
        "refresh_power_grid_track_bd_live_witness",
        POWER_GRID_TRACK_BD_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(POWER_GRID_TRACK_BD_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_grid_track_bd_live_witness_green() {
        assert!(refresh_power_grid_track_bd_live_witness());
    }
}
