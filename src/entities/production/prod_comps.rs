use bevy::utils::{HashMap,HashSet};
use bevy::prelude::*;

use crate::entities::types_of::p_enumz::{ResourceType,ResourceFilter};
use crate::idgen::EntityId;
use crate::traits::rates::*;

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