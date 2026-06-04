//! **@coder A** — dual-queue P1/P2 closure bundle ([`coder_dual_queue_todos_v1.md`](coder_dual_queue_todos_v1.md)).

use std::path::PathBuf;

use serde_json::Value;

const STAGE5: &str = "debug_runs/stage5_full_app_live.json";
const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";
const STAGE7: &str = "debug_runs/stage7_behavioral_live.json";
const UI_SHELL: &str = "debug_runs/ui_shell_migration_live.json";
const WAVE_P: &str = "debug_runs/wave_p_live.json";
const STAGE6: &str = "debug_runs/stage6_virtualization_live.json";

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

/// Refresh witnesses + assert all 14 coder-A dual-queue exit criteria.
#[must_use]
pub fn refresh_coder_a_dual_queue_14_closure() -> bool {
    use crate::gui::editor::world_preview::refresh_coder_a_ui_wp_wave_p_witness;
    use crate::gui::hud::simulation_shell_phase2::refresh_ui_w3_p4_001_live_witness;
    use crate::dev::runtime_witness::{
        refresh_infrastructure_view_isolation_live_witness,
        refresh_wc_d04_stage6_virtualization_live_witness,
    };
    use crate::render::stage5_full_app_harness::{
        refresh_log_e01_and_tactical_vfx_stage5_live_witness,
        refresh_log_e01_fullapp_upgrade_001_live_witness,
    };

    assert!(
        refresh_infrastructure_view_isolation_live_witness(),
        "FIRE7-F7-A-001 infrastructure witness"
    );
    assert!(
        refresh_log_e01_and_tactical_vfx_stage5_live_witness(),
        "P2-FIRE-SPARK-010/011 + P2-WATER-POLISH-001 stage5 tactical VFX"
    );
    assert!(
        refresh_log_e01_fullapp_upgrade_001_live_witness(),
        "LOG-E01-FULLAPP-UPGRADE-001 — fixture/visual keys refreshed"
    );
    assert!(
        refresh_ui_w3_p4_001_live_witness(),
        "P4-VEH-01 + UX-E03 shell witness"
    );
    assert!(
        crate::dev::stage7_behavioral_witness::refresh_s7b_m2_001_live_witness(),
        "S7B-TUNE-DELAY-001 via M2 delay witness"
    );
    assert!(
        crate::dev::stage7_behavioral_witness::refresh_s7b_m3_steward_remedy_001_live_witness(),
        "S7B-M3-STEWARD-REMEDY-001 — s7b_m3_green + s7b_steward_green"
    );
    assert!(
        crate::dev::stage7_behavioral_witness::refresh_s7b_m4_play_remedy_001_live_witness(),
        "S7B-M4-PLAY-REMEDY-001 — last writer for s7b_m4_play_green"
    );
    assert!(
        refresh_coder_a_ui_wp_wave_p_witness(),
        "UI-WP-PIPELINE / L4 / MOTION / LAYOUT-003"
    );
    assert!(
        refresh_wc_d04_stage6_virtualization_live_witness(),
        "INFRA-PERF-001 qualified (WC-D04 + frame budget path)"
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **@coder A** — all 14 dual-queue rows (P1×7 + P2×7) witness closure.
    #[test]
    fn coder_a_dual_queue_14_closure_bundle() {
        assert!(refresh_coder_a_dual_queue_14_closure());

        let infra = read_json(INFRA);
        assert!(pointer_bool(&infra, "/fire7_f7_a_001/green"));
        assert!(pointer_bool(
            &infra,
            "/fire7_f7_a_001/f7_a_per_view_extract_bounded"
        ));

        let stage5 = read_json(STAGE5);
        assert!(pointer_bool(&stage5, "/tactical_vfx_witness/fire_sparks_above_smoke"));
        assert!(pointer_bool(&stage5, "/tactical_vfx_witness/fire_spark_011_green"));
        assert!(pointer_bool(&stage5, "/tactical_vfx_witness/water_w1_river_read_green"));
        let instanced_draw = stage5
            .pointer("/projection_state/instanced_draw")
            .or_else(|| stage5.pointer("/particle_routing/instanced_draw"))
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        let instanced_dispatch = stage5
            .pointer("/readiness/instanced_dispatch_ok")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        assert!(
            instanced_draw || instanced_dispatch,
            "INFRA-GPU-TILE-001: instanced path"
        );
        assert_eq!(
            stage5["log_e01_fullapp_upgrade_001"]["full_visual_confirm"],
            Value::Bool(false)
        );
        assert_eq!(
            stage5["log_e01_visual_confirm_001"]["log_e01_fixture_green"],
            Value::Bool(true)
        );

        let shell = read_json(UI_SHELL);
        assert_eq!(shell["p4_veh_01"]["green"], Value::Bool(true));
        assert_eq!(shell["ux_e03_coder_a"]["green"], Value::Bool(true));
        assert_eq!(
            shell["ux_e03_coder_a"]["strategic_enqueue_from_transmission_ui"],
            Value::Bool(false)
        );

        let stage7 = read_json(STAGE7);
        assert_eq!(stage7["s7b_m3_green"], Value::Bool(true));
        assert_eq!(stage7["s7b_steward_green"], Value::Bool(true));
        assert_eq!(stage7["s7b_m4_play_001"]["green"], Value::Bool(true));
        assert_eq!(stage7["s7b_tune_delay_001"]["green"], Value::Bool(true));
        assert_eq!(stage7["s7b_tune_delay_001"]["dispatch_delay_ticks"], Value::from(8));

        let wave_p = read_json(WAVE_P);
        assert_eq!(wave_p["ui_wp_pipeline_green"], Value::Bool(true));
        assert_eq!(wave_p["ui_wp_l4_001_green"], Value::Bool(true));
        assert_eq!(wave_p["ui_wp_motion_001_green"], Value::Bool(true));
        assert_eq!(wave_p["ui_wp_layout_003_green"], Value::Bool(true));
        assert_eq!(wave_p["coder_a_ui_wp_queue_green"], Value::Bool(true));

        let stage6 = read_json(STAGE6);
        assert!(pointer_bool(&stage6, "/wc_d04/green"));
    }
}
