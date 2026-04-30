//! Serializable-friendly damage DTOs shared by vehicles and buildings.

use bevy::prelude::Component;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DamageState {
    #[default]
    Intact,
    Damaged,
    Destroyed,
}

#[derive(Debug, Clone, Component)]
pub struct RoadVehicleDamageInfo {
    pub structural_integrity: f32,
    pub state: DamageState,
}

impl Default for RoadVehicleDamageInfo {
    fn default() -> Self {
        Self {
            structural_integrity: 1.0,
            state: DamageState::Intact,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct BuildingDamageInfo {
    pub structural_integrity: f32,
    pub machinery_damage: f32,
    pub electrical_connection_quality: f32,
}

impl Default for BuildingDamageInfo {
    fn default() -> Self {
        Self {
            structural_integrity: 1.0,
            machinery_damage: 0.0,
            electrical_connection_quality: 1.0,
        }
    }
}
