use crate::gui::ui_windows::*;
//use editor::;
use crate::engine::states::*;
use bevy::app::AppExit;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts, EguiSettings};

pub struct BaseMenuPlugin;

impl Plugin for BaseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MainMenuState>()
        .add_system(main_menu_ui_setup.in_schedule(OnEnter(MainMenuState::MainMenu)))
        .add_system(main_menu_ui_system.in_set(OnUpdate(BaseState::MainMenu)));
    }
}
//.add_state::<MainMenuState>()
fn main_menu_ui_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ui_state: ResMut<UiState>,
) {
    // Load the necessary fonts, textures, or other assets for the main menu UI
    let font_handle = asset_server.load("data/assets/fonts/FiraMono-Medium.ttf");
    ui_state.font_handle = Some(font_handle);

    // Load any necessary sound effects or music for the main menu

    //ui_state.menu_music_handle = Some(menu_music_handle);
    //ui_state.button_click_sound_handle = Some(button_click_sound_handle);

    // Configure any additional UI elements or settings, such as colors or layout preferences
    ui_state.menu_text_color = Color::rgb(0.9, 0.9, 0.9);
    ui_state.menu_background_color = Color::rgb(0.2, 0.2, 0.2);
}

fn main_menu_ui_system(
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    mut ui_state: ResMut<UiState>,
    mut contexts: EguiContexts,
    mut state: ResMut<State<MainMenuState>>,
    mut app_exit_events: EventWriter<AppExit>,
    mut commands: Commands,
) {
    let mut open_editor = false;
    let mut quit = false;
    let ctx = contexts.ctx_mut();

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
                commands.insert_resource(NextState(Some(BaseState::Simuation)));
            }
            if ui.button("Load World").clicked() {
                commands.insert_resource(NextState(Some(MainMenuState::Load)));
            }

            if ui.button("Load Editor").clicked() {
                open_editor = true;
            }

            if ui.button("Quit").clicked() {
                quit = true;
            }
        });
    });

    if quit {
        if let Ok(primary_window) = primary_window_query.get_single() {
            app_exit_events.send(AppExit);
        }
    }
}
