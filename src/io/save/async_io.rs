//! Async save IO â€” workers write buffers; main thread owns ECS apply.

use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, TryRecvError};

use crate::io::save::dto::SavedChunkBody;
use crate::io::save::manifest::SaveWorldManifest;
use crate::io::save::pipeline::{
    build_incremental_save_manifest, chunk_artifact_path, compress_payload, write_artifact_atomic,
    write_manifest_atomic, SavePipelineJob,
};
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

pub fn poll_save_io_completions(
    mut dispatcher: ResMut<SaveIoDispatcher>,
    mut pending: ResMut<crate::io::save::pipeline::PendingSaveApplyQueue>,
) {
    dispatcher.ensure_started();
    while let Some(completion) = dispatcher.poll_completion() {
        match completion {
            SaveIoCompletion::Ready { jobs, .. } => pending.jobs.extend(jobs),
            SaveIoCompletion::Failed(message) => {
                bevy::log::warn!("save IO worker failed: {message}");
            }
        }
    }
}
