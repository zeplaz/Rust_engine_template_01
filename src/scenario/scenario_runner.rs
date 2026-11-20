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

        app.init_resource::<super::place_corridor::CorridorPlacementLog>();
        app.add_systems(
            Update,
            (
                super::script_host::gate_wait_until_site_operational_system,
                super::place_corridor::run_place_corridor_step_system,
                drain_script_steps,
            )
                .chain()
                .in_set(ScenarioScriptSystemSet::DrainScenarioSteps),
        );
    }
}
