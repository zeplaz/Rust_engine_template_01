// Entity struct
//use std::str::FromStr;
use bevy::prelude::*;
use serde::Deserialize;

use crate::events::ownership_events::*;
use crate::idgen::EntityId;
use crate::traits::AgentOwnable;
use crate::traits::Identifiable;

use crate::entities::types_of::*;
use crate::traits::Spaceialization;

#[derive(Debug, Clone)]
pub struct EntityInfo {
    id: EntityId,
    owner_id: Option<EntityId>,
    entity_type: EntityType,
    position: Vec4,
}

impl EntityInfo {
    // Canonical constructor: `EntityId::new()` issues a process-wide unique id
    // via the atomic counter in `crate::idgen` (replaces the retired
    // `core::id_generator::IdGenerator`).
    pub fn new(
        owner_id: Option<EntityId>,
        entity_type: EntityType,
        position: Vec4,
    ) -> Self {
        EntityInfo {
            id: EntityId::new(),
            owner_id,
            entity_type,
            position,
        }
    }

    pub fn get_entity_type(&self) -> EntityType {
        self.entity_type.clone()
    }

    pub fn set_owner_id(
        &mut self,
        new_owner_id: EntityId,
        mut ownership_change_events: MessageWriter<OwnershipChangeEvent>,
    ) {
        let old_owner_id = self.owner_id;
        self.owner_id = Some(new_owner_id);
        ownership_change_events.write(OwnershipChangeEvent {
            entity_id: self.id,
            old_owner_id,
            new_owner_id,
        });
    }
}

impl Spaceialization for EntityInfo {
    type Position = Vec4;

    fn get_position(&self) -> &Self::Position {
        &self.position
    }
}

impl Identifiable for EntityInfo {
    fn id(&self) -> EntityId {
        self.id
    }
}

impl AgentOwnable for EntityInfo {
    fn set_owner(&mut self, owner_id: EntityId) {
        self.owner_id = Some(owner_id);
    }

    fn get_owner(&self) -> EntityId {
        self.owner_id.unwrap_or_else(|| EntityId::from_u32(0))
    }
}
