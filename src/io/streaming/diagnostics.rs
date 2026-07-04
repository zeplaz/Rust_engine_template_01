//! STREAM_DIAG=1 / PERF=1 — explain why the streaming spine runs or skips each frame.

use bevy::diagnostic::FrameCount;
use bevy::log::info;
use bevy::prelude::*;

use super::{
    interest::interest_chunk_set_signature, ChunkCache, ChunkStreamingScheduler,
    PendingStreamApplyQueue, PendingTileStorageDiffQueue,
};

/// When true, reconstruct / apply / tile-storage systems in the spine chain are skipped.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct StreamingSpineWarmGate {
    pub skip_reconstruct_chain: bool,
}

#[derive(Resource, Debug, Default)]
pub struct StreamingSpineDiagState {
    pub last_pending_len: usize,
    pub last_pending_signature: u64,
    pub last_interest_radius: i32,
    pub last_logged_frame: u32,
    pub logged_budget_config: bool,
    pub last_pending_over_cap_frame: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StreamSpineWorkKind {
    #[default]
    Idle,
    WarmSkip,
    HydrateSync,
    HydrateAsync,
    ReconstructBatch,
    ApplyBodies,
    TileStorageDiffs,
}

#[must_use]
pub fn stream_spine_diag_enabled() -> bool {
    std::env::var("STREAM_DIAG")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[inline]
fn pending_coords_signature(coords: &[IVec2]) -> u64 {
    interest_chunk_set_signature(coords)
}

#[must_use]
pub fn refresh_streaming_spine_warm_gate(
    scheduler: &ChunkStreamingScheduler,
    cache: &ChunkCache,
    apply_queue: &PendingStreamApplyQueue,
    tile_queue: &PendingTileStorageDiffQueue,
) -> StreamingSpineWarmGate {
    let all_cached = !scheduler.pending_chunks.is_empty()
        && scheduler.all_pending_chunks_cached(cache);
    let skip = all_cached
        && scheduler.staged_chunk_bodies.is_empty()
        && scheduler.jobs.is_empty()
        && apply_queue.ready_bodies.is_empty()
        && tile_queue.batch.chunks.is_empty();
    StreamingSpineWarmGate {
        skip_reconstruct_chain: skip,
    }
}

#[must_use]
pub fn classify_streaming_spine_work(
    gate: &StreamingSpineWarmGate,
    scheduler: &ChunkStreamingScheduler,
    cache: &ChunkCache,
    apply_queue: &PendingStreamApplyQueue,
    tile_queue: &PendingTileStorageDiffQueue,
    io_in_flight: bool,
) -> StreamSpineWorkKind {
    if gate.skip_reconstruct_chain {
        return StreamSpineWorkKind::WarmSkip;
    }
    if !scheduler.staged_chunk_bodies.is_empty() {
        return StreamSpineWorkKind::ReconstructBatch;
    }
    if !apply_queue.ready_bodies.is_empty() {
        return StreamSpineWorkKind::ApplyBodies;
    }
    if !tile_queue.batch.chunks.is_empty() {
        return StreamSpineWorkKind::TileStorageDiffs;
    }
    if scheduler.pending_chunks.is_empty() {
        return StreamSpineWorkKind::Idle;
    }
    if io_in_flight {
        return StreamSpineWorkKind::HydrateAsync;
    }
    if scheduler.all_pending_chunks_cached(cache) {
        return StreamSpineWorkKind::Idle;
    }
    if !scheduler.staged_chunk_bodies.is_empty() {
        return StreamSpineWorkKind::ReconstructBatch;
    }
    let missing = scheduler
        .pending_chunks
        .iter()
        .filter(|c| cache.get(**c).is_none())
        .count();
    if missing > 0 {
        StreamSpineWorkKind::HydrateSync
    } else {
        StreamSpineWorkKind::Idle
    }
}

pub fn log_pending_chunks_changed(
    frame: u32,
    world_radius: i32,
    scheduler: &ChunkStreamingScheduler,
    diag: &mut StreamingSpineDiagState,
) {
    if !stream_spine_diag_enabled() {
        return;
    }
    let sig = pending_coords_signature(&scheduler.pending_chunks);
    let len = scheduler.pending_chunks.len();
    let changed = len != diag.last_pending_len
        || sig != diag.last_pending_signature
        || world_radius != diag.last_interest_radius;
    if !changed && frame.saturating_sub(diag.last_logged_frame) < 120 {
        return;
    }
    if changed {
        info!(
            target: "proc_A_dine01::io::streaming::diag",
            frame,
            pending_len = len,
            pending_sig = sig,
            interest_radius_chunks = world_radius,
            jobs = scheduler.jobs.len(),
            staged = scheduler.staged_chunk_bodies.len(),
            "STREAM pending_chunks changed (interest enqueue)"
        );
    }
    diag.last_pending_len = len;
    diag.last_pending_signature = sig;
    diag.last_interest_radius = world_radius;
    diag.last_logged_frame = frame;
}

pub fn log_streaming_spine_frame_summary(
    frame: u32,
    gate: &StreamingSpineWarmGate,
    scheduler: &ChunkStreamingScheduler,
    cache: &ChunkCache,
    apply_queue: &PendingStreamApplyQueue,
    tile_queue: &PendingTileStorageDiffQueue,
    io_in_flight: bool,
    diag: &mut StreamingSpineDiagState,
) {
    if !stream_spine_diag_enabled() {
        return;
    }
    let work = classify_streaming_spine_work(
        gate,
        scheduler,
        cache,
        apply_queue,
        tile_queue,
        io_in_flight,
    );
    let missing = if scheduler.pending_chunks.is_empty() {
        0
    } else {
        scheduler
            .pending_chunks
            .iter()
            .filter(|c| cache.get(**c).is_none())
            .count()
    };
    let all_cached = !scheduler.pending_chunks.is_empty()
        && scheduler.all_pending_chunks_cached(cache);
    let should_log = work != StreamSpineWorkKind::Idle
        || frame.saturating_sub(diag.last_logged_frame) >= 60;
    if !should_log {
        return;
    }
    info!(
        target: "proc_A_dine01::io::streaming::diag",
        frame,
        ?work,
        warm_skip = gate.skip_reconstruct_chain,
        pending_len = scheduler.pending_chunks.len(),
        all_pending_cached = all_cached,
        missing_in_cache = missing,
        jobs = scheduler.jobs.len(),
        staged = scheduler.staged_chunk_bodies.len(),
        apply_ready = apply_queue.ready_bodies.len(),
        tile_diff_chunks = tile_queue.batch.chunks.len(),
        "STREAM spine frame summary"
    );
    diag.last_logged_frame = frame;
}

pub fn refresh_streaming_spine_warm_gate_system(
    scheduler: Res<ChunkStreamingScheduler>,
    cache: Res<ChunkCache>,
    apply_queue: Res<PendingStreamApplyQueue>,
    tile_queue: Res<PendingTileStorageDiffQueue>,
    mut gate: ResMut<StreamingSpineWarmGate>,
) {
    *gate = refresh_streaming_spine_warm_gate(
        scheduler.as_ref(),
        cache.as_ref(),
        apply_queue.as_ref(),
        tile_queue.as_ref(),
    );
}

pub fn log_streaming_spine_frame_summary_system(
    frame: Res<FrameCount>,
    gate: Res<StreamingSpineWarmGate>,
    scheduler: Res<ChunkStreamingScheduler>,
    cache: Res<ChunkCache>,
    apply_queue: Res<PendingStreamApplyQueue>,
    tile_queue: Res<PendingTileStorageDiffQueue>,
    dispatcher: Res<super::ChunkStreamIoDispatcher>,
    mut diag: ResMut<StreamingSpineDiagState>,
) {
    log_streaming_spine_frame_summary(
        frame.0,
        gate.as_ref(),
        scheduler.as_ref(),
        cache.as_ref(),
        apply_queue.as_ref(),
        tile_queue.as_ref(),
        dispatcher.in_flight,
        diag.as_mut(),
    );
}

fn warm_gate_allows_reconstruct_chain(gate: Option<Res<StreamingSpineWarmGate>>) -> bool {
    !gate.is_some_and(|g| g.skip_reconstruct_chain)
}

pub fn streaming_warm_gate_allows_reconstruct() -> impl Fn(Option<Res<StreamingSpineWarmGate>>) -> bool + Clone
{
    warm_gate_allows_reconstruct_chain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warm_gate_requires_pending_and_empty_queues() {
        let scheduler = ChunkStreamingScheduler::default();
        let cache = ChunkCache::default();
        let gate = refresh_streaming_spine_warm_gate(
            &scheduler,
            &cache,
            &PendingStreamApplyQueue::default(),
            &PendingTileStorageDiffQueue::default(),
        );
        assert!(!gate.skip_reconstruct_chain);
    }

    #[test]
    fn pending_signature_stable_for_same_coords() {
        let a = pending_coords_signature(&[IVec2::new(1, 2), IVec2::new(3, 4)]);
        let b = pending_coords_signature(&[IVec2::new(1, 2), IVec2::new(3, 4)]);
        assert_eq!(a, b);
    }

    #[test]
    fn warm_gate_skips_when_pending_all_cached_and_idle() {
        use crate::io::streaming::chunk_cache::{ChunkCache, ChunkCacheEntry};
        use crate::io::streaming::ChunkStreamingScheduler;

        let coord = IVec2::new(1, 2);
        let mut scheduler = ChunkStreamingScheduler::default();
        scheduler.pending_chunks.push(coord);
        let mut cache = ChunkCache::default();
        cache.entries.insert(
            coord,
            ChunkCacheEntry {
                coord,
                material_names: Vec::new(),
                content_hash: 1,
                last_touch: 1,
            },
        );
        let gate = refresh_streaming_spine_warm_gate(
            &scheduler,
            &cache,
            &PendingStreamApplyQueue::default(),
            &PendingTileStorageDiffQueue::default(),
        );
        assert!(gate.skip_reconstruct_chain);
    }
}
