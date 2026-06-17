//! SIM-HUD lane 5 polish — readability tokens + slice green predicates (CODER-B-HUD-L5-POLISH-001).

use crate::gui::minimap_shell::MinimapOverlayMask;
use crate::gui::simulation_minimap_overlay_defaults;
use crate::sim::effects::{format_ops_strip_alerts_line, PlayerEventLog};

use super::panel_state::HudPanelState;
use super::simulation_shell_phase2::{
    format_ops_strip_alert_badge, format_sim_tick_line, phase2c_layout_contract_ok,
    BUILD_RAIL_W_PX, UiShellMigrationWitness,
};

/// Minimum ops-strip body / badge font (design_sim_hud_ops_v1.md).
pub const OPS_STRIP_FONT_MIN_PX: f32 = 11.0;
/// Primary ops-strip mono body size in [`crate::gui::in_game_hud`].
pub const OPS_STRIP_BODY_FONT_PT: f32 = 13.0;
/// Info & overlays egui panel body floor.
pub const INFO_PANEL_BODY_FONT_MIN_PX: f32 = 11.0;

#[must_use]
pub fn sim_hud_slice_ops_polish_green() -> bool {
    OPS_STRIP_BODY_FONT_PT >= OPS_STRIP_FONT_MIN_PX
        && format_sim_tick_line(42, false, 1.0) == "T+00042  RUN    v=1.0x"
        && format_ops_strip_alerts_line(0, &PlayerEventLog::default()) == "ALERTS  0"
        && format_ops_strip_alert_badge(0) == "◆0"
        && format_ops_strip_alert_badge(4) == "◆4"
}

#[must_use]
pub fn sim_hud_slice_minimap_green(mask: &MinimapOverlayMask) -> bool {
    let expected = simulation_minimap_overlay_defaults();
    mask.logistics_heat == expected.logistics_heat
        && mask.fire_heat == expected.fire_heat
        && mask.units == expected.units
        && mask.fow == expected.fow
        && !mask.fire_heat
}

#[must_use]
pub fn sim_hud_slice_dock_green(
    command_tray_collapsed: bool,
    overlay_tray_collapsed: bool,
    witness: &UiShellMigrationWitness,
) -> bool {
    command_tray_collapsed
        && overlay_tray_collapsed
        && witness.floating_egui_shells_gated
}

#[must_use]
pub fn sim_hud_slice_build_green(
    context_tray_collapsed: bool,
    witness: &UiShellMigrationWitness,
) -> bool {
    context_tray_collapsed
        && BUILD_RAIL_W_PX == 52.0
        && witness.build_rail_authoritative
}

#[must_use]
pub fn sim_hud_info_panel_tokens_ok() -> bool {
    INFO_PANEL_BODY_FONT_MIN_PX >= OPS_STRIP_FONT_MIN_PX
}

#[must_use]
pub fn sim_hud_l5_polish_rollup_green() -> bool {
    sim_hud_slice_ops_polish_green()
        && sim_hud_info_panel_tokens_ok()
        && phase2c_layout_contract_ok()
        && !crate::render::infrastructure_overlay_legend_rows().is_empty()
        && crate::gui::construction_growth_inspector::growth_hud_ecology_hint_wired_witness_green()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_l5_polish_predicates_green() {
        assert!(sim_hud_slice_ops_polish_green());
        assert!(sim_hud_info_panel_tokens_ok());
        let mask = simulation_minimap_overlay_defaults();
        assert!(sim_hud_slice_minimap_green(&mask));
        let witness = UiShellMigrationWitness {
            floating_egui_shells_gated: true,
            build_rail_authoritative: true,
            ..Default::default()
        };
        assert!(sim_hud_slice_dock_green(true, true, &witness));
        assert!(sim_hud_slice_build_green(true, &witness));
        assert!(sim_hud_l5_polish_rollup_green());
    }
}
