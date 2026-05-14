//! Stage-5 ecology overlay snapshot (committed sim step; render reads only).

use bevy::prelude::*;

use crate::systems::sim_control::SimStepStamp;

#[derive(Resource, Clone, Debug, Default)]
pub struct EcologyVisualSnapshot {
    pub stamp: SimStepStamp,
    pub ecology_chunk_count: u32,
    pub mean_biomass: f32,
    pub mean_fire_risk: f32,
    /// Per-chunk committed ecology means (length matches `ecology_chunk_count` when published).
    pub chunk_rows: Vec<Vec4>,
}
