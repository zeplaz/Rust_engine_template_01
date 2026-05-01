use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContextSettings, EguiContexts};

use crate::gui::InputBindings;
// will go in gui:
pub fn run_prompt_getter() {
    //    let mut input_box = InputBoxState::new();
}

#[derive(Default, Resource)]
pub struct UiState {
    pub label: String,
    pub value: f32,
    pub inverted: bool,
    pub egui_texture_handle: Option<egui::TextureHandle>,
    pub is_window_open: bool,
    pub font_handle: Option<Handle<Font>>,
    pub button_texture_handle: Option<egui::TextureHandle>,
    pub background_texture_handle: Option<egui::TextureHandle>,
    pub menu_text_color: Color,
    pub menu_background_color: Color,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum UIRegion {
    TopBar,
    HoverInfo,
    OverlayMenu,
    ContextMenu,
    MiniMap,
    EntityMenu,
    MainWorld,
}

pub fn configure_visuals_system(mut contexts: EguiContexts) -> Result {
    contexts.ctx_mut()?.set_visuals(egui::Visuals {
        ..Default::default()
    });
    Ok(())
}

pub fn configure_ui_state_system(mut ui_state: ResMut<UiState>) {
    ui_state.is_window_open = true;
}

pub fn update_ui_scale_factor_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut egui_ctx: Query<&mut EguiContextSettings>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if keyboard_input.just_pressed(bindings.toggle_egui_ui_scale) || toggle_scale_factor.is_none() {
        *toggle_scale_factor = Some(!toggle_scale_factor.unwrap_or(true));

        if let (Ok(window), Ok(mut settings)) = (windows.single(), egui_ctx.single_mut()) {
            let scale_factor = if toggle_scale_factor.unwrap() {
                1.0
            } else {
                1.0 / window.scale_factor()
            };
            settings.scale_factor = scale_factor;
        }
    }
}
