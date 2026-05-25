//! Live witness: `debug_runs/wave_s_hydrate_live.json` (WS-A04).

use bevy::prelude::*;

use crate::engine::states::BaseState;

use super::wave_s_artifacts::WaveSShellHydrateWitness;

pub const WAVE_S_HYDRATE_JSON: &str = "debug_runs/wave_s_hydrate_live.json";

#[derive(Resource, Debug)]
pub struct WaveSLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for WaveSLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 120,
            written: false,
        }
    }
}

#[must_use]
pub fn build_wave_s_hydrate_proof_payload(witness: &WaveSShellHydrateWitness) -> serde_json::Value {
    serde_json::json!({
        "profile": "WAVE_S_HYDRATE",
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
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;
    let body = build_wave_s_hydrate_proof_payload(witness.as_ref());
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "WAVE_S_HYDRATE",
        "wave_s_live_proof",
        WAVE_S_HYDRATE_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(WAVE_S_HYDRATE_JSON, wrapped) {
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
}
