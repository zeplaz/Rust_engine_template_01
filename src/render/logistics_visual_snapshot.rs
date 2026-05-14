//! Stage-5 logistics overlay snapshot (committed sim step; render reads only).

use bevy::prelude::*;

use crate::systems::sim_control::SimStepStamp;

#[derive(Resource, Clone, Debug, Default)]
pub struct LogisticsVisualSnapshot {
    pub stamp: SimStepStamp,
    pub corridor_revision: u64,
    pub active_overlay_rows: u32,
    /// Committed corridor rows (`edge_id`, traffic factor) for GPU projection.
    pub edge_rows: Vec<(u32, f32)>,
}
