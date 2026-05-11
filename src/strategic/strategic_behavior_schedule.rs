//! Cross-plugin **Update** ordering for mission → brain → faction → fracture (CPU sim spine).
//!
//! GPU / tooling plugins attach elsewhere; these sets only coordinate the strategic behavior chain.

use bevy::prelude::*;

use super::hybrid_brain::hybrid_intent_reset_system;

/// Ordered phases for [`crate::strategic::SimulationPlugin`] sub-plugins.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StrategicBehaviorSchedule {
    /// Expire missions, accumulate [`super::PressureField`], apply [`super::ScriptInfluence`] to participants.
    MissionPressure,
    /// Behavior model hook, decision pipeline, mission elapsed tick.
    AgentBrainPrep,
    /// Faction meso drift, cohesion pressure, internal stage.
    FactionDrift,
    /// Informational fracture probability + soft [`super::FractureEvent`] emission (no map / war authority).
    FractureOverlay,
}

pub struct StrategicBehaviorSchedulePlugin;

impl Plugin for StrategicBehaviorSchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                StrategicBehaviorSchedule::AgentBrainPrep.after(StrategicBehaviorSchedule::MissionPressure),
                StrategicBehaviorSchedule::AgentBrainPrep.before(hybrid_intent_reset_system),
                StrategicBehaviorSchedule::FactionDrift.after(hybrid_intent_reset_system),
                StrategicBehaviorSchedule::FractureOverlay.after(StrategicBehaviorSchedule::FactionDrift),
            ),
        );
    }
}
