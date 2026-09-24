//! Fire ecology witness — `debug_runs/fire_ecology_live.json` (DEV-CONTAIN-004).
//!
//! **Single writer:** [`commit_fire_ecology_live_proof`] is the only path that writes
//! this JSON. Runtime cadence and lib harness both call it. Nested `sim_effect_spine`
//! is attached only from post-drain spine payload (no double-wrap, no inflated green).

use bevy::prelude::*;
use serde_json::Value;

use crate::engine::states::BaseState;
use crate::sim::effects::{
    build_sim_effect_spine_proof_payload, SimEffectFactionReactWitness, SimEffectQueue,
    SimEffectSpineWitness, SimEffectTelemetryLedger,
};
use crate::systems::fire::witness_collectors::{build_fire_ecology_proof_payload, FireEcologyWitness};

use super::common::WitnessWriteCadence;
use super::io::{write_enveloped_witness, write_enveloped_witness_unchecked};

pub const FIRE_ECOLOGY_JSON: &str = "debug_runs/fire_ecology_live.json";

/// Serialize lib + runtime writers that share `FIRE_ECOLOGY_JSON` (parallel `cargo test`).
/// Re-entrant on the same thread so refresh+read can share one critical section.
pub struct EcologyWitnessFileGuard {
    _inner: Option<std::sync::MutexGuard<'static, ()>>,
    acquired: bool,
}

impl Drop for EcologyWitnessFileGuard {
    fn drop(&mut self) {
        if self.acquired {
            ECOLOGY_LOCK_HELD.with(|h| h.set(false));
        }
    }
}

std::thread_local! {
    static ECOLOGY_LOCK_HELD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn ecology_witness_file_lock() -> EcologyWitnessFileGuard {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    if ECOLOGY_LOCK_HELD.with(|h| h.get()) {
        return EcologyWitnessFileGuard {
            _inner: None,
            acquired: false,
        };
    }
    let guard = LOCK
        .get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    ECOLOGY_LOCK_HELD.with(|h| h.set(true));
    EcologyWitnessFileGuard {
        _inner: Some(guard),
        acquired: true,
    }
}

/// Test helper — hold while reading `FIRE_ECOLOGY_JSON` after a commit.
#[cfg(test)]
pub fn ecology_witness_file_lock_for_tests() -> EcologyWitnessFileGuard {
    ecology_witness_file_lock()
}

#[derive(Resource, Debug)]
pub struct FireEcologyLiveProofState {
    pub cadence: WitnessWriteCadence,
}

impl Default for FireEcologyLiveProofState {
    fn default() -> Self {
        Self {
            cadence: WitnessWriteCadence {
                write_interval: 90,
                ..Default::default()
            },
        }
    }
}

impl FireEcologyLiveProofState {
    #[must_use]
    pub fn written(&self) -> bool {
        self.cadence.written
    }

    pub fn set_written(&mut self, value: bool) {
        self.cadence.written = value;
    }
}

/// Optional labels written into the ecology body (lib harness gate, etc.).
#[derive(Clone, Debug, Default)]
pub struct FireEcologyCommitLabels {
    pub gate: Option<&'static str>,
    pub lib_harness: bool,
    pub source_system: &'static str,
}

impl FireEcologyCommitLabels {
    #[must_use]
    pub fn runtime() -> Self {
        Self {
            gate: None,
            lib_harness: false,
            source_system: "fire_ecology_live_proof",
        }
    }

    #[must_use]
    pub fn lib_harness() -> Self {
        Self {
            gate: Some("FIRE-ECOLOGY-REFRESH-001"),
            lib_harness: true,
            source_system: "refresh_fire_ecology_lib_harness_witness",
        }
    }
}

/// Flatten [`build_sim_effect_spine_proof_payload`] into a single ecology nest key.
///
/// Payload shape is `{ "sim_effect_spine": {...}, "faction_react_*": ... }`.
/// Nesting that whole object under another `"sim_effect_spine"` double-wraps — refuse that.
#[must_use]
pub fn nest_sim_effect_spine_for_ecology(spine_payload: &Value) -> Value {
    let mut block = spine_payload
        .get("sim_effect_spine")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    if let Some(obj) = block.as_object_mut() {
        for key in ["faction_react_wired", "faction_react_hook_rows"] {
            if let Some(v) = spine_payload.get(key) {
                obj.insert(key.to_string(), v.clone());
            }
        }
    }
    block
}

fn apply_commit_labels(body: &mut Value, labels: &FireEcologyCommitLabels) {
    let Some(obj) = body.as_object_mut() else {
        return;
    };
    if let Some(gate) = labels.gate {
        obj.insert("gate".into(), serde_json::json!(gate));
    }
    if labels.lib_harness {
        obj.insert("lib_harness".into(), serde_json::json!(true));
    }
}

/// Sole writer for `debug_runs/fire_ecology_live.json`.
///
/// When `spine_payload` is `Some`, nest a single honest `sim_effect_spine` block
/// (post-drain metrics). When `None`, omit the key — do not invent drain green.
#[must_use]
pub fn commit_fire_ecology_live_proof(
    witness: &FireEcologyWitness,
    spine_payload: Option<&Value>,
    labels: FireEcologyCommitLabels,
) -> bool {
    commit_fire_ecology_live_proof_inner(witness, spine_payload, labels, false)
}

/// Lib / forced refresh — bypasses runtime witness write gate.
#[must_use]
pub fn commit_fire_ecology_live_proof_unchecked(
    witness: &FireEcologyWitness,
    spine_payload: Option<&Value>,
    labels: FireEcologyCommitLabels,
) -> bool {
    commit_fire_ecology_live_proof_inner(witness, spine_payload, labels, true)
}

fn commit_fire_ecology_live_proof_inner(
    witness: &FireEcologyWitness,
    spine_payload: Option<&Value>,
    labels: FireEcologyCommitLabels,
    unchecked: bool,
) -> bool {
    let _guard = ecology_witness_file_lock();
    let mut body = build_fire_ecology_proof_payload(witness);
    if let Some(spine) = spine_payload {
        if let Some(obj) = body.as_object_mut() {
            obj.insert(
                "sim_effect_spine".into(),
                nest_sim_effect_spine_for_ecology(spine),
            );
        }
    }
    apply_commit_labels(&mut body, &labels);
    let profile = if labels.lib_harness {
        "FIRE-ECOLOGY-REFRESH-001"
    } else {
        "FIRE_ECOLOGY_F1"
    };
    if unchecked {
        write_enveloped_witness_unchecked(profile, labels.source_system, FIRE_ECOLOGY_JSON, body)
    } else {
        write_enveloped_witness(profile, labels.source_system, FIRE_ECOLOGY_JSON, body)
    }
}

fn spine_payload_from_world_resources(
    spine: Option<Res<SimEffectSpineWitness>>,
    queue: Option<Res<SimEffectQueue>>,
    ledger: Option<Res<SimEffectTelemetryLedger>>,
    faction: Option<Res<SimEffectFactionReactWitness>>,
) -> Option<Value> {
    let (spine, queue, ledger) = match (spine, queue, ledger) {
        (Some(s), Some(q), Some(l)) => (s, q, l),
        _ => return None,
    };
    // Honest: only nest after drain has observed activity (finalize_after_drain).
    if !spine.queue_drain_ok && queue.drained_total == 0 && queue.last_drain_count == 0 {
        return None;
    }
    Some(build_sim_effect_spine_proof_payload(
        spine.as_ref(),
        queue.as_ref(),
        ledger.as_ref(),
        faction.as_deref(),
    ))
}

pub fn write_fire_ecology_live_proof_system(
    base: Option<Res<State<BaseState>>>,
    mut state: ResMut<FireEcologyLiveProofState>,
    mut witness: ResMut<FireEcologyWitness>,
    spine: Option<Res<SimEffectSpineWitness>>,
    queue: Option<Res<SimEffectQueue>>,
    ledger: Option<Res<SimEffectTelemetryLedger>>,
    faction: Option<Res<SimEffectFactionReactWitness>>,
) {
    if !matches!(base.as_deref().map(|s| s.get()), Some(BaseState::Simulation)) {
        return;
    }

    witness.proof_json = true;
    let spine_payload = spine_payload_from_world_resources(spine, queue, ledger, faction);
    if commit_fire_ecology_live_proof(
        witness.as_ref(),
        spine_payload.as_ref(),
        FireEcologyCommitLabels::runtime(),
    ) {
        state.set_written(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nest_sim_effect_spine_flattens_double_wrap() {
        let payload = serde_json::json!({
            "sim_effect_spine": {
                "queue_drain_ok": true,
                "dedupe_ok": true,
                "effect_rows": 3
            },
            "faction_react_wired": true,
            "faction_react_hook_rows": 1
        });
        let nested = nest_sim_effect_spine_for_ecology(&payload);
        assert_eq!(nested["queue_drain_ok"], true);
        assert_eq!(nested["faction_react_wired"], true);
        assert!(nested.get("sim_effect_spine").is_none());
    }
}
