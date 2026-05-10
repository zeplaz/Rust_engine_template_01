//! Orchestrator plugin: [`strategic_fields_and_ai_orchestrator_v1.md`](../../prompts/guides/strategic_fields_and_ai_orchestrator_v1.md).

use bevy::prelude::*;

use super::plugin::StrategicFieldsPlugin;
use super::sim::StrategicSimulationPlugin;

/// Adds [`StrategicFieldsPlugin`] (chunk overlays + logistics graph inject) and [`StrategicSimulationPlugin`]
/// (overlay coupling, settlement growth, corridor wear, AI aggregates).
pub struct StrategicFieldsAndAiPlugin;

impl Plugin for StrategicFieldsAndAiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((StrategicFieldsPlugin, StrategicSimulationPlugin));
    }
}
