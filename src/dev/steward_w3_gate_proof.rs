//! **STEWARD-W3-GATE-001** — Wave 1 UI shell + minimap M2 steward bundle (run after coder five-lane).
//!
//! Includes **UI-SHELL-REFRESH-001** re-verify in the same lib pass.

use std::path::PathBuf;

use serde_json::Value;

const UI_SHELL: &str = "debug_runs/ui_shell_migration_live.json";
const STAGE5: &str = "debug_runs/stage5_full_app_live.json";
const MINIMAP: &str = "debug_runs/minimap_compositor_live.json";

/// Spec-aligned shell gates (2A/2B + UI-SHELL-REFRESH sub-check).
const SHELL_SPEC: &[&str] = &[
    "/phase2_zones_live",
    "/phase2a_closed",
    "/phase2b_closed",
    "/ui_p2b_coder_b_green",
    "/ui_p2b_coder_b/green",
    "/ui_p2b_coder_b/build_toolbox_egui_gated",
    "/ui_p2b_coder_b/side_status_rail_egui_gated",
    "/ui_p2b_coder_b/floating_egui_shells_gated",
    "/ui_w3_2a_001/green",
    "/ui_w3_2b_001/green",
];

const STAGE5_GATES: &[&str] = &["/stage5_closure/passes", "/readiness/passes"];

const MINIMAP_M2: &[&str] = &["/composite_ok", "/ui_w3_m2_001/green"];

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

    /// **STEWARD-W3-GATE-001** + **UI-SHELL-REFRESH-001** (same session).
    ///
    /// Prereq: Wave 1 coder witnesses green (`coder_b_ui_w3_witness_001_lib_bundle` or five-lane).
    #[test]
    fn steward_w3_gate_001_lib_bundle() {
        use crate::dev::coder_b_ui_w3_witness_proof::refresh_ui_w3_witness_001_live_witness;

        assert!(
            refresh_ui_w3_witness_001_live_witness(),
            "Wave 1 coder witness refresh must pass first"
        );

        let shell = read_json(UI_SHELL);
        for ptr in SHELL_SPEC {
            assert!(pointer_bool(&shell, ptr), "{UI_SHELL} {ptr} must be true");
        }
        assert_eq!(
            shell.pointer("/egui_pass_count_in_sim")
                .and_then(|x| x.as_u64())
                .expect("egui_pass_count_in_sim"),
            0,
            "UI-SHELL-REFRESH: egui_pass_count_in_sim must be 0"
        );

        let stage5 = read_json(STAGE5);
        for ptr in STAGE5_GATES {
            assert!(pointer_bool(&stage5, ptr), "{STAGE5} {ptr} must be true");
        }

        let minimap = read_json(MINIMAP);
        for ptr in MINIMAP_M2 {
            assert!(pointer_bool(&minimap, ptr), "{MINIMAP} {ptr} must be true");
        }
    }
}
