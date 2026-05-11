use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::objectives::ScenarioObjectiveV1;

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
}
