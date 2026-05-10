//! Orchestrator plugin: [`strategic_fields_and_ai_orchestrator_v1.md`](../../prompts/guides/strategic_fields_and_ai_orchestrator_v1.md).

use bevy::prelude::*;

use super::infrastructure_graph::InfrastructureGraphBridgePlugin;
use super::plugin::StrategicFieldsPlugin;
use super::sim::StrategicSimulationPlugin;

/// Adds [`StrategicFieldsPlugin`] (chunk overlays + logistics graph inject),
/// [`InfrastructureGraphBridgePlugin`] (construction-phase graph mirror), and [`StrategicSimulationPlugin`]
/// (overlay coupling, settlement growth, corridor wear, AI aggregates).
///
/// **Transport ordering:** when the full engine is built, configure
/// [`crate::strategic::plugin::StrategicFieldPipeline::GraphSync`] to run **after**
/// [`crate::systems::transport::TransportSchedule::CostCache`] so the logistics graph matches the latest
/// cost cache (see [`crate::engine::EnginePlugin`](../engine/engine_with_worldgen.rs)).
pub struct StrategicFieldsAndAiPlugin;

impl Plugin for StrategicFieldsAndAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            StrategicFieldsPlugin,
            InfrastructureGraphBridgePlugin,
            StrategicSimulationPlugin,
        ));
    }
}
