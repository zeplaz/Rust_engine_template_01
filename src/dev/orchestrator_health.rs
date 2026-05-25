//! Runtime thread/frame health samples for the build orchestrator (`ORCHESTRATOR_EXPORT_HEALTH=1`).

use bevy::prelude::*;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Resource, Default, Serialize, Clone)]
pub struct OrchestratorThreadHealthExport {
    pub threads: Vec<OrchestratorThreadHealthRow>,
}

#[derive(Clone, Serialize)]
pub struct OrchestratorThreadHealthRow {
    pub name: String,
    pub alive: bool,
    pub stalled_frames: u64,
    pub avg_frame_ms: f32,
    pub notes: String,
}

#[derive(Resource, Default)]
struct MainThreadHealthState {
    stalled_frames: u64,
    ema_frame_ms: f32,
    samples: u32,
}

pub struct OrchestratorHealthPlugin;

impl Plugin for OrchestratorHealthPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrchestratorThreadHealthExport>()
            .init_resource::<MainThreadHealthState>()
            .add_systems(Last, sample_main_thread_health)
            .add_systems(Update, export_orchestrator_health_when_requested);
    }
}

fn sample_main_thread_health(
    time: Res<Time>,
    mut export: ResMut<OrchestratorThreadHealthExport>,
    mut state: ResMut<MainThreadHealthState>,
) {
    let dt_ms = time.delta_secs() * 1000.0;
    if dt_ms > 33.0 {
        state.stalled_frames = state.stalled_frames.saturating_add(1);
    }
    if state.samples == 0 {
        state.ema_frame_ms = dt_ms;
    } else {
        state.ema_frame_ms = state.ema_frame_ms * 0.9 + dt_ms * 0.1;
    }
    state.samples = state.samples.saturating_add(1);

    export.threads.clear();
    export.threads.push(OrchestratorThreadHealthRow {
        name: "main_bevy".into(),
        alive: true,
        stalled_frames: state.stalled_frames,
        avg_frame_ms: state.ema_frame_ms,
        notes: "EMA frame delta; stall when dt > 33ms".into(),
    });
    export.threads.push(OrchestratorThreadHealthRow {
        name: "render_thread".into(),
        alive: true,
        stalled_frames: 0,
        avg_frame_ms: 0.0,
        notes: "Bevy render thread — not sampled separately in v1".into(),
    });
}

fn export_orchestrator_health_when_requested(
    export: Res<OrchestratorThreadHealthExport>,
    mut wrote: Local<bool>,
) {
    if *wrote || std::env::var_os("ORCHESTRATOR_EXPORT_HEALTH").is_none() {
        return;
    }
    if export.threads.is_empty() {
        return;
    }
    let rel = "debug_runs/orchestrator_thread_health.json";
    let body = serde_json::to_value(export.as_ref()).unwrap_or(serde_json::json!({}));
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "ORCHESTRATOR_THREAD_HEALTH",
        "orchestrator_health",
        rel,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(rel, wrapped) {
        *wrote = true;
    }
}

pub fn orchestrator_health_path() -> PathBuf {
    PathBuf::from("debug_runs/orchestrator_thread_health.json")
}

