//! @orchestrator-status IN_PROGRESS
//! @orchestrator-owner ui_layout_agent
//! @orchestrator-do-not-cleanup
//! HUD panel width state machine (Visual Aid v2 VA1).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, SimulationMapPointerGate, SimulationToolPointer,
};
use crate::gui::hud::sim_hud_esc_cascade::{
    resolve_esc_handler, sim_hud_trays_expanded, EscCascadeHandler, HudEscCascadeFrame,
    HudEscCascadeSet,
};
use crate::gui::SimulationMapViewport;

/// Panel chrome state — replaces bool `expanded` on shell resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HudPanelState {
    #[default]
    Collapsed,
    Peek,
    Expanded,
    Pinned,
}

impl HudPanelState {
    pub const WIDTH_COLLAPSED: f32 = 44.0;
    pub const WIDTH_PEEK: f32 = 200.0;
    pub const WIDTH_EXPANDED: f32 = 280.0;

    #[must_use]
    pub const fn target_width(self) -> f32 {
        match self {
            Self::Collapsed => Self::WIDTH_COLLAPSED,
            Self::Peek => Self::WIDTH_PEEK,
            Self::Expanded | Self::Pinned => Self::WIDTH_EXPANDED,
        }
    }

    #[must_use]
    pub const fn shows_content(self) -> bool {
        !matches!(self, Self::Collapsed)
    }

    #[must_use]
    pub const fn is_pinned(self) -> bool {
        matches!(self, Self::Pinned)
    }

    pub fn collapse_unpinned(&mut self) {
        if !self.is_pinned() {
            *self = Self::Collapsed;
        }
    }

    pub fn toggle_pin(&mut self) {
        *self = if *self == Self::Pinned {
            Self::Expanded
        } else {
            Self::Pinned
        };
    }

    pub fn click_open(&mut self) {
        if *self == Self::Collapsed {
            *self = Self::Expanded;
        }
    }

    pub fn hover_peek(&mut self) {
        if *self == Self::Collapsed {
            *self = Self::Peek;
        }
    }
}

/// Optional per-panel animation hook (width lerp later).
#[derive(Component, Debug, Clone)]
pub struct HudPanel {
    pub state: HudPanelState,
    pub anim_t: f32,
}

impl Default for HudPanel {
    fn default() -> Self {
        Self {
            state: HudPanelState::Collapsed,
            anim_t: 0.0,
        }
    }
}

/// Global HUD panel collapse on Escape (unpinned panels only).
pub fn hud_panel_escape_collapse_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut picker: ResMut<crate::gui::hud::SimBuildPickerState>,
    mut road_sheet: ResMut<crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState>,
    mut tray: ResMut<crate::gui::hud::HudOverlayTrayState>,
    mut layout: ResMut<crate::gui::hud::HudCommandShellLayout>,
    mut context_tray: ResMut<super::simulation_shell_phase2::ContextTrayState>,
    keybindings: Option<Res<crate::gui::options_keybindings_ui::KeybindingsUiState>>,
    mut esc_frame: ResMut<HudEscCascadeFrame>,
) {
    // ECS-10: sole writer of the per-frame Esc witness; resolve from pre-mutation state.
    if !keys.just_pressed(KeyCode::Escape) {
        esc_frame.set_if_neq(HudEscCascadeFrame::default());
        return;
    }
    let handler = resolve_esc_handler(
        keybindings.is_some_and(|k| k.capture_slot.is_some()),
        picker.open,
        road_sheet.open,
        sim_hud_trays_expanded(&context_tray, &tray, &layout),
    );
    esc_frame.set_if_neq(HudEscCascadeFrame {
        handled_by: Some(handler),
    });
    match handler {
        // Key-binding capture owns Esc (cancel); HUD stays as-is.
        EscCascadeHandler::KeybindingCapture | EscCascadeHandler::PauseMenu => return,
        EscCascadeHandler::BuildPicker => {
            picker.close();
            return;
        }
        EscCascadeHandler::RoadSheet => {
            road_sheet.close();
            return;
        }
        EscCascadeHandler::TrayCollapse => {}
    }
    super::simulation_shell_phase2::collapse_context_tray_on_escape(&mut context_tray);
    tray.tray_panel_state.collapse_unpinned();
    layout.overlay_tray_state.collapse_unpinned();
    layout.command_tray_state.collapse_unpinned();
    layout.intel_timeline_state.collapse_unpinned();
    layout.command_table_state.collapse_unpinned();
}

fn click_is_inside_map_hole(
    map_vp: &SimulationMapViewport,
    win: &Query<&Window, With<PrimaryWindow>>,
) -> bool {
    let Ok(window) = win.single() else {
        return false;
    };
    let Some(cursor) = window.cursor_position() else {
        return false;
    };
    map_vp.is_adequate_for_camera() && map_vp.contains_cursor(cursor)
}

fn collapse_unpinned_shells(
    tray: &mut crate::gui::hud::HudOverlayTrayState,
    layout: &mut crate::gui::hud::HudCommandShellLayout,
    context_tray: &mut crate::gui::hud::ContextTrayState,
) {
    tray.tray_panel_state.collapse_unpinned();
    layout.overlay_tray_state.collapse_unpinned();
    layout.command_tray_state.collapse_unpinned();
    context_tray.panel_state.collapse_unpinned();
}

/// Simulation click outside Bevy chrome → collapse unpinned shells. No egui context.
pub fn hud_panel_click_outside_collapse_system(
    mouse: Res<ButtonInput<MouseButton>>,
    map_vp: Res<SimulationMapViewport>,
    win: Query<&Window, With<PrimaryWindow>>,
    gate: Option<Res<SimulationMapPointerGate>>,
    mut tray: ResMut<crate::gui::hud::HudOverlayTrayState>,
    mut layout: ResMut<crate::gui::hud::HudCommandShellLayout>,
    mut context_tray: ResMut<crate::gui::hud::ContextTrayState>,
) {
    if !mouse.just_pressed(MouseButton::Left) || click_is_inside_map_hole(map_vp.as_ref(), &win) {
        return;
    }
    if simulation_tool_pointer(gate.as_deref()) == SimulationToolPointer::HeldByChrome {
        return;
    }
    collapse_unpinned_shells(
        tray.as_mut(),
        layout.as_mut(),
        context_tray.as_mut(),
    );
}

pub struct HudPanelStatePlugin;

impl Plugin for HudPanelStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudEscCascadeFrame>().add_systems(
            Update,
            (
                hud_panel_escape_collapse_system.in_set(HudEscCascadeSet::Resolve),
                hud_panel_click_outside_collapse_system
                    .after(HudEscCascadeSet::Resolve)
                    .after(super::simulation_pointer_gate::PointerGateSet::Publish),
            )
                .run_if(crate::gui::ui_gates::in_simulation_or_editor),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_panel_state_widths_and_transitions() {
        assert_eq!(HudPanelState::Collapsed.target_width(), 44.0);
        assert_eq!(HudPanelState::Pinned.target_width(), 280.0);
        let mut s = HudPanelState::Expanded;
        s.collapse_unpinned();
        assert_eq!(s, HudPanelState::Collapsed);
        s = HudPanelState::Pinned;
        s.collapse_unpinned();
        assert_eq!(s, HudPanelState::Pinned);
        s.toggle_pin();
        assert_eq!(s, HudPanelState::Expanded);
        let mut collapsed = HudPanelState::Collapsed;
        collapsed.hover_peek();
        assert_eq!(collapsed, HudPanelState::Peek);
    }
}
