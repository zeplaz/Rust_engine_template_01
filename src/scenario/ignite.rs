//! **VSS-T2-002** — scenario ignite helpers routing through [`SimEffectQueue`] (not harness seed).

use bevy::prelude::UVec2;

use crate::sim::effects::{SimEffectEvent, SimEffectKind, SimEffectQueue, SimEffectSource};
use crate::terrain::generation::tile_chunk_map::world_tile_to_chunk_cell_key;
use crate::terrain::ChunkCellKey;

use super::scenario_steps::{ScenarioIgniteCell, ScenarioWorldTile};

pub const DEFAULT_SCENARIO_IGNITE_SPARK: f32 = 0.55;

#[must_use]
pub fn ignite_cells_from_scenario_cells(cells: &[ScenarioIgniteCell]) -> Vec<(ChunkCellKey, f32)> {
    cells
        .iter()
        .map(|c| {
            (
                ChunkCellKey::new(
                    bevy::math::IVec2::new(c.chunk_x, c.chunk_y),
                    c.cell,
                ),
                c.spark,
            )
        })
        .collect()
}

#[must_use]
pub fn ignite_cell_from_world_tile(
    tile: ScenarioWorldTile,
    cells_per_chunk: UVec2,
    spark: f32,
) -> (ChunkCellKey, f32) {
    let key = world_tile_to_chunk_cell_key(tile.tile_x, tile.tile_z, cells_per_chunk);
    (key, spark)
}

/// Enqueue ignite cells on the SimEffect spine — single writer surface for scenario scripts.
pub fn enqueue_scenario_ignite_sim_effect(
    queue: &mut SimEffectQueue,
    source: SimEffectSource,
    cause_id: impl Into<String>,
    parent_effect_id: Option<u64>,
    cells: Vec<(ChunkCellKey, f32)>,
) -> bool {
    if cells.is_empty() {
        return false;
    }
    queue.push(SimEffectEvent {
        source,
        cause_id: cause_id.into(),
        parent_effect_id,
        kind: SimEffectKind::IgniteCells { cells },
    })
}
