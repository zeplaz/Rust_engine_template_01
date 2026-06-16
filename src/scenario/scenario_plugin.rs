use bevy::prelude::*;

use super::objectives::{
    ObjectiveTargetRef, ScenarioObjectiveKindV1, ScenarioObjectiveMarker, ScenarioObjectiveV1,
};
use super::scenario_runner::ScenarioRunnerPlugin;
use super::scenario_steps::{ScenarioIgniteCell, ScenarioStep};
use super::scenario_types::{ScenarioFileV1, ScenarioMetadata};
use super::script_host::{EngineScriptHost, ScenarioExecutionState};
use super::validation::ScenarioValidationReport;

pub struct ScenarioScriptingPlugin;

impl Plugin for ScenarioScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EngineScriptHost>()
            .register_type::<EngineScriptHost>()
            .register_type::<ScenarioFileV1>()
            .register_type::<ScenarioMetadata>()
            .register_type::<ScenarioStep>()
            .register_type::<ScenarioIgniteCell>()
            .register_type::<ScenarioObjectiveV1>()
            .register_type::<ObjectiveTargetRef>()
            .register_type::<ScenarioObjectiveKindV1>()
            .register_type::<ScenarioObjectiveMarker>()
            .register_type::<ScenarioExecutionState>()
            .register_type::<ScenarioValidationReport>()
            .add_plugins(ScenarioRunnerPlugin);
    }
}
