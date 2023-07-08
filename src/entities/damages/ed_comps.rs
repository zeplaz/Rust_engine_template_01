
use bevy::prelude::*;

pub use super::ed_states::*;



#[derive(Component)]
pub struct Condition {
    pub condition: f32,
}

#[derive(Debug, Component)]
pub struct DamageStateComponent {
    state: DamageState,
    time_in_state: f32,
    fire_state: FireState,
    time_in_fire_state: f32,

}


// DamageInfo component
#[derive(Debug,Component)]
pub struct BuildingDamageInfo {
    pub machinery_damage: f32,
    pub structural_integrity: f32,
    pub electrical_connection_quality: f32,
}

impl Default for BuildingDamageInfo  {
    fn default() -> Self {
        Self {
            machinery_damage: 0.0,
            structural_integrity: 1.0,
            electrical_connection_quality: 1.0,
        }
        }
    }

#[derive(Debug, Component)]
pub struct RoadVehicleDamageInfo {
    wheel_damage: f32,
    engine_damage: f32,
    structural_integrity: f32,
}

impl Default for RoadVehicleDamageInfo {
    fn default() -> Self {
        Self {
            wheel_damage: 0.0,
            engine_damage: 0.0,
            structural_integrity: 1.0, // from 0.0 to 1.0
        }
    }
}