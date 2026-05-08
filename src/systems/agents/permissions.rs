use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::idgen::EntityId;

/// Represents the various domains/systems in the game that can have permissions assigned
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionDomain {
    // Economic domains
    EconomicPolicy,       // Setting tax rates, subsidies, etc.
    TradePolicy,          // Setting tariffs, trade agreements
    ResourceManagement,   // Access to manage resources, stockpiles
    Production,           // Building and managing production facilities
    
    // Infrastructure domains
    RoadConstruction,     // Building and managing roads
    RailConstruction,     // Building and managing railways
    PowerInfrastructure,  // Power plants, distribution networks
    WaterInfrastructure,  // Water systems, dams, irrigation
    
    // Military domains
    MilitaryCommand,      // Direct control of military units
    DefenseInfrastructure,// Building defensive structures
    IntelligenceOps,      // Espionage, information gathering
    MilitaryProduction,   // Production of military equipment
    
    // Governance domains
    DiplomaticRelations,  // Managing relations with other factions
    FactionPolicy,        // Setting overall faction policies
    Research,             // Managing research priorities
    PopulationManagement, // Policies affecting population
    
    // Special domains
    Admin,                // Administrative access (can reassign permissions)
    Observer,             // Can observe but not modify anything
}

/// The level of access granted for a particular domain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    None,       // No access
    ReadOnly,   // Can only view information
    Limited,    // Can make some changes with restrictions
    Full,       // Full control over the domain
    Owner,      // Has ownership-level access (can delegate to others)
}

/// Represents a permission grant with optional time limit and conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub domain: PermissionDomain,
    pub access_level: AccessLevel,
    pub grantor: EntityId,             // Who granted this permission
    pub grant_time: f64,               // When the permission was granted
    pub expiration: Option<f64>,       // When the permission expires (if temporary)
    pub region_limited: Option<HashSet<EntityId>>,  // If limited to specific regions
    pub entity_limited: Option<HashSet<EntityId>>,  // If limited to specific entities
    pub condition: Option<PermissionCondition>,     // Optional condition for the permission
}

/// Represents conditional permissions that only activate under certain circumstances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionCondition {
    // Permission only active during wartime
    Wartime { 
        with_factions: Option<HashSet<EntityId>> 
    },
    
    // Permission active only during economic crisis
    EconomicCrisis { 
        threshold: f32 
    },
    
    // Permission requires approval from another agent
    RequiresApproval { 
        approver: EntityId 
    },
    
    // Permission only active when the agent is physically present in the region
    PresenceRequired,
    
    // Permission limited by quotas (e.g. can only build 5 roads)
    Quota { 
        limit: u32, 
        used: u32 
    },
    
    // Permission only when resource levels are above a threshold
    ResourceThreshold { 
        resource_type: String, 
        min_level: f32 
    },
    
    // Custom condition with a description
    Custom { 
        description: String 
    },
}

/// Component that stores all permissions for an agent
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub permissions: HashMap<PermissionDomain, PermissionGrant>,
    pub delegated_to: HashMap<EntityId, Vec<PermissionGrant>>,  // Permissions this agent has delegated to others
    pub delegated_from: HashMap<EntityId, Vec<PermissionGrant>>,  // Permissions delegated to this agent from others
}

impl Default for AgentPermissions {
    fn default() -> Self {
        Self {
            permissions: HashMap::new(),
            delegated_to: HashMap::new(),
            delegated_from: HashMap::new(),
        }
    }
}

impl AgentPermissions {
    /// Create a new AgentPermissions with owner-level access to all domains
    pub fn new_owner() -> Self {
        let mut permissions = HashMap::new();
        
        // Add owner-level access to all domains
        for domain in [
            PermissionDomain::EconomicPolicy,
            PermissionDomain::TradePolicy,
            PermissionDomain::ResourceManagement,
            PermissionDomain::Production,
            PermissionDomain::RoadConstruction,
            PermissionDomain::RailConstruction,
            PermissionDomain::PowerInfrastructure,
            PermissionDomain::WaterInfrastructure,
            PermissionDomain::MilitaryCommand,
            PermissionDomain::DefenseInfrastructure,
            PermissionDomain::IntelligenceOps,
            PermissionDomain::MilitaryProduction,
            PermissionDomain::DiplomaticRelations,
            PermissionDomain::FactionPolicy,
            PermissionDomain::Research,
            PermissionDomain::PopulationManagement,
            PermissionDomain::Admin,
            PermissionDomain::Observer,
        ].iter() {
            permissions.insert(*domain, PermissionGrant {
                domain: *domain,
                access_level: AccessLevel::Owner,
                grantor: EntityId::default(), // Self-granted
                grant_time: 0.0,  // At game start
                expiration: None,
                region_limited: None,
                entity_limited: None,
                condition: None,
            });
        }
        
        Self {
            permissions,
            delegated_to: HashMap::new(),
            delegated_from: HashMap::new(),
        }
    }
    
    /// Create a new AgentPermissions with observer-only access
    pub fn new_observer() -> Self {
        let mut permissions = HashMap::new();
        
        // Add observer access only
        permissions.insert(PermissionDomain::Observer, PermissionGrant {
            domain: PermissionDomain::Observer,
            access_level: AccessLevel::Full,
            grantor: EntityId::default(),
            grant_time: 0.0,
            expiration: None,
            region_limited: None,
            entity_limited: None,
            condition: None,
        });
        
        Self {
            permissions,
            delegated_to: HashMap::new(),
            delegated_from: HashMap::new(),
        }
    }
    
    /// Check if the agent has at least the specified access level for a domain
    pub fn has_permission(&self, domain: PermissionDomain, min_level: AccessLevel, game_time: f64) -> bool {
        if let Some(grant) = self.permissions.get(&domain) {
            // Check if the permission has expired
            if let Some(expiration) = grant.expiration {
                if game_time > expiration {
                    return false;
                }
            }
            
            // Check if the access level is sufficient
            match (grant.access_level, min_level) {
                (AccessLevel::None, _) => false,
                (_, AccessLevel::None) => true,
                (a, b) if a as u8 >= b as u8 => true,
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Check if the agent has permission for a specific entity
    pub fn has_entity_permission(&self, domain: PermissionDomain, entity_id: EntityId, min_level: AccessLevel, game_time: f64) -> bool {
        if let Some(grant) = self.permissions.get(&domain) {
            // Check if the permission has expired
            if let Some(expiration) = grant.expiration {
                if game_time > expiration {
                    return false;
                }
            }
            
            // Check if there's an entity limitation
            if let Some(entity_limits) = &grant.entity_limited {
                if !entity_limits.contains(&entity_id) {
                    return false;
                }
            }
            
            // Check if the access level is sufficient
            match (grant.access_level, min_level) {
                (AccessLevel::None, _) => false,
                (_, AccessLevel::None) => true,
                (a, b) if a as u8 >= b as u8 => true,
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// Grant a permission to this agent
    pub fn grant_permission(&mut self, grant: PermissionGrant, _granting_agent_id: EntityId) {
        // Store the grant by domain
        self.permissions.insert(grant.domain, grant.clone());
        
        // Track where the permission came from
        if grant.grantor != EntityId::default() {
            self.delegated_from
                .entry(grant.grantor)
                .or_insert_with(Vec::new)
                .push(grant);
        }
    }
    
    /// Delegate a permission to another agent
    pub fn delegate_permission(
        &mut self, 
        to_agent_id: EntityId, 
        domain: PermissionDomain, 
        access_level: AccessLevel, 
        game_time: f64, 
        expiration: Option<f64>,
        region_limited: Option<HashSet<EntityId>>,
        entity_limited: Option<HashSet<EntityId>>,
        condition: Option<PermissionCondition>,
    ) -> Option<PermissionGrant> {
        // Check if this agent has permission to delegate
        let self_grant = self.permissions.get(&domain)?;
        
        // Can't delegate higher access than you have
        if self_grant.access_level as u8 <= access_level as u8 && self_grant.access_level != AccessLevel::Owner {
            return None;
        }
        
        // Can't delegate if you don't have owner/admin permissions
        if self_grant.access_level != AccessLevel::Owner && domain != PermissionDomain::Admin {
            return None;
        }
        
        // Create the grant
        let grant = PermissionGrant {
            domain,
            access_level,
            grantor: self_grant.grantor, // The original owner
            grant_time: game_time,
            expiration,
            region_limited,
            entity_limited,
            condition,
        };
        
        // Record the delegation
        self.delegated_to
            .entry(to_agent_id)
            .or_insert_with(Vec::new)
            .push(grant.clone());
        
        Some(grant)
    }
    
    /// Revoke a permission from this agent
    pub fn revoke_permission(&mut self, domain: PermissionDomain, revoker_id: EntityId) -> bool {
        let Some(grant) = self.permissions.get(&domain).cloned() else {
            return false;
        };
        // Only the grantor or an admin can revoke
        let can_revoke = grant.grantor == revoker_id
            || self
                .permissions
                .get(&PermissionDomain::Admin)
                .is_some_and(|admin_grant| admin_grant.grantor == revoker_id);
        if !can_revoke {
            return false;
        }
        let grantor = grant.grantor;
        self.permissions.remove(&domain);
        if let Some(delegated) = self.delegated_from.get_mut(&grantor) {
            delegated.retain(|g| g.domain != domain);
        }
        true
    }
    
    /// Revoke all permissions delegated to a specific agent
    pub fn revoke_delegated_permissions(&mut self, to_agent_id: EntityId) {
        self.delegated_to.remove(&to_agent_id);
    }
}

/// Component that identifies an agent entity
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub faction_id: EntityId,
    pub agent_type: AgentType,
    pub color: Color,
}

/// Types of agents in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    HumanPlayer,
    AIPlayer,
    Delegate,    // An AI agent delegated by a player
    Observer,    // Can only observe, not act
}

/// Resource that tracks the current game time for permission checking
#[derive(Resource, Default)]
pub struct GameTime {
    pub current_time: f64,
}

/// Event for permission grants
#[derive(Message)]
pub struct PermissionGrantEvent {
    pub to_agent_id: EntityId,
    pub from_agent_id: EntityId,
    pub grant: PermissionGrant,
}

/// Event for permission revocations
#[derive(Message)]
pub struct PermissionRevokeEvent {
    pub from_agent_id: EntityId,
    pub revoker_id: EntityId,
    pub domain: PermissionDomain,
}

/// The result of a permission request
pub enum PermissionResult {
    Granted,
    Denied { reason: String },
    PendingApproval { approver_id: EntityId },
}

/// System to periodically check for and remove expired permissions
pub fn expire_permissions_system(
    mut agent_query: Query<&mut AgentPermissions>,
    game_time: Res<GameTime>,
) {
    for mut permissions in agent_query.iter_mut() {
        // Check each permission for expiration
        let expired_domains: Vec<PermissionDomain> = permissions.permissions
            .iter()
            .filter_map(|(domain, grant)| {
                if let Some(expiration) = grant.expiration {
                    if game_time.current_time > expiration {
                        Some(*domain)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        // Remove expired permissions
        for domain in expired_domains {
            permissions.permissions.remove(&domain);
        }
    }
}

// Plugin to register all agent permission systems
pub struct AgentPermissionsPlugin;

impl Plugin for AgentPermissionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameTime>()
           .add_message::<PermissionGrantEvent>()
           .add_message::<PermissionRevokeEvent>()
           .add_systems(Update, expire_permissions_system);
    }
}