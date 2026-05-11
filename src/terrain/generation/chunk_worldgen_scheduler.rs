//! Chunk-scheduled worldgen backbone: priority queue → U7 [`ChunkDirty`].
//!
//! - **CPU remains authoritative** through the existing materialization pipeline; this scheduler prioritizes
//!   work and widens [`ChunkDirty::passes`] when jobs dispatch.
//! - **Preview / U7:** `invalidate_world` and `mark_chunks_dirty_on_asset_change` already update [`ChunkDirty`].
//!   This module does not mirror the preview queue by default (avoids duplicating or widening masks
//!   after tooling has already set pass-specific dirtiness).
//! - Optional [`GpuChunkGenPipeline`] is a placeholder for future noise / field compute (GPU suggests,
//!   CPU validates).

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

use bevy::prelude::*;

use crate::terrain::generation::passes::p1_fields::fill_fields;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::{ChunkDependency, ChunkDirty, DIRTY_ALL, DIRTY_PASSES_2_THROUGH_6};

// -----------------------------------------------------------------------------
// Reasons & jobs
// -----------------------------------------------------------------------------

/// Why a chunk needs (re)generation. Priority (high → low):
/// `EditorEdit` > `DirtyDependency` > `CameraVisible` > `MissionInfluence` > `WorldInit`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChunkGenReason {
    WorldInit,
    MissionInfluence,
    CameraVisible,
    DirtyDependency,
    EditorEdit,
}

impl ChunkGenReason {
    #[inline]
    fn rank(self) -> u8 {
        match self {
            ChunkGenReason::WorldInit => 1,
            ChunkGenReason::MissionInfluence => 2,
            ChunkGenReason::CameraVisible => 3,
            ChunkGenReason::DirtyDependency => 4,
            ChunkGenReason::EditorEdit => 5,
        }
    }
}

/// Single unit of chunk generation work for the scheduler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkGenJob {
    pub chunk_coord: IVec2,
    /// Fine priority within the same [`ChunkGenReason`] (larger = sooner).
    pub priority: u8,
    pub reason: ChunkGenReason,
    /// Monotonic sequence for stable ordering.
    pub seq: u64,
}

impl PartialOrd for ChunkGenJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChunkGenJob {
    fn cmp(&self, other: &Self) -> Ordering {
        self.reason
            .rank()
            .cmp(&other.reason.rank())
            .then_with(|| self.priority.cmp(&other.priority))
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

// -----------------------------------------------------------------------------
// Resources
// -----------------------------------------------------------------------------

/// Max-heap scheduling front + coord dedup + sequence counter.
#[derive(Resource, Debug)]
pub struct ChunkGenQueue {
    heap: BinaryHeap<ChunkGenJob>,
    queued_coords: HashSet<IVec2>,
    seq: u64,
}

impl Default for ChunkGenQueue {
    fn default() -> Self {
        Self {
            heap: BinaryHeap::new(),
            queued_coords: HashSet::new(),
            seq: 0,
        }
    }
}

impl ChunkGenQueue {
    #[inline]
    fn next_seq(&mut self) -> u64 {
        self.seq = self.seq.wrapping_add(1);
        self.seq
    }

    /// Highest-priority editor jobs: [`ChunkGenReason::EditorEdit`] with `priority = u8::MAX`.
    pub fn push_editor_edit(&mut self, chunk_coord: IVec2) {
        let seq = self.next_seq();
        self.push_job(ChunkGenJob {
            chunk_coord,
            priority: u8::MAX,
            reason: ChunkGenReason::EditorEdit,
            seq,
        });
    }

    /// Schedule `job` unless this chunk is already waiting in the heap.
    pub fn push_job(&mut self, job: ChunkGenJob) {
        if self.queued_coords.insert(job.chunk_coord) {
            self.heap.push(job);
        }
    }

    pub fn pop_job(&mut self) -> Option<ChunkGenJob> {
        let job = self.heap.pop()?;
        self.queued_coords.remove(&job.chunk_coord);
        Some(job)
    }

    #[inline]
    pub fn pending_len(&self) -> usize {
        self.heap.len()
    }
}

#[derive(Resource, Clone, Debug)]
pub struct ChunkGenConfig {
    pub max_jobs_per_frame: usize,
    /// When set, [`ChunkGenCameraWindow`] enqueues a ring each frame (enable for streaming gameplay).
    pub schedule_camera_ring: bool,
}

impl Default for ChunkGenConfig {
    fn default() -> Self {
        Self {
            max_jobs_per_frame: 8,
            // Off by default: tests and minimal apps should not constantly widen dirty flags.
            schedule_camera_ring: false,
        }
    }
}

/// Editor / camera driver for visible-chunk scheduling (strategic sim can update this from a real camera later).
#[derive(Resource, Clone, Debug)]
pub struct ChunkGenCameraWindow {
    pub center_chunk: IVec2,
    pub radius: i32,
}

impl Default for ChunkGenCameraWindow {
    fn default() -> Self {
        Self {
            center_chunk: IVec2::ZERO,
            radius: 2,
        }
    }
}

/// Placeholder for GPU noise / mask targets (no compute shaders wired yet).
#[derive(Resource, Clone, Default, Debug)]
pub struct GpuChunkGenPipeline;

impl GpuChunkGenPipeline {
    /// Future: dispatch compute for `chunk` into internal buffers; CPU pass must still validate.
    pub fn dispatch_gpu_chunk_gen(&mut self, _chunk: IVec2) {
        // Stub: GPU path lands in Phase 3.
    }
}

/// Future hook: normalized rects for preview texture patches per chunk (populated by preview raster).
#[derive(Resource, Clone, Default, Debug)]
pub struct ChunkTexturePatchQueue {
    pub patches: Vec<(IVec2, Rect)>,
}

/// Strategic layer fills this; [`queue_mission_hint_jobs`] drains into [`ChunkGenQueue`] (keeps terrain free of `strategic` imports).
#[derive(Resource, Clone, Default, Debug)]
pub struct ChunkGenMissionChunkHints {
    pub coords: Vec<IVec2>,
}

// -----------------------------------------------------------------------------
// CPU baseline (authoritative fields pass — optional direct call; U7 remains source of truth for full stack)
// -----------------------------------------------------------------------------

/// Passes 1–3 equivalent (height, moisture, temperature) for a chunk matrix using world params.
///
/// Classification / hydrology / materialize still run when [`ChunkDirty`] drives the material plugin.
pub fn generate_chunk_cpu_height_moisture_temp(
    matrix: &mut ChunkCellMatrix,
    chunk_coord: IVec2,
    params: &WorldGenParams,
) {
    fill_fields(matrix, chunk_coord, params, None);
}

// -----------------------------------------------------------------------------
// Systems
// -----------------------------------------------------------------------------

pub fn queue_visible_chunks(
    cfg: Res<ChunkGenConfig>,
    window: Res<ChunkGenCameraWindow>,
    mut queue: ResMut<ChunkGenQueue>,
) {
    if !cfg.schedule_camera_ring {
        return;
    }
    let r = window.radius.max(0);
    let c = window.center_chunk;
    for dy in -r..=r {
        for dx in -r..=r {
            let coord = IVec2::new(c.x + dx, c.y + dy);
            let seq = queue.next_seq();
            queue.push_job(ChunkGenJob {
                chunk_coord: coord,
                priority: 0,
                reason: ChunkGenReason::CameraVisible,
                seq,
            });
        }
    }
}

pub fn queue_mission_hint_jobs(mut hints: ResMut<ChunkGenMissionChunkHints>, mut queue: ResMut<ChunkGenQueue>) {
    for coord in hints.coords.drain(..) {
        let seq = queue.next_seq();
        queue.push_job(ChunkGenJob {
            chunk_coord: coord,
            priority: 0,
            reason: ChunkGenReason::MissionInfluence,
            seq,
        });
    }
}

#[inline]
fn dirty_mask_for_reason(reason: ChunkGenReason) -> u8 {
    match reason {
        ChunkGenReason::DirtyDependency => DIRTY_PASSES_2_THROUGH_6,
        ChunkGenReason::WorldInit
        | ChunkGenReason::MissionInfluence
        | ChunkGenReason::CameraVisible
        | ChunkGenReason::EditorEdit => DIRTY_ALL,
    }
}

pub fn dispatch_chunk_jobs(
    cfg: Res<ChunkGenConfig>,
    mut queue: ResMut<ChunkGenQueue>,
    mut gpu: ResMut<GpuChunkGenPipeline>,
    mut chunks: Query<(Entity, &Chunk, &mut ChunkDirty), With<ChunkDependency>>,
) {
    let mut dispatched = 0usize;
    let max = cfg.max_jobs_per_frame.max(1);

    while dispatched < max {
        let Some(job) = queue.pop_job() else {
            break;
        };

        if job.reason == ChunkGenReason::MissionInfluence || job.reason == ChunkGenReason::CameraVisible {
            gpu.dispatch_gpu_chunk_gen(job.chunk_coord);
        }

        let mut found = false;
        for (_e, chunk, mut dirty) in chunks.iter_mut() {
            if chunk.coord == job.chunk_coord {
                dirty.passes |= dirty_mask_for_reason(job.reason);
                found = true;
                break;
            }
        }
        if found {
            dispatched += 1;
        }
    }
}

pub struct ChunkWorldgenSchedulerPlugin;

impl Plugin for ChunkWorldgenSchedulerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkGenQueue>()
            .init_resource::<ChunkGenConfig>()
            .init_resource::<ChunkGenCameraWindow>()
            .init_resource::<GpuChunkGenPipeline>()
            .init_resource::<ChunkTexturePatchQueue>()
            .init_resource::<ChunkGenMissionChunkHints>()
            .add_systems(
                Update,
                (
                    queue_mission_hint_jobs,
                    queue_visible_chunks,
                    dispatch_chunk_jobs,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_job_ordering_prefers_editor_over_world_init() {
        let a = ChunkGenJob {
            chunk_coord: IVec2::ZERO,
            priority: 0,
            reason: ChunkGenReason::WorldInit,
            seq: 1,
        };
        let b = ChunkGenJob {
            chunk_coord: IVec2::ONE,
            priority: 0,
            reason: ChunkGenReason::EditorEdit,
            seq: 0,
        };
        assert!(b > a);
    }

    #[test]
    fn queue_dedupes_same_coord() {
        let mut q = ChunkGenQueue::default();
        let s1 = q.next_seq();
        q.push_job(ChunkGenJob {
            chunk_coord: IVec2::ZERO,
            priority: 0,
            reason: ChunkGenReason::WorldInit,
            seq: s1,
        });
        let s2 = q.next_seq();
        q.push_job(ChunkGenJob {
            chunk_coord: IVec2::ZERO,
            priority: 0,
            reason: ChunkGenReason::EditorEdit,
            seq: s2,
        });
        assert_eq!(q.pending_len(), 1);
    }
}
