// Entity struct
//use std::str::FromStr;
use serde::Deserialize;
use bevy::prelude::*;

use crate::idgen::{EntityId, IdGenerator};
use crate::traits::ids::Identifiable;
use crate::traits::agents::AgentOwnable;
use crate::events::ownership_events::*;

use super::e_flag_types::*;

#[derive(Bundle, Debug, Clone)]
pub struct EntityInfo {
    id: EntityId,
    owner_id: Option<EntityId>,
    position: Vec2,
    entity_type: EntityType,
}

impl EntityInfo {
    fn new(
        id_generator: &mut IdGenerator,
        owner_id: Option<EntityId>,
        position: Vec2,
        entity_type: EntityType,
    ) -> Self {
        EntityInfo {
            id: id_generator.generate_id(),
            owner_id,
            position,
            entity_type,
        }
    }

    pub fn get_position(&self) -> Vec2 {
        self.position
    }

    pub fn get_entity_type(&self) -> Vec2 {
        self.entity_type
    }

    pub fn set_owner_id(
        &mut self,
        new_owner_id: EntityId,
        mut ownership_change_events: EventWriter<OwnershipChangeEvent>,
    ) {
        let old_owner_id = self.owner_id;
        self.owner_id = Some(new_owner_id);
        ownership_change_events.send(OwnershipChangeEvent {
            entity_id: self.id,
            old_owner_id,
            new_owner_id,
        });
    }
}

impl Identifiable for EntityInfo {
    fn id(&mut self) -> EntityId {
        self.id
    }
}

impl AgentOwnable for EntityInfo {
    fn set_owner(&mut self, owner_id: EntityId) {
        self.owner_id = Some(owner_id);
    }

    fn get_owner(&self) -> EntityId {
        self.owner_id
    }
}

#[derive(Debug, Deserialize)]
pub struct RoadVehicleConfig {
    pub name: String,
    pub vtype: RoadVehicleType,
}