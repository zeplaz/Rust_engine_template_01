
use bevy::prelude::*;
use bevy::utils::{HashMap};
use crate::entities::prelude::*;

use crate::idgen::{EntityId};
pub use crate::entities::entity::*;

use crate::entities::types_aliases::{ResourceRequirementsMap};


#[derive(Component)]
pub struct Textures {
    pub textures: HashMap<String, HashMap<String, HashMap<String, TextureInfo>>>,
}

#[derive(Clone, Debug, Component)]
pub struct AgentOwnable {
    pub owner_id: EntityId,
}

// OccupiedTiles component
#[derive(Component)]
pub struct OccupiedTiles {
    pub tiles: Vec<EntityId>,
}

#[derive(Component)]
pub struct Waypoints{
    pub points:Vec<Vec2>,
    pub current_waypoint_index:usize,
}

#[derive(Component)]
pub struct MaintenanceLevel{
    pub maintenance_level: f32,
}


#[derive(Component)]
pub struct ConstructionStatus
{   
    pub construction_progress: f32,
    pub construction_time: f32,
    pub consturction_resources_requerments: ResourceRequirementsMap,
}
