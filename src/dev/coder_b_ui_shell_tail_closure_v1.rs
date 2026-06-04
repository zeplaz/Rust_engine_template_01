//! **@coder B** — UI shell tail closure: **UI-P3-SHELL-ROLLUP-001**, **UI-OH-P4-001**, **UI-OH-P5-001**.
//!
//! Refreshes `debug_runs/ui_shell_migration_live.json` with compositor-aligned P3 rollup + OH P4/P5 blocks.

use std::path::PathBuf;

use serde_json::Value;

const UI_SHELL: &str = "debug_runs/ui_shell_migration_live.json";

const SHELL_TAIL_CLOSURE: &[&str] = &[
    "/ui_p3_001/closed",
    "/ui_p3_001/compositor_authoritative",
    "/ui_oh_p4_001/green",
    "/ui_oh_p5_001/green",
    "/phase5/pause_menu_bevy",
    "/ui_w3_p4_001/green",
    "/phase4/icon_atlas_loaded",
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

/// Single writer: P3 shell rollup + OH P4/P5 witness tails.
pub fn refresh_coder_b_ui_shell_tail_closure_witness() -> bool {
    use crate::gui::hud::simulation_shell_phase2::refresh_coder_b_ui_shell_tail_closure_witness as refresh;
    refresh()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **@coder B** — UI-P3-SHELL-ROLLUP-001 + UI-OH-P4-001 + UI-OH-P5-001 bundle.
    #[test]
    fn coder_b_ui_shell_tail_closure_001_lib_bundle() {
        assert!(refresh_coder_b_ui_shell_tail_closure_witness());

        let shell = read_json(UI_SHELL);
        for ptr in SHELL_TAIL_CLOSURE {
            assert!(pointer_bool(&shell, ptr), "{UI_SHELL} {ptr} must be true");
        }
    }
}
