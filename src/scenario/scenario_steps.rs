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

/// World tile for [`ScenarioStep::IgniteAt`] — same axes as map editor / [`crate::strategic::BuildSiteTile`].
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Serialize, Deserialize)]
pub struct ScenarioWorldTile {
    pub tile_x: u32,
    pub tile_z: u32,
}

fn default_ignite_spark() -> f32 {
    0.55
}

fn default_scenario_ignite_cause_id() -> String {
    "CAUSE-scenario-ignite".into()
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
    /// **VSS-T2-002** — single world-tile ignite via SimEffect waist (product path).
    IgniteAt {
        tile: ScenarioWorldTile,
        #[serde(default = "default_ignite_spark")]
        spark: f32,
        #[serde(default = "default_scenario_ignite_cause_id")]
        cause_id: String,
    },
    /// **VSS-T2-002** — resolve `assets/scenarios/triggers/{effect_id}.trigger.ron` → SimEffect ignite.
    TriggerEffect {
        effect_id: String,
    },
}

impl ScenarioStep {
    /// True when the step enqueues fire through the SimEffect spine (not harness seed).
    #[must_use]
    pub fn routes_fire_via_sim_effect(&self) -> bool {
        matches!(
            self,
            Self::EmitSimEffect { .. } | Self::IgniteAt { .. } | Self::TriggerEffect { .. }
        )
    }
}
