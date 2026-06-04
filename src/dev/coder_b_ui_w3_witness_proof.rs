//! **UI-W3-WITNESS-001** — @coder B: lib refresh of Wave 3 witness JSONs + Stage 5 cross-check.
//!
//! Operator timestamp refresh: `cargo run -p proc_A_dine01 --release -- --test visual`

use std::path::PathBuf;

use serde_json::Value;

pub const VISUAL_OPERATOR_CMD: &str = "cargo run -p proc_A_dine01 --release -- --test visual";

const UI_SHELL: &str = "debug_runs/ui_shell_migration_live.json";
const STAGE5: &str = "debug_runs/stage5_full_app_live.json";
const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";
const STAGE6: &str = "debug_runs/stage6_virtualization_live.json";
const MINIMAP: &str = "debug_runs/minimap_compositor_live.json";

const WITNESS_SHELL: &[&str] = &[
    "/ui_w3_witness_001/green",
    "/ui_w3_2a_001/green",
    "/ui_w3_2b_001/green",
    "/ui_w3_2c_001/green",
    "/ui_w3_p4_001/green",
    "/ui_w3_p5_001/green",
    "/witness/interaction_block_green",
    "/phase2b_closed",
];

const WITNESS_CROSS: &[(&str, &[&str])] = &[
    (STAGE5, &["/stage5_closure/passes", "/readiness/passes"]),
    (
        INFRA,
        &[
            "/infrastructure_view_isolation_green",
            "/vm_09/triage_vm09_v2_green",
        ],
    ),
    (STAGE6, &["/stage6_virtualization_green", "/wc_d04/green"]),
    (
        MINIMAP,
        &["/composite_path", "/ui_w3_m2_001/green"],
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

fn pointer_str(v: &Value, ptr: &str) -> String {
    v.pointer(ptr)
        .and_then(|x| x.as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| panic!("missing or non-string {ptr}"))
}

fn pointer_u64(v: &Value, ptr: &str) -> u64 {
    v.pointer(ptr)
        .and_then(|x| x.as_u64())
        .unwrap_or_else(|| panic!("missing or non-number {ptr}"))
}

/// Lib refresh: shell (coder B five-lane) + infra + stage6 + minimap M2; then agent index.
pub fn refresh_ui_w3_witness_001_live_witness() -> bool {
    use crate::dev::debug_run_envelope::refresh_agent_debug_index;
    use crate::gui::hud::simulation_shell_phase2::refresh_coder_b_ui_five_lane_witness;
    use crate::render::minimap_compositor::refresh_ui_w3_m2_001_live_witness;
    use crate::dev::runtime_witness::{
        refresh_infrastructure_view_isolation_live_witness,
        refresh_wc_d04_stage6_virtualization_live_witness,
    };

    assert!(
        refresh_coder_b_ui_five_lane_witness(),
        "UI-W3-WITNESS-001: shell five-lane"
    );
    assert!(
        refresh_infrastructure_view_isolation_live_witness(),
        "UI-W3-WITNESS-001: infrastructure_view_isolation"
    );
    assert!(
        refresh_wc_d04_stage6_virtualization_live_witness(),
        "UI-W3-WITNESS-001: stage6_virtualization"
    );
    assert!(
        refresh_ui_w3_m2_001_live_witness(),
        "UI-W3-WITNESS-001: minimap compositor M2"
    );
    refresh_agent_debug_index().expect("agent_debug_index");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **UI-W3-WITNESS-001** — lib JSON refresh bundle (operator: [`VISUAL_OPERATOR_CMD`]).
    #[test]
    fn coder_b_ui_w3_witness_001_lib_bundle() {
        assert!(refresh_ui_w3_witness_001_live_witness());

        let shell = read_json(UI_SHELL);
        for ptr in WITNESS_SHELL {
            assert!(pointer_bool(&shell, ptr), "{UI_SHELL} {ptr} must be true");
        }
        assert_eq!(
            pointer_u64(&shell, "/egui_pass_count_in_sim"),
            0,
            "shell egui pass count"
        );
        assert_eq!(
            shell.pointer("/ui_w3_witness_001/visual_operator")
                .and_then(|v| v.as_str()),
            Some(VISUAL_OPERATOR_CMD)
        );

        for (path, pointers) in WITNESS_CROSS {
            let v = read_json(path);
            for ptr in *pointers {
                if *ptr == "/composite_path" {
                    assert_eq!(
                        pointer_str(&v, ptr),
                        "GpuCompute",
                        "{path} composite_path"
                    );
                } else {
                    assert!(pointer_bool(&v, ptr), "{path} {ptr} must be true");
                }
            }
        }
    }
}
