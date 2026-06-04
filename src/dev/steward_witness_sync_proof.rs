//! STEWARD-WITNESS-SYNC-001 — reconcile lib-refreshed proofs + [`agent_debug_index.json`](../../debug_runs/agent_debug_index.json).

use std::path::PathBuf;

use serde_json::Value;

use crate::dev::debug_run_envelope::{refresh_agent_debug_index, AGENT_DEBUG_INDEX_PATH};

const BUNDLE: &[(&str, &[&str])] = &[
    (
        "debug_runs/ui_shell_migration_live.json",
        &["/phase2b_closed", "/ui_p2a_coder_b/green"],
    ),
    (
        "debug_runs/infrastructure_view_isolation_live.json",
        &[
            "/infrastructure_view_isolation_green",
            "/vm_09/triage_vm09_coder_b_green",
        ],
    ),
    (
        "debug_runs/stage7_play_live.json",
        &["/s7p_steward_green", "/concrete_chain_e2e/production_green"],
    ),
    (
        "debug_runs/industrial_activation_live.json",
        &["/activation_green"],
    ),
    (
        "debug_runs/logistics_throughput_live.json",
        &["/throughput_green"],
    ),
    (
        "debug_runs/construction_stage_live.json",
        &["/operational_green"],
    ),
    (
        "debug_runs/minimap_compositor_live.json",
        &["/ui_p3_m3_green"],
    ),
    (
        "debug_runs/stage5_full_app_live.json",
        &[
            "/stage5_closure/passes",
            "/tactical_vfx_witness/all_green",
            "/water_surface/water_witness_001_green",
        ],
    ),
    (
        "debug_runs/stage7_behavioral_live.json",
        &[
            "/behavioral_contract_ok",
            "/s7b_m1_green",
            "/s7b_m2_green",
            "/s7b_m3_green",
            "/s7b_steward_green",
        ],
    ),
];

fn repo_root() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_json(rel: &str) -> Value {
    let path = repo_root().join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {rel}: {e}"))
}

fn pointer_bool(v: &Value, ptr: &str) -> bool {
    v.pointer(ptr)
        .and_then(|x| x.as_bool())
        .unwrap_or_else(|| panic!("missing or non-bool pointer {ptr} in JSON"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// STEWARD-WITNESS-SYNC-001 — lib bundle gates + refresh agent index.
    #[test]
    fn steward_witness_sync_001_lib_bundle() {
        use crate::gui::hud::shell_diagnostics::ProductShellDiagnostics;
        use crate::gui::hud::simulation_shell_phase2::{
            commit_ui_shell_migration_live_proof, ContextTrayState, UiShellMigrationWitness,
        };

        let shell_witness = UiShellMigrationWitness {
            phase2_zones_live: true,
            alert_click_expanded_tray: true,
            intel_map_camera_request: true,
            escape_collapsed_tray: true,
            minimap_chrome_aligned: true,
            flat_v2_tab_chrome: true,
            ops_zones_wired: true,
            mock_zone_parity: true,
            build_rail_synced: true,
            build_rail_authoritative: true,
            build_toolbox_egui_gated: true,
            side_status_rail_egui_gated: true,
            floating_egui_shells_gated: true,
            ops_zone_hover_token: true,
            icon_atlas_loaded: true,
            last_minimap_rect_delta_px: 1.0,
            ..Default::default()
        };
        assert!(commit_ui_shell_migration_live_proof(
            &shell_witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        ));

        use crate::dev::stage7_behavioral_witness::refresh_s7b_steward_001_live_witness;
        assert!(
            refresh_s7b_steward_001_live_witness(),
            "stage7 behavioral steward witness refresh"
        );

        for (path, pointers) in BUNDLE {
            let v = read_json(path);
            for ptr in *pointers {
                assert!(pointer_bool(&v, ptr), "{path} {ptr} must be true");
            }
        }
        refresh_agent_debug_index().expect("agent_debug_index");
        let index = read_json(AGENT_DEBUG_INDEX_PATH);
        assert!(
            index
                .get("proof_count")
                .and_then(|c| c.as_u64())
                .unwrap_or(0)
                >= 10,
            "expected indexed proofs"
        );
    }
}
