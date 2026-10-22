use bevy::prelude::*;

use crate::idgen::EntityId;
use crate::systems::agents::permissions::PermissionDomain;

/// UI state for the permissions window
#[derive(Resource)]
pub struct PermissionsUiState {
    pub visible: bool,
    pub selected_agent: Option<EntityId>,
    pub selected_target: Option<EntityId>,
    pub filter_domain: Option<PermissionDomain>,
    pub domain_scroll_pos: f32,
    pub temp_grant_expiration: Option<f64>,
}

impl Default for PermissionsUiState {
    fn default() -> Self {
        Self {
            visible: false,
            selected_agent: None,
            selected_target: None,
            filter_domain: None,
            domain_scroll_pos: 0.0,
            temp_grant_expiration: None,
        }
    }
}

/// Event to toggle the permissions UI
#[derive(Message)]
pub struct TogglePermissionsUiEvent;

/// System to handle UI visibility toggle
pub fn toggle_permissions_ui(
    mut events: MessageReader<TogglePermissionsUiEvent>,
    mut ui_state: ResMut<PermissionsUiState>,
) {
    for _ in events.read() {
        ui_state.visible = !ui_state.visible;
    }
}

/// System to render the permissions UI — RetiredEgui, returns Result.
#[allow(clippy::too_many_arguments)]
pub fn permissions_ui_system() -> Result { Ok(()) }

/// Resource to track the local player's agent
#[derive(Resource)]
pub struct LocalPlayerAgent {
    pub agent_id: EntityId,
}

/// Plugin for agent permissions UI
pub struct AgentPermissionsUiPlugin;

impl Plugin for AgentPermissionsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PermissionsUiState>()
           .add_message::<TogglePermissionsUiEvent>()
           .add_systems(Update, toggle_permissions_ui);
    }
}