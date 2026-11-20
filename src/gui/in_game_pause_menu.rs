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
    mut next_menu: ResMut<NextState<InGameMenuState>>,
    mut pending: ResMut<PauseMenuPendingAction>,
    esc_frame: Res<super::hud::sim_hud_esc_cascade::HudEscCascadeFrame>,
) {
    if !keys.just_pressed(bindings.cancel_keybinding_capture) {
        return;
    }
    // CQ-06 / ECS-10: honor upstream claim *before* open/close pause (capture must not also
    // dismiss the pause menu on the same press).
    if esc_frame.consumed_before_pause() {
        return;
    }
    if *menu.get() == InGameMenuState::Pause {
        NextState::set_if_different(&mut *next_menu, InGameMenuState::Normal);
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
    ) {
        return;
    }
    NextState::set_if_different(&mut *next_menu, InGameMenuState::Pause);
}

pub struct InGamePauseMenuPlugin;

impl Plugin for InGamePauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InGameMenuState>()
            .init_resource::<PauseMenuPendingAction>()
            .init_resource::<super::hud::sim_hud_esc_cascade::HudEscCascadeFrame>()
            .add_plugins(super::pause_menu_bevy::PauseMenuBevyPlugin)
            .add_systems(
                Update,
                toggle_pause_menu_on_escape
                    .run_if(in_simulation_or_editor)
                    .after(crate::gui::hud::sim_hud_esc_cascade::HudEscCascadeSet::Resolve),
            );
    }
}

#[cfg(test)]
mod ecs10_esc_chain_tests {
    use super::*;
    use crate::gui::hud::panel_state::hud_panel_escape_collapse_system;
    use crate::gui::hud::sim_hud_esc_cascade::{
        EscCascadeHandler, HudEscCascadeFrame, HudEscCascadeSet,
    };
    use crate::gui::hud::{
        ContextTrayState, HudCommandShellLayout, HudOverlayTrayState, HudPanelState,
        SimBuildPickerState,
    };
    use crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState;
    use crate::gui::options_keybindings_ui::{BindingSlot, KeybindingsUiState};
    use bevy::state::app::StatesPlugin;

    fn chain_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<InGameMenuState>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputBindings>()
            .init_resource::<SimBuildPickerState>()
            .init_resource::<SimRoadToolSheetState>()
            .init_resource::<ContextTrayState>()
            .init_resource::<HudOverlayTrayState>()
            .init_resource::<HudCommandShellLayout>()
            .init_resource::<PauseMenuPendingAction>()
            .init_resource::<KeybindingsUiState>()
            .init_resource::<HudEscCascadeFrame>()
            .add_systems(
                Update,
                (
                    hud_panel_escape_collapse_system.in_set(HudEscCascadeSet::Resolve),
                    toggle_pause_menu_on_escape.after(HudEscCascadeSet::Resolve),
                ),
            );
        app
    }

    fn press_esc_and_update(app: &mut App) {
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.release(KeyCode::Escape);
            keys.clear();
            keys.press(KeyCode::Escape);
        }
        app.update();
    }

    fn pause_requested(app: &App) -> bool {
        // The transition may already be applied (StateTransition) or still queued.
        *app.world().resource::<State<InGameMenuState>>().get() == InGameMenuState::Pause
            || matches!(
                app.world().resource::<NextState<InGameMenuState>>(),
                NextState::Pending(InGameMenuState::Pause)
                    | NextState::PendingIfDifferent(InGameMenuState::Pause)
            )
    }

    fn clear_pending(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<InGameMenuState>>()
            .reset();
    }

    #[test]
    fn ecs10_esc_closing_picker_does_not_fall_through_to_pause() {
        let mut app = chain_app();
        app.world_mut().resource_mut::<SimBuildPickerState>().open = true;
        press_esc_and_update(&mut app);
        assert!(!app.world().resource::<SimBuildPickerState>().open, "picker consumed Esc");
        assert_eq!(
            app.world().resource::<HudEscCascadeFrame>().handled_by,
            Some(EscCascadeHandler::BuildPicker)
        );
        assert!(!pause_requested(&app), "same Esc must not also open pause");

        clear_pending(&mut app);
        press_esc_and_update(&mut app);
        assert!(pause_requested(&app), "second Esc reaches the pause handler");
    }

    #[test]
    fn ecs10_esc_with_road_sheet_open_does_not_fall_through_to_pause() {
        let mut app = chain_app();
        app.world_mut().resource_mut::<SimRoadToolSheetState>().open = true;
        press_esc_and_update(&mut app);
        // The sheet belongs to the armed line tool: the build cancel takes this Esc (clear the
        // path, then drop the tool, which hides the sheet). The cascade only keeps it from pause.
        assert_eq!(
            app.world().resource::<HudEscCascadeFrame>().handled_by,
            Some(EscCascadeHandler::RoadSheet)
        );
        assert!(!pause_requested(&app));
    }

    #[test]
    fn ecs10_esc_collapsing_tray_does_not_fall_through_to_pause() {
        let mut app = chain_app();
        app.world_mut().resource_mut::<ContextTrayState>().panel_state = HudPanelState::Expanded;
        press_esc_and_update(&mut app);
        assert_eq!(
            app.world().resource::<ContextTrayState>().panel_state,
            HudPanelState::Collapsed
        );
        assert_eq!(
            app.world().resource::<HudEscCascadeFrame>().handled_by,
            Some(EscCascadeHandler::TrayCollapse)
        );
        assert!(!pause_requested(&app));
    }

    #[test]
    fn ecs10_keybinding_capture_esc_does_not_open_pause_or_collapse_hud() {
        let mut app = chain_app();
        app.world_mut().resource_mut::<KeybindingsUiState>().capture_slot =
            Some(BindingSlot::FrontHold);
        app.world_mut().resource_mut::<ContextTrayState>().panel_state = HudPanelState::Expanded;
        press_esc_and_update(&mut app);
        assert_eq!(
            app.world().resource::<HudEscCascadeFrame>().handled_by,
            Some(EscCascadeHandler::KeybindingCapture)
        );
        assert!(!pause_requested(&app));
        assert_eq!(
            app.world().resource::<ContextTrayState>().panel_state,
            HudPanelState::Expanded,
            "capture owns Esc; HUD must not collapse"
        );
    }

    #[test]
    fn cq06_capture_esc_does_not_also_close_open_pause_menu() {
        let mut app = chain_app();
        NextState::set_if_different(
            &mut *app.world_mut().resource_mut::<NextState<InGameMenuState>>(),
            InGameMenuState::Pause,
        );
        app.update();
        assert_eq!(
            *app.world().resource::<State<InGameMenuState>>().get(),
            InGameMenuState::Pause
        );
        app.world_mut().resource_mut::<KeybindingsUiState>().capture_slot =
            Some(BindingSlot::FrontHold);
        press_esc_and_update(&mut app);
        assert_eq!(
            app.world().resource::<HudEscCascadeFrame>().handled_by,
            Some(EscCascadeHandler::KeybindingCapture)
        );
        assert_eq!(
            *app.world().resource::<State<InGameMenuState>>().get(),
            InGameMenuState::Pause,
            "same Esc must cancel capture only; pause stays open"
        );
    }

    #[test]
    fn cq06_rebound_cancel_key_drives_esc_cascade() {
        let mut app = chain_app();
        app.world_mut()
            .resource_mut::<InputBindings>()
            .cancel_keybinding_capture = KeyCode::Backspace;
        app.world_mut().resource_mut::<SimBuildPickerState>().open = true;
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(KeyCode::Backspace);
        }
        app.update();
        assert!(!app.world().resource::<SimBuildPickerState>().open);
        assert_eq!(
            app.world().resource::<HudEscCascadeFrame>().handled_by,
            Some(EscCascadeHandler::BuildPicker)
        );
        // Hardcoded Escape must not also fire the cascade this frame.
        assert!(!pause_requested(&app));
    }

    #[test]
    fn ecs10_esc_with_no_upstream_handler_opens_pause() {
        let mut app = chain_app();
        press_esc_and_update(&mut app);
        assert_eq!(
            app.world().resource::<HudEscCascadeFrame>().handled_by,
            Some(EscCascadeHandler::PauseMenu)
        );
        assert!(pause_requested(&app));
    }

    #[test]
    fn ecs10_frame_witness_resets_when_esc_not_pressed() {
        let mut app = chain_app();
        app.world_mut().resource_mut::<SimBuildPickerState>().open = true;
        press_esc_and_update(&mut app);
        assert!(app.world().resource::<HudEscCascadeFrame>().consumed_before_pause());
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();
        app.update();
        assert_eq!(*app.world().resource::<HudEscCascadeFrame>(), HudEscCascadeFrame::default());
    }
}
