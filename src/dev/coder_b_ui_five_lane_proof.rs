//! **@coder B** — five-lane UI witness bundle: **2B**, **2C**, **P5**, **witness**, **P4**.
//!
//! Refreshes `debug_runs/ui_shell_migration_live.json` in one lib pass (preserves 2A fields).

use std::path::PathBuf;

use serde_json::Value;

const UI_SHELL: &str = "debug_runs/ui_shell_migration_live.json";

/// Fifth lane is **P4** (icon atlas + build-rail auth). Queue shorthand **P6** is not a shell gate.
const FIVE_LANE_SHELL: &[&str] = &[
    "/phase2b_closed",
    "/ui_w3_2b_001/green",
    "/ui_oh_2b_001/green",
    "/egui_pass_count_in_sim",
    "/phase2c/phase2c_closed",
    "/ui_w3_2c_001/green",
    "/ui_p5_pause_001_green",
    "/phase5/pause_menu_bevy",
    "/ui_oh_p5_001/green",
    "/ui_w3_p5_001/green",
    "/witness/interaction_block_green",
    "/ui_w3_p4_001/green",
    "/ui_oh_p4_001/green",
    "/phase4/icon_atlas_loaded",
    "/phase4/p5_br_tab_wired",
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

/// Single writer: **2B** + **2C** + **P5** + **witness** + **P4** shell witness.
pub fn refresh_coder_b_ui_five_lane_witness() -> bool {
    use crate::gui::hud::simulation_shell_phase2::refresh_coder_b_ui_five_lane_witness as refresh;
    refresh()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **@coder B** — 2B, 2C, P5, witness, P4 (5) witness bundle.
    #[test]
    fn coder_b_ui_five_lane_001_lib_bundle() {
        assert!(refresh_coder_b_ui_five_lane_witness());

        let shell = read_json(UI_SHELL);
        for ptr in FIVE_LANE_SHELL {
            if *ptr == "/egui_pass_count_in_sim" {
                assert_eq!(
                    pointer_u64(&shell, ptr),
                    0,
                    "{UI_SHELL} {ptr} must be 0"
                );
            } else {
                assert!(pointer_bool(&shell, ptr), "{UI_SHELL} {ptr} must be true");
            }
        }
    }
}
