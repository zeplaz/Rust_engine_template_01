//! **COD-SIM-HUD-ESC-CASCADE-001** — Esc order: picker → trays → pause menu.
//!
//! # Chain of responsibility (ECS-10 · LEGACY `hid`)
//!
//! Ref-books **ECS-10** · Chains of Responsibility (GEA 4e II §17.8.7, gea4v2 p639–641) with
//! LEGACY keyword `hid` (gea_v1_e4 p538 / p578): one Esc press has **exactly one** handler; the
//! first handler that claims it consumes the event and nothing after it may react to the same
//! press. Pointer twin: [`crate::render::view_runtime::InputCaptureOwner`] (UI → surface → world).
//!
//! ```text
//! KeybindingCapture → BuildPicker → RoadSheet → TrayCollapse → PauseMenu
//! ```
//!
//! **Single writer:** [`hud_panel_escape_collapse_system`](crate::gui::hud::panel_state::hud_panel_escape_collapse_system)
//! (in [`HudEscCascadeSet::Resolve`]) resolves the handler from **pre-mutation** state and writes
//! [`HudEscCascadeFrame`] every frame. Downstream Esc readers
//! ([`toggle_pause_menu_on_escape`](crate::gui::toggle_pause_menu_on_escape)) run
//! `.after(HudEscCascadeSet::Resolve)` and must honor [`HudEscCascadeFrame::consumed`] — without
//! it the same Esc that closed the picker would also open the pause menu (state already mutated).

use bevy::prelude::*;

use crate::gui::hud::{
    ContextTrayState, HudCommandShellLayout, HudOverlayTrayState,
    SimBuildPickerState,
};
use crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState;

const MAX_BUILD_READ_LINE: usize = 48;

/// Ordered Esc handlers (first claim wins).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EscCascadeHandler {
    /// Options → Key bindings is waiting for a key; Esc cancels the capture (not pause / HUD).
    KeybindingCapture,
    BuildPicker,
    RoadSheet,
    /// Any expanded tray: unpinned ones collapse; pinned ones keep blocking pause.
    TrayCollapse,
    /// Nothing upstream claimed Esc — pause menu may open.
    PauseMenu,
}

/// Named anchor for the Esc chain resolver (no bare-fn `.after`).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HudEscCascadeSet {
    /// Resolves + applies the first handler; writes [`HudEscCascadeFrame`].
    Resolve,
}

/// Per-frame Esc consumption witness; sole writer is the `Resolve` system.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HudEscCascadeFrame {
    /// Handler that consumed this frame's Esc press (`None` when Esc was not pressed).
    pub handled_by: Option<EscCascadeHandler>,
}

impl HudEscCascadeFrame {
    /// True when a handler **upstream of the pause menu** consumed this frame's Esc.
    #[inline]
    #[must_use]
    pub fn consumed_before_pause(self) -> bool {
        !matches!(self.handled_by, None | Some(EscCascadeHandler::PauseMenu))
    }
}

/// Pure chain resolution from pre-mutation state (ECS-10).
#[inline]
#[must_use]
pub fn resolve_esc_handler(
    keybinding_capture_active: bool,
    picker_open: bool,
    road_sheet_open: bool,
    trays_expanded: bool,
) -> EscCascadeHandler {
    if keybinding_capture_active {
        EscCascadeHandler::KeybindingCapture
    } else if picker_open {
        EscCascadeHandler::BuildPicker
    } else if road_sheet_open {
        EscCascadeHandler::RoadSheet
    } else if trays_expanded {
        EscCascadeHandler::TrayCollapse
    } else {
        EscCascadeHandler::PauseMenu
    }
}

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
    fn ecs10_esc_chain_first_claim_wins() {
        use EscCascadeHandler::*;
        assert_eq!(resolve_esc_handler(true, true, true, true), KeybindingCapture);
        assert_eq!(resolve_esc_handler(false, true, true, true), BuildPicker);
        assert_eq!(resolve_esc_handler(false, false, true, true), RoadSheet);
        assert_eq!(resolve_esc_handler(false, false, false, true), TrayCollapse);
        assert_eq!(resolve_esc_handler(false, false, false, false), PauseMenu);
    }

    #[test]
    fn ecs10_only_upstream_handlers_block_pause() {
        let none = HudEscCascadeFrame::default();
        assert!(!none.consumed_before_pause());
        let pause = HudEscCascadeFrame { handled_by: Some(EscCascadeHandler::PauseMenu) };
        assert!(!pause.consumed_before_pause());
        for h in [
            EscCascadeHandler::KeybindingCapture,
            EscCascadeHandler::BuildPicker,
            EscCascadeHandler::RoadSheet,
            EscCascadeHandler::TrayCollapse,
        ] {
            assert!(HudEscCascadeFrame { handled_by: Some(h) }.consumed_before_pause());
        }
    }

    #[test]
    fn truncate_build_read_line_at_48() {
        let long = "BUILD  ·  Blocked ✗ · reason that is way too long for one strip line";
        let t = truncate_build_read_line(long);
        assert!(t.chars().count() <= MAX_BUILD_READ_LINE);
        assert!(t.ends_with('…'));
    }
}
