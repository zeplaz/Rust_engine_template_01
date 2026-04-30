
use bevy::prelude::*;

use crate::entities::types_of::VehicleType;


#[derive(Component)]  
pub struct Vehicle {
    pub vehicle_type: VehicleType,
    pub max_speed: f32,
    pub mass: f32,
    pub velocity: Vec3,
}