use crate::engine::{BaseState, MainMenuState, WorldGenFlowState};
use crate::gui::ui_windows::*;
use crate::gui::AppStartState;
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, WorldMarker,
};
use bevy::app::AppExit;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

pub struct BaseMenuPlugin;

impl Plugin for BaseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<BaseState>()
            .init_state::<MainMenuState>()
            // Shared egui menu state (`crate::gui::ui_windows::UiState`); must exist before `OnEnter(MainMenuState::MainMenu)`.
            .init_resource::<UiState>()
            .add_systems(OnEnter(MainMenuState::MainMenu), main_menu_ui_setup)
            // Egui rendering goes in EguiPrimaryContextPass (bevy_egui 0.39)
            .add_systems(
                EguiPrimaryContextPass,
                main_menu_ui_system.run_if(
                    in_state(AppStartState::Menu).and(in_state(BaseState::MainMenu)),
                ),
            )
            .add_systems(
                EguiPrimaryContextPass,
                load_menu_ui_system.run_if(
                    in_state(AppStartState::Menu)
                        .and(in_state(BaseState::MainMenu))
                        .and(in_state(MainMenuState::Load)),
                ),
            );
    }
}

fn main_menu_ui_setup(
    _commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_state: ResMut<UiState>,
) {
    let font_handle = asset_server.load("fonts/FiraMono-Medium.ttf");
    ui_state.font_handle = Some(font_handle);
    ui_state.menu_text_color = Color::srgb(0.9, 0.9, 0.9);    // explicit sRGB
    ui_state.menu_background_color = Color::srgb(0.2, 0.2, 0.2);
}

fn main_menu_ui_system(
    _primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    world_roots: Query<Entity, With<WorldMarker>>,
    mut ui_state: ResMut<UiState>,
    mut contexts: EguiContexts,
    mut next_base: ResMut<NextState<BaseState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut next_world_flow: ResMut<NextState<WorldGenFlowState>>,
    mut app_exit_events: MessageWriter<AppExit>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let _egui_texture_handle = ui_state
        .egui_texture_handle
        .get_or_insert_with(|| {
            ctx.load_texture(
                "example-image",
                egui::ColorImage::example(),
                Default::default(),
            )
        })
        .clone();

    egui::TopBottomPanel::top("main_menu_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("New World").clicked() {
                despawn_generated_world_entities(&mut commands, &world_roots);
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::NewWorldSetup);
            }
            if ui.button("Debug_Enter").clicked() {
                // Bevy 0.18: `set` re-runs OnEnter/OnExit even when already in state; prefer `set_if_neq`.
                NextState::set_if_neq(&mut *next_base, BaseState::Simulation);
            }
            if ui.button("Load World").clicked() {
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::LoadingSave);
                NextState::set_if_neq(&mut *next_menu, MainMenuState::Load);
                info!(
                    "Load World: procedural generation is disabled in this flow; deserialize when saves are wired."
                );
            }
            if ui.button("New map in editor").clicked() {
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::Idle);
                NextState::set_if_neq(&mut *next_base, BaseState::Editor);
            }
            if ui.button("Quit").clicked() {
                app_exit_events.write(AppExit::Success);
            }
        });
    });
    Ok(())
}

/// Placeholder load screen: path field + stub entry into simulation (no deserialization yet).
fn load_menu_ui_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    world_roots: Query<Entity, With<WorldMarker>>,
    mut next_base: ResMut<NextState<BaseState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut next_world_flow: ResMut<NextState<WorldGenFlowState>>,
    mut path_buf: Local<String>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    if path_buf.is_empty() {
        *path_buf = "saves/slot_0.ron".to_string();
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Load World");
        ui.label("Stub: set path below. Real build will use a native/OS file picker, validate, then deserialize into ECS.");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.add(egui::TextEdit::singleline(&mut *path_buf).desired_width(320.0));
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button("Cancel").clicked() {
                NextState::set_if_neq(&mut *next_menu, MainMenuState::MainMenu);
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::Idle);
            }
            if ui.button("Open saved map in editor (stub)").clicked() {
                info!(
                    "Open saved map in editor (stub): no file read yet — would load {:?}; entering editor shell until M5 hydrate.",
                    path_buf.as_str()
                );
                despawn_generated_world_entities(&mut commands, &world_roots);
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::Idle);
                NextState::set_if_neq(&mut *next_base, BaseState::Editor);
                NextState::set_if_neq(&mut *next_menu, MainMenuState::MainMenu);
            }
            if ui.button("Load into game (stub)").clicked() {
                despawn_generated_world_entities(&mut commands, &world_roots);
                info!(
                    "Load stub: no file read yet — would load {:?}; entering simulation, world gen flow Idle.",
                    path_buf.as_str()
                );
                NextState::set_if_neq(&mut *next_world_flow, WorldGenFlowState::Idle);
                NextState::set_if_neq(&mut *next_base, BaseState::Simulation);
                NextState::set_if_neq(&mut *next_menu, MainMenuState::MainMenu);
            }
        });
    });
    Ok(())
}
