//! Chunk ecology fields (biomass, fire risk, regrowth) — design: flora / fire runbooks.

mod chunk_ecology;

pub use chunk_ecology::{chunk_ecology_tick, ChunkEcology};

use bevy::prelude::*;

use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use chunk_ecology::spawn_chunk_ecology_on_new_chunk;

pub struct EcologyPlugin;

impl Plugin for EcologyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_chunk_ecology_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                chunk_ecology_tick.in_set(ChunkEnvironmentSet::Ecology),
            )
                .chain(),
        );
    }
}
