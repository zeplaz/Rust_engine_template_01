//! **COD-SIM-HUD-ESC-CASCADE-001** — Esc order: picker → trays → pause menu.

use bevy::prelude::*;

use crate::gui::hud::{
    ContextTrayState, HudCommandShellLayout, HudOverlayTrayState,
    SimBuildPickerState,
};
use crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState;

const MAX_BUILD_READ_LINE: usize = 48;

#[must_use]
pub fn sim_hud_trays_expanded(
    context_tray: &ContextTrayState,
    tray: &HudOverlayTrayState,
    layout: &HudCommandShellLayout,
) -> bool {
    context_tray.panel_state.shows_content()
        || tray.tray_panel_state.shows_content()
        || layout.overlay_tray_state.shows_content()
        || layout.command_tray_state.shows_content()
        || layout.intel_timeline_state.shows_content()
        || layout.command_table_state.shows_content()
}

#[must_use]
pub fn sim_hud_esc_cascade_ready_for_pause(
    picker: &SimBuildPickerState,
    road_sheet: &SimRoadToolSheetState,
    context_tray: &ContextTrayState,
    tray: &HudOverlayTrayState,
    layout: &HudCommandShellLayout,
) -> bool {
    !picker.open
        && !road_sheet.open
        && !sim_hud_trays_expanded(context_tray, tray, layout)
}

/// Truncate build-read lines per DES-BUILD-READ-HUD-002.
#[must_use]
pub fn truncate_build_read_line(line: &str) -> String {
    // The limit is in displayed characters, not UTF-8 bytes (lines contain `·`, `✗`, `…`).
    if line.chars().count() <= MAX_BUILD_READ_LINE {
        return line.to_string();
    }
    let mut s = line.chars().take(MAX_BUILD_READ_LINE.saturating_sub(1)).collect::<String>();
    s.push('…');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_build_read_line_at_48() {
        let long = "BUILD  ·  Blocked ✗ · reason that is way too long for one strip line";
        let t = truncate_build_read_line(long);
        assert!(t.chars().count() <= MAX_BUILD_READ_LINE);
        assert!(t.ends_with('…'));
    }
}
