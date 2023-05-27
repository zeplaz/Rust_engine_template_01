use crate::gui::gui_state::InGameUiPluginState;
use crate::gui::main_menu::MenuStatePlugin;

pub struct InGameUiPlugin;

impl Plugin for InGameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_ui_visibility.run_if())
    }
}

fn update_ui_visibility(
    mut query: Query<(&UIRegion, &mut Visible)>,
    in_game_menu_state: Res<State<InGameMenuState>>,
) {
    for (ui_region, mut visible) in query.iter_mut() {
        match *ui_region {
            UIRegion::EntityMenu => {
                visible.is_visible = *in_game_menu_state.current() == InGameMenuState::EntityMenu;
            }
            // Add cases for other UIRegions as needed
            _ => {}
        }
    }
}

fn in_game_pause_ui_system() {}

fn in_game_settings_ui_system() {}

fn if_entity_selected_ui_sytem() {}
