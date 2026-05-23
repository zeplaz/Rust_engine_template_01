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
pub mod combustion;
mod fire_fuel;
pub mod live_proof;
pub mod types;

pub(crate) use fire_fuel::{fire_fuel_field_tick, spawn_fire_fuel_field_on_new_chunk};

pub use chunk_fuel_profile::{chunk_fuel_profile_from_vegetation, ChunkFuelProfile};
pub use crate::terrain::fire::FuelLayer;
pub use chunk_fire_overlay::chunk_fire_overlay_tick;
pub use chunk_smoke_field::{
    chunk_smoke_field_pull_from_advected_atmosphere, chunk_smoke_field_tick, ChunkSmokeField,
    ATMOSPHERE_TO_CHUNK_SMOKE_BLEND,
};
pub use fire_fuel::{derive_fire_fuel_from_vegetation, FireFuelField};
pub use chunk_surface_fire::{chunk_surface_fire_tick, ChunkSurfaceFire};
pub use fire_light_emission::FireLightEmission;
pub use ember_spot_ignition::{
    apply_ember_spot_ignitions, emit_ember_spot_ignition_events, resolve_spot_ignite_cell,
    EmberSpotIgnitionEvent,
};
pub use surface_water::{init_surface_water_fire_gate, SurfaceWaterFireGate};
pub use live_proof::{
    finalize_fire_ecology_witness_frame, write_fire_ecology_live_proof_system, FireEcologyLiveProofState,
    FireEcologyWitness,
};
pub use types::ChunkFireOverlay;

pub(crate) use chunk_fuel_profile::{chunk_fuel_profile_tick, spawn_chunk_fuel_profile_on_new_chunk};

use bevy::prelude::*;

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
            .init_resource::<live_proof::FireEcologyWitness>()
            .init_resource::<live_proof::FireEcologyLiveProofState>()
            .add_systems(Startup, surface_water::init_surface_water_fire_gate)
            .add_message::<ember_spot_ignition::EmberSpotIgnitionEvent>()
            .add_systems(
            Update,
            (
                spawn_chunk_surface_fire_on_new_chunk.in_set(ChunkEnvironmentSet::Fire),
                spawn_chunk_smoke_field_on_new_chunk.in_set(ChunkEnvironmentSet::Fire),
                spawn_chunk_fire_overlay_on_matrix.in_set(ChunkEnvironmentSet::Fire),
                chunk_fire_overlay_tick.in_set(ChunkEnvironmentSet::Fire),
                live_proof::finalize_fire_ecology_witness_frame
                    .after(chunk_fire_overlay_tick)
                    .in_set(ChunkEnvironmentSet::Fire),
                live_proof::write_fire_ecology_live_proof_system
                    .after(live_proof::finalize_fire_ecology_witness_frame),
                emit_ember_spot_ignition_events.in_set(ChunkEnvironmentSet::Fire),
                apply_ember_spot_ignitions.in_set(ChunkEnvironmentSet::Fire),
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
            )
                .chain(),
        );
    }
}
