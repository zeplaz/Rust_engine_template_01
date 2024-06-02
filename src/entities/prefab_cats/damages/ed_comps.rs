
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



// New structs and enums for damage accumulation, thresholds, and fire propagation

#[derive(Debug, Component)]
pub struct DamageAccumulator {
    pub time_in_overcapacity: f32,
    pub time_in_fire: f32,
}

impl Default for DamageAccumulator {
    fn default() -> Self {
        Self {
            time_in_overcapacity: 0.0,
            time_in_fire: 0.0,
        }
    }
}

#[derive(Debug, Component)]
pub struct DamageThreshold {
    pub machinery_damage_threshold: f32,
    pub structural_integrity_threshold: f32,
    pub electrical_connection_threshold: f32,
}

impl Default for DamageThreshold {
    fn default() -> Self {
        Self {
            machinery_damage_threshold: 0.5,
            structural_integrity_threshold: 0.5,
            electrical_connection_threshold: 0.5,
        }
    }
}

#[derive(Debug, Component)]
pub struct FirePropagation {
    pub propagation_radius: f32,
    pub propagation_chance: f32,
}

impl Default for FirePropagation {
    fn default() -> Self {
        Self {
            propagation_radius: 5.0,
            propagation_chance: 0.1,
        }
    }
}
