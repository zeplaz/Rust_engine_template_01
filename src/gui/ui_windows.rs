use bevy::{prelude::*, window::PrimaryWindow};

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
    pub egui_texture_handle: Option<()>,
    pub is_window_open: bool,
    pub font_handle: Option<Handle<Font>>,
    pub button_texture_handle: Option<()>,
    pub background_texture_handle: Option<()>,
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

/// Deprecated: global egui theme is applied by [`crate::gui::UiThemePlugin`] from [`crate::gui::UiPalette`].
pub fn configure_visuals_system() -> Result { Ok(()) }

pub fn configure_ui_state_system(mut ui_state: ResMut<UiState>) {
    ui_state.is_window_open = true;
}

pub fn update_ui_scale_factor_system() {}
