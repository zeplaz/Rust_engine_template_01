use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::idgen::EntityId;
use crate::systems::agents::permissions::{
    Agent, AgentType, AgentPermissions, PermissionDomain, AccessLevel, PermissionGrant
};
use crate::systems::agents::agent_manager::{AgentManager, CreateAgentCommand};

/// Represents a player connection in a multiplayer game
#[derive(Component)]
pub struct PlayerConnection {
    pub player_id: String,           // Unique ID for this player/connection
    pub agent_id: Option<EntityId>,  // The agent entity assigned to this player
    pub connected: bool,             // Whether the player is currently connected
}

/// Keeps track of all player connections
#[derive(Resource, Default)]
pub struct MultiplayerManager {
    pub connections: HashMap<String, Entity>,
    pub player_names: HashMap<String, String>,
}

/// Command to add a new player to the game
#[derive(Debug, Clone, Component)]
pub struct AddPlayerCommand {
    pub player_id: String,
    pub player_name: String,
    pub faction_id: Option<EntityId>,
    pub color: Option<Color>,
    pub permissions: Option<AgentPermissions>,
}

/// System that handles adding new players
pub fn add_player_system(
    mut commands: Commands,
    mut multiplayer_manager: ResMut<MultiplayerManager>,
    add_player_commands: Query<(Entity, &AddPlayerCommand)>,
) {
    for (cmd_entity, cmd) in add_player_commands.iter() {
        // Create a player connection
        let connection_entity = commands.spawn(PlayerConnection {
            player_id: cmd.player_id.clone(),
            agent_id: None,
            connected: true,
        }).id();
        
        // Register the connection
        multiplayer_manager.connections.insert(cmd.player_id.clone(), connection_entity);
        multiplayer_manager.player_names.insert(cmd.player_id.clone(), cmd.player_name.clone());
        
        // Create an agent for this player
        let agent_cmd = CreateAgentCommand {
            name: cmd.player_name.clone(),
            faction_id: cmd.faction_id,
            agent_type: AgentType::HumanPlayer,
            color: cmd.color,
            permissions: cmd.permissions.clone(),
        };
        
        // Spawn the create agent command
        commands.spawn(agent_cmd);
        
        // Remove the add player command
        commands.entity(cmd_entity).despawn();
    }
}

/// Event for player connection changes
#[derive(Message)]
pub struct PlayerConnectionEvent {
    pub player_id: String,
    pub connected: bool,
}

/// System to handle player connection events
pub fn handle_player_connections(
    mut events: MessageReader<PlayerConnectionEvent>,
    mut connection_query: Query<&mut PlayerConnection>,
    multiplayer_manager: Res<MultiplayerManager>,
) {
    for event in events.read() {
        if let Some(&connection_entity) = multiplayer_manager.connections.get(&event.player_id) {
            if let Ok(mut connection) = connection_query.get_mut(connection_entity) {
                connection.connected = event.connected;
                
                if event.connected {
                    info!("Player {} connected", event.player_id);
                } else {
                    info!("Player {} disconnected", event.player_id);
                }
            }
        }
    }
}

/// Command to assign an agent to a player
#[derive(Debug, Clone, Component)]
pub struct AssignAgentCommand {
    pub player_id: String,
    pub agent_id: EntityId,
}

/// System to handle agent assignments
pub fn assign_agent_system(
    mut commands: Commands,
    mut connection_query: Query<&mut PlayerConnection>,
    multiplayer_manager: Res<MultiplayerManager>,
    agent_manager: Res<AgentManager>,
    assign_commands: Query<(Entity, &AssignAgentCommand)>,
) {
    for (cmd_entity, cmd) in assign_commands.iter() {
        if let Some(&connection_entity) = multiplayer_manager.connections.get(&cmd.player_id) {
            if let Ok(mut connection) = connection_query.get_mut(connection_entity) {
                // Ensure the agent exists
                if agent_manager.get_agent_entity(cmd.agent_id).is_some() {
                    connection.agent_id = Some(cmd.agent_id);
                }
            }
        }
        
        // Remove the command
        commands.entity(cmd_entity).despawn();
    }
}

/// Command to create an AI delegate for a player
#[derive(Debug, Clone, Component)]
pub struct CreateDelegateCommand {
    pub player_id: String,
    pub delegate_name: String,
    pub permissions: Vec<(PermissionDomain, AccessLevel)>,
    pub region_limited: Option<Vec<EntityId>>,
    pub entity_limited: Option<Vec<EntityId>>,
    pub expiration: Option<f64>,
}

/// System to handle delegate creation
pub fn create_delegate_system(
    mut commands: Commands,
    connection_query: Query<&PlayerConnection>,
    multiplayer_manager: Res<MultiplayerManager>,
    agent_manager: Res<AgentManager>,
    agent_query: Query<&Agent>,
    create_commands: Query<(Entity, &CreateDelegateCommand)>,
    game_time: Res<crate::systems::agents::permissions::GameTime>,
) {
    for (cmd_entity, cmd) in create_commands.iter() {
        // Find the player's connection
        if let Some(&connection_entity) = multiplayer_manager.connections.get(&cmd.player_id) {
            if let Ok(connection) = connection_query.get(connection_entity) {
                if let Some(player_agent_id) = connection.agent_id {
                    if let Some(player_entity) = agent_manager.get_agent_entity(player_agent_id) {
                        if let Ok(player_agent) = agent_query.get(player_entity) {
                            // Create permissions for the delegate
                            let mut permissions = AgentPermissions::default();
                            
                            // Convert region and entity limitations
                            let region_limited = cmd.region_limited.clone().map(|ids| ids.into_iter().collect());
                            let entity_limited = cmd.entity_limited.clone().map(|ids| ids.into_iter().collect());
                            
                            // Add specified permissions
                            for (domain, level) in &cmd.permissions {
                                let grant = PermissionGrant {
                                    domain: *domain,
                                    access_level: *level,
                                    grantor: player_agent_id,
                                    grant_time: game_time.current_time,
                                    expiration: cmd.expiration,
                                    region_limited: region_limited.clone(),
                                    entity_limited: entity_limited.clone(),
                                    condition: None,
                                };
                                
                                permissions.grant_permission(grant, player_agent_id);
                            }
                            
                            // Create the delegate agent
                            let agent_cmd = CreateAgentCommand {
                                name: format!("{}'s Delegate: {}", player_agent.name, cmd.delegate_name),
                                faction_id: Some(player_agent.faction_id),
                                agent_type: AgentType::Delegate,
                                color: Some(player_agent.color),
                                permissions: Some(permissions),
                            };
                            
                            commands.spawn(agent_cmd);
                        }
                    }
                }
            }
        }
        
        // Remove the command
        commands.entity(cmd_entity).despawn();
    }
}

/// Plugin for multiplayer functionality
pub struct MultiplayerPlugin;

impl Plugin for MultiplayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultiplayerManager>()
           .add_message::<PlayerConnectionEvent>()
           .add_systems(Update, (
               add_player_system,
               handle_player_connections,
               assign_agent_system,
               create_delegate_system,
           ));
    }
}