//! **@coder A** — five-lane UI witness bundle: **2A**, **M2**, **P4**, **M3**, **theme**.
//!
//! Refreshes `ui_shell_migration_live.json` + `minimap_compositor_live.json` in one lib pass.

use std::path::PathBuf;

use serde_json::Value;

const UI_SHELL: &str = "debug_runs/ui_shell_migration_live.json";
const MINIMAP: &str = "debug_runs/minimap_compositor_live.json";

const FIVE_LANE_SHELL: &[&str] = &[
    "/ui_w3_2a_001/green",
    "/ui_w3_p4_001/green",
    "/ui_w3_p4_001/icon_atlas_loaded",
    "/ui_w3_theme_001/green",
    "/phase2a_closed",
    "/phase4/icon_atlas_loaded",
    "/ui_p2a_tail/p4_auth_green",
];

const FIVE_LANE_MINIMAP: &[&str] = &[
    "/ui_w3_m2_001/green",
    "/ui_w3_m3_001/green",
    "/ui_w3_m3_001/operational_green",
    "/ui_p3_001_green",
    "/ui_p3_m3_green",
    "/logistics_rows",
];

const FIVE_LANE_SHELL_P4: &[&str] = &[
    "/ui_w3_p4_001/petroleum_panel_tab_wired",
    "/ui_w3_p4_001/p5_br_tab_wired",
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

fn pointer_u64(v: &Value, ptr: &str) -> u64 {
    v.pointer(ptr)
        .and_then(|x| x.as_u64())
        .unwrap_or_else(|| panic!("missing or non-number {ptr}"))
}

/// Single writer: **2A** + **P4** + **theme** shell witness, then **M2** + **M3** minimap witness.
pub fn refresh_coder_a_ui_five_lane_witness() -> bool {
    use crate::dev::steward_ui_oh_gate_proof::refresh_ui_oh_gate_001_shell_witness;
    use crate::render::minimap_compositor::refresh_ui_w3_m3_001_stage7_operational_witness;

    assert!(
        refresh_ui_oh_gate_001_shell_witness(),
        "shell: 2A/2B/P4 via UI-OH-GATE writer"
    );
    assert!(
        refresh_ui_w3_m3_001_stage7_operational_witness(),
        "minimap: UI-W3-M3-001 Stage 7 operational"
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **@coder A** — 2A, M2, P4, M3, theme (5) witness bundle.
    #[test]
    fn coder_a_ui_five_lane_001_lib_bundle() {
        assert!(refresh_coder_a_ui_five_lane_witness());

        let shell = read_json(UI_SHELL);
        for ptr in FIVE_LANE_SHELL {
            assert!(pointer_bool(&shell, ptr), "{UI_SHELL} {ptr} must be true");
        }
        for ptr in FIVE_LANE_SHELL_P4 {
            assert!(pointer_bool(&shell, ptr), "{UI_SHELL} {ptr} must be true");
        }

        let minimap = read_json(MINIMAP);
        for ptr in FIVE_LANE_MINIMAP {
            if ptr.ends_with("logistics_rows") {
                continue;
            }
            assert!(pointer_bool(&minimap, ptr), "{MINIMAP} {ptr} must be true");
        }
        assert!(
            pointer_u64(&minimap, "/logistics_rows") >= 2,
            "M2 logistics_rows"
        );
        assert!(
            pointer_u64(&minimap, "/construction_rows") > 0,
            "M2/M3 construction_rows"
        );
    }
}
