use crate::engine::{BaseState, MainMenuState};
use crate::gui::ui_windows::*;
use bevy::app::AppExit;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

pub struct BaseMenuPlugin;

impl Plugin for BaseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MainMenuState>()
            .add_systems(OnEnter(MainMenuState::MainMenu), main_menu_ui_setup)
            // Egui rendering goes in EguiPrimaryContextPass (bevy_egui 0.39)
            .add_systems(
                EguiPrimaryContextPass,
                main_menu_ui_system.run_if(in_state(BaseState::MainMenu)),
            );
    }
}

fn main_menu_ui_setup(
    mut commands: Commands,
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
    mut ui_state: ResMut<UiState>,
    mut contexts: EguiContexts,
    mut next_base: ResMut<NextState<BaseState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut app_exit_events: MessageWriter<AppExit>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let egui_texture_handle = ui_state
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
            if ui.button("Debug_Enter").clicked() {
                // Bevy 0.18: `set` re-runs OnEnter/OnExit even when already in state; prefer `set_if_neq`.
                NextState::set_if_neq(&mut *next_base, BaseState::Simulation);
            }
            if ui.button("Load World").clicked() {
                NextState::set_if_neq(&mut *next_menu, MainMenuState::Load);
            }
            if ui.button("Load Editor").clicked() {
                NextState::set_if_neq(&mut *next_base, BaseState::Editor);
            }
            if ui.button("Quit").clicked() {
                app_exit_events.write(AppExit::Success);
            }
        });
    });
    Ok(())
}
