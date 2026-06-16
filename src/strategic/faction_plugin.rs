//! **Faction dynamics** — meso tick, cohesion pressure, internal stage (primary sim drift).

use bevy::prelude::*;

use crate::sim::effects::SimEffectSystemSet;

use super::behavior_fracture::{
    faction_cohesion_pressure_system, faction_internal_stage_system,
    faction_meso_internal_tick_system, fracture_probability_overlay_system,
};
use super::behavior_sim_effect_react::apply_sim_effect_telemetry_faction_stress_system;
use super::strategic_behavior_schedule::StrategicBehaviorSchedule;

pub struct FactionPlugin;

impl Plugin for FactionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                apply_sim_effect_telemetry_faction_stress_system,
                faction_meso_internal_tick_system,
                faction_cohesion_pressure_system,
                faction_internal_stage_system,
                fracture_probability_overlay_system,
            )
                .chain()
                .in_set(StrategicBehaviorSchedule::FactionDrift)
                .after(SimEffectSystemSet::Drain),
        );
    }
}
