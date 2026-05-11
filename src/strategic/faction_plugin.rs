//! **Faction dynamics** — meso tick, cohesion pressure, internal stage (primary sim drift).

use bevy::prelude::*;

use super::behavior_fracture::{
    faction_cohesion_pressure_system, faction_internal_stage_system,
    faction_meso_internal_tick_system, fracture_probability_overlay_system,
};
use super::strategic_behavior_schedule::StrategicBehaviorSchedule;

pub struct FactionPlugin;

impl Plugin for FactionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                faction_meso_internal_tick_system,
                faction_cohesion_pressure_system,
                faction_internal_stage_system,
                fracture_probability_overlay_system,
            )
                .chain()
                .in_set(StrategicBehaviorSchedule::FactionDrift),
        );
    }
}
