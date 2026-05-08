//! Surface fire proxies (CPU) — heat/fuel per [`Chunk`](crate::terrain::generation::Chunk).
//!
//! GPU visuals: [`crate::render::GpuWeatherFireFieldPlugin`] reads aggregated means via
//! [`crate::render::WeatherFireFieldUniforms`](crate::render::WeatherFireFieldUniforms).

mod chunk_surface_fire;

pub use chunk_surface_fire::{chunk_surface_fire_tick, ChunkSurfaceFire};

use bevy::prelude::*;

use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use chunk_surface_fire::spawn_chunk_surface_fire_on_new_chunk;

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_chunk_surface_fire_on_new_chunk.in_set(ChunkEnvironmentSet::Fire),
                chunk_surface_fire_tick.in_set(ChunkEnvironmentSet::Fire),
            )
                .chain(),
        );
    }
}
