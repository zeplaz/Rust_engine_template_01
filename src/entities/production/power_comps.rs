
use bevy::utils::{HashMap, HashSet};
use bevy::prelude::*;

use super::p_enumz::PowerDistributionType;


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