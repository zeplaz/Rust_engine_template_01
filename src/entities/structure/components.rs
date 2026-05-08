use bevy::prelude::*;
use crate::idgen::EntityId;
use crate::entities::types_of::BuildingType;

// Building component
#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
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

#[derive(Component)]
pub struct Tree {}