//! **Wave 2–4** — scenario script panel (Editor / map-editor chrome).
//!
//! Bevy body lives on [`crate::gui::editor::map_editor::MapEditorBevyShell`]
//! (palette | minimap | scenario). Hotkey: [`InputBindings::toggle_scenario_script_panel`] (F10).
//! Spec: `prompts/guides/scenario_campaign_scripted_tools_runbook_v1.md`.

use bevy::prelude::*;

use crate::gui::InputBindings;

const RUNBOOK_PATH: &str = "prompts/guides/scenario_campaign_scripted_tools_runbook_v1.md";

#[derive(Resource, Debug, Clone)]
pub struct ScenarioScriptPanelState {
    /// Path relative to crate root or absolute (e.g. `assets/scenarios/tests/minimal_wave1.scenario.ron`).
    pub file_path: String,
    pub status_line: String,
    pub autoscroll_log: bool,
    /// Window visibility: hotkey + palette can reopen.
    pub window_open: bool,
    /// **Editor — Scenario tools** anchor (map-editor session only).
    pub tools_entry_visible: bool,
    /// Wave 4+: camera / overlay focus for selected objective id.
    pub inspector_focus_id: Option<String>,
}

impl Default for ScenarioScriptPanelState {
    fn default() -> Self {
        Self {
            file_path: "assets/scenarios/tests/minimal_wave1.scenario.ron".into(),
            status_line: String::new(),
            autoscroll_log: true,
            window_open: false,
            tools_entry_visible: true,
            inspector_focus_id: None,
        }
    }
}

pub fn toggle_scenario_script_panel_hotkey(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut panel: ResMut<ScenarioScriptPanelState>,
) {
    if keys.just_pressed(bindings.toggle_scenario_script_panel) {
        panel.window_open = !panel.window_open;
        if panel.window_open && panel.status_line.is_empty() {
            panel.status_line = format!("runbook: {RUNBOOK_PATH}");
        }
    }
}

/// Retired egui entry — Bevy chrome via `MapEditorBevyShell`.
pub fn scenario_editor_tools_entry_window() -> Result {
    Ok(())
}

/// Retired egui body — Bevy chrome via `sync_map_editor_bevy_shell`.
pub fn scenario_script_panel_system() -> Result {
    Ok(())
}
