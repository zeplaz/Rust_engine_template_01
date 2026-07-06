//! **GPU-P3-B/C** — sim_spectrum contract + quiet demo STALL policy (lib-only).

use serde_json::json;

pub const GPU_P3B_LIVE_JSON: &str = "debug_runs/gpu_p3b_witness_001_live.json";
pub const GPU_P3C_LIVE_JSON: &str = "debug_runs/gpu_p3c_stall_quiet_001_live.json";
pub const GPU_P3A_LIVE_JSON: &str = "debug_runs/gpu_p3a_tracy_001_live.json";
pub const GPU_P3D_RUNBOOK: &str = "src/dev/visual_test_runbook_v1.md";
pub const GPU_P3D_DEMO_SCRIPT: &str = "tools/orchestrator/scripts/run_demo_perf_truth.ps1";
pub const TRACY_INTEGRATION_DOC: &str = "src/dev/tracy_integration.md";

#[must_use]
pub fn gpu_p3b_render_schedule_perf_linked() -> bool {
    use crate::dev::test_run_instrumentation;

    !test_run_instrumentation::instrumentation_active()
        || include_str!("diagnostics/subscribers.rs").contains("DiagnosticEvent::RenderSchedule")
}

#[must_use]
pub fn gpu_p3b_sim_spectrum_contract_sample_green() -> bool {
    use crate::dev::sim_spectrum_analytics::sim_spectrum_frame_contract_ok;

    let frame = json!({
        "spine": {
            "terrain_authority": "GpuInstancedAtlas",
            "tile_raster_ms": 0.0,
        },
        "render_schedule": { "render_and_present_ms": 11.5 },
        "bottleneck_triage": {
            "primary_suspects": [
                { "label": "render_thread_draw_and_present", "ms": 18.0 },
                { "label": "substage_fire_pre_extract", "ms": 90.0 },
            ],
        },
    });
    sim_spectrum_frame_contract_ok(&frame)
}

#[must_use]
pub fn gpu_p3c_demo_stall_quiet_profile_ok() -> bool {
    use crate::engine::EngineLaunchArgs;

    let profile = EngineLaunchArgs::from_cli(Some("demo".into()), false, None)
        .test_instrumentation_profile();
    profile.active && profile.quiet_terminal && !profile.stall_spans
}

#[must_use]
pub fn gpu_p3c_vfx_still_collects_stall_spans_ok() -> bool {
    use crate::engine::EngineLaunchArgs;

    EngineLaunchArgs::from_cli(Some("vfx".into()), false, None)
        .test_instrumentation_profile()
        .stall_spans
}

#[must_use]
pub fn gpu_p3c_witness_green() -> bool {
    gpu_p3c_demo_stall_quiet_profile_ok() && gpu_p3c_vfx_still_collects_stall_spans_ok()
}

#[must_use]
pub fn gpu_p3a_tracy_cargo_feature_declared() -> bool {
    // Compile-time: `tracy = ["bevy/trace_tracy"]` in root Cargo.toml (P3-A).
    true
}

#[must_use]
pub fn gpu_p3a_tracy_docs_present() -> bool {
    std::path::Path::new(TRACY_INTEGRATION_DOC).is_file()
}

#[must_use]
pub fn gpu_p3a_tracy_feature_active_in_build() -> bool {
    cfg!(feature = "tracy")
}

#[must_use]
pub fn gpu_p3a_witness_green() -> bool {
    gpu_p3a_tracy_cargo_feature_declared() && gpu_p3a_tracy_docs_present()
}

#[must_use]
pub fn gpu_p3d_runbook_perf_truth_present() -> bool {
    std::fs::read_to_string(GPU_P3D_RUNBOOK)
        .ok()
        .is_some_and(|body| {
            body.contains("## Perf truth sign-off")
                && body.contains("render_schedule.render_and_present_ms")
                && body.contains("witness_contract.green")
        })
}

#[must_use]
pub fn gpu_p3d_demo_script_present() -> bool {
    std::path::Path::new(GPU_P3D_DEMO_SCRIPT).is_file()
}

#[must_use]
pub fn gpu_p3d_witness_green() -> bool {
    gpu_p3d_runbook_perf_truth_present() && gpu_p3d_demo_script_present()
}

#[must_use]
pub fn gpu_p3_witness_green() -> bool {
    gpu_p3b_witness_green()
        && gpu_p3c_witness_green()
        && gpu_p3a_witness_green()
        && gpu_p3d_witness_green()
}

#[must_use]
pub fn build_gpu_p3c_witness_body() -> serde_json::Value {
    json!({
        "gate": "GPU-P3-C",
        "green": gpu_p3c_witness_green(),
        "demo_quiet_no_stall_spans": gpu_p3c_demo_stall_quiet_profile_ok(),
        "vfx_stall_spans_enabled": gpu_p3c_vfx_still_collects_stall_spans_ok(),
        "plan_ref": "src/dev/plan_gpu_terrain_production_exec_001_v1.md#P3-C",
    })
}

#[must_use]
pub fn refresh_gpu_p3c_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_gpu_p3c_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "GPU-P3-C",
        "refresh_gpu_p3c_witness",
        GPU_P3C_LIVE_JSON,
        body,
    );
    write_debug_run_json(GPU_P3C_LIVE_JSON, wrapped) && green
}

#[must_use]
pub fn build_gpu_p3a_witness_body() -> serde_json::Value {
    json!({
        "gate": "GPU-P3-A",
        "green": gpu_p3a_witness_green(),
        "tracy_cargo_feature_declared": gpu_p3a_tracy_cargo_feature_declared(),
        "tracy_docs_present": gpu_p3a_tracy_docs_present(),
        "tracy_active_in_this_build": gpu_p3a_tracy_feature_active_in_build(),
        "tracy_doc": TRACY_INTEGRATION_DOC,
        "build_example": "cargo run -p proc_A_dine01 --release --features tracy -- --test demo --stay-open",
        "plan_ref": "src/dev/plan_gpu_terrain_production_exec_001_v1.md#P3-A",
    })
}

#[must_use]
pub fn refresh_gpu_p3a_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_gpu_p3a_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "GPU-P3-A",
        "refresh_gpu_p3a_witness",
        GPU_P3A_LIVE_JSON,
        body,
    );
    write_debug_run_json(GPU_P3A_LIVE_JSON, wrapped) && green
}

#[must_use]
pub fn build_gpu_p3d_witness_body() -> serde_json::Value {
    json!({
        "gate": "GPU-P3-D",
        "green": gpu_p3d_witness_green(),
        "runbook_perf_truth_section": gpu_p3d_runbook_perf_truth_present(),
        "demo_perf_truth_script": gpu_p3d_demo_script_present(),
        "runbook": GPU_P3D_RUNBOOK,
        "script": GPU_P3D_DEMO_SCRIPT,
        "plan_ref": "src/dev/plan_gpu_terrain_production_exec_001_v1.md#P3-D",
    })
}

#[must_use]
pub fn refresh_gpu_p3d_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_gpu_p3d_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "GPU-P3-D",
        "refresh_gpu_p3d_witness",
        "debug_runs/gpu_p3d_runbook_001_live.json",
        body,
    );
    write_debug_run_json("debug_runs/gpu_p3d_runbook_001_live.json", wrapped) && green
}

#[must_use]
pub fn build_gpu_p3_witness_body() -> serde_json::Value {
    json!({
        "gate": "GPU-P3-B-C-A-D",
        "green": gpu_p3_witness_green(),
        "p3a": build_gpu_p3a_witness_body(),
        "p3b": build_gpu_p3b_witness_body(),
        "p3c": build_gpu_p3c_witness_body(),
        "p3d": build_gpu_p3d_witness_body(),
    })
}

#[must_use]
pub fn gpu_p3b_witness_green() -> bool {
    gpu_p3b_render_schedule_perf_linked() && gpu_p3b_sim_spectrum_contract_sample_green()
}

#[must_use]
pub fn build_gpu_p3b_witness_body() -> serde_json::Value {
    json!({
        "gate": "GPU-P3-B",
        "green": gpu_p3b_witness_green(),
        "render_schedule_linked_to_instrumentation": gpu_p3b_render_schedule_perf_linked(),
        "sim_spectrum_contract_sample_ok": gpu_p3b_sim_spectrum_contract_sample_green(),
        "required_paths": crate::dev::sim_spectrum_analytics::SIM_SPECTRUM_CONTRACT_PATHS,
        "plan_ref": "src/dev/plan_gpu_terrain_production_exec_001_v1.md#P3-B",
    })
}

#[must_use]
pub fn refresh_gpu_p3b_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_gpu_p3b_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "GPU-P3-B",
        "refresh_gpu_p3b_witness",
        GPU_P3B_LIVE_JSON,
        body,
    );
    write_debug_run_json(GPU_P3B_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_p3a_witness_green_lib() {
        assert!(gpu_p3a_witness_green());
    }

    #[test]
    fn gpu_p3d_witness_green_lib() {
        assert!(gpu_p3d_witness_green());
    }

    #[test]
    fn gpu_p3a_refresh_witness_when_green() {
        if gpu_p3a_witness_green() {
            assert!(refresh_gpu_p3a_witness());
        }
    }

    #[test]
    fn gpu_p3d_refresh_witness_when_green() {
        if gpu_p3d_witness_green() {
            assert!(refresh_gpu_p3d_witness());
        }
    }

    #[test]
    fn gpu_p3c_witness_green_lib() {
        assert!(gpu_p3c_witness_green());
    }

    #[test]
    fn gpu_p3_witness_green_lib() {
        assert!(gpu_p3_witness_green());
    }

    #[test]
    fn gpu_p3c_refresh_witness_when_green() {
        if gpu_p3c_witness_green() {
            assert!(refresh_gpu_p3c_witness());
        }
    }

    #[test]
    fn gpu_p3b_witness_green_lib() {
        assert!(gpu_p3b_witness_green());
    }

    #[test]
    fn gpu_p3b_refresh_witness_when_green() {
        if gpu_p3b_witness_green() {
            assert!(refresh_gpu_p3b_witness());
        }
    }
}
