//! Async world streaming spine — priority scoring, staging, and upload enqueue.

mod apply;
mod chunk_cache;
mod hydrate;
mod interest;
mod preview_ghost;
mod residency;
mod task_pool;
mod tile_storage_apply;
mod tile_storage_contract;
mod wave_c_prerequisites;
mod wave_c_live_proof;
mod wave_c_readiness;

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
pub use interest::{
    highest_priority_orb, interest_orbs_from_lod_zones, merge_interest_chunk_coords,
    merge_interest_chunk_coords_with_ghost_bands, merge_interest_orbs_deduped,
    primary_interest_orb, priority_for_chunk, InterestOrb, InterestOrbKind,
};
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
    tile_storage_diff_for_chunk, PendingTileStorageDiffQueue, TileStorageDiffBatch,
    TileStorageDiffChunk, TILE_STORAGE_DIFF_CONTRACT_BQ,
};
pub use wave_c_prerequisites::{
    gather_wave_c_prerequisites, wave_c_prerequisites_passes, WaveCPrerequisitesReport,
    WAVE_C_OPEN_BACKLOG_ITEMS,
};
pub use wave_c_readiness::{
    gather_wave_c_readiness, wave_c_readiness_passes, WaveCReadinessReport,
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
        self.pending_chunks = chunks.to_vec();
        self.jobs.clear();
        self.staged_chunk_bodies.clear();
        for &chunk in chunks {
            let distance = (chunk - focus).as_vec2().length();
            let visible = distance <= visible_radius as f32;
            let priority = weights.score(distance, sim_importance, visible)
                + orb_priority(chunk) as f32 * 0.01;
            self.jobs.push(ChunkStreamJob {
                chunk,
                stage: ChunkStreamStage::Disk,
                priority,
            });
        }
        self
            .jobs
            .sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
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
) {
    const CHUNK_TILES: UVec2 = UVec2::splat(32);
    let mut orbs = vec![primary_interest_orb(&world)];
    orbs.extend(interest_orbs_from_lod_zones(&zones.zones, CHUNK_TILES));
    let orbs = merge_interest_orbs_deduped(&orbs);
    let coords = merge_interest_chunk_coords(&orbs);
    let weights = ChunkStreamingPriority {
        distance_weight: -1.0,
        simulation_weight: 1.5,
        visibility_weight: 2.0,
    };
    let orb_priority = |coord: IVec2| priority_for_chunk(coord, &orbs);
    scheduler.enqueue_chunk_coords(
        &coords,
        world.focus_chunk,
        weights,
        world.gameplay_importance,
        world.interest_radius_chunks.max(1),
        orb_priority,
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
) {
    if scheduler.jobs.is_empty() || !scheduler.staged_chunk_bodies.is_empty() {
        return;
    }
    let Some(manifest) = load_manifest_for_streaming(&settings.bundle_dir) else {
        return;
    };
    let chunks = scheduler.pending_chunks.clone();
    if chunks.is_empty() {
        return;
    }
    scheduler.staged_chunk_bodies =
        hydrate_stream_chunks_from_manifest(&settings.bundle_dir, &manifest, &chunks);
    for job in &mut scheduler.jobs {
        if job.stage == ChunkStreamStage::Disk {
            job.stage = ChunkStreamStage::Deserialize;
        }
    }
}

pub fn reconstruct_staged_chunks_into_cache(
    mut scheduler: ResMut<ChunkStreamingScheduler>,
    mut cache: ResMut<ChunkCache>,
    tier: Res<ChunkCacheTierSettings>,
    mut spill: ResMut<ChunkCacheDiskSpill>,
    mut tile_diffs: ResMut<PendingTileStorageDiffQueue>,
    mut apply_queue: ResMut<PendingStreamApplyQueue>,
) {
    if scheduler.staged_chunk_bodies.is_empty() {
        return;
    }
    for (coord, body) in scheduler.staged_chunk_bodies.drain(..) {
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
    if scheduler.jobs.is_empty() {
        return;
    }
    let all_cached = scheduler.jobs.iter().all(|job| {
        job.stage != ChunkStreamStage::DomainReconstruct || cache.get(job.chunk).is_some()
    });
    if all_cached {
        scheduler.advance_job_stages();
    }
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
            .init_resource::<wave_c_live_proof::WaveCLiveProofState>()
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
                    attrib_streaming_reconstruct_before,
                    reconstruct_staged_chunks_into_cache,
                    apply::apply_pending_stream_chunk_bodies,
                    crate::render::clear_async_domain_apply_labels_after_stream_apply
                        .after(apply::apply_pending_stream_chunk_bodies),
                    apply_pending_tile_storage_diffs,
                    stall_after_tile_storage_apply,
                    finalize_stream_domain_reconstruct,
                    tick_tile_storage_smooth_transitions,
                    attrib_streaming_reconstruct_after,
                    wave_c_live_proof::write_wave_c_live_proof_system,
                )
                    .chain()
                    .after(crate::gui::WorldRepresentationSystemSet::ComputeFrame),
            );
    }
}

pub use wave_c_live_proof::{
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
