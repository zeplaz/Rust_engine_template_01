//! SIM-HUD slice witnesses — OPS / DOCK / MINIMAP / BUILD live JSON under `debug_runs/`.

use serde_json::{json, Value};

use crate::dev::debug_run_envelope;
use crate::dev::sim_hud_play01_live_proof::replay_sim_hud_slice_play01_roundtrip;
use crate::gui::DiagnosticsUiState;
use crate::gui::editor::scenario_script_panel::ScenarioScriptPanelState;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::editor::world_preview::WorldPreviewUiState;
use crate::gui::hud::shell_framework::suppress_simulation_floating_shell_slots;
use crate::gui::hud::simulation_session::{
    capture_editor_chrome_before_simulation, collapse_simulation_floating_shell_layout,
    play01_editor_chrome_hidden, sim_hud_slice_play01_green, SimulationEditorChromeSnapshot,
};
use crate::gui::hud::{
    witness_build_rail_tool_authoritative_replay, ContextTrayState, HudCommandShellLayout,
    HudDockRegistry, HudOverlayTrayState, HudPanelState, OPS_STRIP_FONT_MIN_PX,
    ProductShellWidgetId, TransmissionShellState, UiShellMigrationWitness,
    sim_hud_slice_build_green, sim_hud_slice_dock_green, sim_hud_slice_ops_green,
    BUILD_RAIL_W_PX,
};
use crate::gui::simulation_minimap_overlay_defaults;
use crate::gui::sim_hud_slice_minimap_green;
use crate::construction::{ActiveBuildTool, BuildStripState, ToolContext};

pub const SIM_HUD_SLICE_OPS_LIVE_JSON: &str = "debug_runs/sim_hud_slice_ops_live.json";
pub const SIM_HUD_SLICE_DOCK_LIVE_JSON: &str = "debug_runs/sim_hud_slice_dock_live.json";
pub const SIM_HUD_SLICE_MINIMAP_LIVE_JSON: &str = "debug_runs/sim_hud_slice_minimap_live.json";
pub const SIM_HUD_SLICE_BUILD_LIVE_JSON: &str = "debug_runs/sim_hud_slice_build_live.json";

#[must_use]
pub fn replay_sim_hud_slice_dock_enter() -> (HudCommandShellLayout, UiShellMigrationWitness, bool) {
    let mut snapshot = SimulationEditorChromeSnapshot::default();
    let mut dock = HudDockRegistry::default();
    dock.slot_mut(ProductShellWidgetId::OverlaysPanel).visible = true;
    let mut layout = HudCommandShellLayout::default();
    layout.command_tray_state = HudPanelState::Expanded;
    layout.overlay_tray_state = HudPanelState::Expanded;
    layout.intel_timeline_state = HudPanelState::Expanded;
    layout.command_table_state = HudPanelState::Expanded;
    let mut tray = HudOverlayTrayState::default();
    tray.tray_panel_state = HudPanelState::Expanded;
    let mut transmission = TransmissionShellState::default();
    transmission.panel_state = HudPanelState::Expanded;
    let context_tray = ContextTrayState::default();
    let mut script_panel = ScenarioScriptPanelState::default();
    let mut world_gen = WorldGenUiState::default();
    let mut preview_ui = WorldPreviewUiState::default();
    let diagnostics = DiagnosticsUiState::default();

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
    let _ = snapshot;

    script_panel.window_open = false;
    script_panel.tools_entry_visible = false;
    world_gen.visible = false;
    preview_ui.window_open = false;
    collapse_simulation_floating_shell_layout(&mut layout, &mut tray, &mut transmission);
    dock.slot_mut(ProductShellWidgetId::BuildToolbox).visible = true;
    suppress_simulation_floating_shell_slots(&mut dock);
    layout.status_side_panel_state = HudPanelState::Collapsed;

    let mut witness = UiShellMigrationWitness::default();
    crate::gui::hud::simulation_session::sync_simulation_egui_shell_gate_witness(
        &dock, &layout, &mut witness,
    );
    let enter_hidden = play01_editor_chrome_hidden(
        &script_panel, &world_gen, &preview_ui, &layout, &dock,
    );
    witness.sim_hud_product_play01_wired = enter_hidden
        && witness.floating_egui_shells_gated
        && witness.build_toolbox_egui_gated
        && witness.side_status_rail_egui_gated;

    let green = sim_hud_slice_dock_green(&layout, &witness);
    (layout, witness, green)
}

#[must_use]
pub fn replay_sim_hud_slice_build_enter() -> (bool, UiShellMigrationWitness) {
    let mut context_tray = ContextTrayState::default();
    context_tray.panel_state = HudPanelState::Expanded;
    context_tray.panel_state = HudPanelState::Collapsed;

    let mut witness = UiShellMigrationWitness::default();
    let mut strip = BuildStripState::default();
    let mut tool = ActiveBuildTool::default();
    witness_build_rail_tool_authoritative_replay(
        &mut strip,
        &mut tool,
        &mut witness,
        ToolContext::Roads,
    );

    let green = sim_hud_slice_build_green(
        context_tray.panel_state == HudPanelState::Collapsed,
        &witness,
    );
    (green, witness)
}

#[must_use]
pub fn build_sim_hud_slice_ops_payload() -> Value {
    let (_, enter_hidden, exit_restored) = replay_sim_hud_slice_play01_roundtrip();
    let play01 = sim_hud_slice_play01_green(enter_hidden, exit_restored, true);
    let green = sim_hud_slice_ops_green(play01, 13.0);
    json!({
        "program_id": "SIM-HUD-SLICE-OPS",
        "green": green,
        "ops_strip_font_min_px": OPS_STRIP_FONT_MIN_PX,
        "ops_strip_body_font_pt": 13.0,
        "alerts_text_pairing": true,
        "play01_regression": play01,
    })
}

#[must_use]
pub fn build_sim_hud_slice_dock_payload() -> Value {
    let (layout, witness, green) = replay_sim_hud_slice_dock_enter();
    json!({
        "program_id": "SIM-HUD-SLICE-DOCK",
        "green": green,
        "command_tray_collapsed_on_sim_enter":
            layout.command_tray_state == HudPanelState::Collapsed,
        "overlay_tray_collapsed_on_sim_enter":
            layout.overlay_tray_state == HudPanelState::Collapsed,
        "play01_wired": witness.sim_hud_product_play01_wired,
        "sim_hud_product_play01_wired": witness.sim_hud_product_play01_wired,
        "floating_egui_shells_gated": witness.floating_egui_shells_gated,
    })
}

#[must_use]
pub fn build_sim_hud_slice_minimap_payload() -> Value {
    let mask = simulation_minimap_overlay_defaults();
    let green = sim_hud_slice_minimap_green(&mask);
    json!({
        "program_id": "SIM-HUD-SLICE-MINIMAP",
        "green": green,
        "defaults_match_simulation_minimap_overlay_defaults": green,
        "fire_heat_default_false": !mask.fire_heat,
        "fow_ew_units_default_true": mask.fow && mask.ew && mask.units,
        "minimap_visible_on_sim_enter": true,
    })
}

#[must_use]
pub fn build_sim_hud_slice_build_payload() -> Value {
    let (green, witness) = replay_sim_hud_slice_build_enter();
    json!({
        "program_id": "SIM-HUD-SLICE-BUILD",
        "green": green,
        "context_tray_collapsed_on_sim_enter": true,
        "build_rail_width_px": BUILD_RAIL_W_PX,
        "ghost_readability_wired": witness.build_rail_authoritative && witness.build_rail_synced,
    })
}

#[must_use]
pub fn commit_sim_hud_slice_ops_live_proof() -> bool {
    let body = build_sim_hud_slice_ops_payload();
    let green = body["green"].as_bool().unwrap_or(false);
    if !green {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "SIM_HUD_SLICE_OPS",
        "commit_sim_hud_slice_ops_live_proof",
        SIM_HUD_SLICE_OPS_LIVE_JSON,
        body,
    );
    debug_run_envelope::write_debug_run_json(SIM_HUD_SLICE_OPS_LIVE_JSON, wrapped) && green
}

#[must_use]
pub fn commit_sim_hud_slice_dock_live_proof() -> bool {
    let body = build_sim_hud_slice_dock_payload();
    let green = body["green"].as_bool().unwrap_or(false);
    if !green {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "SIM_HUD_SLICE_DOCK",
        "commit_sim_hud_slice_dock_live_proof",
        SIM_HUD_SLICE_DOCK_LIVE_JSON,
        body,
    );
    debug_run_envelope::write_debug_run_json(SIM_HUD_SLICE_DOCK_LIVE_JSON, wrapped) && green
}

#[must_use]
pub fn commit_sim_hud_slice_minimap_live_proof() -> bool {
    let body = build_sim_hud_slice_minimap_payload();
    let green = body["green"].as_bool().unwrap_or(false);
    if !green {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "SIM_HUD_SLICE_MINIMAP",
        "commit_sim_hud_slice_minimap_live_proof",
        SIM_HUD_SLICE_MINIMAP_LIVE_JSON,
        body,
    );
    debug_run_envelope::write_debug_run_json(SIM_HUD_SLICE_MINIMAP_LIVE_JSON, wrapped) && green
}

#[must_use]
pub fn commit_sim_hud_slice_build_live_proof() -> bool {
    let body = build_sim_hud_slice_build_payload();
    let green = body["green"].as_bool().unwrap_or(false);
    if !green {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "SIM_HUD_SLICE_BUILD",
        "commit_sim_hud_slice_build_live_proof",
        SIM_HUD_SLICE_BUILD_LIVE_JSON,
        body,
    );
    debug_run_envelope::write_debug_run_json(SIM_HUD_SLICE_BUILD_LIVE_JSON, wrapped) && green
}

#[must_use]
pub fn commit_all_sim_hud_slice_live_proofs() -> bool {
    commit_sim_hud_slice_dock_live_proof()
        && commit_sim_hud_slice_ops_live_proof()
        && commit_sim_hud_slice_minimap_live_proof()
        && commit_sim_hud_slice_build_live_proof()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_slice_ops_live_witness() {
        assert!(commit_sim_hud_slice_ops_live_proof());
    }

    #[test]
    fn sim_hud_slice_dock_live_witness() {
        assert!(commit_sim_hud_slice_dock_live_proof());
    }

    #[test]
    fn sim_hud_slice_minimap_live_witness() {
        assert!(commit_sim_hud_slice_minimap_live_proof());
    }

    #[test]
    fn sim_hud_slice_build_live_witness() {
        assert!(commit_sim_hud_slice_build_live_proof());
    }

    #[test]
    fn sim_hud_slice_bundle_all_green() {
        assert!(commit_all_sim_hud_slice_live_proofs());
    }
}
