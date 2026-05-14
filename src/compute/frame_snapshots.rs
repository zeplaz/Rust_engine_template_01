//! Read-only **CPU snapshot** resources consumed by [`super::ComputeDispatchGraph`] (no ECS queries inside dispatch).

use bevy::prelude::*;

use crate::systems::sim_control::SimStepStamp;

/// Agent / crowd state snapshot for GPU steering and lightweight policy (stub until agent extract lands).
#[derive(Resource, Default, Debug, Clone)]
pub struct AgentFrame {
    pub stamp: SimStepStamp,
    pub agent_count: u32,
}

/// Navigation / cost-field snapshot for GPU pathfinding kernels (stub until nav extract lands).
#[derive(Resource, Default, Debug, Clone)]
pub struct NavFieldFrame {
    pub stamp: SimStepStamp,
    pub cell_count: u32,
}
