use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::sim::effects::SimEffectSource;

use super::objectives::ScenarioObjectiveV1;

/// One ignite cell for [`ScenarioStep::EmitSimEffect`] (RON: `chunk_x`, `chunk_y`, `cell`, `spark`).
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Serialize, Deserialize)]
pub struct ScenarioIgniteCell {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub cell: u32,
    pub spark: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub enum ScenarioStep {
    NoOp,
    SimAdvance { ticks: u32 },
    Log { message: String },
    /// Spawn [`super::objectives::ScenarioObjectiveMarker`] entities (Wave 3).
    RegisterObjectives {
        clear_existing: bool,
        objectives: Vec<ScenarioObjectiveV1>,
    },
    /// **SCENARIO-TRIGGER-001 / G-PLAY-FIRE-001** — enqueue sim-effect ignite (Path A, not harness seed).
    EmitSimEffect {
        source: SimEffectSource,
        cause_id: String,
        #[serde(default)]
        parent_effect_id: Option<u64>,
        cells: Vec<ScenarioIgniteCell>,
    },
}
