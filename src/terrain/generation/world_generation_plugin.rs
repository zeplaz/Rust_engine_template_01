use bevy::prelude::*;

use crate::terrain::generation::world_generator_enhanced::WorldGeneratorPlugin;
use crate::gui::editor::world_gen_ui::WorldGenUiPlugin;
use crate::gui::editor::world_preview::WorldPreviewPlugin;

// Main plugin for world generation that combines all the subplugins
pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WorldGeneratorPlugin,
            WorldGenUiPlugin,
            WorldPreviewPlugin,
        ));
    }
}

// A simple keyboard input system to toggle the world gen UI
pub fn world_gen_key_input(
    mut toggle_events: EventWriter<crate::gui::editor::world_gen_ui::ToggleWorldGenUiEvent>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F8) {
        toggle_events.send(crate::gui::editor::world_gen_ui::ToggleWorldGenUiEvent);
    }
}

// System set for adding the world generation tools to the game
pub struct WorldGenToolsPlugin;

impl Plugin for WorldGenToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldGenerationPlugin)
           .add_systems(Update, world_gen_key_input);
        
        info!("World Generation Tools initialized. Press F8 to open the World Generator UI.");
    }
}