//! Async save IO — workers write buffers; main thread owns ECS apply.
//!
//! **CONC-09** — When to make asynchronous requests (gea4v2 17.7.2 / 17.7.2.1): issue early, consume a frame later.
//! LEGACY: sync request / job / streaming I/O latency hiding (gea_v2_e4 p621-624).
//!
//! Request half = [SaveIoDispatcher::submit] (non-blocking send; never touches ECS state).
//! Consume half = [poll_save_io_completions] -> [pply_save_io_completion] (	ry_recv, main thread).
//! No sync wait on the critical path; no worker-side writes to [PendingSaveApplyQueue](crate::io::save::pipeline::PendingSaveApplyQueue).

use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, TryRecvError};

use crate::io::save::dto::SavedChunkBody;
use crate::io::save::manifest::SaveWorldManifest;
use crate::io::save::pipeline::{
    build_incremental_save_manifest, chunk_artifact_path, write_artifact_atomic,
    write_manifest_atomic, SavePipelineJob,
};
use crate::io::save::wire_format::compress_payload;
use crate::io::save::registry_snapshot::write_registry_snapshot_artifacts;

/// Request to flush dirty chunk bodies off the main thread.
#[derive(Clone, Debug)]
pub struct SaveIoWorkOrder {
    pub bundle_dir: PathBuf,
    pub world_seed: u64,
    pub chunk_bodies: Vec<(IVec2, SavedChunkBody)>,
}

#[derive(Debug)]
pub enum SaveIoCompletion {
    Ready {
        manifest: SaveWorldManifest,
        jobs: Vec<SavePipelineJob>,
    },
    Failed(String),
}

#[derive(Resource, Default)]
pub struct SaveIoDispatcher {
    work_sender: Option<Sender<SaveIoWorkOrder>>,
    completion_receiver: Mutex<Option<Receiver<SaveIoCompletion>>>,
}

impl SaveIoDispatcher {
    pub fn ensure_started(&mut self) {
        if self.work_sender.is_some() {
            return;
        }
        let (work_tx, work_rx) = crossbeam_channel::unbounded::<SaveIoWorkOrder>();
        let (done_tx, done_rx) = crossbeam_channel::unbounded::<SaveIoCompletion>();
        self.work_sender = Some(work_tx);
        *self.completion_receiver.lock().expect("save IO receiver lock") = Some(done_rx);
        thread::spawn(move || {
            while let Ok(order) = work_rx.recv() {
                let result = run_save_io_work_order(order);
                if done_tx.send(result).is_err() {
                    break;
                }
            }
        });
    }

    pub fn submit(&self, order: SaveIoWorkOrder) -> bool {
        self.work_sender
            .as_ref()
            .is_some_and(|sender| sender.send(order).is_ok())
    }

    pub fn poll_completion(&self) -> Option<SaveIoCompletion> {
        let guard = self.completion_receiver.lock().expect("save IO receiver lock");
        let receiver = guard.as_ref()?;
        match receiver.try_recv() {
            Ok(completion) => Some(completion),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
}

fn run_save_io_work_order(order: SaveIoWorkOrder) -> SaveIoCompletion {
    if let Err(error) = write_registry_snapshot_artifacts(&order.bundle_dir) {
        return SaveIoCompletion::Failed(error.to_string());
    }
    let mut jobs = Vec::new();
    for (coord, body) in order.chunk_bodies {
        let encoded = match crate::io::save::dto::encode_chunk_body_ron(&body) {
            Ok(bytes) => bytes,
            Err(error) => return SaveIoCompletion::Failed(error.to_string()),
        };
        let compressed = compress_payload(&encoded);
        let artifact_path = chunk_artifact_path(&order.bundle_dir, coord);
        if let Err(error) = write_artifact_atomic(&artifact_path, &compressed) {
            return SaveIoCompletion::Failed(error.to_string());
        }
        jobs.push(SavePipelineJob {
            chunk: coord,
            artifact_path,
            body_bytes: compressed,
        });
    }
    let manifest = build_incremental_save_manifest(order.world_seed, &jobs);
    if let Err(error) = write_manifest_atomic(&order.bundle_dir, &manifest) {
        return SaveIoCompletion::Failed(error.to_string());
    }
    SaveIoCompletion::Ready { manifest, jobs }
}

/// **CONC-09** main-thread consume half: the only place a worker product reaches
/// [`PendingSaveApplyQueue`](crate::io::save::pipeline::PendingSaveApplyQueue).
/// Workers never call this; `submit` (request half) never touches the queue.
pub fn apply_save_io_completion(
    completion: SaveIoCompletion,
    pending: &mut crate::io::save::pipeline::PendingSaveApplyQueue,
) {
    match completion {
        SaveIoCompletion::Ready { jobs, .. } => pending.jobs.extend(jobs),
        SaveIoCompletion::Failed(message) => {
            bevy::log::warn!("save IO worker failed: {message}");
        }
    }
}

pub fn poll_save_io_completions(
    mut dispatcher: ResMut<SaveIoDispatcher>,
    mut pending: ResMut<crate::io::save::pipeline::PendingSaveApplyQueue>,
) {
    dispatcher.ensure_started();
    // Non-blocking `try_recv` drain — never waits on the worker (CONC-09: consume a frame later).
    while let Some(completion) = dispatcher.poll_completion() {
        apply_save_io_completion(completion, &mut pending);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::save::pipeline::PendingSaveApplyQueue;

    fn temp_bundle(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "conc09_{tag}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn submit_before_start_is_rejected_not_blocking() {
        // Request half never spins up a hidden sync fallback: un-started dispatcher refuses.
        let dispatcher = SaveIoDispatcher::default();
        let accepted = dispatcher.submit(SaveIoWorkOrder {
            bundle_dir: temp_bundle("unstarted"),
            world_seed: 1,
            chunk_bodies: Vec::new(),
        });
        assert!(!accepted);
        assert!(dispatcher.poll_completion().is_none());
    }

    #[test]
    fn request_does_not_apply_until_main_thread_poll() {
        let dir = temp_bundle("req_apply");
        let mut dispatcher = SaveIoDispatcher::default();
        dispatcher.ensure_started();
        let mut pending = PendingSaveApplyQueue::default();

        let body = SavedChunkBody {
            schema_version: crate::io::save::SAVED_CHUNK_BODY_SCHEMA_VERSION,
            chunk: [3, 4],
            cells: Vec::new(),
        };
        // Request half: returns immediately; queue is untouched by submit.
        assert!(dispatcher.submit(SaveIoWorkOrder {
            bundle_dir: dir.clone(),
            world_seed: 9,
            chunk_bodies: vec![(IVec2::new(3, 4), body)],
        }));
        assert!(pending.jobs.is_empty(), "submit must not apply products");

        // Consume half: poll (try_recv) until the worker product lands, then apply on main.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let completion = loop {
            if let Some(done) = dispatcher.poll_completion() {
                break done;
            }
            assert!(std::time::Instant::now() < deadline, "worker never completed");
            std::thread::yield_now();
        };
        assert!(
            pending.jobs.is_empty(),
            "poll_completion alone must not apply products"
        );
        apply_save_io_completion(completion, &mut pending);
        assert_eq!(pending.jobs.len(), 1);
        assert_eq!(pending.jobs[0].chunk, IVec2::new(3, 4));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failed_completion_does_not_enqueue_apply_jobs() {
        let mut pending = PendingSaveApplyQueue::default();
        apply_save_io_completion(SaveIoCompletion::Failed("disk full".into()), &mut pending);
        assert!(pending.jobs.is_empty());
    }
}
