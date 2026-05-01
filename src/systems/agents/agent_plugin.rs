use bevy::prelude::*;

use crate::systems::agents::permissions::AgentPermissionsPlugin;
use crate::systems::agents::agent_manager::AgentManagerPlugin;
use crate::systems::agents::multiplayer::MultiplayerPlugin;
use crate::events::ownership_events::OwnershipPlugin;
use crate::gui::AgentPermissionsUiPlugin;

/// Master plugin for all agent-related functionality
pub struct AgentSystemsPlugin;

impl Plugin for AgentSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // Core agent systems
            AgentPermissionsPlugin,
            AgentManagerPlugin,
            
            // Ownership and events
            OwnershipPlugin,
            
            // Multiplayer functionality
            MultiplayerPlugin,
            
            // UI
            AgentPermissionsUiPlugin,
        ));
        
        info!("Agent systems initialized. Press F7 to open the Agent Permissions UI.");
    }
}

/// System to toggle the permissions UI with a keyboard shortcut
pub fn permissions_ui_shortcut(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bindings: Res<crate::gui::InputBindings>,
    mut toggle_events: MessageWriter<crate::gui::TogglePermissionsUiEvent>,
) {
    if keyboard_input.just_pressed(bindings.toggle_agent_permissions) {
        toggle_events.write(crate::gui::TogglePermissionsUiEvent);
    }
}