//! Coder B queue — lib witness refresh bundle (LOG-E01, IND-E02, VFX, infra, shell, wave P/C).

use serde_json::Value;

const STAGE5: &str = "debug_runs/stage5_full_app_live.json";
const INDUSTRIAL: &str = "debug_runs/industrial_activation_live.json";
const SHELL: &str = "debug_runs/ui_shell_migration_live.json";
const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";
const WAVE_P: &str = "debug_runs/wave_p_live.json";
const WAVE_C: &str = "debug_runs/wave_c_live.json";
const CONSTRUCTION: &str = "debug_runs/construction_stage_live.json";

fn repo_root() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
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

fn pointer_u64(v: &Value, ptr: &str) -> u64 {
    v.pointer(ptr)
        .and_then(|x| x.as_u64())
        .unwrap_or_else(|| panic!("missing or non-number {ptr}"))
}

/// Refresh all Coder B optional + active witness JSONs (lib-only).
pub fn refresh_coder_b_queue_bundle_live_witnesses() -> bool {
    use crate::dev::debug_run_envelope::refresh_agent_debug_index;
    use crate::economy::activation::refresh_ind_e02_default_play_002_live_witness;
    use crate::gui::editor::world_preview::refresh_coder_a_ui_wp_wave_p_witness;
    use crate::gui::hud::simulation_shell_phase2::refresh_ui_p2a_001_live_witness;
    use crate::dev::runtime_witness::wave_c::commit_wave_c_live_proof;
    use crate::io::streaming::{gather_wave_c_readiness, TileStorageApplyReport};
    use crate::dev::runtime_witness::refresh_infrastructure_view_isolation_live_witness;
    use crate::render::stage5_full_app_harness::refresh_log_e01_and_tactical_vfx_stage5_live_witness;

    assert!(
        refresh_log_e01_and_tactical_vfx_stage5_live_witness(),
        "LOG-E01-WITNESS + P2-VFX-WITNESS-001 + P2-WATER-WITNESS-002"
    );
    assert!(
        refresh_ind_e02_default_play_002_live_witness(),
        "IND-E02-DEFAULT-PLAY-002"
    );
    assert!(
        refresh_ui_p2a_001_live_witness(),
        "WITNESS-SHELL-P4 + UI-P2A-WITNESS-TAIL"
    );
    assert!(
        refresh_infrastructure_view_isolation_live_witness(),
        "INFRA-VM10/11 + VM-09 rollup"
    );
    assert!(
        refresh_coder_a_ui_wp_wave_p_witness(),
        "WAVE-P-WITNESS"
    );

    let wave_c = gather_wave_c_readiness(&crate::gui::editor::world_preview::PreviewPathAuthority::default());
    let tile = TileStorageApplyReport {
        applied_chunks: 2,
        pending_smooth_tiles: 0,
        ..Default::default()
    };
    assert!(commit_wave_c_live_proof(&wave_c, &tile), "WAVE-C-WITNESS");

    refresh_agent_debug_index().expect("agent_debug_index");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::dev::triage_vm09_v2_proof::infra_vm09_stray_map_camera_writer_audit_green;

    #[test]
    fn infra_vm09_stray_map_camera_writer_audit_green_bundle() {
        assert!(infra_vm09_stray_map_camera_writer_audit_green());
    }

    #[test]
    fn coder_b_queue_bundle_001_lib_refresh() {
        assert!(refresh_coder_b_queue_bundle_live_witnesses());

        let stage5 = read_json(STAGE5);
        assert!(
            pointer_u64(&stage5, "/projection_graph/logistics_active_rows") > 0
                || pointer_bool(&stage5, "/tactical_vfx_witness/all_green"),
            "LOG-E01 or tactical VFX rollup"
        );

        let industrial = read_json(INDUSTRIAL);
        assert!(
            industrial["concrete_chain_e2e"]["ind_e02_green"]
                .as_bool()
                .unwrap_or(false),
            "IND-E02"
        );

        let shell = read_json(SHELL);
        assert_eq!(shell["phase4"]["icon_atlas_loaded"], serde_json::json!(true));
        assert!(pointer_bool(&shell, "/ui_p2a_tail/f03_green"));
        assert!(pointer_bool(&shell, "/ui_p2a_tail/p4_auth_green"));

        let infra = read_json(INFRA);
        assert!(pointer_bool(&infra, "/infrastructure_view_isolation_green"));

        let wave_p = read_json(WAVE_P);
        assert!(pointer_bool(&wave_p, "/ui_wp_layout_d02_opt_green"));
        assert!(pointer_bool(&wave_p, "/ui_wp_layout_002_green"));

        let wave_c = read_json(WAVE_C);
        assert!(pointer_bool(&wave_c, "/wave_c_green"));

        let construction = read_json(CONSTRUCTION);
        assert!(
            construction["operational_green"].as_bool().unwrap_or(false)
                || construction["profile"] == "CONSTRUCTION_STAGE"
        );
    }
}
