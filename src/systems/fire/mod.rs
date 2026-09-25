//! Surface fire proxies (CPU) — heat/fuel per [`Chunk`](crate::terrain::generation::Chunk).
//!
//! GPU visuals: [`crate::render::GpuWeatherFireFieldPlugin`] reads aggregated means via
//! [`crate::render::WeatherFireFieldUniforms`](crate::render::WeatherFireFieldUniforms).
//!
//! Design upgrade: [`terrain::fire`](crate::terrain::fire) fuel taxonomy + [`ChunkFuelProfile`](chunk_fuel_profile::ChunkFuelProfile)
//! + [`ChunkSmokeField`](chunk_smoke_field::ChunkSmokeField) (`prompts/guides/base_fire_sim.md`).

mod chunk_fire_overlay;
mod surface_water;
mod chunk_fuel_profile;
mod chunk_smoke_field;
mod chunk_surface_fire;
mod ember_spot_ignition;
mod fire_light_emission;
mod play_fire_visibility;
pub mod combustion;
mod fire_fuel;
pub mod witness_collectors;
pub mod types;

pub(crate) use fire_fuel::{fire_fuel_field_tick, spawn_fire_fuel_field_on_new_chunk};

pub use play_fire_visibility::{
    sync_sim_map_fire_overlay_when_sim_has_heat, triage_fire_play_vis_001_green,
    triage_fire_play_vis_001_witness_json, TriageFirePlayVis001Inputs,
};
pub use chunk_fuel_profile::{chunk_fuel_profile_from_vegetation, ChunkFuelProfile};
pub use crate::terrain::fire::FuelLayer;
pub use chunk_fire_overlay::chunk_fire_overlay_tick;
pub use chunk_smoke_field::{
    chunk_smoke_field_pull_from_advected_atmosphere, chunk_smoke_field_tick, ChunkSmokeField,
    ATMOSPHERE_TO_CHUNK_SMOKE_BLEND, CHUNK_SMOKE_L0_PULL,
};
pub use fire_fuel::{derive_fire_fuel_from_vegetation, FireFuelField};
pub use chunk_surface_fire::{chunk_surface_fire_tick, ChunkSurfaceFire};
pub use fire_light_emission::FireLightEmission;
pub use ember_spot_ignition::{
    apply_ember_spot_ignitions, emit_ember_spot_ignition_events, resolve_spot_ignite_cell,
    EmberSpotIgnitionEvent,
};
pub use surface_water::{init_surface_water_fire_gate, SurfaceWaterFireGate};
pub use witness_collectors::{
    finalize_fire_ecology_witness_frame, write_fire_ecology_live_proof_system, FireEcologyLiveProofState,
    FireEcologyWitness,
};
pub use types::ChunkFireOverlay;

pub(crate) use chunk_fuel_profile::{chunk_fuel_profile_tick, spawn_chunk_fuel_profile_on_new_chunk};

use bevy::prelude::*;

use crate::sim::effects::SimEffectSystemSet;
use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use chunk_fire_overlay::spawn_chunk_fire_overlay_on_matrix;
use chunk_smoke_field::spawn_chunk_smoke_field_on_new_chunk;
use chunk_surface_fire::spawn_chunk_surface_fire_on_new_chunk;
use fire_light_emission::{
    maintain_fire_light_emission_from_surface_fire, update_fire_light_emission_flicker,
};

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<surface_water::SurfaceWaterFireGate>()
            .init_resource::<witness_collectors::FireEcologyWitness>()
            .init_resource::<witness_collectors::FireEcologyLiveProofState>()
            .add_systems(Startup, surface_water::init_surface_water_fire_gate)
            .add_message::<ember_spot_ignition::EmberSpotIgnitionEvent>()
            .add_systems(
                Update,
                (
                    spawn_chunk_surface_fire_on_new_chunk.in_set(ChunkEnvironmentSet::Fire),
                    spawn_chunk_smoke_field_on_new_chunk.in_set(ChunkEnvironmentSet::Fire),
                    spawn_chunk_fire_overlay_on_matrix.in_set(ChunkEnvironmentSet::Fire),
                    chunk_fire_overlay_tick.in_set(ChunkEnvironmentSet::Fire),
                    witness_collectors::finalize_fire_ecology_witness_frame
                        .after(chunk_fire_overlay_tick)
                        .in_set(ChunkEnvironmentSet::Fire),
                    // Sole ecology JSON writer — after drain finalize so nested sim_effect_spine is honest.
                    witness_collectors::write_fire_ecology_live_proof_system
                        .after(witness_collectors::finalize_fire_ecology_witness_frame)
                        .after(SimEffectSystemSet::Drain)
                        .run_if(crate::dev::runtime_witness::fire_ecology_live_proof_due),
                    // VSS-T3-001: emit → SimEffect drain → apply (sole EmberSpot MessageWriter = drain).
                    emit_ember_spot_ignition_events
                        .after(chunk_fire_overlay_tick)
                        .before(SimEffectSystemSet::Drain)
                        .in_set(ChunkEnvironmentSet::Fire),
                    apply_ember_spot_ignitions
                        .after(SimEffectSystemSet::Drain)
                        .before(chunk_surface_fire_tick)
                        .in_set(ChunkEnvironmentSet::Fire),
                    chunk_surface_fire_tick.in_set(ChunkEnvironmentSet::Fire),
                    maintain_fire_light_emission_from_surface_fire
                        .after(chunk_surface_fire_tick)
                        .in_set(ChunkEnvironmentSet::Fire),
                    update_fire_light_emission_flicker
                        .after(maintain_fire_light_emission_from_surface_fire)
                        .in_set(ChunkEnvironmentSet::Fire),
                    chunk_smoke_field_tick
                        .after(update_fire_light_emission_flicker)
                        .in_set(ChunkEnvironmentSet::Fire),
                    // TRIAGE-FIRE-PLAY-VIS-001 / VFX-ABSENT: product SimulationMap fire_heat
                    // follows real sim heat (was exported but never scheduled → dead overlay).
                    sync_sim_map_fire_overlay_when_sim_has_heat
                        .after(chunk_fire_overlay_tick)
                        .run_if(in_state(crate::engine::states::BaseState::Simulation)),
                ),
            );
    }
}

#[cfg(test)]
mod fire_schedule_locks {
    #[test]
    fn sync_sim_map_fire_overlay_when_sim_has_heat_is_in_fire_schedule() {
        let src = include_str!("mod.rs");
        let plugin = src
            .split_once("impl Plugin for FirePlugin")
            .expect("FirePlugin")
            .1;
        let plugin = plugin.split("mod fire_schedule_locks").next().unwrap_or(plugin);
        let code: String = plugin
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let idx = code
            .find("sync_sim_map_fire_overlay_when_sim_has_heat")
            .expect("sync_sim_map_fire_overlay_when_sim_has_heat missing from FirePlugin");
        let window = &code[idx..(idx + 180).min(code.len())];
        assert!(
            window.contains(".after(chunk_fire_overlay_tick)"),
            "sync_sim_map_fire_overlay_when_sim_has_heat must be scheduled after chunk_fire_overlay_tick"
        );
    }
}
