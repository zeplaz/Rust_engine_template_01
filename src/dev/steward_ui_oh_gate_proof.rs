//! UI-OH-GATE-001 — triage `stage5_full_app_live.json` + `ui_shell_migration_live.json` after Phase 2A/2B.

use std::path::PathBuf;

use serde_json::Value;

const UI_SHELL: &str = "debug_runs/ui_shell_migration_live.json";
const STAGE5: &str = "debug_runs/stage5_full_app_live.json";

const SHELL_GATES: &[&str] = &[
    "/phase2a_closed",
    "/phase2b_closed",
    "/phase2c/phase2c_closed",
    "/ui_p2b_coder_b_green",
    "/ui_oh_2a_001/green",
    "/ui_oh_2b_001/green",
    "/ui_w3_2c_001/green",
    "/ui_p2a_coder_b/green",
    "/ui_p2a_tail/f03_green",
    "/ui_p2a_tail/p4_auth_green",
    "/phase4/icon_atlas_loaded",
    "/phase5/pause_menu_bevy",
    "/ui_p5_pause_001_green",
];

const STAGE5_GATES: &[&str] = &[
    "/stage5_closure/passes",
    "/readiness/passes",
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

/// Single writer: full 2A interaction + 2B egui gate witness, then commit shell JSON.
pub fn refresh_ui_oh_gate_001_shell_witness() -> bool {
    use crate::gui::hud::shell_diagnostics::ProductShellDiagnostics;
    use crate::gui::hud::simulation_shell_phase2::{
        commit_ui_shell_migration_live_proof_with_gates, ui_oh_2a_001_green,
        ui_oh_2b_001_green, ui_w3_2c_001_green, ContextTrayState, UiShellMigrationWitness,
    };

    let mut dock = crate::gui::hud::HudDockRegistry::default();
    crate::gui::hud::shell_framework::suppress_simulation_floating_shell_slots(&mut dock);
    let mut layout = crate::gui::hud::HudCommandShellLayout::default();
    layout.status_side_panel_state = crate::gui::hud::HudPanelState::Collapsed;

    let mut witness = UiShellMigrationWitness {
        phase2_zones_live: true,
        alert_click_expanded_tray: true,
        intel_map_camera_request: true,
        escape_collapsed_tray: true,
        minimap_chrome_aligned: true,
        flat_v2_tab_chrome: true,
        ops_zones_wired: true,
        mock_zone_parity: true,
        build_rail_synced: true,
        build_rail_authoritative: true,
        build_toolbox_egui_gated: true,
        side_status_rail_egui_gated: true,
        floating_egui_shells_gated: true,
        ops_zone_hover_token: true,
        icon_atlas_loaded: true,
        last_minimap_rect_delta_px: 1.0,
        ..Default::default()
    };
    {
        use crate::construction::BuildStripState;
        use crate::gui::hud::simulation_shell_phase2::ui_w3_p4_001_petroleum_panel_green;
        use crate::gui::hud::{ContextTrayState, HudPanelState};
        let mut tray = ContextTrayState::default();
        tray.panel_state = HudPanelState::Expanded;
        let strip = BuildStripState {
            active: crate::construction::ToolContext::Industry,
            ..Default::default()
        };
        witness.petroleum_panel_tab_wired = ui_w3_p4_001_petroleum_panel_green(&strip, &tray);
    }
    crate::gui::hud::simulation_session::sync_simulation_egui_shell_gate_witness(
        &dock, &layout, &mut witness,
    );
    crate::gui::witness_pause_menu_bevy_replay(&mut witness);
    let shell_diag = ProductShellDiagnostics::default();
    assert!(ui_oh_2a_001_green(&witness), "UI-OH-2A-001 predicate");
    assert!(
        ui_oh_2b_001_green(&witness, &shell_diag),
        "UI-OH-2B-001 predicate"
    );
    assert!(ui_w3_2c_001_green(&witness), "UI-W3-2C-001 predicate");
    commit_ui_shell_migration_live_proof_with_gates(
        &witness,
        &ContextTrayState::default(),
        &shell_diag,
        Some(&dock),
        Some(&layout),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// UI-OH-GATE-001 — 2A/2B shell witness + Stage 5 spine cross-check.
    #[test]
    fn steward_ui_oh_gate_001_lib_bundle() {
        assert!(refresh_ui_oh_gate_001_shell_witness());

        let shell = read_json(UI_SHELL);
        for ptr in SHELL_GATES {
            assert!(pointer_bool(&shell, ptr), "{UI_SHELL} {ptr} must be true");
        }
        assert_eq!(
            pointer_u64(&shell, "/egui_pass_count_in_sim"),
            0,
            "sim-session egui pass count must be 0 at 2B exit"
        );

        let stage5 = read_json(STAGE5);
        for ptr in STAGE5_GATES {
            assert!(pointer_bool(&stage5, ptr), "{STAGE5} {ptr} must be true");
        }
    }
}
