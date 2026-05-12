use std::collections::HashSet;

use bevy::prelude::*;

pub use crate::entities::entity::*;
use crate::entities::types_aliases::ResourceRequirementsMap;
pub use crate::entities::types_of::{EmergencyType, MalfunctionType, OperationalStatus};
use crate::idgen::EntityId;

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
pub struct Waypoints {
    pub points: Vec<Vec2>,
    pub current_waypoint_index: usize,
}

#[derive(Component)]
pub struct Operational {
    pub maintenance_level: f32,
    pub malfunctions: HashSet<MalfunctionType>,
    pub emergencies: HashSet<EmergencyType>,
    pub operational_status: OperationalStatus,
}

/// Repeating maintenance interval for [`Operational`] — advanced by [`crate::entities::production::core::operational_maintenance_timer_tick`].
#[derive(Component, Debug, Clone)]
pub struct MaintenanceTimer {
    pub check: Timer,
}

impl Default for MaintenanceTimer {
    fn default() -> Self {
        Self {
            check: Timer::from_seconds(60.0, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
pub struct ConstructionStatus {
    pub construction_progress: f32,
    pub construction_time: f32,
    pub consturction_resources_requerments: ResourceRequirementsMap,
}

#[derive(Component)]
pub struct Distribution {
    pub radius: f32,
}
