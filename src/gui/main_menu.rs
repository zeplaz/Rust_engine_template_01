use crate::engine::{BaseState, MainMenuState};
use crate::gui::ui_windows::UiState;
use bevy::prelude::*;

/// Registers **core menu-related state** + shared [`UiState`] (fonts / legacy hooks).
/// Main menu **layout** lives in [`super::app_shell::AppShellPlugin`](crate::gui::AppShellPlugin).
pub struct BaseMenuPlugin;

impl Plugin for BaseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<BaseState>()
            .init_state::<MainMenuState>()
            .init_resource::<UiState>();
    }
}
