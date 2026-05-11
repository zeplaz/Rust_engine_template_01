//! Orchestrator plugin: [`strategic_fields_and_ai_orchestrator_v1.md`](../../prompts/guides/strategic_fields_and_ai_orchestrator_v1.md).

use bevy::prelude::*;

use super::simulation_plugin::SimulationPlugin;

/// **Strategic fields + AI** — thin alias for [`SimulationPlugin`] (chunk overlays, logistics bridge, hybrid sim, behavior chain).
///
/// **Transport ordering:** configure [`crate::strategic::plugin::StrategicFieldPipeline::GraphSync`] **after**
/// [`crate::systems::transport::TransportSchedule::CostCache`] in the root [`crate::engine::EnginePlugin`](../../engine/engine_with_worldgen.rs).
pub struct StrategicFieldsAndAiPlugin;

impl Plugin for StrategicFieldsAndAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SimulationPlugin);
    }
}
