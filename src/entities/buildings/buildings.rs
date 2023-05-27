use crate::idgen::{EntityId, IdGenerator};

use crate::systems::DamageState;
use crate::traits::damage::{DamageInfoProvider, TakesDamage};

use entity::{EntityInfo, MilitaryCivilian};

// Building component
#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
    pub military_civilian: MilitaryCivilian,
    pub distribution_radius: f32,
    pub electrical_supply: f32,
}

// DamageInfo component
#[derive(Component)]
pub struct BuildingDamageInfo {
    pub machinery_damage: f32,
    pub structural_integrity: f32,
    pub electrical_connection: f32,
}

// OccupiedTiles component
#[derive(Component)]
pub struct OccupiedTiles {
    pub tiles: Vec<EntityId>,
}

// NearbyRoads component
#[derive(Component)]
pub struct NearbyRoads {
    pub roads: Vec<EntityId>,
}

// NearbyRails component
#[derive(Component)]
pub struct NearbyRails {
    pub rails: Vec<EntityId>,
}

// Sources component
#[derive(Component)]
pub struct Sources {
    pub sources: Vec<EntityId>,
}

// Destinations component
#[derive(Component)]
pub struct Destinations {
    pub destinations: Vec<EntityId>,
}

impl DamageInfoProvider for BuildingDamageInfo {
    type DamageInfo = BuildingDamageInfo;

    fn get_damage_info(&self) -> &Self::DamageInfo {
        &self
    }
}
