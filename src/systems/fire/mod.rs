//! Surface fire proxies (CPU) — heat/fuel per [`Chunk`](crate::terrain::generation::Chunk).
//!
//! GPU visuals: [`crate::render::GpuWeatherFireFieldPlugin`] reads aggregated means via
//! [`crate::render::WeatherFireFieldUniforms`](crate::render::WeatherFireFieldUniforms).
//!
//! Design upgrade: [`terrain::fire`](crate::terrain::fire) fuel taxonomy + [`ChunkFuelProfile`](chunk_fuel_profile::ChunkFuelProfile)
//! + [`ChunkSmokeField`](chunk_smoke_field::ChunkSmokeField) (`prompts/guides/base_fire_sim.md`).

mod chunk_fire_overlay;
mod chunk_fuel_profile;
mod chunk_smoke_field;
mod chunk_surface_fire;
mod ember_spot_ignition;
pub mod combustion;
mod fire_fuel;
pub mod types;

pub(crate) use fire_fuel::{fire_fuel_field_tick, spawn_fire_fuel_field_on_new_chunk};

pub use chunk_fuel_profile::{chunk_fuel_profile_from_vegetation, ChunkFuelProfile};
pub use chunk_fire_overlay::chunk_fire_overlay_tick;
pub use chunk_smoke_field::{chunk_smoke_field_tick, ChunkSmokeField};
pub use fire_fuel::{derive_fire_fuel_from_vegetation, FireFuelField};
pub use chunk_surface_fire::{chunk_surface_fire_tick, ChunkSurfaceFire};
pub use ember_spot_ignition::{
    apply_ember_spot_ignitions, emit_ember_spot_ignition_events, resolve_spot_ignite_cell,
    EmberSpotIgnitionEvent,
};
pub use types::ChunkFireOverlay;

pub(crate) use chunk_fuel_profile::{chunk_fuel_profile_tick, spawn_chunk_fuel_profile_on_new_chunk};

use bevy::prelude::*;

use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use chunk_fire_overlay::spawn_chunk_fire_overlay_on_matrix;
use chunk_smoke_field::spawn_chunk_smoke_field_on_new_chunk;
use chunk_surface_fire::spawn_chunk_surface_fire_on_new_chunk;

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ember_spot_ignition::EmberSpotIgnitionEvent>()
            .add_systems(
            Update,
            (
                spawn_chunk_surface_fire_on_new_chunk.in_set(ChunkEnvironmentSet::Fire),
                spawn_chunk_smoke_field_on_new_chunk.in_set(ChunkEnvironmentSet::Fire),
                spawn_chunk_fire_overlay_on_matrix.in_set(ChunkEnvironmentSet::Fire),
                chunk_fire_overlay_tick.in_set(ChunkEnvironmentSet::Fire),
                emit_ember_spot_ignition_events.in_set(ChunkEnvironmentSet::Fire),
                apply_ember_spot_ignitions.in_set(ChunkEnvironmentSet::Fire),
                chunk_surface_fire_tick.in_set(ChunkEnvironmentSet::Fire),
                chunk_smoke_field_tick.in_set(ChunkEnvironmentSet::Fire),
            )
                .chain(),
        );
    }
}
