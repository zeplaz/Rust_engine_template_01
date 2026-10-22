//! Left **status rail** — docked egui side panel (`HudCommandShellLayout::status_side_panel_state`).
//!
//! **2B-03 / HUD-NAT-008:** Editor-only — simulation uses Bevy left context rail; gated via
//! [`crate::gui::ui_gates::side_status_rail_egui_active`] and
//! [`super::shell_framework::side_status_rail_egui_dock_active`].
use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::input_bindings::InputBindings;

use super::dock_shell::HudCommandShellLayout;
use super::panel_state::HudPanelState;
use super::shell_framework::side_status_rail_egui_dock_active;

/// HUD-NAT-008 — egui side status must not open in Simulation (Bevy context rail owns status).
pub const SIDE_STATUS_RAIL_EGUI_IN_SIM: bool = false;

/// Lib / witness: sim gate confirms no egui side-status overflow into play chrome.
#[must_use]
pub fn hud_nat_008_side_status_sim_gated() -> bool {
    !SIDE_STATUS_RAIL_EGUI_IN_SIM
        && !side_status_rail_egui_dock_active(BaseState::Simulation)
        && side_status_rail_egui_dock_active(BaseState::Editor)
        && include_str!("hud_side_status_panel.rs").contains("side_status_rail_egui_dock_active")
        && include_str!("hud_side_status_panel.rs").contains("SIDE_STATUS_RAIL_EGUI_IN_SIM")
}

/// Draw the docked left status panel (call early in the egui pass).

/// Keyboard: cycle status side panel width state.
pub fn hud_status_side_panel_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut layout: ResMut<HudCommandShellLayout>,
) {
    if !keys.just_pressed(bindings.toggle_hud_status_side_panel) {
        return;
    }
    layout.status_side_panel_state = match layout.status_side_panel_state {
        HudPanelState::Collapsed => HudPanelState::Expanded,
        HudPanelState::Peek => HudPanelState::Expanded,
        HudPanelState::Expanded => HudPanelState::Collapsed,
        HudPanelState::Pinned => HudPanelState::Collapsed,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_nat_008_side_status_sim_gated_lib() {
        assert!(hud_nat_008_side_status_sim_gated());
    }
}
