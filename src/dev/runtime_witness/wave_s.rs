//! Wave S hydrate witness — `debug_runs/wave_s_hydrate_live.json` (WS-A04).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::io::save::WaveSShellHydrateWitness;

use super::common::{tick_live_proof_cadence, LiveProofCadence};
use super::io::write_enveloped_witness;

pub const WAVE_S_HYDRATE_JSON: &str = "debug_runs/wave_s_hydrate_live.json";

const PROFILE: &str = "WAVE_S_HYDRATE";
const SOURCE: &str = "wave_s_live_proof";

/// Slice B compat alias.
pub type WaveSLiveProofState = LiveProofCadence;

#[must_use]
pub fn build_wave_s_hydrate_proof_payload(witness: &WaveSShellHydrateWitness) -> serde_json::Value {
    serde_json::json!({
        "profile": PROFILE,
        "shell_loaded": witness.shell_loaded,
        "blueprint_count": witness.blueprint_count,
        "layout_widget_count": witness.layout_widget_count,
        "autoload_enabled": witness.autoload_enabled,
        "restore_triggered": witness.restore_triggered,
        "last_error": witness.last_error,
        "wave_s_hydrate_green": witness.shell_loaded || witness.last_error.is_none(),
    })
}

pub fn write_wave_s_hydrate_live_proof_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<WaveSLiveProofState>,
    witness: Res<WaveSShellHydrateWitness>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    if !tick_live_proof_cadence(&mut state) {
        return;
    }
    let body = build_wave_s_hydrate_proof_payload(witness.as_ref());
    if write_enveloped_witness(PROFILE, SOURCE, WAVE_S_HYDRATE_JSON, body) {
        state.written = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_s_hydrate_proof_payload_fields() {
        let witness = WaveSShellHydrateWitness {
            shell_loaded: true,
            blueprint_count: 2,
            layout_widget_count: 4,
            autoload_enabled: false,
            restore_triggered: true,
            last_error: None,
        };
        let body = build_wave_s_hydrate_proof_payload(&witness);
        assert_eq!(body.get("shell_loaded").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            body.get("wave_s_hydrate_green").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    /// Slice B — contract keys preserved after containment move.
    #[test]
    fn wave_s_hydrate_live_json_contract_keys() {
        const KEYS: &[&str] = &[
            "profile",
            "shell_loaded",
            "blueprint_count",
            "layout_widget_count",
            "autoload_enabled",
            "restore_triggered",
            "last_error",
            "wave_s_hydrate_green",
        ];
        let witness = WaveSShellHydrateWitness::default();
        let body = build_wave_s_hydrate_proof_payload(&witness);
        for key in KEYS {
            assert!(body.get(key).is_some(), "missing contract key: {key}");
        }
    }
}
