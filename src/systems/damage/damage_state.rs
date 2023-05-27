use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Clone, Eq, States, Default)]
pub enum DamageState {
    #[default]
    FullyOperational,
    Damaged,
    Disabled,
    Wrecked,
}

#[derive(Debug, Component)]
pub struct DamageStateComponent {
    state: DamageState,
    time_in_state: f32,
}

#[derive(Debug, Component)]
pub struct RoadVehicleDamage {
    wheel_damage: f32,
    engine_damage: f32,
    structural_integrity: f32,
}

impl Default for RoadVehicleDamage {
    fn default() -> Self {
        Self {
            wheel_damage: 0.0,
            engine_damage: 0.0,
            structural_integrity: 1.0, // from 0.0 to 1.0
        }
    }
}
