//! SIM-HUD-SLICE-PLAY01 live witness — `debug_runs/sim_hud_play01_live.json`.

use serde_json::{json, Value};

use crate::dev::debug_run_envelope;
use crate::gui::DiagnosticsUiState;
use crate::gui::editor::scenario_script_panel::ScenarioScriptPanelState;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::editor::world_preview::WorldPreviewUiState;
use crate::gui::hud::simulation_session::{
    capture_editor_chrome_before_simulation, collapse_simulation_floating_shell_layout,
    play01_editor_chrome_hidden, restore_editor_chrome_from_snapshot,
    sim_hud_slice_play01_green, SimulationEditorChromeSnapshot,
};
use crate::gui::hud::{
    ContextTrayState, HudCommandShellLayout, HudDockRegistry, HudOverlayTrayState, HudPanelState,
    ProductShellWidgetId, TransmissionShellState,
};
use crate::gui::hud::shell_framework::suppress_simulation_floating_shell_slots;

pub const SIM_HUD_PLAY01_LIVE_JSON: &str = "debug_runs/sim_hud_play01_live.json";

#[must_use]
pub fn replay_sim_hud_slice_play01_roundtrip() -> (bool, bool, bool) {
    let mut snapshot = SimulationEditorChromeSnapshot::default();
    let mut dock = HudDockRegistry::default();
    dock.slot_mut(ProductShellWidgetId::BuildToolbox).visible = true;
    dock.slot_mut(ProductShellWidgetId::OverlaysPanel).visible = true;
    let mut layout = HudCommandShellLayout::default();
    layout.command_tray_state = HudPanelState::Expanded;
    layout.overlay_tray_state = HudPanelState::Expanded;
    layout.status_side_panel_state = HudPanelState::Expanded;
    let mut tray = HudOverlayTrayState::default();
    tray.tray_panel_state = HudPanelState::Expanded;
    let mut transmission = TransmissionShellState::default();
    transmission.panel_state = HudPanelState::Expanded;
    let mut context_tray = ContextTrayState::default();
    context_tray.panel_state = HudPanelState::Expanded;
    let mut script_panel = ScenarioScriptPanelState::default();
    script_panel.window_open = true;
    script_panel.tools_entry_visible = true;
    let mut world_gen = WorldGenUiState::default();
    world_gen.visible = true;
    let mut preview_ui = WorldPreviewUiState::default();
    preview_ui.window_open = true;
    let mut diagnostics = DiagnosticsUiState::default();
    diagnostics.sections_default_open = true;

    capture_editor_chrome_before_simulation(
        &mut snapshot,
        &dock,
        &layout,
        &tray,
        &transmission,
        &context_tray,
        &script_panel,
        &world_gen,
        &preview_ui,
        &diagnostics,
    );
    let snapshot_captured = snapshot.captured;

    script_panel.window_open = false;
    script_panel.tools_entry_visible = false;
    world_gen.visible = false;
    preview_ui.window_open = false;
    collapse_simulation_floating_shell_layout(&mut layout, &mut tray, &mut transmission);
    context_tray.panel_state = HudPanelState::Collapsed;
    suppress_simulation_floating_shell_slots(&mut dock);
    layout.status_side_panel_state = HudPanelState::Collapsed;
    diagnostics.sections_default_open = false;

    let enter_hidden = play01_editor_chrome_hidden(
        &script_panel,
        &world_gen,
        &preview_ui,
        &layout,
        &dock,
    );

    let exit_restored = restore_editor_chrome_from_snapshot(
        &snapshot,
        &mut dock,
        &mut layout,
        &mut tray,
        &mut transmission,
        &mut context_tray,
        &mut script_panel,
        &mut world_gen,
        &mut preview_ui,
        &mut diagnostics,
    );

    (snapshot_captured, enter_hidden, exit_restored)
}

#[must_use]
pub fn build_sim_hud_play01_proof_payload() -> Value {
    let (snapshot_captured, enter_hidden, exit_restored) = replay_sim_hud_slice_play01_roundtrip();
    let green = sim_hud_slice_play01_green(enter_hidden, exit_restored, snapshot_captured);
    json!({
        "gate": "SIM-HUD-SLICE-PLAY01",
        "profile": "SIM_HUD_PLAY01",
        "sim_hud_slice_play01": {
            "green": green,
            "enter_hidden": enter_hidden,
            "snapshot_captured": snapshot_captured,
            "exit_restore_wired": exit_restored,
            "product_egui_shell_in_simulation": false,
            "modules": [
                "src/gui/hud/simulation_session.rs",
                "src/gui/ui_gates.rs"
            ],
        },
        "sim_hud_product_001_link": "debug_runs/ui_shell_migration_live.json",
    })
}

#[must_use]
pub fn commit_sim_hud_play01_live_proof() -> bool {
    let body = build_sim_hud_play01_proof_payload();
    let green = body["sim_hud_slice_play01"]["green"].as_bool().unwrap_or(false);
    if !green {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "SIM_HUD_PLAY01",
        "commit_sim_hud_play01_live_proof",
        SIM_HUD_PLAY01_LIVE_JSON,
        body,
    );
    debug_run_envelope::write_debug_run_json(SIM_HUD_PLAY01_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_play01_live_witness() {
        assert!(commit_sim_hud_play01_live_proof());
        let text = std::fs::read_to_string(SIM_HUD_PLAY01_LIVE_JSON).expect("proof json");
        let v: Value = serde_json::from_str(&text).expect("parse");
        assert!(v["sim_hud_slice_play01"]["green"].as_bool().unwrap_or(false));
    }
}
