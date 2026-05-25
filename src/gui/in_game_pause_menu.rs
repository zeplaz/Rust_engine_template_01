//! In-game pause menu (Escape) — simulation shell overlay distinct from sim pause (P).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::engine::states::{InGameMenuState, MainMenuState, WorldGenFlowState};
use crate::engine::{AppState, PauseState, WorldGenChromeLatch, WorldGenState};
use crate::gui::editor::world_gen_ui::{CancelActiveWorldGenEvent, WorldGenUiState};
use crate::gui::editor::world_preview::WorldPreviewUiState;
use crate::gui::pause_menu_confirm::{
    execute_pause_exit_to_main_menu, execute_pause_exit_to_world_gen, PauseMenuConfirm,
    PauseMenuPendingAction, world_gen_work_active,
};
use crate::gui::std_floating;
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::gui::InputBindings;
use crate::terrain::generation::world_generator_enhanced::{
    WorldGenJobSlot, WorldGenProgress, WorldMarker,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PauseMenuChoice {
    Resume,
    Save,
    Load,
    WorldGenerator,
    MainMenu,
    Quit,
}

#[derive(SystemParam)]
pub(crate) struct PauseMenuNavParams<'w, 's> {
    pending: ResMut<'w, PauseMenuPendingAction>,
    next_menu: ResMut<'w, NextState<InGameMenuState>>,
    next_base: ResMut<'w, NextState<crate::engine::states::BaseState>>,
    next_flow: ResMut<'w, NextState<WorldGenFlowState>>,
    next_main_menu: ResMut<'w, NextState<MainMenuState>>,
    next_app: ResMut<'w, NextState<AppState>>,
    next_wg: ResMut<'w, NextState<WorldGenState>>,
    next_pause: ResMut<'w, NextState<PauseState>>,
    commands: Commands<'w, 's>,
    world_roots: Query<'w, 's, Entity, With<WorldMarker>>,
    cancel_world_gen: MessageWriter<'w, CancelActiveWorldGenEvent>,
    app_exit: MessageWriter<'w, AppExit>,
}

#[derive(SystemParam)]
pub(crate) struct PauseMenuWorldGenParams<'w> {
    chrome_latch: ResMut<'w, WorldGenChromeLatch>,
    world_gen_ui: ResMut<'w, WorldGenUiState>,
    world_preview_ui: ResMut<'w, WorldPreviewUiState>,
    job_slot: Res<'w, WorldGenJobSlot>,
    progress: Res<'w, WorldGenProgress>,
}

pub fn toggle_pause_menu_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    menu: Res<State<InGameMenuState>>,
    mut next_menu: ResMut<NextState<InGameMenuState>>,
    mut pending: ResMut<PauseMenuPendingAction>,
) {
    if !keys.just_pressed(bindings.cancel_keybinding_capture) {
        return;
    }
    if *menu.get() == InGameMenuState::Pause {
        NextState::set_if_neq(&mut *next_menu, InGameMenuState::Normal);
        pending.clear();
    } else {
        NextState::set_if_neq(&mut *next_menu, InGameMenuState::Pause);
    }
}

pub(crate) fn pause_menu_egui_system(
    mut contexts: EguiContexts,
    menu: Res<State<InGameMenuState>>,
    mut nav: PauseMenuNavParams,
    mut wg: PauseMenuWorldGenParams,
) -> Result {
    if *menu.get() != InGameMenuState::Pause {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    if nav.pending.confirm != PauseMenuConfirm::None {
        let (title, body, confirm_label) = match nav.pending.confirm {
            PauseMenuConfirm::ExitToWorldGen => (
                "Leave current world?",
                "Opening World Generator will exit the current simulation world. Unsaved progress may be lost.",
                "Exit to World Generator",
            ),
            PauseMenuConfirm::ExitToMainMenu => (
                "Return to main menu?",
                if world_gen_work_active(&wg.job_slot, &wg.progress) {
                    "World generation is still running. Returning to the main menu will cancel generation and discard partial work."
                } else {
                    "Return to the main menu and exit the current simulation world. Unsaved progress may be lost."
                },
                "Return to Main Menu",
            ),
            PauseMenuConfirm::None => unreachable!(),
        };
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(body);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        nav.pending.clear();
                    }
                    if ui.button(confirm_label).clicked() {
                        match nav.pending.confirm {
                            PauseMenuConfirm::ExitToWorldGen => {
                                execute_pause_exit_to_world_gen(
                                    &mut nav.commands,
                                    &nav.world_roots,
                                    &mut wg.chrome_latch,
                                    &mut nav.next_base,
                                    &mut nav.next_flow,
                                    &mut nav.next_app,
                                    &mut nav.next_wg,
                                    &mut nav.next_pause,
                                    &mut wg.world_gen_ui,
                                    &mut wg.world_preview_ui,
                                    &mut nav.cancel_world_gen,
                                    &wg.job_slot,
                                    &wg.progress,
                                );
                            }
                            PauseMenuConfirm::ExitToMainMenu => {
                                execute_pause_exit_to_main_menu(
                                    &mut nav.commands,
                                    &nav.world_roots,
                                    &mut wg.chrome_latch,
                                    &mut nav.next_base,
                                    &mut nav.next_flow,
                                    &mut nav.next_main_menu,
                                    &mut nav.next_app,
                                    &mut nav.next_wg,
                                    &mut nav.next_pause,
                                    &mut nav.cancel_world_gen,
                                    &wg.job_slot,
                                    &wg.progress,
                                );
                            }
                            PauseMenuConfirm::None => {}
                        }
                        nav.pending.clear();
                        NextState::set_if_neq(&mut *nav.next_menu, InGameMenuState::Normal);
                    }
                });
            });
        return Ok(());
    }

    std_floating(egui::Window::new("Paused"))
        .collapsible(false)
        .default_width(280.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Simulation paused (menu). Sim tick pause is separate (P).");
            ui.separator();
            for (label, choice) in [
                ("Resume", PauseMenuChoice::Resume),
                ("Save game (stub)", PauseMenuChoice::Save),
                ("Load game (stub)", PauseMenuChoice::Load),
                ("World Generator…", PauseMenuChoice::WorldGenerator),
                ("Return to Main Menu", PauseMenuChoice::MainMenu),
                ("Exit program", PauseMenuChoice::Quit),
            ] {
                if ui.button(label).clicked() {
                    match choice {
                        PauseMenuChoice::Resume => {
                            NextState::set_if_neq(&mut *nav.next_menu, InGameMenuState::Normal);
                        }
                        PauseMenuChoice::Save => {
                            info!("Pause menu: Save — wire to WorldSaveSpine when ready.");
                        }
                        PauseMenuChoice::Load => {
                            NextState::set_if_neq(&mut *nav.next_flow, WorldGenFlowState::LoadingSave);
                            NextState::set_if_neq(&mut *nav.next_main_menu, MainMenuState::Load);
                            NextState::set_if_neq(&mut *nav.next_menu, InGameMenuState::Normal);
                        }
                        PauseMenuChoice::WorldGenerator => {
                            nav.pending.confirm = PauseMenuConfirm::ExitToWorldGen;
                        }
                        PauseMenuChoice::MainMenu => {
                            if world_gen_work_active(&wg.job_slot, &wg.progress) {
                                nav.pending.confirm = PauseMenuConfirm::ExitToMainMenu;
                            } else {
                                execute_pause_exit_to_main_menu(
                                    &mut nav.commands,
                                    &nav.world_roots,
                                    &mut wg.chrome_latch,
                                    &mut nav.next_base,
                                    &mut nav.next_flow,
                                    &mut nav.next_main_menu,
                                    &mut nav.next_app,
                                    &mut nav.next_wg,
                                    &mut nav.next_pause,
                                    &mut nav.cancel_world_gen,
                                    &wg.job_slot,
                                    &wg.progress,
                                );
                                NextState::set_if_neq(&mut *nav.next_menu, InGameMenuState::Normal);
                            }
                        }
                        PauseMenuChoice::Quit => {
                            nav.app_exit.write(AppExit::Success);
                        }
                    }
                }
            }
        });
    Ok(())
}

pub struct InGamePauseMenuPlugin;

impl Plugin for InGamePauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InGameMenuState>()
            .init_resource::<PauseMenuPendingAction>()
            .add_systems(
                Update,
                toggle_pause_menu_on_escape.run_if(in_simulation_or_editor),
            )
            .add_systems(EguiPrimaryContextPass, pause_menu_egui_system);
    }
}
