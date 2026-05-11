use bevy::prelude::*;

use crate::systems::sim_control::SimControlSystemSet;

use super::script_host::drain_script_steps;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScenarioScriptSystemSet {
    DrainScenarioSteps,
}

pub struct ScenarioRunnerPlugin;

impl Plugin for ScenarioRunnerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            ScenarioScriptSystemSet::DrainScenarioSteps.before(SimControlSystemSet::AdvanceSimTick),
        );

        app.add_systems(
            Update,
            drain_script_steps.in_set(ScenarioScriptSystemSet::DrainScenarioSteps),
        );
    }
}
