//! In-game pause menu (Escape) — simulation shell overlay distinct from sim tick pause (P).
//! Plan: `prompts/guides/ui/ui_phase5_pause_menu_plan_v1.md` (PLAN-UI-P5-PAUSE-001).
//! UI: [`pause_menu_bevy`] (Bevy); destructive nav in [`pause_menu_confirm`].

use bevy::prelude::*;

use crate::engine::states::InGameMenuState;
use crate::gui::hud::ContextTrayState;
use crate::gui::pause_menu_confirm::PauseMenuPendingAction;
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::gui::InputBindings;

pub fn toggle_pause_menu_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    menu: Res<State<InGameMenuState>>,
    picker: Res<crate::gui::hud::SimBuildPickerState>,
    road_sheet: Res<crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState>,
    context_tray: Res<ContextTrayState>,
    tray: Res<crate::gui::hud::HudOverlayTrayState>,
    layout: Res<crate::gui::hud::HudCommandShellLayout>,
    transmission: Res<crate::gui::hud::TransmissionShellState>,
    mut next_menu: ResMut<NextState<InGameMenuState>>,
    mut pending: ResMut<PauseMenuPendingAction>,
) {
    if !keys.just_pressed(bindings.cancel_keybinding_capture) {
        return;
    }
    if *menu.get() == InGameMenuState::Pause {
        NextState::set_if_neq(&mut *next_menu, InGameMenuState::Normal);
        pending.clear();
        return;
    }
    if pending.is_pending() {
        return;
    }
    if !super::hud::sim_hud_esc_cascade::sim_hud_esc_cascade_ready_for_pause(
        &picker,
        &road_sheet,
        &context_tray,
        &tray,
        &layout,
        &transmission,
    ) {
        return;
    }
    NextState::set_if_neq(&mut *next_menu, InGameMenuState::Pause);
}

pub struct InGamePauseMenuPlugin;

impl Plugin for InGamePauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InGameMenuState>()
            .init_resource::<PauseMenuPendingAction>()
            .add_plugins(super::hud::sim_hud_esc_cascade::SimHudEscCascadePlugin)
            .add_plugins(super::pause_menu_bevy::PauseMenuBevyPlugin)
            .add_systems(
                Update,
                toggle_pause_menu_on_escape
                    .run_if(in_simulation_or_editor)
                    .after(crate::gui::hud::panel_state::hud_panel_escape_collapse_system),
            );
    }
}
