// Entity struct
//use std::str::FromStr;
use bevy::prelude::*;
use serde::Deserialize;

use crate::events::ownership_events::*;
use crate::idgen::{EntityId, IdGenerator};
use crate::traits::agents::AgentOwnable;
use crate::traits::ids::Identifiable;

use super::types_of::*;
use crate::traits::spacial::Spaceialization;

#[derive(Debug, Clone)]
pub struct EntityInfo {
    id: EntityId,
    owner_id: Option<EntityId>,
    entity_type: EntityType,
    position: Vec4,
}

impl EntityInfo {
    fn new(
        id_generator: &mut IdGenerator,
        owner_id: Option<EntityId>,
        entity_type: EntityType,
        position: Vec4,
    ) -> Self {
        EntityInfo {
            id: id_generator.generate_id(),
            owner_id,
            entity_type,
            position,
        }
    }

    pub fn get_entity_type(&self) -> EntityType {
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

impl Spaceialization for Entity {
    tyoe Position  = Vec4;

    fn get_position(&self) -> &self::Position {
        &self.position
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
