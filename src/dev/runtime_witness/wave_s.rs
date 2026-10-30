//! Wave S hydrate witness — `debug_runs/wave_s_hydrate_live.json` (WS-A04).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::io::save::WaveSShellHydrateWitness;

use super::common::LiveProofCadence;
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
    let body = build_wave_s_hydrate_proof_payload(witness.as_ref());
    if write_enveloped_witness(PROFILE, SOURCE, WAVE_S_HYDRATE_JSON, body) {
        state.written = true;
    }
}
