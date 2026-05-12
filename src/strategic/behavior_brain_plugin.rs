//! **Agent brain prep** — sync ids, pluggable [`ActiveBehaviorModel`], [`DecisionPipelineSink`].

use bevy::prelude::*;

use super::ai_explainability::{decision_explainability_capture_system, DecisionExplainabilitySnapshot};
use super::behavior_emergence_log::{
    strategic_emergence_log_hybrid_resolution_system, StrategicEmergenceLog,
};
use super::behavior_entities::behavior_sync_entity_ids_system;
use super::behavior_interface::{
    behavior_model_evaluation_hook_system, ActiveBehaviorModel, DecisionPipelineSink,
};
use super::behavior_pipeline::decision_pipeline_composition_system;
use super::hybrid_brain::hybrid_resolve_and_feedback_system;
use super::strategic_behavior_schedule::StrategicBehaviorSchedule;

pub struct BehaviorPlugin;

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveBehaviorModel>()
            .init_resource::<DecisionPipelineSink>()
            .init_resource::<DecisionExplainabilitySnapshot>()
            .init_resource::<StrategicEmergenceLog>()
            .add_systems(PreUpdate, behavior_sync_entity_ids_system)
            .add_systems(
                Update,
                (
                    behavior_model_evaluation_hook_system,
                    decision_pipeline_composition_system,
                    decision_explainability_capture_system,
                )
                    .chain()
                    .in_set(StrategicBehaviorSchedule::AgentBrainPrep),
            )
            .add_systems(
                PostUpdate,
                strategic_emergence_log_hybrid_resolution_system
                    .after(hybrid_resolve_and_feedback_system),
            );
    }
}
