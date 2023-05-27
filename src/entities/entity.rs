// Entity struct
use std::str::FromStr;

use crate::idgen::{EntityId, IdGenerator};
use traits::ids::Identifiable;
use traits::Agents::AgentOwnable;

#[derive(Debug)]
pub enum BuildingType {
    House,
    Factory,
    RailDepot,
    Warehouse,
    Dépanneur,
    Warehouse,
    FeildDepo,
    Burocracy,
    PowerPlant,
}

#[derive(Debug)]
pub enum RoadVehicleType {
    Bus,
    Truck,
    Car,
    Cargo,
}

#[derive(Debug)]
pub enum EntityType {
    Building(BuildingType),
    Tree,
    Tile,
    RoadVehicle(RoadVehicleType),
    Train,
    Rail,
    Road,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilitaryCivilian {
    Civilian,
    Military,
}

impl FromStr for MilitaryCivilian {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "civilian" | "Civilian" => Ok(MilitaryCivilian::Civilian),
            "military" | "Military" => Ok(MilitaryCivilian::Military),
            _ => Err(()),
        }
    }
}

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
        owner_id: EntityId,
        position: Vec2,
        entity_type: EntityType,
    ) -> Self {
        MyEntity {
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
        self.owner_id = new_owner_id;
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
        self.owner_id = owner_id;
    }

    fn get_owner(&self) -> EntityId {
        self.owner_id
    }
}
