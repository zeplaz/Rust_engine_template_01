use bevy::prelude::*;
use crate::idgen::EntityId;
use crate::entities::{ConstructionStates,MilitaryCivilian};
use crate::entities::strukturave::BuildingType;
use crate::entities::strukturave::RoadSurfaceType;

// Building component
#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
    pub construction_state: ConstructionStates, 
    pub distribution_radius: f32,
    pub military_civilian: MilitaryCivilian,

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
struct Road {
    lanes: u32,
    surface: RoadSurfaceType,
    construction_state: ConstructionStates, 

}


#[derive(Component)]  
struct RoadSegment {}

#[derive(Component)]  
struct RoadConnection {}


#[derive(Component)]  
pub struct Tree {}

#[derive(Component)]  
struct Rrails {}