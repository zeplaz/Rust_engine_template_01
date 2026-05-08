use bevy::prelude::*;
use std::collections::HashMap;

use crate::idgen::EntityId;
use crate::events::ownership_events::FactionColors;
use crate::systems::agents::permissions::{
    Agent, AgentType, AgentPermissions, PermissionDomain, AccessLevel,
    PermissionGrantEvent, PermissionRevokeEvent,
};

/// Resource that manages all agents in the game
#[derive(Resource)]
pub struct AgentManager {
    pub agents: HashMap<EntityId, Entity>,
    pub human_players: Vec<EntityId>,
    pub ai_players: Vec<EntityId>,
}

impl Default for AgentManager {
    fn default() -> Self {
        Self {
            agents: HashMap::new(),
            human_players: Vec::new(),
            ai_players: Vec::new(),
        }
    }
}

impl AgentManager {
    /// Register a new agent with the manager
    pub fn register_agent(&mut self, agent_id: EntityId, entity: Entity, agent_type: AgentType) {
        self.agents.insert(agent_id, entity);
        
        // Add to appropriate list
        match agent_type {
            AgentType::HumanPlayer => self.human_players.push(agent_id),
            AgentType::AIPlayer => self.ai_players.push(agent_id),
            _ => {} // No special list for other types
        }
    }
    
    /// Get the entity for an agent
    pub fn get_agent_entity(&self, agent_id: EntityId) -> Option<Entity> {
        self.agents.get(&agent_id).copied()
    }
    
    /// Remove an agent from the manager
    pub fn remove_agent(&mut self, agent_id: EntityId) {
        self.agents.remove(&agent_id);
        self.human_players.retain(|id| *id != agent_id);
        self.ai_players.retain(|id| *id != agent_id);
    }
}

/// Command for creating a new agent
#[derive(Debug, Clone, Component)]
pub struct CreateAgentCommand {
    pub name: String,
    pub faction_id: Option<EntityId>,
    pub agent_type: AgentType,
    pub color: Option<Color>,
    pub permissions: Option<AgentPermissions>,
}

/// System that handles agent creation
pub fn create_agent_system(
    mut commands: Commands,
    mut agent_manager: ResMut<AgentManager>,
    mut faction_colors: ResMut<FactionColors>,
    create_agent_requests: Query<(Entity, &CreateAgentCommand)>,
) {
    for (request_entity, cmd) in create_agent_requests.iter() {
        // Generate a new entity ID
        let agent_id = EntityId::new();
        
        // Determine faction ID (defaults to agent's own ID)
        let faction_id = cmd.faction_id.unwrap_or(agent_id);
        
        // Determine color (defaults to faction color or grey)
        let color = cmd.color.unwrap_or_else(|| faction_colors.get_color(faction_id));
        
        // Register the color
        faction_colors.register_color(agent_id, color);
        
        // Determine permissions
        let permissions = match &cmd.permissions {
            Some(perms) => perms.clone(),
            None => match cmd.agent_type {
                AgentType::HumanPlayer | AgentType::AIPlayer => AgentPermissions::new_owner(),
                AgentType::Delegate => {
                    // Delegates get limited permissions by default
                    let mut perms = AgentPermissions::default();
                    
                    // Add some basic permissions
                    perms.grant_permission(
                        crate::systems::agents::permissions::PermissionGrant {
                            domain: PermissionDomain::Observer,
                            access_level: AccessLevel::Full,
                            grantor: faction_id,
                            grant_time: 0.0,
                            expiration: None,
                            region_limited: None,
                            entity_limited: None,
                            condition: None,
                        },
                        faction_id
                    );
                    
                    perms
                },
                AgentType::Observer => AgentPermissions::new_observer(),
            },
        };
        
        // Create the agent entity
        let agent_entity = commands.spawn((
            Agent {
                name: cmd.name.clone(),
                faction_id,
                agent_type: cmd.agent_type,
                color,
            },
            permissions,
        )).id();
        
        // Register with the agent manager
        agent_manager.register_agent(agent_id, agent_entity, cmd.agent_type);
        
        // Remove the request entity
        commands.entity(request_entity).despawn();
    }
}

/// System that updates the game state based on agent permissions
pub fn agent_authority_system(
    _agent_manager: Res<AgentManager>,
    _agent_query: Query<(&Agent, &AgentPermissions)>,
) {
    // This system would validate actions against permissions
    // For example:
    // - Check if an agent can build in a region
    // - Check if an agent can issue commands to units
    // - Check if an agent can modify economic policies
    
    // The implementation would depend on the specific game mechanics
}

/// Command to delegate authority from one agent to another
#[derive(Debug, Clone, Component)]
pub struct DelegateAuthorityCommand {
    pub from_agent_id: EntityId,
    pub to_agent_id: EntityId,
    pub domain: PermissionDomain,
    pub access_level: AccessLevel,
    pub expiration: Option<f64>,
    pub region_limited: Option<Vec<EntityId>>,
    pub entity_limited: Option<Vec<EntityId>>,
}

/// System that handles authority delegation
pub fn handle_delegation_system(
    mut commands: Commands,
    agent_manager: Res<AgentManager>,
    mut agent_query: Query<&mut AgentPermissions>,
    delegation_requests: Query<(Entity, &DelegateAuthorityCommand)>,
    game_time: Res<crate::systems::agents::permissions::GameTime>,
    mut permission_events: MessageWriter<crate::systems::agents::permissions::PermissionGrantEvent>,
) {
    for (request_entity, cmd) in delegation_requests.iter() {
        // Get the delegating agent's permissions
        if let Some(from_entity) = agent_manager.get_agent_entity(cmd.from_agent_id) {
            if let Ok(mut from_permissions) = agent_query.get_mut(from_entity) {
                // Convert region and entity limitations
                let region_limited = cmd.region_limited.clone().map(|ids| ids.into_iter().collect());
                let entity_limited = cmd.entity_limited.clone().map(|ids| ids.into_iter().collect());
                
                // Attempt to delegate
                if let Some(grant) = from_permissions.delegate_permission(
                    cmd.to_agent_id,
                    cmd.domain,
                    cmd.access_level,
                    game_time.current_time,
                    cmd.expiration,
                    region_limited,
                    entity_limited,
                    None, // No condition for now
                ) {
                    // If successful, send a grant event
                    permission_events.write(crate::systems::agents::permissions::PermissionGrantEvent {
                        to_agent_id: cmd.to_agent_id,
                        from_agent_id: cmd.from_agent_id,
                        grant,
                    });
                }
            }
        }
        
        // Remove the request entity
        commands.entity(request_entity).despawn();
    }
}

/// Bridge `EntityId` → Bevy `Entity` using [`AgentManager`], then apply grant payloads.
pub fn handle_permission_grants(
    mut events: MessageReader<PermissionGrantEvent>,
    mut agent_query: Query<&mut AgentPermissions>,
    agent_manager: Res<AgentManager>,
) {
    for event in events.read() {
        if let Some(entity) = agent_manager.get_agent_entity(event.to_agent_id) {
            if let Ok(mut permissions) = agent_query.get_mut(entity) {
                permissions.grant_permission(event.grant.clone(), event.from_agent_id);
            }
        }
    }
}

pub fn handle_permission_revokes(
    mut events: MessageReader<PermissionRevokeEvent>,
    mut agent_query: Query<&mut AgentPermissions>,
    agent_manager: Res<AgentManager>,
) {
    for event in events.read() {
        if let Some(entity) = agent_manager.get_agent_entity(event.from_agent_id) {
            if let Ok(mut permissions) = agent_query.get_mut(entity) {
                permissions.revoke_permission(event.domain, event.revoker_id);
            }
        }
    }
}

/// Agent Manager Plugin to register all agent-related systems
pub struct AgentManagerPlugin;

impl Plugin for AgentManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AgentManager>()
           .add_systems(Update, (
               create_agent_system,
               handle_delegation_system,
               agent_authority_system,
               handle_permission_grants,
               handle_permission_revokes,
           ));
    }
}