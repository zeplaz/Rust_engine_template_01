//! @orchestrator-status IN_PROGRESS
//! @orchestrator-owner ui_layout_agent
//! @orchestrator-do-not-cleanup
//! HUD panel width state machine (Visual Aid v2 VA1).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

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

/// Witness for VISUAL-AID-V2-01 predicates.
#[derive(Resource, Clone, Debug, Default)]
pub struct HudPanelStateWitness {
    pub cycle_ok: bool,
    pub last_esc_collapsed: bool,
}

/// Global HUD panel collapse on Escape (unpinned panels only).
pub fn hud_panel_escape_collapse_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut tray: ResMut<crate::gui::hud::HudOverlayTrayState>,
    mut layout: ResMut<crate::gui::hud::HudCommandShellLayout>,
    mut transmission: ResMut<crate::gui::hud::TransmissionShellState>,
    mut context_tray: ResMut<super::simulation_shell_phase2::ContextTrayState>,
    mut ui_shell_witness: ResMut<super::simulation_shell_phase2::UiShellMigrationWitness>,
    mut witness: ResMut<HudPanelStateWitness>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    super::simulation_shell_phase2::collapse_context_tray_on_escape(
        &mut context_tray,
        &mut ui_shell_witness,
    );
    let before_tray = tray.tray_panel_state;
    let before_cmd = layout.command_tray_state;
    let before_side = layout.status_side_panel_state;
    tray.tray_panel_state.collapse_unpinned();
    layout.overlay_tray_state.collapse_unpinned();
    layout.status_side_panel_state.collapse_unpinned();
    layout.command_tray_state.collapse_unpinned();
    layout.intel_timeline_state.collapse_unpinned();
    layout.command_table_state.collapse_unpinned();
    transmission.panel_state.collapse_unpinned();
    witness.last_esc_collapsed = before_tray != HudPanelState::Collapsed
        || before_cmd != HudPanelState::Collapsed
        || before_side != HudPanelState::Collapsed
        || transmission.panel_state != HudPanelState::Collapsed;
    witness.cycle_ok = true;
}

/// Click outside egui panels → collapse unpinned shells.
pub fn hud_panel_click_outside_collapse_system(
    mut contexts: EguiContexts,
    mouse: Res<ButtonInput<MouseButton>>,
    map_vp: Res<SimulationMapViewport>,
    win: Query<&Window, With<PrimaryWindow>>,
    mut tray: ResMut<crate::gui::hud::HudOverlayTrayState>,
    mut layout: ResMut<crate::gui::hud::HudCommandShellLayout>,
    mut transmission: ResMut<crate::gui::hud::TransmissionShellState>,
    mut context_tray: ResMut<crate::gui::hud::ContextTrayState>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if let Ok(window) = win.single() {
        if let Some(cursor) = window.cursor_position() {
            if map_vp.is_adequate_for_camera() && map_vp.contains_cursor(cursor) {
                return;
            }
        }
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if ctx.is_pointer_over_area() {
        return;
    }
    tray.tray_panel_state.collapse_unpinned();
    layout.overlay_tray_state.collapse_unpinned();
    layout.status_side_panel_state.collapse_unpinned();
    layout.command_tray_state.collapse_unpinned();
    transmission.panel_state.collapse_unpinned();
    context_tray.panel_state.collapse_unpinned();
}

pub struct HudPanelStatePlugin;

impl Plugin for HudPanelStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudPanelStateWitness>().add_systems(
            Update,
            (
                hud_panel_escape_collapse_system,
                hud_panel_click_outside_collapse_system.after(hud_panel_escape_collapse_system),
                super::hud_side_status_panel::hud_status_side_panel_toggle_system
                    .run_if(crate::gui::ui_gates::side_status_rail_egui_active),
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
