//! Ecology: macro [`ChunkEcology`](chunk_ecology::ChunkEcology) + meso [`vegetation_field::VegetationField`] (CPU authority).

mod chunk_ecology;
mod vegetation_field;

pub use chunk_ecology::{chunk_ecology_tick, ChunkEcology};
pub use vegetation_field::{
    derive_vegetation_structure, integrate_vegetation_field_step, succession_stage_from_vegetation,
    EcologicalSuccessionStage, VegetationField, VegetationStructure,
};

use bevy::prelude::*;

use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use crate::systems::fire::{
    chunk_fuel_profile_tick, fire_fuel_field_tick, spawn_chunk_fuel_profile_on_new_chunk,
    spawn_fire_fuel_field_on_new_chunk,
};
use crate::terrain::generation::Chunk;
use crate::terrain::material::{invalidate_world, InvalidationReason, WorldPreviewState};
use chunk_ecology::spawn_chunk_ecology_on_new_chunk;
use vegetation_field::{spawn_vegetation_field_on_new_chunk, vegetation_field_tick};

fn ecology_preview_bump_on_vegetation_change(
    preview_state: Option<ResMut<WorldPreviewState>>,
    q: Query<(&Chunk, &VegetationField), Changed<VegetationField>>,
) {
    let Some(mut state) = preview_state else {
        return;
    };
    let coords: Vec<IVec2> = q.iter().map(|(c, _)| c.coord).collect();
    if coords.is_empty() {
        return;
    }
    invalidate_world(
        InvalidationReason::EcologyFields,
        &mut state,
        coords.into_iter(),
    );
}

pub struct EcologyPlugin;

impl Plugin for EcologyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_vegetation_field_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                spawn_chunk_ecology_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                spawn_chunk_fuel_profile_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                spawn_fire_fuel_field_on_new_chunk.in_set(ChunkEnvironmentSet::Ecology),
                chunk_ecology_tick.in_set(ChunkEnvironmentSet::Ecology),
                vegetation_field_tick.in_set(ChunkEnvironmentSet::Ecology),
                fire_fuel_field_tick.in_set(ChunkEnvironmentSet::Ecology),
                chunk_fuel_profile_tick.in_set(ChunkEnvironmentSet::Ecology),
                ecology_preview_bump_on_vegetation_change.in_set(ChunkEnvironmentSet::Ecology),
            )
                .chain(),
        );
    }
}
