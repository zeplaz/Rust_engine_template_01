use bevy::prelude::*;

use crate::gui::AppStartState;

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

/// Tooling-only plugin boundary (editor/testing workflows).
pub struct WorldGenerationToolsUiPlugin;

impl Plugin for WorldGenerationToolsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WorldGenUiPlugin, WorldPreviewPlugin));
    }
}

/// Runtime/in-game boundary (no tooling windows).
pub struct WorldGenerationInGamePlugin;

impl Plugin for WorldGenerationInGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldGeneratorPlugin);
    }
}

// A simple keyboard input system to toggle the world gen UI
pub fn world_gen_key_input(
    mut toggle_events: MessageWriter<crate::gui::editor::world_gen_ui::ToggleWorldGenUiEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bindings: Res<crate::gui::InputBindings>,
) {
    if keyboard_input.just_pressed(bindings.toggle_world_generator) {
        toggle_events.write(crate::gui::editor::world_gen_ui::ToggleWorldGenUiEvent);
    }
}

// System set for adding the world generation tools to the game
pub struct WorldGenToolsPlugin;

impl Plugin for WorldGenToolsPlugin {
    fn build(&self, app: &mut App) {
        // Explicitly keep tools UI in this plugin; in-game runtime uses WorldGenerationInGamePlugin.
        app.add_plugins((WorldGenerationInGamePlugin, WorldGenerationToolsUiPlugin))
            .add_systems(
                Update,
                world_gen_key_input.run_if(not(in_state(AppStartState::Splash))),
            );
        
        info!("World Generation Tools initialized. Default key for UI is F8; change under Options → key bindings when KeybindingsOptionsPlugin is loaded.");
    }
}