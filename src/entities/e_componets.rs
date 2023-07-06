
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use crate::entities::prelude::*;

use crate::idgen::{EntityId};
pub use crate::entities::entity::*;

use crate::entities::types_aliases::{ResourceRequirementsMap};

use crate::traits::rates::*;


#[derive(Component)]
pub struct Textures {
    pub textures: HashMap<String, HashMap<String, HashMap<String, TextureInfo>>>,
}

#[derive(Component)]
pub struct Condition {
    pub condition: f32,
}

#[derive(Component)]
pub struct MaintenanceLevel{
    pub maintenance_level: f32,
}

#[derive(Clone, Debug)]
pub struct AgentOwnable {
    pub owner_id: EntityId,
}

#[derive(Component)]
pub struct ConstructionStatus
{   
    pub construction_progress: f32,
    pub construction_time: f32,
    pub consturction_resources_requerments: ResourceRequirementsMap,
}

// Building component
#[derive(Component)]
pub struct Building {
    pub building_type: BuildingType,
    pub construction_state: ConstructionStates, 
    pub distribution_radius: f32,
    pub military_civilian: MilitaryCivilian,

}

#[derive(Component)]
pub struct ConsumptionComponent {
    pub consumption_rate: HashMap<ResourceType, f32>,
    pub current_rate: f32, // current overall production rate
    pub max_rate: f32, // maximum overall production rate
    pub storage: HashMap<ResourceType, f32>,
   
}

impl RateCalculatable for ConsumptionComponent {
    fn calculate_total_rate(&self) -> f32 {
        self.consumption_rates.values().sum()
    }
}

#[derive(Component)]
pub struct StorageComponent{
    pub storage: HashMap<ResourceType, f32>,
    
}

#[derive(Component)]
pub struct ProductionComponent {
    pub production_rate: HashMap<ResourceType, f32>,
    pub current_rate: f32, // current overall production rate
    pub max_rate: f32, // maximum overall production rate
    pub storage: HashMap<ResourceType, f32>,
   
}

impl RateCalculatable for ProductionComponent {
    fn calculate_total_rate(&self) -> f32 {
        self.production_rates.values().sum()
    }
}

#[derive(Component)]
pub struct DistroRadius{
    pub radius: f32,
}



#[derive(Component)]
pub struct ElectricalCapacity {
    pub capacity: f32,
}

#[derive(Component)]
pub struct ElectricalLoad {
    pub base_load: f32, // base load is the load when the entity is idle
    pub current_load: f32, // current load can change based on entity activity
}

//ElectricalGrid  needs TransformerComponent, PowerLineComponent, and ElectricalLoad
#[derive(Component)]
pub struct ElectricalGrid {
    pub members: HashSet<Entity>,
    pub connected_grids: HashSet<Entity>,
    pub total_load: f32,
    pub total_capacity: f32,
}

#[derive(Component)]
pub struct SubstationComponent
{
    pub capacity: f32,
    pub radius: f32,
    pub max_transfer: f32,
    pub input_voltage: f32,
    pub output_voltages: HashMap<PowerDistributionType, f32>,
    pub current_tempature: f32,
    pub max_tempature: f32,
   
}

#[derive(Component)]
pub struct PowerLineComponent
{
    pub capacity: f32,
    pub radius: f32,
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

#[derive(Component)]  
pub struct Resource {
    pub resource_type: ResourceType,
    pub quantity: f32,
}
#[derive(Component)]
pub struct ResourceFilterComponent {
    pub whitelist: HashSet<ResourceFilter>,
    pub blacklist: HashSet<ResourceFilter>,
}

#[derive(Component)]
pub struct ResourseCarrier {
    pub current_load: f32,
    pub capacity: f32,
    pub max_capacity: f32,
    pub cargo: HashMap<ResourceType, f32>,
}

#[derive(Component)]
pub struct Waypoints{
    pub points:Vec<Vec2>,
    pub current_waypoint_index:usize,
}

#[derive(Component)]  
pub struct Vehicle {
    pub vehicle_type: VehicleType,
    pub max_speed: f32,
    pub mass: f32,
    pub velocity: Vec3,
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


// OccupiedTiles component
#[derive(Component)]
pub struct OccupiedTiles {
    pub tiles: Vec<EntityId>,
}








