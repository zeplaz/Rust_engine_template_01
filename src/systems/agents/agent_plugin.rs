use bevy::prelude::*;

use crate::systems::agents::permissions::AgentPermissionsPlugin;
use crate::systems::agents::agent_manager::AgentManagerPlugin;
use crate::systems::agents::multiplayer::MultiplayerPlugin;
use crate::events::ownership_events::OwnershipPlugin;

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
        ));
    }
}
