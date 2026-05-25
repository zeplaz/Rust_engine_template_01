//! Shared pause-menu destructive navigation (single confirm surface).

use bevy::prelude::*;

use crate::engine::states::{BaseState, MainMenuState, WorldGenFlowState};
use crate::engine::{ux_return_to_main_menu, AppState, WorldGenChromeLatch, PauseState, WorldGenState};
use crate::gui::editor::world_gen_ui::CancelActiveWorldGenEvent;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::editor::world_preview::WorldPreviewUiState;
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, WorldGenJobSlot, WorldGenProgress, WorldMarker,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PauseMenuConfirm {
    #[default]
    None,
    ExitToWorldGen,
    ExitToMainMenu,
}

/// Pending destructive navigation from pause menu.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct PauseMenuPendingAction {
    pub confirm: PauseMenuConfirm,
}

impl PauseMenuPendingAction {
    #[must_use]
    pub const fn is_pending(self) -> bool {
        !matches!(self.confirm, PauseMenuConfirm::None)
    }

    pub fn clear(&mut self) {
        self.confirm = PauseMenuConfirm::None;
    }
}

#[must_use]
pub fn world_gen_work_active(job: &WorldGenJobSlot, progress: &WorldGenProgress) -> bool {
    job.is_busy() || progress.running
}

pub fn execute_pause_exit_to_world_gen(
    commands: &mut Commands,
    world_roots: &Query<Entity, With<WorldMarker>>,
    chrome_latch: &mut WorldGenChromeLatch,
    next_base: &mut NextState<BaseState>,
    next_flow: &mut NextState<WorldGenFlowState>,
    next_app: &mut NextState<AppState>,
    next_wg: &mut NextState<WorldGenState>,
    next_pause: &mut NextState<PauseState>,
    world_gen_ui: &mut WorldGenUiState,
    world_preview_ui: &mut WorldPreviewUiState,
    cancel_world_gen: &mut MessageWriter<CancelActiveWorldGenEvent>,
    job: &WorldGenJobSlot,
    progress: &WorldGenProgress,
) {
    if world_gen_work_active(job, progress) {
        cancel_world_gen.write(CancelActiveWorldGenEvent);
    }
    despawn_generated_world_entities(commands, world_roots);
    chrome_latch.reset_for_new_flow();
    NextState::set_if_neq(next_base, BaseState::Editor);
    NextState::set_if_neq(next_flow, WorldGenFlowState::NewWorldSetup);
    next_app.set(AppState::WorldGen);
    next_wg.set(WorldGenState::Preview);
    next_pause.set(PauseState::Off);
    world_gen_ui.visible = true;
    world_preview_ui.window_open = true;
}

pub fn execute_pause_exit_to_main_menu(
    commands: &mut Commands,
    world_roots: &Query<Entity, With<WorldMarker>>,
    chrome_latch: &mut WorldGenChromeLatch,
    next_base: &mut NextState<BaseState>,
    next_flow: &mut NextState<WorldGenFlowState>,
    next_main_menu: &mut NextState<MainMenuState>,
    next_app: &mut NextState<AppState>,
    next_wg: &mut NextState<WorldGenState>,
    next_pause: &mut NextState<PauseState>,
    cancel_world_gen: &mut MessageWriter<CancelActiveWorldGenEvent>,
    job: &WorldGenJobSlot,
    progress: &WorldGenProgress,
) {
    if world_gen_work_active(job, progress) {
        cancel_world_gen.write(CancelActiveWorldGenEvent);
    }
    despawn_generated_world_entities(commands, world_roots);
    ux_return_to_main_menu(next_app, next_wg, next_pause, chrome_latch);
    NextState::set_if_neq(next_base, BaseState::MainMenu);
    NextState::set_if_neq(next_flow, WorldGenFlowState::Idle);
    NextState::set_if_neq(next_main_menu, MainMenuState::MainMenu);
}
