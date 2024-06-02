use crate::entities::prelude::*;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

pub use crate::entities::entity::*;
use crate::entities::types_aliases::ResourceRequirementsMap;
pub use crate::entities::types_of::{EmergencyType, MalfunctionType, OperationalStatus};
use crate::idgen::EntityId;

use std::time::Duration;

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

struct MaintenanceTimer {
    timer: Timer,
    interval: Duration,
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
