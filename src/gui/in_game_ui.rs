// LEGACY MODULE (not actively wired):
// This file references:
//   - `Visible` (removed in Bevy 0.11; replaced by `Visibility`)
//   - `crate::gui::gui_state::InGameUiPluginState` (module does not exist)
//   - `add_system(x.run_if())` (0.10/0.11 pattern)
//   - `in_game_menu_state.current()` (old State<S> API)
//
// TODO: rewrite with:
//   - `Visibility` component
//   - `add_systems(Update, x.run_if(in_state(InGameMenuState::X)))`
//   - Canonical InGameMenuState from `engine::states`

use bevy::prelude::*;

pub struct InGameUiPlugin;

impl Plugin for InGameUiPlugin {
    fn build(&self, _app: &mut App) {
        // Intentionally empty: systems have been removed pending rewrite.
    }
}
