//! TEMP-EGUI: minimal world-map editor shell (runbook M1). Replace with Bevy UI per `gui_runbook_v1` when
//! widgets reach parity. Registered from `EnginePlugin` only so the `world_generator` binary stays free
//! of `BaseState`.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::engine::{BaseState, MainMenuState, WorldGenFlowState};
use crate::gui::AppStartState;

pub fn map_editor_shell_system(
    mut contexts: EguiContexts,
    mut next_base: ResMut<NextState<BaseState>>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
) -> Result {
    egui::Window::new("Map editor (M1)")
        .anchor(egui::Align2::LEFT_TOP, [8.0, 8.0])
        .collapsible(false)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label("TEMP-EGUI placeholder shell. Tools in M2–M5.");
            if ui.button("Exit to main menu").clicked() {
                NextState::set_if_neq(&mut *next_base, BaseState::MainMenu);
                NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                NextState::set_if_neq(&mut *next_menu, MainMenuState::MainMenu);
            }
        });
    Ok(())
}

pub struct MapEditorShellPlugin;

impl Plugin for MapEditorShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            map_editor_shell_system.run_if(
                in_state(AppStartState::Menu).and(in_state(BaseState::Editor)),
            ),
        );
    }
}
