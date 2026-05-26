//! **@coder A** — **UI-W3-P4-001** (icon atlas + petroleum) + **UI-W3-M3-001** (Stage 7 operational minimap).

use std::path::PathBuf;

use serde_json::Value;

const UI_SHELL: &str = "debug_runs/ui_shell_migration_live.json";
const MINIMAP: &str = "debug_runs/minimap_compositor_live.json";
const STAGE7: &str = "debug_runs/stage7_behavioral_live.json";

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

/// **UI-W3-P4-001** + **UI-W3-M3-001** — dedicated Wave 3 witness refresh (shell + minimap + Stage 7).
pub fn refresh_coder_a_ui_w3_p4_m3_witness() -> bool {
    use crate::gui::hud::simulation_shell_phase2::refresh_ui_w3_p4_001_live_witness;
    use crate::render::minimap_compositor::refresh_ui_w3_m3_001_stage7_operational_witness;

    assert!(
        refresh_ui_w3_p4_001_live_witness(),
        "UI-W3-P4-001 shell witness"
    );
    assert!(
        refresh_ui_w3_m3_001_stage7_operational_witness(),
        "UI-W3-M3-001 Stage 7 operational minimap witness"
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **UI-W3-P4-001** + **UI-W3-M3-001** — Coder A Wave 3 pair witness bundle.
    #[test]
    fn coder_a_ui_w3_p4_m3_001_lib_bundle() {
        assert!(refresh_coder_a_ui_w3_p4_m3_witness());

        let shell = read_json(UI_SHELL);
        assert_eq!(shell["ui_w3_p4_001"]["green"], Value::Bool(true));
        assert_eq!(
            shell["ui_w3_p4_001"]["petroleum_panel_tab_wired"],
            Value::Bool(true)
        );
        assert_eq!(shell["phase4"]["icon_atlas_loaded"], Value::Bool(true));
        assert_eq!(shell["phase4"]["p5_br_tab_wired"], Value::Bool(true));

        let minimap = read_json(MINIMAP);
        assert_eq!(minimap["ui_w3_m3_001"]["green"], Value::Bool(true));
        assert_eq!(minimap["ui_w3_m3_001"]["operational_green"], Value::Bool(true));
        assert_eq!(minimap["ui_p3_001_green"], Value::Bool(true));

        let stage7 = read_json(STAGE7);
        assert_eq!(stage7["s7b_m3_green"], Value::Bool(true));
    }
}
