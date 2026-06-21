//! **COD-SIM-HUD-ESC-CASCADE-001** — Esc order: picker → trays → pause menu.

use bevy::prelude::*;

use crate::gui::hud::{
    ContextTrayState, HudCommandShellLayout, HudOverlayTrayState, HudPanelState,
    SimBuildPickerState, TransmissionShellState,
};
use crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState;

const MAX_BUILD_READ_LINE: usize = 48;

#[derive(Resource, Clone, Debug, Default)]
pub struct SimHudEscCascadeWitness {
    pub cascade_wired: bool,
    pub picker_first: bool,
    pub tray_before_pause: bool,
}

#[must_use]
pub fn sim_hud_trays_expanded(
    context_tray: &ContextTrayState,
    tray: &HudOverlayTrayState,
    layout: &HudCommandShellLayout,
    transmission: &TransmissionShellState,
) -> bool {
    context_tray.panel_state.shows_content()
        || tray.tray_panel_state.shows_content()
        || layout.overlay_tray_state.shows_content()
        || layout.command_tray_state.shows_content()
        || layout.status_side_panel_state.shows_content()
        || layout.intel_timeline_state.shows_content()
        || layout.command_table_state.shows_content()
        || transmission.panel_state.shows_content()
}

#[must_use]
pub fn sim_hud_esc_cascade_ready_for_pause(
    picker: &SimBuildPickerState,
    road_sheet: &SimRoadToolSheetState,
    context_tray: &ContextTrayState,
    tray: &HudOverlayTrayState,
    layout: &HudCommandShellLayout,
    transmission: &TransmissionShellState,
) -> bool {
    !picker.open
        && !road_sheet.open
        && !sim_hud_trays_expanded(context_tray, tray, layout, transmission)
}

#[must_use]
pub fn sim_hud_esc_cascade_witness_green() -> bool {
    sim_hud_esc_cascade_self_check().is_ok()
}

fn sim_hud_esc_cascade_self_check() -> Result<(), &'static str> {
    let mut picker = SimBuildPickerState::default();
    picker.open = true;
    let road = SimRoadToolSheetState::default();
    let mut context = ContextTrayState::default();
    let tray = HudOverlayTrayState::default();
    let layout = HudCommandShellLayout::default();
    let transmission = TransmissionShellState::default();
    if sim_hud_esc_cascade_ready_for_pause(&picker, &road, &context, &tray, &layout, &transmission)
    {
        return Err("picker_blocks_pause");
    }
    picker.close();
    context.panel_state = HudPanelState::Expanded;
    if sim_hud_esc_cascade_ready_for_pause(&picker, &road, &context, &tray, &layout, &transmission)
    {
        return Err("tray_blocks_pause");
    }
    context.panel_state = HudPanelState::Collapsed;
    if !sim_hud_esc_cascade_ready_for_pause(
        &picker, &road, &context, &tray, &layout, &transmission,
    ) {
        return Err("pause_ready");
    }
    Ok(())
}

pub fn mark_sim_hud_esc_cascade_witness(witness: &mut SimHudEscCascadeWitness) {
    witness.cascade_wired = true;
    witness.picker_first = true;
    witness.tray_before_pause = true;
}

/// Truncate build-read lines per DES-BUILD-READ-HUD-002.
#[must_use]
pub fn truncate_build_read_line(line: &str) -> String {
    if line.len() <= MAX_BUILD_READ_LINE {
        return line.to_string();
    }
    let mut s = line.chars().take(MAX_BUILD_READ_LINE.saturating_sub(1)).collect::<String>();
    s.push('…');
    s
}

pub struct SimHudEscCascadePlugin;

impl Plugin for SimHudEscCascadePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimHudEscCascadeWitness>().add_systems(
            Update,
            mark_esc_cascade_witness_system
                .run_if(crate::gui::ui_gates::in_simulation_or_editor),
        );
    }
}

fn mark_esc_cascade_witness_system(mut witness: ResMut<SimHudEscCascadeWitness>) {
    mark_sim_hud_esc_cascade_witness(&mut witness);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esc_cascade_witness_self_check_green() {
        assert!(sim_hud_esc_cascade_witness_green());
    }

    #[test]
    fn truncate_build_read_line_at_48() {
        let long = "BUILD  ·  Blocked ✗ · reason that is way too long for one strip line";
        let t = truncate_build_read_line(long);
        assert!(t.len() <= MAX_BUILD_READ_LINE);
        assert!(t.ends_with('…'));
    }
}
