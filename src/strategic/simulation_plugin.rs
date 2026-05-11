//! **Simulation spine** — fields + hybrid tick + mission/brain/faction/fracture plugins (CPU orchestration).
//!
//! See [`super::strategic_behavior_schedule::StrategicBehaviorSchedule`] for Update ordering.

use bevy::prelude::*;

use super::agent_batch_scoring::AgentBatchScoringPlugin;
use super::behavior_brain_plugin::BehaviorPlugin;
use super::fracture_plugin::FracturePlugin;
use super::infrastructure_graph::InfrastructureGraphBridgePlugin;
use super::mission_plugin::MissionPlugin;
use super::faction_plugin::FactionPlugin;
use super::plugin::StrategicFieldsPlugin;
use super::sim::StrategicSimulationPlugin;
use super::spatial_network::SpatialNetworkPlugin;
use super::strategic_behavior_schedule::StrategicBehaviorSchedulePlugin;

/// Core strategic + behavior stack: chunk overlays, construction bridge, hybrid sim, then phased behavior chain.
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            StrategicFieldsPlugin,
            InfrastructureGraphBridgePlugin,
            SpatialNetworkPlugin,
            StrategicSimulationPlugin,
            StrategicBehaviorSchedulePlugin,
            MissionPlugin,
            AgentBatchScoringPlugin,
            BehaviorPlugin,
            FracturePlugin,
            FactionPlugin,
        ));
    }
}
