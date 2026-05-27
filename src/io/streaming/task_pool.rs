//! Off-thread chunk hydrate — main thread owns ECS / GPU apply.

use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, TryRecvError};

use crate::io::save::SavedChunkBody;

use super::hydrate::{hydrate_stream_chunks_from_manifest, load_manifest_for_streaming};
use super::{ChunkCache, ChunkStreamStage, ChunkStreamingScheduler};

#[derive(Resource, Default)]
pub struct StreamHydrateDiagnostics {
    pub logged_missing_manifest: bool,
    pub suppressed_missing_manifest_warnings: u64,
    pub last_failure_message: Option<String>,
    pub suppressed_worker_failures: u64,
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

pub fn submit_stream_hydrate_work(
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    scheduler: Res<ChunkStreamingScheduler>,
    cache: Res<ChunkCache>,
    mut dispatcher: ResMut<ChunkStreamIoDispatcher>,
    mut diagnostics: ResMut<StreamHydrateDiagnostics>,
) {
    if scheduler.pending_chunks.is_empty() || dispatcher.in_flight {
        return;
    }
    // PERF-PLAY-001: only hydrate chunks not already in the hot cache — re-submitting the full
    // pending window every completion frame was forcing ~650ms reconstruct/apply loops.
    let chunks: Vec<IVec2> = scheduler
        .pending_chunks
        .iter()
        .copied()
        .filter(|coord| cache.get(*coord).is_none())
        .collect();
    if chunks.is_empty() {
        return;
    }
    if load_manifest_for_streaming(&settings.bundle_dir).is_none() {
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
        match completion {
            StreamIoCompletion::Ready(bodies) => {
                scheduler.staged_chunk_bodies = bodies;
                for job in &mut scheduler.jobs {
                    if job.stage == ChunkStreamStage::Disk {
                        job.stage = ChunkStreamStage::Deserialize;
                    }
                }
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
}
