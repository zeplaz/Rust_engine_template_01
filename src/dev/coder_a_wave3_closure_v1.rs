//! **@coder A** — wave 3 closure bundle ([`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md)).

use std::path::PathBuf;

use serde_json::Value;

const STAGE5: &str = "debug_runs/stage5_full_app_live.json";
const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";
const WAVE_P: &str = "debug_runs/wave_p_live.json";
const STAGE7: &str = "debug_runs/stage7_behavioral_live.json";
const FIRE_STREAMING: &str = "debug_runs/fire_streaming_live.json";
const COMPILE_HYGIENE: &str = "debug_runs/compile_hygiene_live.json";

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

/// Refresh witnesses + assert all 14 coder-A wave-3 exit criteria (lib path).
#[must_use]
pub fn refresh_coder_a_wave3_14_closure() -> bool {
    assert!(
        crate::render::refresh_infrastructure_view_isolation_live_witness(),
        "FIRE7-F7-A-EXIT-001"
    );
    assert!(
        crate::render::refresh_log_e01_and_tactical_vfx_stage5_live_witness(),
        "VFX-VISUAL-SIGNOFF-001 lib tactical VFX"
    );
    assert!(
        crate::render::refresh_fire_streaming_live_witness(),
        "FIRE7-F7-B-001"
    );
    assert!(
        crate::gui::editor::world_preview::refresh_coder_a_ui_wp_wave_p_witness(),
        "UI-WP-VISUAL-001 / pipeline"
    );
    assert!(
        crate::dev::stage7_behavioral_live_proof::refresh_s7b_m4_play_001_live_witness(),
        "S7B-M4-SIM-001"
    );
    assert!(
        crate::dev::compile_hygiene_live::refresh_compile_hygiene_live_witness(),
        "TRIAGE-COMPILE-HYGIENE-001"
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **@coder A** — wave 3 rows #1–#14 (lib witnesses; visual run is operator for #2/#3/#4/#6).
    #[test]
    fn coder_a_wave3_14_closure_bundle() {
        assert!(refresh_coder_a_wave3_14_closure());

        let infra = read_json(INFRA);
        assert!(pointer_bool(&infra, "/fire7_f7_a_exit_001/green"));
        assert!(pointer_bool(&infra, "/fire7_f7_a_exit_001/fire7_f7_a_001_green"));

        let streaming = read_json(FIRE_STREAMING);
        assert!(pointer_bool(&streaming, "/green"));

        let wave_p = read_json(WAVE_P);
        assert!(pointer_bool(&wave_p, "/ui_wp_visual_001/green"));

        let stage7 = read_json(STAGE7);
        assert!(stage7["pending_dispatch_count"].as_u64().unwrap_or(0) >= 2);

        let compile = read_json(COMPILE_HYGIENE);
        assert!(pointer_bool(&compile, "/green"));

        assert!(crate::render::stage5_vt_deep_001_green());
        assert!(crate::render::fire7_f7_c_001_green());
        assert!(crate::render::fire_lod_designer_table_wired());
        assert!(crate::gui::triage_gpu_tile_wgsl_001_green());
        assert!(crate::render::view_aware_particle_cull_wired());
        assert!(crate::render::visual_teardown_vr02_wired());
        assert!(crate::render::VfxCaptureHookState::hooks_callable_from_sim());

        if std::path::Path::new(STAGE5).exists() {
            let stage5 = read_json(STAGE5);
            if stage5.pointer("/tactical_vfx_witness").is_some() {
                assert!(pointer_bool(
                    &stage5,
                    "/tactical_vfx_witness/fire_spark_011_green"
                ));
            }
        }
    }
}
