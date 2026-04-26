use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::idgen::EntityId;
use crate::systems::agents::permissions::{PermissionDomain, AccessLevel, PermissionGrantEvent};

/// Event triggered when an entity's ownership changes
#[derive(Event)]
pub struct OwnershipChangeEvent {
    pub entity_id: EntityId,
    pub old_owner_id: EntityId,
    pub new_owner_id: EntityId,
}

/// Resource that stores faction/agent colors for visual representation
#[derive(Resource)]
pub struct FactionColors {
    colors: HashMap<EntityId, Color>,
    default_color: Color,
}

impl Default for FactionColors {
    fn default() -> Self {
        let mut colors = HashMap::new();
        
        // Add some default colors for common factions
        colors.insert(EntityId::from_u32(1), Color::rgb(0.8, 0.0, 0.0));  // Red
        colors.insert(EntityId::from_u32(2), Color::rgb(0.0, 0.0, 0.8));  // Blue
        colors.insert(EntityId::from_u32(3), Color::rgb(0.0, 0.8, 0.0));  // Green
        colors.insert(EntityId::from_u32(4), Color::rgb(0.8, 0.8, 0.0));  // Yellow
        colors.insert(EntityId::from_u32(5), Color::rgb(0.8, 0.0, 0.8));  // Purple
        colors.insert(EntityId::from_u32(6), Color::rgb(0.0, 0.8, 0.8));  // Cyan
        
        Self {
            colors,
            default_color: Color::rgb(0.5, 0.5, 0.5),  // Grey default
        }
    }
}

impl FactionColors {
    /// Get the color for a faction/agent
    pub fn get_color(&self, owner_id: EntityId) -> Color {
        *self.colors.get(&owner_id).unwrap_or(&self.default_color)
    }
    
    /// Register a new faction/agent color
    pub fn register_color(&mut self, owner_id: EntityId, color: Color) {
        self.colors.insert(owner_id, color);
    }
}

/// System to handle ownership change events
pub fn ownership_change_listener(
    mut events: EventReader<OwnershipChangeEvent>,
    mut sprite_query: Query<(&mut Sprite, &Handle<Image>)>,
    mut mesh_query: Query<&mut Handle<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    faction_colors: Res<FactionColors>,
    mut permission_events: EventWriter<PermissionGrantEvent>,
) {
    for event in events.read() {
        // Get the new owner's color
        let color = faction_colors.get_color(event.new_owner_id);
        
        // Update 2D sprite color if the entity has one
        if let Ok((mut sprite, _)) = sprite_query.get_mut(event.entity_id.as_u32() as Entity) {
            sprite.color = color;
        }
        
        // Update 3D material color if the entity has one
        if let Ok(mut material_handle) = mesh_query.get_mut(event.entity_id.as_u32() as Entity) {
            if let Some(material) = materials.get_mut(&*material_handle) {
                material.base_color = color;
            }
        }
        
        // Transfer ownership permissions for this entity
        permission_events.send(PermissionGrantEvent {
            to_agent_id: event.new_owner_id,
            from_agent_id: EntityId::default(), // System-granted
            grant: crate::systems::agents::permissions::PermissionGrant {
                domain: PermissionDomain::Admin,
                access_level: AccessLevel::Owner,
                grantor: EntityId::default(),
                grant_time: 0.0, // Current time would be better
                expiration: None,
                region_limited: None,
                entity_limited: Some([event.entity_id].into_iter().collect()),
                condition: None,
            },
        });
    }
}

// Plugin to register ownership-related systems
pub struct OwnershipPlugin;

impl Plugin for OwnershipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionColors>()
           .add_event::<OwnershipChangeEvent>()
           .add_systems(Update, ownership_change_listener);
    }
}