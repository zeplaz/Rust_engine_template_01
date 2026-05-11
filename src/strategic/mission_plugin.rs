//! **Mission pressure** — [`ActiveMissions`], [`PressureField`], narrative influence injection.

use bevy::prelude::*;

use crate::terrain::generation::chunk_worldgen_scheduler::{queue_mission_hint_jobs, ChunkGenMissionChunkHints};

use super::behavior_mission::{
    active_missions_advance_elapsed_system, active_missions_expire_system,
    narrative_mission_influence_apply_system, pressure_field_from_active_missions_system, ActiveMissions,
};
use super::behavior_pressure::PressureField;
use super::strategic_behavior_schedule::StrategicBehaviorSchedule;

fn sync_mission_influence_chunk_hints(
    missions: Res<ActiveMissions>,
    mut hints: ResMut<ChunkGenMissionChunkHints>,
) {
    hints.coords.clear();
    for m in &missions.missions {
        hints.coords.extend_from_slice(&m.influenced_chunks);
    }
}

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveMissions>()
            .init_resource::<PressureField>()
            .add_systems(
                Update,
                sync_mission_influence_chunk_hints.before(queue_mission_hint_jobs),
            )
            .add_systems(
                Update,
                (
                    active_missions_expire_system,
                    pressure_field_from_active_missions_system,
                    narrative_mission_influence_apply_system,
                )
                    .chain()
                    .in_set(StrategicBehaviorSchedule::MissionPressure),
            )
            .add_systems(
                Update,
                active_missions_advance_elapsed_system.in_set(StrategicBehaviorSchedule::AgentBrainPrep),
            );
    }
}
