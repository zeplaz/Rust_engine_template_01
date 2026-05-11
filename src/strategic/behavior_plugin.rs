//! Deprecated bundle — prefer [`super::simulation_plugin::SimulationPlugin`] or individual plugins.

use bevy::prelude::*;

use super::behavior_brain_plugin::BehaviorPlugin;
use super::faction_plugin::FactionPlugin;
use super::fracture_plugin::FracturePlugin;
use super::mission_plugin::MissionPlugin;
use super::strategic_behavior_schedule::StrategicBehaviorSchedulePlugin;

/// Adds mission → brain → fracture resources → faction chains (see [`StrategicBehaviorSchedulePlugin`]).
#[deprecated(
    note = "use SimulationPlugin or StrategicBehaviorSchedulePlugin + MissionPlugin + BehaviorPlugin + FracturePlugin + FactionPlugin"
)]
pub struct BehaviorScaffoldPlugin;

#[allow(deprecated)]
impl Plugin for BehaviorScaffoldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            StrategicBehaviorSchedulePlugin,
            MissionPlugin,
            BehaviorPlugin,
            FracturePlugin,
            FactionPlugin,
        ));
    }
}
