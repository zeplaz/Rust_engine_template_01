//! **Legacy (transitional B)** ECS placeholders for roads / rail — not wired to runtime nav or
//! [`crate::gui::editor::map_editor::MapEditorRoadMarkerV1`] /
//! [`crate::systems::transport::TransportTopology`].
//!
//! **MIGRATION:** **INFRA-E0-003** — gated by `legacy_transport_ecs_stubs` feature (default off).
//! Retire when [`crate::infrastructure::profiles::RailProfile`] + transport graph own the lane
//! ([`src/dev/plan_infrastructure_world_layers_exec_001_v1.md`](../../dev/plan_infrastructure_world_layers_exec_001_v1.md)).
//! Prefer R8 snapshots + editor bake for new work.
#![allow(dead_code)]

use bevy::prelude::*;
use crate::entities::types_of::RoadSurfaceType;

#[derive(Component)]
pub struct Road {
    pub lanes: u32,
    pub surface: RoadSurfaceType,
}

#[derive(Component)]
pub struct RoadSegment;

#[derive(Component)]
pub struct RoadConnection;

#[derive(Component)]
pub struct Rrails;

#[derive(Debug, PartialEq, Clone)]
pub enum GaugeType {
    Narrow,
    Standard,
    Broad,
    Metre,
    TwoFoot,
    Brunel,
    FiveFoot,
}

#[derive(Component)]
pub struct RailGauge {
    pub gauge_type: GaugeType,
    pub width_mm: f32,
    pub name: String,
    pub max_speed: f32,
    pub max_load_capacity: f32,
    pub stability_factor: f32,
    pub turn_radius: f32,
    pub infrastructure_cost_factor: f32,
    pub rolling_stock_interchangeability: bool,
}
