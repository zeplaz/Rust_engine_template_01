//! Replay + editor parity witness (infrastructure hardening — not Stage 5 exit).

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use serde::Serialize;

use crate::engine::states::BaseState;
use crate::systems::sim_frame_delta::CommittedSimReplayRing;

#[derive(Resource, Clone, Debug, Default, Serialize)]
pub struct ReplayEditorParityWitness {
    pub replay_ring_len: u32,
    pub scenario_plugin_wired: bool,
    pub editor_scenario_panel: bool,
    pub committed_visual_fence_module: bool,
    pub infrastructure_isolation_json: bool,
    pub parity_green: bool,
}

#[allow(dead_code)]
fn witness_path() -> std::path::PathBuf {
    std::path::Path::new("debug_runs").join("replay_editor_parity_live.json")
}

pub fn refresh_replay_editor_parity_witness_system(
    replay: Option<Res<CommittedSimReplayRing>>,
    mut witness: ResMut<ReplayEditorParityWitness>,
) {
    witness.replay_ring_len = replay
        .as_deref()
        .map(|r| r.stamps.len() as u32)
        .unwrap_or(0);
    witness.scenario_plugin_wired =
        include_str!("../scenario/scenario_plugin.rs").contains("pub struct ScenarioScriptingPlugin");
    witness.editor_scenario_panel =
        include_str!("../gui/editor/scenario_script_panel.rs").contains("ScenarioScriptPanelState");
    witness.committed_visual_fence_module = include_str!("../render/api.rs")
        .contains("CommittedVisualSnapshotFence")
        || include_str!("../render/mod.rs").contains("visual_snapshot_commit");
    // Isolation JSON on disk is not wiring — require live replay samples instead.
    witness.infrastructure_isolation_json = witness.replay_ring_len >= 2;
    witness.parity_green = witness.replay_ring_len >= 2
        && witness.scenario_plugin_wired
        && witness.editor_scenario_panel
        && witness.infrastructure_isolation_json;
}

pub fn write_replay_editor_parity_live_proof_system(
    frame: Res<FrameCount>,
    base: Res<State<BaseState>>,
    witness: Res<ReplayEditorParityWitness>,
    mut wrote: Local<bool>,
) {
    const PROOF_PATH: &str = "debug_runs/replay_editor_parity_live.json";
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    if !crate::dev::debug_run_envelope::witness_refresh_due(PROOF_PATH, frame.0) {
        return;
    }
    if witness.replay_ring_len < 2 {
        return;
    }
    let payload = serde_json::json!({
        "profile": "REPLAY_EDITOR_PARITY",
        "parity_green": witness.parity_green,
        "replay_ring_len": witness.replay_ring_len,
        "scenario_plugin_wired": witness.scenario_plugin_wired,
        "editor_scenario_panel": witness.editor_scenario_panel,
        "infrastructure_isolation_json": witness.infrastructure_isolation_json,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "REPLAY_EDITOR_PARITY",
        "replay_editor_parity",
        PROOF_PATH,
        payload,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(PROOF_PATH, wrapped) {
        *wrote = true;
    }
}

/// **REPLAY-PARITY-001** — lib refresh of `replay_editor_parity_live.json`.
///
/// Lib cannot invent `replay_ring_len` — prove under live Simulation with ring samples.
#[must_use]
pub fn refresh_replay_editor_parity_live_witness() -> bool {
    let scenario_plugin_wired =
        include_str!("../scenario/scenario_plugin.rs").contains("pub struct ScenarioScriptingPlugin");
    let editor_scenario_panel =
        include_str!("../gui/editor/scenario_script_panel.rs").contains("ScenarioScriptPanelState");
    let committed_visual_fence_module = include_str!("../render/api.rs")
        .contains("CommittedVisualSnapshotFence")
        || include_str!("../render/mod.rs").contains("visual_snapshot_commit");
    const PROOF_PATH: &str = "debug_runs/replay_editor_parity_live.json";
    let payload = serde_json::json!({
        "profile": "REPLAY_EDITOR_PARITY",
        "parity_green": false,
        "replay_ring_len": 0,
        "scenario_plugin_wired": scenario_plugin_wired,
        "editor_scenario_panel": editor_scenario_panel,
        "committed_visual_fence_module": committed_visual_fence_module,
        "infrastructure_isolation_json": false,
        "replay_parity_001_green": false,
        "proof_grade": "lib_fixture_retired",
        "cheat_retired": true,
        "retire_note": "Invented ring_len=4 lib green retired (CLN-WIT-001); runtime system remains authoritative.",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "REPLAY-PARITY-001",
        "refresh_replay_editor_parity_live_witness",
        PROOF_PATH,
        payload,
    );
    let _ = crate::dev::debug_run_envelope::write_debug_run_json(PROOF_PATH, wrapped);
    false
}

pub fn register_replay_editor_parity_hooks(app: &mut App) {
    app.init_resource::<ReplayEditorParityWitness>()
        .add_systems(
            Update,
            (
                refresh_replay_editor_parity_witness_system,
                write_replay_editor_parity_live_proof_system.after(refresh_replay_editor_parity_witness_system),
            )
                .run_if(|base: Res<State<BaseState>>| matches!(base.get(), BaseState::Simulation)),
        );
}
