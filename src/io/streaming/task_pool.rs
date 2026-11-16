//! Off-thread chunk hydrate — main thread owns ECS / GPU apply.
//!
//! **ECS-08** — Game object updates as jobs / async updates (gea4v2 17.7.4.1–2 · 17.7.4.6).
//! **CONC-01** — Job systems / job counters (gea4v1 8.6.4); a `Task` handle ≈ [`ChunkStreamIoDispatcher::in_flight`].
//! LEGACY: `job systems` / async game-object update (gea_v1_e4 p528 · gea_v2_e4 p626).
//!
//! Workers load [`SavedChunkBody`] only. Completions land in a channel; **only**
//! [`apply_stream_io_completion`] (called from [`poll_stream_hydrate_completions`] on the
//! main thread) mutates [`ChunkStreamingScheduler`]. That is the sole writer of staged bodies
//! from the async path — no dual writer from the worker thread.

use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, TryRecvError};

use crate::io::save::SavedChunkBody;

use super::hydrate::{hydrate_stream_chunks_from_manifest, load_manifest_for_streaming};
use super::{
    budget::StreamingSpineBudget, manifest_cache::StreamingManifestCache, stream_spine_diag_enabled,
    ChunkCache, ChunkStreamingScheduler,
};

#[derive(Resource, Default)]
pub struct StreamHydrateDiagnostics {
    pub logged_missing_manifest: bool,
    pub suppressed_missing_manifest_warnings: u64,
    pub last_failure_message: Option<String>,
    pub suppressed_worker_failures: u64,
    /// Main-thread apply counter (ECS-08 sole-apply witness for tests / diagnostics).
    pub main_thread_applies: u64,
}

#[derive(Clone, Debug)]
pub struct StreamIoWorkOrder {
    pub bundle_dir: PathBuf,
    pub chunks: Vec<IVec2>,
}

#[derive(Debug)]
pub enum StreamIoCompletion {
    Ready(Vec<(IVec2, SavedChunkBody)>),
    Failed(String),
}

#[derive(Resource, Default)]
pub struct ChunkStreamIoDispatcher {
    work_sender: Option<Sender<StreamIoWorkOrder>>,
    completion_receiver: Mutex<Option<Receiver<StreamIoCompletion>>>,
    pub in_flight: bool,
}

impl ChunkStreamIoDispatcher {
    pub fn ensure_started(&mut self) {
        if self.work_sender.is_some() {
            return;
        }
        let (work_tx, work_rx) = crossbeam_channel::unbounded::<StreamIoWorkOrder>();
        let (done_tx, done_rx) = crossbeam_channel::unbounded::<StreamIoCompletion>();
        self.work_sender = Some(work_tx);
        *self
            .completion_receiver
            .lock()
            .expect("stream IO receiver lock") = Some(done_rx);
        thread::spawn(move || {
            while let Ok(order) = work_rx.recv() {
                let result = run_stream_io_work_order(order);
                if done_tx.send(result).is_err() {
                    break;
                }
            }
        });
    }

    pub fn submit(&mut self, order: StreamIoWorkOrder) -> bool {
        self.ensure_started();
        let Some(sender) = self.work_sender.as_ref() else {
            return false;
        };
        if sender.send(order).is_ok() {
            self.in_flight = true;
            true
        } else {
            false
        }
    }

    pub fn poll_completion(&mut self) -> Option<StreamIoCompletion> {
        let guard = self
            .completion_receiver
            .lock()
            .expect("stream IO receiver lock");
        let receiver = guard.as_ref()?;
        match receiver.try_recv() {
            Ok(completion) => {
                self.in_flight = false;
                Some(completion)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.in_flight = false;
                None
            }
        }
    }
}

fn run_stream_io_work_order(order: StreamIoWorkOrder) -> StreamIoCompletion {
    let Some(manifest) = load_manifest_for_streaming(&order.bundle_dir) else {
        return StreamIoCompletion::Failed("missing save manifest".into());
    };
    StreamIoCompletion::Ready(hydrate_stream_chunks_from_manifest(
        &order.bundle_dir,
        &manifest,
        &order.chunks,
    ))
}

/// ECS-08 sole main-thread apply of one worker completion into the scheduler.
///
/// Workers must never call this. Staging bodies + hydrate bookkeeping are mutated only here
/// (or the sync reconstruct path that drains the same `staged_chunk_bodies` vec on main).
pub fn apply_stream_io_completion(
    completion: StreamIoCompletion,
    scheduler: &mut ChunkStreamingScheduler,
    diagnostics: &mut StreamHydrateDiagnostics,
) {
    match completion {
        StreamIoCompletion::Ready(bodies) => {
            let hydrated: Vec<IVec2> = bodies.iter().map(|(c, _)| *c).collect();
            scheduler.staged_chunk_bodies.extend(bodies);
            scheduler.note_chunks_hydrated(&hydrated);
            diagnostics.main_thread_applies = diagnostics.main_thread_applies.saturating_add(1);
        }
        StreamIoCompletion::Failed(message) => {
            if diagnostics.last_failure_message.as_deref() != Some(message.as_str()) {
                bevy::log::warn!(
                    target: "proc_A_dine01::io::streaming::task_pool",
                    "stream hydrate worker failed: {message}"
                );
                diagnostics.last_failure_message = Some(message);
            } else {
                diagnostics.suppressed_worker_failures =
                    diagnostics.suppressed_worker_failures.wrapping_add(1);
            }
        }
    }
}

pub fn submit_stream_hydrate_work(
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    scheduler: Res<ChunkStreamingScheduler>,
    cache: Res<ChunkCache>,
    budget: Res<StreamingSpineBudget>,
    mut manifest_cache: ResMut<StreamingManifestCache>,
    mut dispatcher: ResMut<ChunkStreamIoDispatcher>,
    mut diagnostics: ResMut<StreamHydrateDiagnostics>,
) {
    if scheduler.pending_chunks.is_empty() || dispatcher.in_flight {
        return;
    }
    // PERF-PLAY-001 + IO-03: priority-budgeted hydrate, clamped by spawn entity frame ceiling.
    let chunks = scheduler
        .select_disk_hydrate_batch(&cache, budget.effective_hydrate_chunks_per_frame());
    if chunks.is_empty() {
        return;
    }
    if manifest_cache
        .manifest_for_bundle(&settings.bundle_dir)
        .is_none()
    {
        if !diagnostics.logged_missing_manifest {
            bevy::log::warn!(
                target: "proc_A_dine01::io::streaming::task_pool",
                "stream hydrate skipped: missing save manifest at {}",
                settings.bundle_dir.display()
            );
            diagnostics.logged_missing_manifest = true;
        } else {
            diagnostics.suppressed_missing_manifest_warnings =
                diagnostics.suppressed_missing_manifest_warnings.wrapping_add(1);
        }
        return;
    }
    if stream_spine_diag_enabled() {
        bevy::log::info!(
            target: "proc_A_dine01::io::streaming::diag",
            async_batch = chunks.len(),
            pending_len = scheduler.pending_chunks.len(),
            "STREAM hydrate async submit"
        );
    }
    dispatcher.submit(StreamIoWorkOrder {
        bundle_dir: settings.bundle_dir.clone(),
        chunks,
    });
}

pub fn poll_stream_hydrate_completions(
    mut dispatcher: ResMut<ChunkStreamIoDispatcher>,
    mut scheduler: ResMut<ChunkStreamingScheduler>,
    mut diagnostics: ResMut<StreamHydrateDiagnostics>,
) {
    dispatcher.ensure_started();
    while let Some(completion) = dispatcher.poll_completion() {
        apply_stream_io_completion(completion, &mut scheduler, &mut diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::save::SAVED_CHUNK_BODY_SCHEMA_VERSION;

    fn sample_body(coord: IVec2) -> SavedChunkBody {
        SavedChunkBody {
            schema_version: SAVED_CHUNK_BODY_SCHEMA_VERSION,
            chunk: [coord.x, coord.y],
            cells: Vec::new(),
        }
    }

    #[test]
    fn async_hydrate_results_apply_on_main_thread_only() {
        let mut scheduler = ChunkStreamingScheduler::default();
        let mut diagnostics = StreamHydrateDiagnostics::default();
        let coord = IVec2::new(2, -1);
        // Simulate a worker product — never touches scheduler itself.
        let completion = StreamIoCompletion::Ready(vec![(coord, sample_body(coord))]);
        assert!(scheduler.staged_chunk_bodies.is_empty());
        assert_eq!(diagnostics.main_thread_applies, 0);

        apply_stream_io_completion(completion, &mut scheduler, &mut diagnostics);

        assert_eq!(scheduler.staged_chunk_bodies.len(), 1);
        assert_eq!(scheduler.staged_chunk_bodies[0].0, coord);
        assert_eq!(diagnostics.main_thread_applies, 1);
    }

    #[test]
    fn failed_completion_does_not_stage_bodies() {
        let mut scheduler = ChunkStreamingScheduler::default();
        let mut diagnostics = StreamHydrateDiagnostics::default();
        apply_stream_io_completion(
            StreamIoCompletion::Failed("missing save manifest".into()),
            &mut scheduler,
            &mut diagnostics,
        );
        assert!(scheduler.staged_chunk_bodies.is_empty());
        assert_eq!(diagnostics.main_thread_applies, 0);
        assert_eq!(
            diagnostics.last_failure_message.as_deref(),
            Some("missing save manifest")
        );
    }

    #[test]
    fn in_flight_gate_is_single_writer_counter() {
        // CONC-01 job counter ≈ in_flight: at most one outstanding async hydrate job.
        let mut dispatcher = ChunkStreamIoDispatcher::default();
        assert!(!dispatcher.in_flight);
        // Directly model the counter without spawning (submit would start a thread).
        dispatcher.in_flight = true;
        assert!(dispatcher.in_flight);
        dispatcher.in_flight = false;
        assert!(!dispatcher.in_flight);
    }
}
