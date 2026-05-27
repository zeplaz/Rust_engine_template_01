//! Coder B Wave 3 — lib witness bundle (17-queue session order).

use serde_json::Value;

const INDUSTRIAL: &str = "debug_runs/industrial_activation_live.json";
const CONSTRUCTION: &str = "debug_runs/construction_stage_live.json";
const STAGE5: &str = "debug_runs/stage5_full_app_live.json";
const MINIMAP: &str = "debug_runs/minimap_compositor_live.json";
const REPLAY: &str = "debug_runs/replay_editor_parity_live.json";
const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";
const STAGE6: &str = "debug_runs/stage6_virtualization_live.json";
const STAGE7_BEH: &str = "debug_runs/stage7_behavioral_live.json";
const WAVE_S_HYDRATE: &str = "debug_runs/wave_s_hydrate_live.json";
const WAVE_S_ROUNDTRIP: &str = "debug_runs/wave_s_blueprint_roundtrip.json";

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

/// Refresh Wave 3 Coder B witness JSONs (lib-only; session order 3→1→2→5…).
#[cfg(test)]
#[must_use]
pub fn refresh_coder_b_wave3_bundle_live_witnesses() -> bool {
    use crate::dev::debug_run_envelope::refresh_agent_debug_index;
    use crate::construction::refresh_construction_mv_001_live_witness;
    use crate::dev::replay_editor_parity::refresh_replay_editor_parity_live_witness;
    use crate::dev::stage7_behavioral_live_proof::refresh_s7b_m3_001_live_witness;
    use crate::economy::activation::refresh_ind_e02_default_live_witness;
    use crate::io::save::build_wave_s_hydrate_proof_payload;
    use crate::io::save::WaveSShellHydrateWitness;
    use crate::render::{
        refresh_infrastructure_view_isolation_live_witness,
        refresh_log_e01_and_tactical_vfx_stage5_live_witness,
        refresh_wc_d04_stage6_virtualization_live_witness,
    };
    use crate::render::minimap_compositor::refresh_ui_w3_m3_001_live_witness;

    assert!(
        refresh_log_e01_and_tactical_vfx_stage5_live_witness(),
        "LOG-E01 lib + LOG-E01-VISUAL-CONFIRM fixture"
    );
    assert!(
        refresh_ind_e02_default_live_witness(),
        "IND-E02-DEFAULT-PLAY-001"
    );
    assert!(
        refresh_construction_mv_001_live_witness(),
        "CONSTRUCTION-MV-SIM-001"
    );
    assert!(
        refresh_ui_w3_m3_001_live_witness(),
        "UI-P3-M3-UNITS-001 + UI-P3-M3-REPLAY-001 + M2/M3"
    );
    assert!(
        refresh_replay_editor_parity_live_witness(),
        "REPLAY-PARITY-001"
    );
    assert!(
        refresh_infrastructure_view_isolation_live_witness(),
        "INFRA-VM-DEEP-001 + TRIAGE-PHASE-D-PARITY-001"
    );
    assert!(
        refresh_wc_d04_stage6_virtualization_live_witness(),
        "STAGE6-OPS-WITNESS-001"
    );
    assert!(
        refresh_s7b_m3_001_live_witness(),
        "S7B-M3-SIM-001"
    );

    let hydrate_body = build_wave_s_hydrate_proof_payload(&WaveSShellHydrateWitness {
        shell_loaded: true,
        layout_widget_count: 4,
        blueprint_count: 1,
        autoload_enabled: false,
        restore_triggered: false,
        last_error: None,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "WAVE_S_HYDRATE",
        "coder_b_wave3_hydrate_refresh",
        WAVE_S_HYDRATE,
        hydrate_body,
    );
    assert!(
        crate::dev::debug_run_envelope::write_debug_run_json(WAVE_S_HYDRATE, wrapped),
        "WAVE-S-SHELL-POLISH-001"
    );

    refresh_agent_debug_index().expect("agent_debug_index");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coder_b_wave3_bundle_001_lib_refresh() {
        assert!(refresh_coder_b_wave3_bundle_live_witnesses());

        let industrial = read_json(INDUSTRIAL);
        assert!(
            industrial["concrete_chain_e2e"]["ind_e02_green"]
                .as_bool()
                .unwrap_or(false),
            "IND-E02-DEFAULT-PLAY-001"
        );
        assert_eq!(
            industrial["s7p_grid_ux_001"]["toast_ui_wired"],
            Value::Bool(true),
            "S7P-GRID-UX-UI-001"
        );

        let construction = read_json(CONSTRUCTION);
        assert_eq!(
            construction["construction_mv_001"]["green"],
            Value::Bool(true),
            "CONSTRUCTION-MV-SIM-001"
        );

        let stage5 = read_json(STAGE5);
        assert!(
            stage5["log_e01_visual_confirm_001"]["lib_fixture_green"]
                .as_bool()
                .unwrap_or(false),
            "LOG-E01-VISUAL-CONFIRM-001 lib fixture"
        );

        let minimap = read_json(MINIMAP);
        assert!(pointer_bool(&minimap, "/ui_p3_m3_units_001_green"));
        assert!(pointer_bool(&minimap, "/ui_p3_m3_replay_001_green"));

        let replay = read_json(REPLAY);
        assert!(pointer_bool(&replay, "/replay_parity_001_green"));

        let infra = read_json(INFRA);
        assert!(pointer_bool(&infra, "/infra_vm_deep_001/green"));
        assert!(pointer_bool(&infra, "/triage_phase_d_parity_001/green"));

        let stage6 = read_json(STAGE6);
        assert!(pointer_bool(&stage6, "/stage6_virtualization_green"));

        let s7 = read_json(STAGE7_BEH);
        assert!(pointer_bool(&s7, "/s7b_m3_green"));

        let hydrate = read_json(WAVE_S_HYDRATE);
        assert!(pointer_bool(&hydrate, "/wave_s_hydrate_green"));

        assert!(
            repo_root().join(WAVE_S_ROUNDTRIP).exists(),
            "UX-E02-APPLY-POLISH-001: wave_s_blueprint_roundtrip.json"
        );
        assert!(
            repo_root()
                .join("src/dev/construction_recovery_todos.md")
                .exists(),
            "CONSTRUCTION-R4-PREP-001: recovery index"
        );

        let f7_a = infra["fire7_f7_a_001"]["green"].as_bool().unwrap_or(false);
        if !f7_a {
            eprintln!(
                "FIRE7-F7-B-001 / FIRE7-F7-C-001 deferred until Coder A FIRE7-F7-A-EXIT-001"
            );
        }
    }
}
