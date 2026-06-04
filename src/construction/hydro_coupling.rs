//! Construction → hydrology event bus bridge (B-H2 / PLAN-CONSTRUCTION-HYDRO-COUPLING-001).
//!
//! **Execute paths only** — preview, ghost, and pending queue must not call emit helpers.

use std::collections::HashMap;

use bevy::math::{IVec2, UVec2};
use bevy::prelude::*;

use crate::strategic::{footprint_affected_chunk_coords, BuildSiteTile, FootprintTiles};
use crate::substrate::hydrology::{
    HydrologyConstructionCouplingWitness, HydrologyDirtyEvent, HydrologyDirtyReason,
    HydrologyEventQueue,
};
use crate::substrate::ChunkKey;

pub use crate::substrate::hydrology::construction_hydro_coupling_witness_green;

/// Lib witness refresh: one execute emit + drain cycle (no preview emits).
pub fn seed_hydro_coupling_lib_witness(
    queue: &mut HydrologyEventQueue,
    coupling: &mut HydrologyConstructionCouplingWitness,
    registry: &mut crate::substrate::WorldSubstrateRegistry,
    key: ChunkKey,
) {
    use crate::substrate::hydrology::drain::drain_construction_hydro_events;

    coupling.bridge_registered = true;
    emit_construction_hydro_dirty(
        queue,
        coupling,
        key,
        HydrologyDirtyReason::ConstructionComplete { structure_id: 1 },
        1,
        [0_u32],
    );
    drain_construction_hydro_events(queue, registry);
}

/// Marks B-H2 bridge registered when substrate plugin is enabled.
pub fn register_construction_hydro_coupling_bridge(
    mut coupling: Option<ResMut<HydrologyConstructionCouplingWitness>>,
) {
    if let Some(c) = coupling.as_mut() {
        c.bridge_registered = true;
    }
}

/// Construction-facing enqueue — never touches slab hydrology fields.
pub fn emit_construction_hydro_dirty(
    queue: &mut HydrologyEventQueue,
    coupling: &mut HydrologyConstructionCouplingWitness,
    key: ChunkKey,
    reason: HydrologyDirtyReason,
    structure_id: u64,
    affected_cells: impl IntoIterator<Item = u32>,
) {
    coupling.execute_emit_count = coupling.execute_emit_count.saturating_add(1);
    let _ = queue.push(HydrologyDirtyEvent {
        key,
        reason,
        structure_id,
        affected_cells: affected_cells.into_iter().collect(),
    });
}

/// Site / building execute commit ([`crate::strategic::commit_construction_site_system`]).
pub fn emit_site_execute_hydro_dirty(
    queue: &mut HydrologyEventQueue,
    coupling: &mut HydrologyConstructionCouplingWitness,
    structure_id: u64,
    origin: BuildSiteTile,
    footprint: FootprintTiles,
    cells_per_chunk: UVec2,
) {
    let chunk_coords = footprint_affected_chunk_coords(origin, footprint, cells_per_chunk);
    let cells = footprint_cell_indices(origin, footprint, cells_per_chunk);
    for cc in chunk_coords {
        let key = ChunkKey::from(cc);
        let affected: Vec<u32> = cells
            .iter()
            .filter(|(chunk, _)| *chunk == cc)
            .map(|(_, idx)| *idx)
            .collect();
        emit_construction_hydro_dirty(
            queue,
            coupling,
            key,
            HydrologyDirtyReason::ConstructionComplete { structure_id },
            structure_id,
            affected,
        );
    }
}

/// Road / rail segment execute ([`super::construction_pipeline::execute_construction_plans_system`]).
pub fn emit_road_execute_hydro_dirty(
    queue: &mut HydrologyEventQueue,
    coupling: &mut HydrologyConstructionCouplingWitness,
    structure_id: u64,
    tiles: &[BuildSiteTile],
    cells_per_chunk: UVec2,
) {
    let cw = cells_per_chunk.x.max(1) as i32;
    let ch = cells_per_chunk.y.max(1) as i32;
    let mut by_chunk: HashMap<IVec2, Vec<u32>> = HashMap::new();
    for tile in tiles {
        let tx = tile.x as i32;
        let tz = tile.z as i32;
        let cc = IVec2::new(tx.div_euclid(cw), tz.div_euclid(ch));
        let local_x = tx.rem_euclid(cw) as u32;
        let local_z = tz.rem_euclid(ch) as u32;
        let idx = local_z * cw as u32 + local_x;
        by_chunk.entry(cc).or_default().push(idx);
    }
    for (cc, affected) in by_chunk {
        emit_construction_hydro_dirty(
            queue,
            coupling,
            ChunkKey::from(cc),
            HydrologyDirtyReason::ConstructionComplete { structure_id },
            structure_id,
            affected,
        );
    }
}

fn footprint_cell_indices(
    origin: BuildSiteTile,
    fp: FootprintTiles,
    cells_per_chunk: UVec2,
) -> Vec<(IVec2, u32)> {
    let cw = cells_per_chunk.x.max(1) as i32;
    let ch = cells_per_chunk.y.max(1) as i32;
    let ox = origin.x as i32;
    let oz = origin.z as i32;
    let mut out = Vec::new();
    for dz in 0..fp.depth {
        for dx in 0..fp.width {
            let tx = ox + dx as i32;
            let tz = oz + dz as i32;
            let cc = IVec2::new(tx.div_euclid(cw), tz.div_euclid(ch));
            let local_x = tx.rem_euclid(cw) as u32;
            let local_z = tz.rem_euclid(ch) as u32;
            let idx = local_z * cw as u32 + local_x;
            out.push((cc, idx));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::hydrology::drain::hydrology_drain_construction_events_system;
    use crate::substrate::{SubstratePlugin, WorldSubstrateRegistry};
    use bevy::state::app::StatesPlugin;
    use crate::engine::states::BaseState;

    #[test]
    fn execute_emit_enqueued_preview_emits_zero() {
        let mut queue = HydrologyEventQueue::default();
        let mut coupling = HydrologyConstructionCouplingWitness {
            bridge_registered: true,
            ..Default::default()
        };
        emit_construction_hydro_dirty(
            &mut queue,
            &mut coupling,
            ChunkKey::new(0, 0),
            HydrologyDirtyReason::ConstructionComplete { structure_id: 42 },
            42,
            [0_u32, 1],
        );
        assert_eq!(queue.pending.len(), 1);
        assert_eq!(coupling.execute_emit_count, 1);
        assert_eq!(coupling.preview_emit_count, 0);
    }

    #[test]
    fn site_execute_emits_construction_complete_per_chunk() {
        let mut queue = HydrologyEventQueue::default();
        let mut coupling = HydrologyConstructionCouplingWitness::default();
        emit_site_execute_hydro_dirty(
            &mut queue,
            &mut coupling,
            7,
            BuildSiteTile { x: 0, z: 0 },
            FootprintTiles {
                width: 2,
                depth: 1,
            },
            UVec2::new(32, 32),
        );
        assert!(!queue.pending.is_empty());
        assert!(queue.pending.iter().all(|e| matches!(
            e.reason,
            HydrologyDirtyReason::ConstructionComplete { structure_id: 7 }
        )));
    }

    #[test]
    fn drain_marks_substrate_dirty_without_writing_hydrology() {
        use crate::substrate::WorldChunkState;

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, SubstratePlugin))
            .init_resource::<HydrologyConstructionCouplingWitness>()
            .init_state::<BaseState>()
            .insert_state(BaseState::Simulation)
            .add_systems(Update, hydrology_drain_construction_events_system);

        let key = ChunkKey::new(0, 0);
        {
            let mut reg = app.world_mut().resource_mut::<WorldSubstrateRegistry>();
            reg.chunks
                .insert(key, WorldChunkState::new_empty(key, 16));
            reg.chunks.set_resident(key, true);
        }
        {
            app.world_mut()
                .resource_mut::<HydrologyConstructionCouplingWitness>()
                .bridge_registered = true;
            let pushed = app
                .world_mut()
                .resource_mut::<HydrologyEventQueue>()
                .push(HydrologyDirtyEvent {
                    key,
                    reason: HydrologyDirtyReason::ConstructionComplete { structure_id: 1 },
                    structure_id: 1,
                    affected_cells: vec![0],
                });
            assert!(pushed);
            app.world_mut()
                .resource_mut::<HydrologyConstructionCouplingWitness>()
                .execute_emit_count = 1;
        }

        let depth_before = app
            .world()
            .resource::<WorldSubstrateRegistry>()
            .chunks
            .get(key)
            .unwrap()
            .hydrology
            .water_depth
            .clone();

        app.update();

        let reg = app.world().resource::<WorldSubstrateRegistry>();
        assert!(reg.chunks.dirty.contains(&key));
        assert_eq!(
            reg.chunks.get(key).unwrap().hydrology.water_depth,
            depth_before
        );
        let queue = app.world().resource::<HydrologyEventQueue>();
        assert_eq!(queue.construction_events_drained, 1);
        let coupling = app.world().resource::<HydrologyConstructionCouplingWitness>();
        assert!(construction_hydro_coupling_witness_green(coupling, queue));
    }
}
