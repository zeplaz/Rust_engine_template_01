
use bevy::utils::{HashMap, HashSet};
use bevy::prelude::*;

use crate::entities::types_of::p_enumz::{OperationMechanism,SwitchCloseBehavior,PowerDistributionType, PowerPlantType,SwitchState,ResourceMechanism };

use crate::entities::types_of::e_flagz::{OperationalStatus};

#[derive(Component)]
pub struct ThermalComponent {
    pub current_temperature: f32,
    pub max_temperature: f32,
}

#[derive(Component)]
pub struct ElectricalComponent {
    pub base_load: f32,
    pub current_load: f32,
    pub max_transfer: f32,
    pub capacity: f32,
}

#[derive(Component)]
pub struct TransformerComponent {
    pub input_voltage: f32,
    pub output_voltage: f32,
}

#[derive(Component)]
pub struct SubstationComponent {
    pub input_voltage:  HashMap<PowerDistributionType, f32>,
    pub output_voltages: HashMap<PowerDistributionType, f32>,

}

#[derive(Component)]
pub struct Fuel{
    pub name: String,
    pub current_fuel: f32,
    pub max_fuel: f32,
    pub property: ResourceMechanism, 
}

#[derive(Component)]
pub struct PowerPlant {
    pub plant_type: PowerPlantType,
    pub max_output: f32,
    pub current_output: f32,
    pub status: OperationalStatus,
    pub efficiency: f32, // only relevant for some types
}


//ElectricalGrid  needs TransformerComponent, PowerLineComponent, and ElectricalLoad
#[derive(Component)]
pub struct ElectricalGrid {
    pub members: HashSet<Entity>,
    pub connected_grids: HashSet<Entity>,
    pub total_load: f32,
    pub total_capacity: f32,
    // Note: total_capacity can be computed by summing the Capacity components of all members
}

// PowerLineComponent now only has specific properties
#[derive(Component)]
pub struct PowerLineComponent {}

#[derive(Component)]
pub struct SwitchComponent {
    pub state: SwitchState,
    pub max_current: f32,
    pub connected_entities: (Entity, Entity),
    pub retry_duration: Option<f32>, // Only relevant for Automatic switches
    pub operation_time: Option<f32>, // Time taken to change state, 
    pub elapsed_time: f32, // Used to track time for retries and operations
    pub retry_count: u32, // Used to track number of retries
    pub operation_mechanism: OperationMechanism,
    pub event_behavior: SwitchCloseBehavior,
}

