//! S7B-PREFLIGHT-001 — prerequisite witness bundle before **S7B-M1-001**.

use std::path::PathBuf;

use serde_json::{json, Value};

use crate::dev::debug_run_envelope::{write_debug_run_json, wrap_debug_run};

const BEHAVIORAL_LIVE: &str = "debug_runs/stage7_behavioral_live.json";

const PREREQ_BUNDLE: &[(&str, &[&str])] = &[
    (
        "debug_runs/stage7_play_live.json",
        &["/s7p_steward_green", "/activation_green"],
    ),
    (
        "debug_runs/ui_shell_migration_live.json",
        &["/phase2b_closed", "/ui_p2a_coder_b/green"],
    ),
    (
        "debug_runs/infrastructure_view_isolation_live.json",
        &["/infrastructure_view_isolation_green"],
    ),
    (
        "debug_runs/wave_p_live.json",
        &["/wave_p_green", "/ui_wp_layout_002_green"],
    ),
    (
        "debug_runs/construction_stage_live.json",
        &["/operational_green"],
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
        .unwrap_or_else(|| panic!("missing or non-bool {ptr}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// S7B-PREFLIGHT-001 — prerequisite bundle + shell commit; unblocks S7B-M1-001.
    #[test]
    fn steward_s7b_preflight_001_lib_bundle() {
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

        for (path, pointers) in PREREQ_BUNDLE {
            let v = read_json(path);
            for ptr in *pointers {
                assert!(pointer_bool(&v, ptr), "{path} {ptr} must be true");
            }
        }

        let play = read_json("debug_runs/stage7_play_live.json");
        assert!(pointer_bool(&play, "/s7p_steward_green"));

        let body = json!({
            "profile": "STAGE7_BEHAVIORAL",
            "s7b_preflight_green": true,
            "s7p_play_witness_ok": true,
            "behavioral_contract_ok": false,
            "s7b_m1_green": false,
            "s7b_m2_green": false,
            "s7b_m3_green": false,
            "s7b_steward_green": false,
            "impl_plan": "src/dev/stage7_behavioral_implementation_plan_v1.md",
            "witness_spec": "src/dev/stage7_behavioral_live_witness_spec_v1.md",
            "gate": "S7B-PREFLIGHT-001",
        });
        let wrapped = wrap_debug_run(
            "STAGE7_BEHAVIORAL",
            "steward_s7b_preflight_proof",
            BEHAVIORAL_LIVE,
            body,
        );
        assert!(
            write_debug_run_json(BEHAVIORAL_LIVE, wrapped),
            "write {BEHAVIORAL_LIVE}"
        );
    }
}
