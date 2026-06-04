//! CS-003 — sim hydrate bridge: `Chunk` + [`ChunkCellMatrix`] → slab (witness + persist target).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

use super::registry::{WorldSubstrateRegistry, WssSubstrateWitness};
use super::slab::ChunkKey;
use super::types::{hydrate_skeleton_chunk, substrate_cell_count, WorldChunkState};

/// Copy terrain cells from ECS chunk matrix into slab (no `ChunkWeather` / fire reads).
pub fn hydrate_chunk_into_substrate(
    registry: &mut WorldSubstrateRegistry,
    chunk: &Chunk,
    matrix: &ChunkCellMatrix,
) {
    let key = ChunkKey::from(chunk.coord);
    if registry.chunks.contains(key) {
        return;
    }
    let n = substrate_cell_count(matrix.size);
    let mut state = WorldChunkState::new_empty(key, n);
    for y in 0..matrix.size.y {
        for x in 0..matrix.size.x {
            let i = matrix.idx(x, y);
            state.terrain.height[i] = matrix.elevation[i];
            state.terrain.material_ids[i] = matrix.family[i].0;
            state.terrain.porosity[i] = matrix.moisture[i];
            state.terrain.hardness[i] = matrix.temperature[i];
            // HY-002: hydrate local hydrology masks/scalars from worldgen matrix fields.
            let elev = matrix.elevation[i];
            let moisture = matrix.moisture[i].clamp(0.0, 1.0);
            state.hydrology.water_depth[i] = (moisture * 0.8).max(0.01);
            state.hydrology.saturation[i] = moisture;
            state.hydrology.salinity[i] = if elev < 0.16 { 0.6 } else { 0.05 };
            if moisture > 0.72 {
                state.hydrology.river_mask[i] = 1;
            }
            if elev < 0.16 {
                state.hydrology.ocean_mask[i] = 1;
            }
            state.atmosphere.local.soil_moisture = state.atmosphere.local.soil_moisture.max(moisture);
            state.atmosphere.local.fog_density = state.atmosphere.local.fog_density.max(moisture * 0.12);
            state.contamination.airborne[i] = state.contamination.airborne[i].max(moisture * 0.2);
        }
    }
    registry.chunks.insert(key, state);
    registry.chunks.set_resident(key, true);
}

/// Simulation: hydrate from live chunk entities; skeleton fallback when none spawned yet.
pub fn sync_substrate_hydrate_system(
    base: Res<State<BaseState>>,
    mut witness: ResMut<WssSubstrateWitness>,
    mut registry: ResMut<WorldSubstrateRegistry>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    mut skeleton_bootstrapped: Local<bool>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }

    let mut hydrated = false;
    for (chunk, matrix) in &chunks {
        let key = ChunkKey::from(chunk.coord);
        if !registry.chunks.contains(key) {
            hydrate_chunk_into_substrate(registry.as_mut(), chunk, matrix);
            hydrated = true;
        }
    }

    if registry.chunks.is_empty() && !*skeleton_bootstrapped {
        hydrate_skeleton_chunk(registry.as_mut(), IVec2::ZERO);
        hydrated = true;
        *skeleton_bootstrapped = true;
    }

    if hydrated || !registry.chunks.is_empty() {
        witness.hydrate_wired = true;
        witness.chunk_environment_order_preserved = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;

    #[test]
    fn river_mask_nonzero_on_fixture() {
        let mut registry = WorldSubstrateRegistry::default();
        let chunk = Chunk {
            coord: IVec2::new(4, 4),
        };
        let mut matrix = ChunkCellMatrix::new(UVec2::new(4, 4));
        for cell in matrix.moisture.iter_mut() {
            *cell = 0.85;
        }
        hydrate_chunk_into_substrate(&mut registry, &chunk, &matrix);
        let key = ChunkKey::from(chunk.coord);
        let state = registry.chunks.get(key).expect("hydrated chunk");
        assert!(
            state.hydrology.river_mask.iter().any(|m| *m > 0),
            "high moisture fixture should produce river_mask cells"
        );
    }
}
