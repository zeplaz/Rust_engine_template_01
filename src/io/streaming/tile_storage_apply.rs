//! BQ-101 TileStorage diff apply + smooth transition bookkeeping for renderer consumers.

use std::collections::HashMap;
use std::time::Instant;

use bevy::prelude::*;

use super::tile_storage_contract::{
    PendingTileStorageDiffQueue, TileStorageApplyTiming,
};

const TILE_STORAGE_SMOOTH_STEPS: u8 = 4;

/// Per-chunk changed tile indices mid smooth blend (0 = just applied, `TILE_STORAGE_SMOOTH_STEPS` = ready).
#[derive(Resource, Debug, Default, Clone)]
pub struct TileStorageSmoothTransitionState {
    pub pending_by_chunk: HashMap<IVec2, HashMap<u32, u8>>,
}

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct TileStorageApplyReport {
    pub last_apply_timing: TileStorageApplyTiming,
    pub applied_chunks: u32,
    pub pending_smooth_tiles: u32,
}

pub fn apply_pending_tile_storage_diffs(
    mut queue: ResMut<PendingTileStorageDiffQueue>,
    mut state: ResMut<TileStorageSmoothTransitionState>,
    mut report: ResMut<TileStorageApplyReport>,
    update_attrib: Option<ResMut<crate::render::FrameUpdateAttrib>>,
) {
    let t0 = Instant::now();
    if queue.batch.chunks.is_empty() {
        return;
    }
    if matches!(queue.batch.timing, TileStorageApplyTiming::Bq101Deferred) {
        return;
    }
    for chunk in queue.batch.chunks.drain(..) {
        let slot = state.pending_by_chunk.entry(chunk.chunk).or_default();
        for idx in chunk.changed_tile_indices {
            slot.insert(idx, 0);
        }
        report.applied_chunks = report.applied_chunks.saturating_add(1);
    }
    report.last_apply_timing = TileStorageApplyTiming::AfterDomainReconstruct;
    queue.batch.timing = report.last_apply_timing;
    report.pending_smooth_tiles = state
        .pending_by_chunk
        .values()
        .map(|per_chunk| per_chunk.len() as u32)
        .sum();
    crate::render::record_tile_storage_apply_ms(update_attrib, t0.elapsed().as_secs_f32() * 1000.0);
}

pub fn tick_tile_storage_smooth_transitions(
    mut state: ResMut<TileStorageSmoothTransitionState>,
    mut report: ResMut<TileStorageApplyReport>,
) {
    for per_chunk in state.pending_by_chunk.values_mut() {
        per_chunk.retain(|_, step| {
            if *step < TILE_STORAGE_SMOOTH_STEPS {
                *step = step.saturating_add(1);
                true
            } else {
                false
            }
        });
    }
    state.pending_by_chunk.retain(|_, per_chunk| !per_chunk.is_empty());
    report.pending_smooth_tiles = state
        .pending_by_chunk
        .values()
        .map(|per_chunk| per_chunk.len() as u32)
        .sum();
}

#[must_use]
pub fn tile_storage_indices_ready_for_render(
    state: &TileStorageSmoothTransitionState,
    chunk: IVec2,
) -> Vec<u32> {
    state
        .pending_by_chunk
        .get(&chunk)
        .map(|per_chunk| {
            per_chunk
                .iter()
                .filter(|(_, step)| **step >= TILE_STORAGE_SMOOTH_STEPS)
                .map(|(idx, _)| *idx)
                .collect()
        })
        .unwrap_or_default()
}

#[must_use]
pub fn tile_storage_indices_blending(
    state: &TileStorageSmoothTransitionState,
    chunk: IVec2,
) -> Vec<u32> {
    state
        .pending_by_chunk
        .get(&chunk)
        .map(|per_chunk| {
            per_chunk
                .iter()
                .filter(|(_, step)| **step < TILE_STORAGE_SMOOTH_STEPS)
                .map(|(idx, _)| *idx)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::streaming::tile_storage_contract::TILE_STORAGE_DIFF_CONTRACT_BQ;
    use crate::io::streaming::tile_storage_diff_for_chunk;

    #[test]
    fn tile_storage_apply_contract_is_bq_101() {
        assert_eq!(TILE_STORAGE_DIFF_CONTRACT_BQ, "BQ-101");
    }

    #[test]
    fn smooth_transition_advances_changed_indices_to_render_ready() {
        let mut app = App::new();
        app.init_resource::<PendingTileStorageDiffQueue>();
        app.init_resource::<TileStorageSmoothTransitionState>();
        app.init_resource::<TileStorageApplyReport>();
        {
            let mut queue = app.world_mut().resource_mut::<PendingTileStorageDiffQueue>();
            queue.batch.chunks.push(tile_storage_diff_for_chunk(
                IVec2::new(1, 2),
                vec![3, 7],
            ));
        }
        app.add_systems(Update, apply_pending_tile_storage_diffs);
        app.update();
        let state = app.world().resource::<TileStorageSmoothTransitionState>();
        assert_eq!(tile_storage_indices_blending(state, IVec2::new(1, 2)).len(), 2);
        app.add_systems(Update, tick_tile_storage_smooth_transitions);
        for _ in 0..TILE_STORAGE_SMOOTH_STEPS {
            app.update();
        }
        let state = app.world().resource::<TileStorageSmoothTransitionState>();
        assert!(tile_storage_indices_blending(state, IVec2::new(1, 2)).is_empty());
        assert_eq!(
            tile_storage_indices_ready_for_render(state, IVec2::new(1, 2)).len(),
            2
        );
    }
}
