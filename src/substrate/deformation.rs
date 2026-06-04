//! WSS-DEFORMATION-SLAB-L2 — apply `DeformationState` into terrain per sim tick (before hydrology).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::substrate::registry::WorldSubstrateRegistry;
use crate::substrate::slab::ChunkKey;
use crate::substrate::types::WorldChunkState;

pub const WSS_DEFORMATION_SLAB_GATE: &str = "WSS-DEFORMATION-SLAB-001";

#[derive(Resource, Clone, Debug, Default)]
pub struct DeformationTickState {
    pub apply_ticks: u32,
    pub cells_applied: u64,
    pub height_delta_applied_max: f32,
    pub compaction_delta_max: f32,
}

/// Apply pending `height_delta` into `terrain.height` for one chunk (shared by sim + lib fixture).
pub fn apply_deformation_to_chunk(
    chunk: &mut WorldChunkState,
    tick: &mut DeformationTickState,
    sim_tick: u64,
) {
    let n = chunk
        .deformation
        .height_delta
        .len()
        .min(chunk.terrain.height.len());
    if n == 0 {
        return;
    }

    let mut applied_any = false;
    for i in 0..n {
        let delta = chunk.deformation.height_delta[i];
        if delta.abs() <= f32::EPSILON {
            continue;
        }
        chunk.terrain.height[i] += delta;
        if i < chunk.deformation.compaction.len() {
            let compaction = (chunk.deformation.compaction[i] + delta.abs() * 0.1).min(1.0);
            chunk.deformation.compaction[i] = compaction;
            tick.compaction_delta_max = tick.compaction_delta_max.max(compaction);
        }
        tick.height_delta_applied_max = tick.height_delta_applied_max.max(delta.abs());
        chunk.deformation.height_delta[i] = 0.0;
        tick.cells_applied += 1;
        applied_any = true;
    }

    if applied_any {
        chunk.deformation.last_mutation_tick = sim_tick;
        chunk.version = chunk.version.saturating_add(1);
        tick.apply_ticks = tick.apply_ticks.saturating_add(1);
    }
}

pub fn deformation_apply_tick_system(
    base: Res<State<BaseState>>,
    sim_tick: Option<Res<crate::systems::sim_control::SimTick>>,
    mut registry: ResMut<WorldSubstrateRegistry>,
    mut tick: ResMut<DeformationTickState>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }

    tick.height_delta_applied_max = 0.0;
    tick.compaction_delta_max = 0.0;
    let sim_tick = sim_tick.map(|t| t.0).unwrap_or(0);
    let keys: Vec<ChunkKey> = registry.chunks.resident.iter().copied().collect();
    for key in keys {
        let Some(chunk) = registry.chunks.get_mut(key) else {
            continue;
        };
        apply_deformation_to_chunk(chunk, &mut tick, sim_tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::types::WorldChunkState;
    use bevy::math::IVec2;

    #[test]
    fn deformation_apply_moves_height_and_clears_delta() {
        let key = crate::substrate::slab::ChunkKey::from(IVec2::ZERO);
        let mut chunk = WorldChunkState::new_empty(key, 4);
        chunk.terrain.height[0] = 1.0;
        chunk.deformation.height_delta[0] = 0.25;

        let mut tick = DeformationTickState::default();
        apply_deformation_to_chunk(&mut chunk, &mut tick, 7);

        assert!((chunk.terrain.height[0] - 1.25).abs() < 1e-5);
        assert_eq!(chunk.deformation.height_delta[0], 0.0);
        assert_eq!(tick.cells_applied, 1);
        assert_eq!(tick.apply_ticks, 1);
        assert_eq!(chunk.deformation.last_mutation_tick, 7);
    }
}
