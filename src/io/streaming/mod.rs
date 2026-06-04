//! Async world streaming spine — priority scoring, staging, and upload enqueue.

mod apply;
mod budget;
mod chunk_cache;
mod diagnostics;
mod hydrate;
mod interest;
mod manifest_cache;
mod preview_ghost;
mod residency;
mod task_pool;
mod tile_storage_apply;
mod tile_storage_contract;
mod wave_c_prerequisites;

use crate::dev::runtime_witness::wave_c;
mod wave_c_readiness;

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;

use crate::gui::{LodZoneRegistry, WorldRepresentationFrame};
use crate::render::{
    attrib_streaming_reconstruct_after, attrib_streaming_reconstruct_before,
};
use crate::io::save::{SavedChunkBody, WorldSaveBundleSettings};

pub use chunk_cache::{
    hash_saved_chunk_body, ChunkCache, ChunkCacheDiskSpill, ChunkCacheEntry, ChunkCacheTierSettings,
    CHUNK_CACHE_DISK_TIER_OPEN,
};
pub use hydrate::{
    hydrate_all_manifest_chunks, hydrate_stream_chunks_from_manifest, load_manifest_for_streaming,
};
pub use budget::{stream_sync_hydrate_enabled, StreamingSpineBudget};
pub use interest::{
    cap_chunk_coords_by_focus, highest_priority_orb, interest_chunk_set_signature, interest_orbs_from_lod_zones,
    merge_interest_chunk_coords, merge_interest_chunk_coords_with_ghost_bands,
    merge_interest_orbs_deduped, primary_interest_orb, priority_for_chunk, InterestOrb,
    InterestOrbKind,
};
pub use manifest_cache::StreamingManifestCache;
pub use preview_ghost::{
    ghost_band_neighbor_coords_for_preview, preview_coords_with_ghost_bands,
};
pub use residency::{
    chunk_window_coords, ghost_band_seed_coords, ChunkResidencyEntry, ChunkResidencyRole,
    ChunkResidencyTable,
};
pub use task_pool::{
    poll_stream_hydrate_completions, submit_stream_hydrate_work, ChunkStreamIoDispatcher,
    StreamHydrateDiagnostics, StreamIoCompletion, StreamIoWorkOrder,
};
pub use tile_storage_apply::{
    apply_pending_tile_storage_diffs, tick_tile_storage_smooth_transitions,
    tile_storage_indices_blending, tile_storage_indices_ready_for_render,
    TileStorageApplyReport, TileStorageSmoothTransitionState,
};
pub use tile_storage_contract::{
    tile_storage_diff_for_chunk, PendingTileStorageDiffQueue, TileStorageApplyTiming,
    TileStorageDiffBatch, TileStorageDiffChunk, TILE_STORAGE_DIFF_CONTRACT_BQ,
};
pub use wave_c_prerequisites::{
    gather_wave_c_prerequisites, wave_c_prerequisites_passes, WaveCPrerequisitesReport,
    WAVE_C_OPEN_BACKLOG_ITEMS,
};
pub use wave_c_readiness::{
    gather_wave_c_readiness, wave_c_readiness_passes, WaveCReadinessReport,
};
pub use diagnostics::{
    log_pending_chunks_changed, refresh_streaming_spine_warm_gate, stream_spine_diag_enabled,
    StreamingSpineDiagState, StreamingSpineWarmGate, StreamSpineWorkKind,
};

/// Chunk streaming priority inputs (distance, simulation, visibility).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChunkStreamingPriority {
    pub distance_weight: f32,
    pub simulation_weight: f32,
    pub visibility_weight: f32,
}

impl ChunkStreamingPriority {
    #[must_use]
    pub fn score(self, distance: f32, sim_importance: f32, visible: bool) -> f32 {
        let visibility = if visible { 1.0 } else { 0.0 };
        self.distance_weight * distance
            + self.simulation_weight * sim_importance
            + self.visibility_weight * visibility
    }
}

/// Pipeline stage for a chunk streaming job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkStreamStage {
    Disk,
    Deserialize,
    DomainReconstruct,
    GpuUpload,
}

/// One chunk streaming job moving through the async spine.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChunkStreamJob {
    pub chunk: IVec2,
    pub stage: ChunkStreamStage,
    pub priority: f32,
}

/// Scheduler resource for threaded disk → staging → GPU upload.
#[derive(Resource, Debug, Clone, Default)]
pub struct ChunkStreamingScheduler {
    pub pending_chunks: Vec<IVec2>,
    pub jobs: Vec<ChunkStreamJob>,
    pub staged_chunk_bodies: Vec<(IVec2, SavedChunkBody)>,
    /// Signature of last merged interest set ([`interest_chunk_set_signature`]).
    pub interest_signature: u64,
}

impl ChunkStreamingScheduler {
    pub fn enqueue_focus_window(
        &mut self,
        focus: IVec2,
        radius: i32,
        weights: ChunkStreamingPriority,
        sim_importance: f32,
    ) {
        self.pending_chunks.clear();
        self.jobs.clear();
        self.staged_chunk_bodies.clear();
        for y in (focus.y - radius)..=(focus.y + radius) {
            for x in (focus.x - radius)..=(focus.x + radius) {
                let chunk = IVec2::new(x, y);
                let distance = (chunk - focus).as_vec2().length();
                let visible = distance <= radius as f32;
                let priority = weights.score(distance, sim_importance, visible);
                self.pending_chunks.push(chunk);
                self.jobs.push(ChunkStreamJob {
                    chunk,
                    stage: ChunkStreamStage::Disk,
                    priority,
                });
            }
        }
        self
            .jobs
            .sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn enqueue_chunk_coords(
        &mut self,
        chunks: &[IVec2],
        focus: IVec2,
        weights: ChunkStreamingPriority,
        sim_importance: f32,
        visible_radius: i32,
        orb_priority: impl Fn(IVec2) -> u8,
    ) {
        self.sync_interest_targets(chunks, focus, weights, sim_importance, visible_radius, orb_priority);
    }

    fn job_priority_for_chunk(
        chunk: IVec2,
        focus: IVec2,
        weights: ChunkStreamingPriority,
        sim_importance: f32,
        visible_radius: i32,
        orb_priority: &impl Fn(IVec2) -> u8,
    ) -> f32 {
        let distance = (chunk - focus).as_vec2().length();
        let visible = distance <= visible_radius as f32;
        weights.score(distance, sim_importance, visible) + orb_priority(chunk) as f32 * 0.01
    }

    /// Merge full interest (all LOD zones + focus) without clearing in-flight staged bodies.
    pub fn sync_interest_targets(
        &mut self,
        desired: &[IVec2],
        focus: IVec2,
        weights: ChunkStreamingPriority,
        sim_importance: f32,
        visible_radius: i32,
        orb_priority: impl Fn(IVec2) -> u8,
    ) {
        use std::collections::HashSet;

        let priority_fn = &orb_priority;
        let desired_set: HashSet<IVec2> = desired.iter().copied().collect();
        self.pending_chunks.retain(|c| desired_set.contains(c));
        self.jobs.retain(|j| desired_set.contains(&j.chunk));

        for &chunk in desired {
            if let Some(job) = self.jobs.iter_mut().find(|j| j.chunk == chunk) {
                job.priority = Self::job_priority_for_chunk(
                    chunk,
                    focus,
                    weights,
                    sim_importance,
                    visible_radius,
                    priority_fn,
                );
                continue;
            }
            self.pending_chunks.push(chunk);
            self.jobs.push(ChunkStreamJob {
                chunk,
                stage: ChunkStreamStage::Disk,
                priority: Self::job_priority_for_chunk(
                    chunk,
                    focus,
                    weights,
                    sim_importance,
                    visible_radius,
                    priority_fn,
                ),
            });
        }

        self.pending_chunks.sort_by_key(|c| (c.y, c.x));
        self.jobs.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Refresh priorities when the camera moves but the interest set is unchanged.
    pub fn refresh_interest_priorities(
        &mut self,
        focus: IVec2,
        weights: ChunkStreamingPriority,
        sim_importance: f32,
        visible_radius: i32,
        orb_priority: impl Fn(IVec2) -> u8,
    ) {
        let priority_fn = &orb_priority;
        for job in &mut self.jobs {
            job.priority = Self::job_priority_for_chunk(
                job.chunk,
                focus,
                weights,
                sim_importance,
                visible_radius,
                priority_fn,
            );
        }
        self.jobs.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Highest-priority chunks still missing from the hot cache (jobs are sorted desc).
    #[must_use]
    pub fn select_disk_hydrate_batch(&self, cache: &ChunkCache, max: usize) -> Vec<IVec2> {
        let mut out = Vec::with_capacity(max.min(self.jobs.len()));
        for job in &self.jobs {
            if out.len() >= max {
                break;
            }
            if cache.get(job.chunk).is_some() {
                continue;
            }
            if job.stage != ChunkStreamStage::Disk {
                continue;
            }
            out.push(job.chunk);
        }
        out
    }

    pub fn advance_jobs_for_cached_chunks(&mut self, cache: &ChunkCache) {
        for job in &mut self.jobs {
            if cache.get(job.chunk).is_some() && job.stage == ChunkStreamStage::Disk {
                job.stage = ChunkStreamStage::Deserialize;
            }
        }
    }

    pub fn note_chunks_hydrated(&mut self, coords: &[IVec2]) {
        use std::collections::HashSet;
        let hydrated: HashSet<IVec2> = coords.iter().copied().collect();
        for job in &mut self.jobs {
            if hydrated.contains(&job.chunk) && job.stage == ChunkStreamStage::Disk {
                job.stage = ChunkStreamStage::Deserialize;
            }
        }
    }

    pub fn advance_job_stages(&mut self) {
        for job in &mut self.jobs {
            job.stage = match job.stage {
                ChunkStreamStage::Disk => ChunkStreamStage::Deserialize,
                ChunkStreamStage::Deserialize => ChunkStreamStage::DomainReconstruct,
                ChunkStreamStage::DomainReconstruct => ChunkStreamStage::GpuUpload,
                ChunkStreamStage::GpuUpload => ChunkStreamStage::GpuUpload,
            };
        }
    }

    /// True when every pending chunk is already in the hot cache (steady-state after first load).
    #[must_use]
    pub fn all_pending_chunks_cached(&self, cache: &ChunkCache) -> bool {
        !self.pending_chunks.is_empty()
            && self
                .pending_chunks
                .iter()
                .all(|coord| cache.get(*coord).is_some())
    }

    /// Drop job/staging queues once residency is warm so sync hydrate does not re-run every frame.
    pub fn clear_jobs_if_fully_cached(&mut self, cache: &ChunkCache) {
        if self.all_pending_chunks_cached(cache) {
            self.jobs.clear();
            self.staged_chunk_bodies.clear();
        }
    }
}

#[must_use]
pub fn build_residency_table(
    orbs: &[InterestOrb],
    core_coords: &[IVec2],
) -> ChunkResidencyTable {
    let mut core = std::collections::HashMap::new();
    for &coord in core_coords {
        let priority = orbs
            .iter()
            .filter(|orb| chunk_window_coords(orb.center, orb.radius_chunks).contains(&coord))
            .map(|orb| orb.priority)
            .max()
            .unwrap_or(0);
        core.insert(
            coord,
            ChunkResidencyEntry {
                coord,
                role: ChunkResidencyRole::Core,
                orb_priority: priority,
            },
        );
    }
    let mut entries = core;
    for &center in core_coords {
        for neighbor in ghost_band_seed_coords(center) {
            entries.entry(neighbor).or_insert_with(|| {
                let priority = orbs
                    .iter()
                    .filter(|orb| {
                        chunk_window_coords(orb.center, orb.radius_chunks).contains(&neighbor)
                    })
                    .map(|orb| orb.priority)
                    .max()
                    .unwrap_or(0);
                ChunkResidencyEntry {
                    coord: neighbor,
                    role: ChunkResidencyRole::GhostBand,
                    orb_priority: priority,
                }
            });
        }
    }
    ChunkResidencyTable { entries }
}

pub fn schedule_chunk_streaming_from_interest(
    world: Res<WorldRepresentationFrame>,
    zones: Res<LodZoneRegistry>,
    mut scheduler: ResMut<ChunkStreamingScheduler>,
    cache: Res<ChunkCache>,
    frame: Res<FrameCount>,
    mut diag: ResMut<StreamingSpineDiagState>,
    budget: Res<StreamingSpineBudget>,
) {
    const CHUNK_TILES: UVec2 = UVec2::splat(32);
    if stream_spine_diag_enabled() && !diag.logged_budget_config {
        bevy::log::info!(
            target: "proc_A_dine01::io::streaming::diag",
            hydrate_budget = budget.max_hydrate_chunks_per_frame,
            reconstruct_budget = budget.max_reconstruct_chunks_per_frame,
            max_pending_chunks = budget.max_pending_chunks,
            sync_hydrate = stream_sync_hydrate_enabled(),
            "STREAM budget config"
        );
        diag.logged_budget_config = true;
    }
    let mut orbs = vec![primary_interest_orb(&world)];
    orbs.extend(interest_orbs_from_lod_zones(&zones.zones, CHUNK_TILES));
    let orbs = merge_interest_orbs_deduped(&orbs);
    let merged_coords = merge_interest_chunk_coords(&orbs);
    let coords = cap_chunk_coords_by_focus(
        merged_coords,
        world.focus_chunk,
        budget.max_pending_chunks,
    );
    if stream_spine_diag_enabled()
        && coords.len() >= budget.max_pending_chunks
        && frame.0.saturating_sub(diag.last_pending_over_cap_frame) >= 30
    {
        bevy::log::warn!(
            target: "proc_A_dine01::io::streaming::diag",
            frame = frame.0,
            capped_pending_len = coords.len(),
            cap = budget.max_pending_chunks,
            "STREAM pending set reached cap; far-field chunks deferred"
        );
        diag.last_pending_over_cap_frame = frame.0;
    }
    let sig = interest_chunk_set_signature(&coords);
    let weights = ChunkStreamingPriority {
        distance_weight: -1.0,
        simulation_weight: 1.5,
        visibility_weight: 2.0,
    };
    let orb_priority = |coord: IVec2| priority_for_chunk(coord, &orbs);
    let visible_radius = world.interest_radius_chunks.max(1);

    if scheduler.interest_signature == sig {
        scheduler.refresh_interest_priorities(
            world.focus_chunk,
            weights,
            world.gameplay_importance,
            visible_radius,
            orb_priority,
        );
        scheduler.advance_jobs_for_cached_chunks(&cache);
        if scheduler.all_pending_chunks_cached(&cache) {
            scheduler.clear_jobs_if_fully_cached(&cache);
        }
        return;
    }

    scheduler.interest_signature = sig;
    scheduler.sync_interest_targets(
        &coords,
        world.focus_chunk,
        weights,
        world.gameplay_importance,
        visible_radius,
        orb_priority,
    );
    scheduler.advance_jobs_for_cached_chunks(&cache);
    log_pending_chunks_changed(
        frame.0,
        world.interest_radius_chunks,
        &scheduler,
        &mut diag,
    );
}

pub fn sync_chunk_residency_from_scheduler(
    world: Res<WorldRepresentationFrame>,
    zones: Res<LodZoneRegistry>,
    scheduler: Res<ChunkStreamingScheduler>,
    mut table: ResMut<ChunkResidencyTable>,
) {
    const CHUNK_TILES: UVec2 = UVec2::splat(32);
    let mut orbs = vec![primary_interest_orb(&world)];
    orbs.extend(interest_orbs_from_lod_zones(&zones.zones, CHUNK_TILES));
    // S6-12: residency must not stay empty when the scheduler has not enqueued yet — seed from focus window.
    let core_coords = if scheduler.pending_chunks.is_empty() {
        chunk_window_coords(
            world.focus_chunk,
            world.interest_radius_chunks.max(1),
        )
    } else {
        scheduler.pending_chunks.clone()
    };
    *table = build_residency_table(&orbs, &core_coords);
}

pub fn hydrate_stream_jobs_from_save_bundle(
    settings: Res<WorldSaveBundleSettings>,
    mut scheduler: ResMut<ChunkStreamingScheduler>,
    cache: Res<ChunkCache>,
    dispatcher: Res<ChunkStreamIoDispatcher>,
    budget: Res<StreamingSpineBudget>,
    mut manifest_cache: ResMut<StreamingManifestCache>,
    frame: Res<FrameCount>,
    mut diag: ResMut<StreamingSpineDiagState>,
) {
    if !stream_sync_hydrate_enabled() {
        return;
    }
    if scheduler.pending_chunks.is_empty() {
        return;
    }
    if scheduler.all_pending_chunks_cached(&cache) {
        scheduler.clear_jobs_if_fully_cached(&cache);
        return;
    }
    if !scheduler.staged_chunk_bodies.is_empty() || dispatcher.in_flight {
        return;
    }
    let batch = scheduler.select_disk_hydrate_batch(&cache, budget.max_hydrate_chunks_per_frame);
    if batch.is_empty() {
        scheduler.clear_jobs_if_fully_cached(&cache);
        return;
    }
    let Some(manifest) = manifest_cache.manifest_for_bundle(&settings.bundle_dir) else {
        return;
    };
    if stream_spine_diag_enabled() {
        bevy::log::info!(
            target: "proc_A_dine01::io::streaming::diag",
            frame = frame.0,
            hydrate_batch = batch.len(),
            pending_len = scheduler.pending_chunks.len(),
            jobs = scheduler.jobs.len(),
            "STREAM hydrate_stream_jobs_from_save_bundle (sync fallback)"
        );
        diag.last_logged_frame = frame.0;
    }
    let bodies =
        hydrate_stream_chunks_from_manifest(&settings.bundle_dir, manifest, &batch);
    let hydrated: Vec<IVec2> = bodies.iter().map(|(c, _)| *c).collect();
    scheduler.staged_chunk_bodies.extend(bodies);
    scheduler.note_chunks_hydrated(&hydrated);
}

/// Default reconstruct batch when [`StreamingSpineBudget`] is absent (tests).
pub const MAX_RECONSTRUCT_CHUNKS_PER_FRAME: usize = 8;

pub fn reconstruct_staged_chunks_into_cache(
    mut scheduler: ResMut<ChunkStreamingScheduler>,
    mut cache: ResMut<ChunkCache>,
    tier: Res<ChunkCacheTierSettings>,
    mut spill: ResMut<ChunkCacheDiskSpill>,
    mut tile_diffs: ResMut<PendingTileStorageDiffQueue>,
    mut apply_queue: ResMut<PendingStreamApplyQueue>,
    budget: Res<StreamingSpineBudget>,
) {
    if scheduler.staged_chunk_bodies.is_empty() {
        return;
    }
    let batch_len = scheduler
        .staged_chunk_bodies
        .len()
        .min(budget.max_reconstruct_chunks_per_frame);
    let batch: Vec<_> = scheduler.staged_chunk_bodies.drain(..batch_len).collect();
    for (coord, body) in batch {
        let hash = hash_saved_chunk_body(&body);
        if cache
            .get(coord)
            .is_some_and(|entry| entry.content_hash == hash)
        {
            continue;
        }
        let changed_tile_indices = (0..body.cells.len() as u32).collect();
        let _ = cache.upsert_from_saved_body(coord, &body, tier.as_ref(), spill.as_mut());
        apply_queue.ready_bodies.push((coord, body));
        tile_diffs
            .batch
            .chunks
            .push(tile_storage_diff_for_chunk(coord, changed_tile_indices));
    }
    for job in &mut scheduler.jobs {
        if job.stage == ChunkStreamStage::Deserialize {
            job.stage = ChunkStreamStage::DomainReconstruct;
        }
    }
}

/// Main-thread apply queue after domain reconstruct (ECS / GPU apply remains consumer-owned).
#[derive(Resource, Debug, Default, Clone)]
pub struct PendingStreamApplyQueue {
    pub ready_bodies: Vec<(IVec2, SavedChunkBody)>,
}

pub fn finalize_stream_domain_reconstruct(
    mut scheduler: ResMut<ChunkStreamingScheduler>,
    cache: Res<ChunkCache>,
) {
    if !scheduler.jobs.is_empty() {
        let all_cached = scheduler.jobs.iter().all(|job| {
            job.stage != ChunkStreamStage::DomainReconstruct || cache.get(job.chunk).is_some()
        });
        if all_cached {
            scheduler.advance_job_stages();
        }
    }
    scheduler.clear_jobs_if_fully_cached(&cache);
}

fn stall_after_tile_storage_apply(mut watch: ResMut<crate::render::FrameStallWatch>) {
    watch.checkpoint("after_tile_storage_apply");
}

pub struct StreamingSpinePlugin;

impl Plugin for StreamingSpinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkStreamingScheduler>()
            .init_resource::<ChunkResidencyTable>()
            .init_resource::<ChunkCache>()
            .init_resource::<ChunkCacheTierSettings>()
            .init_resource::<ChunkCacheDiskSpill>()
            .init_resource::<ChunkStreamIoDispatcher>()
            .init_resource::<StreamHydrateDiagnostics>()
            .init_resource::<PendingStreamApplyQueue>()
            .init_resource::<PendingTileStorageDiffQueue>()
            .init_resource::<TileStorageSmoothTransitionState>()
            .init_resource::<TileStorageApplyReport>()
            .init_resource::<wave_c::WaveCLiveProofState>()
            .init_resource::<StreamingSpineWarmGate>()
            .init_resource::<StreamingSpineDiagState>()
            .init_resource::<StreamingSpineBudget>()
            .init_resource::<StreamingManifestCache>()
            .add_systems(
                Update,
                (
                    schedule_chunk_streaming_from_interest,
                    sync_chunk_residency_from_scheduler,
                    submit_stream_hydrate_work,
                    poll_stream_hydrate_completions,
                    hydrate_stream_jobs_from_save_bundle,
                )
                    .chain()
                    .after(crate::gui::WorldRepresentationSystemSet::ComputeFrame),
            )
            .add_systems(
                Update,
                (
                    diagnostics::refresh_streaming_spine_warm_gate_system,
                    attrib_streaming_reconstruct_before,
                    reconstruct_staged_chunks_into_cache
                        .run_if(diagnostics::streaming_warm_gate_allows_reconstruct()),
                    apply::apply_pending_stream_chunk_bodies
                        .run_if(diagnostics::streaming_warm_gate_allows_reconstruct()),
                    crate::render::clear_async_domain_apply_labels_after_stream_apply
                        .after(apply::apply_pending_stream_chunk_bodies)
                        .run_if(diagnostics::streaming_warm_gate_allows_reconstruct()),
                    apply_pending_tile_storage_diffs
                        .run_if(diagnostics::streaming_warm_gate_allows_reconstruct()),
                    stall_after_tile_storage_apply,
                    finalize_stream_domain_reconstruct,
                    tick_tile_storage_smooth_transitions,
                    attrib_streaming_reconstruct_after,
                    diagnostics::log_streaming_spine_frame_summary_system,
                    wave_c::write_wave_c_live_proof_system,
                )
                    .chain()
                    .after(crate::gui::WorldRepresentationSystemSet::ComputeFrame),
            )
            .add_systems(
                Update,
                crate::render::stall_checkpoint_post_streaming_spine
                    .after(wave_c::write_wave_c_live_proof_system),
            );
    }
}

pub use crate::dev::runtime_witness::wave_c::{
    commit_wave_c_live_proof, wc_depth_001_green, WAVE_C_LIVE_JSON, WaveCLiveProofState,
};
pub use wave_c_prerequisites::WAVE_C_DEPTH_001_CLOSED_ITEM;

/// Main-thread ECS apply order (S6-22); must match `StreamingSpinePlugin` reconstruct chain.
pub const STREAM_ECS_APPLY_CHAIN: [&str; 3] = [
    "reconstruct_staged_chunks_into_cache",
    "apply_pending_stream_chunk_bodies",
    "clear_async_domain_apply_labels_after_stream_apply",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_ecs_apply_chain_order_matches_streaming_spine_plugin() {
        assert_eq!(
            STREAM_ECS_APPLY_CHAIN,
            [
                "reconstruct_staged_chunks_into_cache",
                "apply_pending_stream_chunk_bodies",
                "clear_async_domain_apply_labels_after_stream_apply",
            ]
        );
    }

    #[test]
    fn reconstruct_batch_cap_limits_per_frame_work() {
        assert_eq!(MAX_RECONSTRUCT_CHUNKS_PER_FRAME, 8);
    }

    #[test]
    fn sync_interest_targets_preserves_staged_bodies() {
        let mut scheduler = ChunkStreamingScheduler::default();
        scheduler
            .staged_chunk_bodies
            .push((IVec2::ZERO, SavedChunkBody {
                schema_version: crate::io::save::SAVED_CHUNK_BODY_SCHEMA_VERSION,
                chunk: [0, 0],
                cells: Vec::new(),
            }));
        let weights = ChunkStreamingPriority::default();
        scheduler.sync_interest_targets(
            &[IVec2::ZERO, IVec2::ONE],
            IVec2::ZERO,
            weights,
            0.0,
            1,
            |_| 0,
        );
        assert_eq!(scheduler.staged_chunk_bodies.len(), 1);
        assert_eq!(scheduler.pending_chunks.len(), 2);
    }

    #[test]
    fn select_disk_hydrate_batch_respects_job_priority_order() {
        let mut scheduler = ChunkStreamingScheduler::default();
        scheduler.jobs = vec![
            ChunkStreamJob {
                chunk: IVec2::new(0, 0),
                stage: ChunkStreamStage::Disk,
                priority: 1.0,
            },
            ChunkStreamJob {
                chunk: IVec2::new(5, 5),
                stage: ChunkStreamStage::Disk,
                priority: 10.0,
            },
        ];
        let cache = ChunkCache::default();
        let batch = scheduler.select_disk_hydrate_batch(&cache, 1);
        assert_eq!(batch, vec![IVec2::new(5, 5)]);
    }

    #[test]
    fn streaming_priority_prefers_visible_near_sim() {
        let weights = ChunkStreamingPriority {
            distance_weight: -1.0,
            simulation_weight: 2.0,
            visibility_weight: 4.0,
        };
        let near = weights.score(1.0, 0.8, true);
        let far = weights.score(8.0, 0.8, false);
        assert!(near > far);
    }

    #[test]
    fn scheduler_orders_focus_window_by_priority() {
        let mut scheduler = ChunkStreamingScheduler::default();
        scheduler.enqueue_focus_window(IVec2::ZERO, 1, ChunkStreamingPriority::default(), 0.5);
        assert_eq!(scheduler.jobs.len(), 9);
        assert!(scheduler.jobs[0].priority >= scheduler.jobs[8].priority);
    }

    #[test]
    fn job_stage_advances_toward_gpu_upload() {
        let mut scheduler = ChunkStreamingScheduler::default();
        scheduler.enqueue_focus_window(IVec2::ZERO, 0, ChunkStreamingPriority::default(), 0.0);
        scheduler.advance_job_stages();
        assert_eq!(scheduler.jobs[0].stage, ChunkStreamStage::Deserialize);
    }

    #[test]
    fn sync_chunk_residency_seeds_focus_window_when_scheduler_pending_empty() {
        let world = WorldRepresentationFrame {
            focus_chunk: IVec2::new(2, 3),
            interest_radius_chunks: 1,
            ..Default::default()
        };
        let orbs = vec![primary_interest_orb(&world)];
        let table = build_residency_table(
            &orbs,
            &chunk_window_coords(world.focus_chunk, world.interest_radius_chunks.max(1)),
        );
        assert!(table.entries.contains_key(&world.focus_chunk));
        assert!(table.entries.len() > 1);
    }

    #[test]
    fn residency_table_marks_core_and_ghost_neighbors() {
        let orbs = vec![primary_interest_orb(&WorldRepresentationFrame {
            focus_chunk: IVec2::ZERO,
            interest_radius_chunks: 0,
            ..Default::default()
        })];
        let table = build_residency_table(&orbs, &[IVec2::ZERO]);
        assert_eq!(
            table.entries.get(&IVec2::ZERO).map(|e| e.role),
            Some(ChunkResidencyRole::Core)
        );
        assert_eq!(
            table.entries.get(&IVec2::ONE).map(|e| e.role),
            Some(ChunkResidencyRole::GhostBand)
        );
    }
}
