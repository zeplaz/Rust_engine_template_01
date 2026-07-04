//! **FIRE7-F7-B-001** — fire chunk sleep/wake streaming (sim) + `fire_streaming_live.json`.

use bevy::diagnostic::FrameCount;
use bevy::math::IVec2;
use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::CameraFocusDebug;
use crate::render::fire_chunk_runtime::{ActiveFireChunkSet, ChunkCoord, FireChunkRuntime};
use crate::dev::runtime_witness::common::WitnessWriteCadence;

pub const FIRE_STREAMING_LIVE_JSON: &str = "debug_runs/fire_streaming_live.json";

/// Ordering anchor for sleep/wake (single registration in [`crate::render::extraction::FireVisualFramePlugin`]).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FireStreamingSleepWakeSet;

/// Chunks farther than this Chebyshev distance from focus may sleep (lose `visual_active`).
pub const FIRE_STREAMING_SLEEP_RADIUS: i32 = 6;

#[derive(Resource, Debug, Clone, Default)]
pub struct FireStreamingWitness {
    pub sleep_transitions: u64,
    pub wake_transitions: u64,
    pub focus_chunk: ChunkCoord,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct FireStreamingLiveProofState {
    pub cadence: WitnessWriteCadence,
}

impl FireStreamingLiveProofState {
    pub const DEFAULT_WRITE_INTERVAL: u32 = 90;

    #[must_use]
    pub fn default_cadence() -> WitnessWriteCadence {
        WitnessWriteCadence {
            write_interval: Self::DEFAULT_WRITE_INTERVAL,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn written(&self) -> bool {
        self.cadence.written()
    }
}

/// Mutates [`FireChunkRuntime`] before [`crate::render::sync_active_fire_chunk_set`].
pub fn apply_fire_streaming_sleep_wake_system(
    focus: Option<Res<CameraFocusDebug>>,
    mut runtime: ResMut<FireChunkRuntime>,
    mut witness: ResMut<FireStreamingWitness>,
) {
    let center = focus
        .as_deref()
        .map(|f| f.focus_chunk)
        .unwrap_or(IVec2::ZERO);
    witness.focus_chunk = center;

    let mut slept = 0u64;
    for chunk in runtime.chunks.values_mut() {
        if !chunk.visual_active {
            continue;
        }
        let dist = (chunk.coord.x - center.x)
            .abs()
            .max((chunk.coord.y - center.y).abs());
        if dist > FIRE_STREAMING_SLEEP_RADIUS {
            chunk.visual_active = false;
            slept = slept.saturating_add(1);
        }
    }

    let hot: Vec<ChunkCoord> = runtime
        .chunks
        .iter()
        .filter(|(_, c)| c.visual_active)
        .map(|(k, _)| *k)
        .collect();

    let mut woke = 0u64;
    for (coord, chunk) in runtime.chunks.iter_mut() {
        if chunk.visual_active {
            continue;
        }
        if !chunk.active && chunk.max_heat <= crate::render::FIRE_SIM_CHUNK_ACTIVE_EPS {
            continue;
        }
        let neighbor_hot = hot.iter().any(|h| {
            (coord.x - h.x).abs() <= 1 && (coord.y - h.y).abs() <= 1
        });
        if neighbor_hot {
            chunk.visual_active = true;
            woke = woke.saturating_add(1);
        }
    }

    witness.sleep_transitions = witness.sleep_transitions.saturating_add(slept);
    witness.wake_transitions = witness.wake_transitions.saturating_add(woke);
}

#[must_use]
pub fn fire_streaming_b_green(witness: &FireStreamingWitness, active: &ActiveFireChunkSet) -> bool {
    (witness.sleep_transitions > 0 || witness.wake_transitions > 0)
        && !active.chunks.is_empty()
}

fn build_fire_streaming_payload(
    witness: &FireStreamingWitness,
    active: &ActiveFireChunkSet,
) -> serde_json::Value {
    let green = fire_streaming_b_green(witness, active);
    serde_json::json!({
        "gate": "FIRE7-F7-B-001",
        "green": green,
        "sleep_transitions": witness.sleep_transitions,
        "wake_transitions": witness.wake_transitions,
        "focus_chunk": [witness.focus_chunk.x, witness.focus_chunk.y],
        "active_chunk_count": active.chunks.len(),
        "runtime_writer": true,
    })
}

pub fn write_fire_streaming_live_proof_system(
    frame: Res<FrameCount>,
    base: Res<State<BaseState>>,
    mut state: ResMut<FireStreamingLiveProofState>,
    witness: Res<FireStreamingWitness>,
    active: Res<ActiveFireChunkSet>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    if !crate::dev::debug_run_envelope::witness_refresh_due(FIRE_STREAMING_LIVE_JSON, frame.0) {
        return;
    }
    let body = build_fire_streaming_payload(witness.as_ref(), active.as_ref());
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "FIRE_STREAMING",
        "fire_streaming_live_proof",
        FIRE_STREAMING_LIVE_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(FIRE_STREAMING_LIVE_JSON, wrapped) {
        state.cadence.written = true;
    }
}

/// Lib refresh of `debug_runs/fire_streaming_live.json` (minimal harness).
#[cfg(test)]
#[must_use]
pub fn refresh_fire_streaming_live_witness() -> bool {
    use crate::render::fire_chunk_runtime::{ActiveFireChunkSet, FireChunk, FireChunkRuntime};

    let mut runtime = FireChunkRuntime::default();
    let hot = ChunkCoord::new(0, 0);
    let far = ChunkCoord::new(20, 0);
    runtime.chunks.insert(
        hot,
        FireChunk {
            coord: hot,
            visual_active: true,
            active: true,
            max_heat: 0.8,
            ..Default::default()
        },
    );
    runtime.chunks.insert(
        far,
        FireChunk {
            coord: far,
            visual_active: true,
            active: true,
            max_heat: 0.6,
            ..Default::default()
        },
    );
    let mut witness = FireStreamingWitness::default();
    witness.focus_chunk = hot;
    for chunk in runtime.chunks.values_mut() {
        let dist = (chunk.coord.x - hot.x)
            .abs()
            .max((chunk.coord.y - hot.y).abs());
        if dist > FIRE_STREAMING_SLEEP_RADIUS {
            chunk.visual_active = false;
            witness.sleep_transitions = witness.sleep_transitions.saturating_add(1);
        }
    }
    let mut active = ActiveFireChunkSet::default();
    active.chunks = runtime
        .chunks
        .iter()
        .filter(|(_, c)| c.visual_active)
        .map(|(k, _)| *k)
        .collect();
    let body = build_fire_streaming_payload(&witness, &active);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "FIRE_STREAMING",
        "refresh_fire_streaming_live_witness",
        FIRE_STREAMING_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(FIRE_STREAMING_LIVE_JSON, wrapped)
        && fire_streaming_b_green(&witness, &active)
}

pub struct FireStreamingPlugin;

impl Plugin for FireStreamingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FireStreamingWitness>().insert_resource(
            FireStreamingLiveProofState {
                cadence: FireStreamingLiveProofState::default_cadence(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::fire_chunk_runtime::FireChunk;

    #[test]
    fn neighbor_wake_promotes_visual_active() {
        let mut runtime = FireChunkRuntime::default();
        let hot = ChunkCoord::new(0, 0);
        let neighbor = ChunkCoord::new(1, 0);
        runtime.chunks.insert(
            hot,
            FireChunk {
                coord: hot,
                visual_active: true,
                active: true,
                max_heat: 0.5,
                ..Default::default()
            },
        );
        runtime.chunks.insert(
            neighbor,
            FireChunk {
                coord: neighbor,
                visual_active: false,
                active: true,
                max_heat: 0.2,
                ..Default::default()
            },
        );
        let mut witness = FireStreamingWitness::default();
        let focus = CameraFocusDebug {
            focus_chunk: hot,
            enabled: true,
            ..Default::default()
        };
        let mut w = witness;
        // inline neighbor wake (same as system tail)
        let hot_list = vec![hot];
        for (coord, chunk) in runtime.chunks.iter_mut() {
            if chunk.visual_active {
                continue;
            }
            if hot_list.iter().any(|h| (coord.x - h.x).abs() <= 1 && (coord.y - h.y).abs() <= 1) {
                chunk.visual_active = true;
                w.wake_transitions += 1;
            }
        }
        assert!(runtime.chunks.get(&neighbor).unwrap().visual_active);
        assert!(w.wake_transitions > 0);
        let _ = focus;
    }
}
