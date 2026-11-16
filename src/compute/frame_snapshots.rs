//! Read-only **CPU snapshot** resources consumed by [`super::ComputeDispatchGraph`] (no ECS queries inside dispatch).

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::systems::sim_control::SimStepStamp;

/// Agent / crowd state snapshot for GPU steering and lightweight policy (stub until agent extract lands).
#[derive(Resource, Default, Debug, Clone)]
pub struct AgentFrame {
    pub stamp: SimStepStamp,
    pub agent_count: u32,
}

/// One chunk heat → nav cost sample (**DH-03b** RO from dense heat).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavHeatCostSample {
    pub chunk: IVec2,
    /// 0..1 path penalty derived from heat (does not write heat).
    pub heat_cost: f32,
}

/// One chunk wildlife soft cost (**MOVE-10** RO from ecology herd).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NavWildlifeSoftSample {
    pub chunk: IVec2,
    /// 0..cap path penalty from wildlife occupancy (nav does not write wildlife).
    pub soft_cost: f32,
}

/// Navigation / cost-field snapshot for pathfinding kernels.
///
/// **DH-03b:** `heat_cost_samples` are RO projections of dense/sparse heat — nav never writes heat.
/// **MOVE-10:** `wildlife_soft_samples` are RO projections from ecology herd occupancy.
#[derive(Resource, Default, Debug, Clone)]
pub struct NavFieldFrame {
    pub stamp: SimStepStamp,
    pub cell_count: u32,
    pub heat_cost_samples: Vec<NavHeatCostSample>,
    pub mean_heat_cost: f32,
    pub wildlife_soft_samples: Vec<NavWildlifeSoftSample>,
}
