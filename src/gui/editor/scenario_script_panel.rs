//! **Wave 2–4** — TEMP-EGUI scenario script panel (Editor only).
//! - UX: tooltips, help foldout, next-step preview, runbook pointer, validation messages, objective inspector stub.
//! - Discovery: **Editor — Scenario tools** anchor window + hotkey [`InputBindings::toggle_scenario_script_panel`] (default **F10**).
//! - Spec: `prompts/guides/scenario_campaign_scripted_tools_runbook_v1.md`.
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContexts;

use crate::gui::std_floating;
use crate::gui::InputBindings;
use crate::gui::style::{
    error_text, framed_group, muted_text, path_hint, primary_text, scenario_execution_badge,
    section_heading, success_text, v_space, warning_text, widget_scroll_vertical_capped,
    widget_scroll_vertical_fill, CmdHeadingStyle, UiPalette, UiSpacing, VertSpace,
};
use crate::scenario::scenario_steps::ScenarioStep;
use crate::scenario::scenario_types::ScenarioFileV1;
use crate::scenario::script_host::EngineScriptHost;

const RUNBOOK_PATH: &str = "prompts/guides/scenario_campaign_scripted_tools_runbook_v1.md";

fn manifest_join_relative(user_path: &str) -> PathBuf {
    let trimmed = user_path.trim();
    let p = Path::new(trimmed);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(trimmed)
    }
}

fn export_json_path_for(ron_path: &Path) -> PathBuf {
    let mut p = ron_path.to_path_buf();
    p.set_extension("json");
    p
}

#[derive(Resource, Debug, Clone)]
pub struct ScenarioScriptPanelState {
    /// Path relative to crate root or absolute (e.g. `assets/scenarios/tests/minimal_wave1.scenario.ron`).
    pub file_path: String,
    pub status_line: String,
    pub autoscroll_log: bool,
    /// Window visibility: hotkey + palette can reopen.
    pub window_open: bool,
    /// **Editor — Scenario tools** anchor window (map-editor session only).
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
    }
}

/// **Editor → Scenario tools** entry window (predictable discovery; complements F10 + script panel).
pub fn scenario_editor_tools_entry_window(
    mut contexts: EguiContexts,
    bindings: Res<InputBindings>,
    mut panel: ResMut<ScenarioScriptPanelState>,
    palette: Res<UiPalette>,
    spacing: Res<UiSpacing>,
) -> Result {
    if !panel.tools_entry_visible {
        return Ok(());
    }
    let pal: &UiPalette = &palette;
    let sp: &UiSpacing = &spacing;
    std_floating(egui::Window::new("Editor — Scenario tools"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 52.0))
        .collapsible(true)
        .default_size(egui::vec2(280.0, 220.0))
        .show(contexts.ctx_mut()?, |ui| {
            section_heading(ui, pal, CmdHeadingStyle::Gt, "Scenario tools");
            muted_text(ui, pal, "Script runner (*.scenario.ron)");
            path_hint(ui, pal, RUNBOOK_PATH);
            v_space(ui, sp, VertSpace::Inter);
            muted_text(
                ui,
                pal,
                format!("Toggle: {}", InputBindings::format_key(bindings.toggle_scenario_script_panel)),
            );
            if ui
                .button("Open script runner…")
                .on_hover_text("Main scenario host window (Load / Run / objectives).")
                .clicked()
            {
                panel.window_open = true;
            }
        });
    Ok(())
}

pub fn scenario_script_panel_system(
    mut contexts: EguiContexts,
    mut host: ResMut<EngineScriptHost>,
    mut panel: ResMut<ScenarioScriptPanelState>,
    palette: Res<UiPalette>,
    spacing: Res<UiSpacing>,
) -> Result {
    let pal: &UiPalette = &palette;
    let sp: &UiSpacing = &spacing;
    let mut open = panel.window_open;
    std_floating(egui::Window::new(
        "Scenario script — Editor / Scenario tools (TEMP-EGUI)",
    ))
        .open(&mut open)
        .default_size([440.0, 560.0])
        .show(contexts.ctx_mut()?, |ui| {
            widget_scroll_vertical_fill("scenario_script_panel_body_scroll", ui.available_height()).show(ui, |ui| {
            framed_group(ui, pal, |ui| {
                section_heading(ui, pal, CmdHeadingStyle::Gt, "Scenario script runner");
                path_hint(ui, pal, RUNBOOK_PATH);
                v_space(ui, sp, VertSpace::Xs);
                muted_text(
                    ui,
                    pal,
                    "One script step per frame. Wave 5+: dock into editor shell; today uses a standard egui window.",
                );
            });
            v_space(ui, sp, VertSpace::Inter);

            ui.collapsing("Help — scripting UX", |ui| {
                muted_text(
                    ui,
                    pal,
                    "Paths: relative to crate root or absolute (CARGO_MANIFEST_DIR).",
                );
                muted_text(
                    ui,
                    pal,
                    "Load parses *.scenario.ron; validation errors block load; warnings allow load.",
                );
                muted_text(
                    ui,
                    pal,
                    "Save writes full authoritative RON (lossless). Export JSON is runtime subset only.",
                );
                muted_text(
                    ui,
                    pal,
                    "Stop pauses the queue; Run/resume continues or replays when the queue is empty.",
                );
                muted_text(
                    ui,
                    pal,
                    "RegisterObjectives → ScenarioObjectiveMarker; clear_existing despawns only those markers.",
                );
                v_space(ui, sp, VertSpace::Xs);
                section_heading(ui, pal, CmdHeadingStyle::Tilde, "Example scenarios");
                path_hint(ui, pal, "assets/scenarios/tests/minimal_wave1.scenario.ron");
                path_hint(ui, pal, "assets/scenarios/tests/wave3_objectives.scenario.ron");
            });
            v_space(ui, sp, VertSpace::Xs);

            section_heading(
                ui,
                pal,
                CmdHeadingStyle::None,
                "Authoring: *.scenario.ron",
            );
            muted_text(
                ui,
                pal,
                "Stable objective_id, ObjectiveTargetRef, factions, tags.",
            );
            v_space(ui, sp, VertSpace::Xs);

            ui.horizontal(|ui| {
                muted_text(ui, pal, "Path:");
                ui.text_edit_singleline(&mut panel.file_path)
                    .on_hover_text("Relative to project root, e.g. assets/scenarios/tests/wave3_objectives.scenario.ron");
            });

            if let Some(next) = host.pending_steps.front() {
                muted_text(
                    ui,
                    pal,
                    format!("Next step (preview): {next:?}"),
                );
            } else if host.active_script.is_some() {
                muted_text(ui, pal, "Next step: (queue empty — use Run to replay)");
            }

            ui.horizontal(|ui| {
                if ui
                    .button("Load")
                    .on_hover_text("Read file → parse RON → validate → replace queued steps.")
                    .clicked()
                {
                    panel.status_line.clear();
                    let path = manifest_join_relative(&panel.file_path);
                    match std::fs::read_to_string(&path) {
                        Ok(text) => match ron::from_str::<ScenarioFileV1>(&text) {
                            Ok(file) => {
                                host.load_script(file);
                                if host.last_error.is_none() {
                                    panel.status_line =
                                        format!("Loaded {} ({} steps)", path.display(), host.pending_steps.len());
                                }
                            }
                            Err(e) => {
                                panel.status_line = format!("RON parse error: {e}");
                            }
                        },
                        Err(e) => {
                            panel.status_line = format!("Read {}: {e}", path.display());
                        }
                    }
                }

                if ui
                    .button("Save")
                    .on_hover_text("Write active scenario (from last successful Load) to Path as **full** RON.")
                    .clicked()
                {
                    panel.status_line.clear();
                    let Some(active) = host.active_script.as_ref() else {
                        panel.status_line = "Nothing to save — load a scenario first.".into();
                        return;
                    };
                    let path = manifest_join_relative(&panel.file_path);
                    match active.to_ron_string_pretty() {
                        Ok(s) => {
                            if let Some(dir) = path.parent() {
                                let _ = std::fs::create_dir_all(dir);
                            }
                            match std::fs::write(&path, format!("{}\n", s.trim_end())) {
                                Ok(()) => {
                                    panel.status_line = format!("Wrote {}", path.display());
                                }
                                Err(e) => {
                                    panel.status_line = format!("Write {}: {e}", path.display());
                                }
                            }
                        }
                        Err(e) => {
                            panel.status_line = format!("RON serialize: {e}");
                        }
                    }
                }

                if ui
                    .button("Export JSON (subset)")
                    .on_hover_text("Runtime/interchange subset beside Path (same basename, .json). Not a full save.")
                    .clicked()
                {
                    panel.status_line.clear();
                    let Some(active) = host.active_script.as_ref() else {
                        panel.status_line = "Nothing to export — load a scenario first.".into();
                        return;
                    };
                    let ron_path = manifest_join_relative(&panel.file_path);
                    let json_path = export_json_path_for(&ron_path);
                    match active.export_runtime_json_subset() {
                        Ok(s) => {
                            if let Some(dir) = json_path.parent() {
                                let _ = std::fs::create_dir_all(dir);
                            }
                            match std::fs::write(&json_path, format!("{s}\n")) {
                                Ok(()) => {
                                    panel.status_line =
                                        format!("Exported runtime JSON {}", json_path.display());
                                }
                                Err(e) => {
                                    panel.status_line =
                                        format!("Write {}: {e}", json_path.display());
                                }
                            }
                        }
                        Err(e) => {
                            panel.status_line = format!("JSON serialize: {e}");
                        }
                    }
                }

                if ui
                    .button("Run / resume")
                    .on_hover_text("Resume after Stop, or replay entire script when the queue has finished.")
                    .clicked()
                {
                    if !host.pending_steps.is_empty() && !host.running {
                        host.resume();
                    } else if host.active_script.is_some() && host.pending_steps.is_empty() {
                        host.restart_from_active();
                    } else if host.active_script.is_none() {
                        panel.status_line = "Load a scenario before Run.".into();
                    }
                }

                if ui
                    .button("Stop")
                    .on_hover_text("Pause step drain; pending steps stay queued.")
                    .clicked()
                {
                    host.stop();
                    panel.status_line = "Scenario execution stopped (pending steps kept).".into();
                }
            });

            v_space(ui, sp, VertSpace::Xs);
            ui.horizontal(|ui| {
                muted_text(ui, pal, "State:");
                scenario_execution_badge(ui, pal, host.current_state);
                muted_text(ui, pal, "· pending:");
                ui.monospace(format!("{}", host.pending_steps.len()));
            });

            if let Some(report) = host.last_validation.as_ref() {
                for w in &report.warnings {
                    warning_text(ui, pal, format!("Warning: {w}"));
                }
            }

            if !panel.status_line.is_empty() {
                let s = panel.status_line.as_str();
                let is_success = s.starts_with("Loaded ")
                    || s.starts_with("Wrote ")
                    || s.starts_with("Exported ");
                let is_warn = !is_success
                    && (s.contains("error")
                        || s.contains("Error")
                        || s.contains("Nothing to")
                        || s.starts_with("Read "));
                if is_success {
                    success_text(ui, pal, s);
                } else if is_warn {
                    warning_text(ui, pal, s);
                } else {
                    primary_text(ui, pal, s);
                }
            }
            if let Some(err) = &host.last_error {
                error_text(ui, pal, err.as_str());
            }

            ui.collapsing("Objectives (inspector stub)", |ui| {
                let Some(script) = host.active_script.as_ref() else {
                    muted_text(ui, pal, "Load a scenario to list objectives.");
                    return;
                };
                for step in &script.steps {
                    let ScenarioStep::RegisterObjectives { objectives, .. } = step else {
                        continue;
                    };
                    for o in objectives {
                        let sel = panel
                            .inspector_focus_id
                            .as_deref()
                            == Some(o.objective_id.as_str());
                        let label = format!("{} — {}", o.objective_id, o.label);
                        if ui.selectable_label(sel, label).clicked() {
                            panel.inspector_focus_id = Some(o.objective_id.clone());
                            panel.status_line = format!(
                                "Inspector focus: {} (Wave 4+: camera + overlay; infrastructure hooks)",
                                o.objective_id
                            );
                        }
                    }
                }
            });
            });

            v_space(ui, sp, VertSpace::Inter);
            ui.checkbox(&mut panel.autoscroll_log, "Autoscroll log");
            ui.separator();
            section_heading(ui, pal, CmdHeadingStyle::None, "Execution log");
            let log_len = host.execution_log.len();
            widget_scroll_vertical_capped("scenario_script_execution_log_scroll", 280.0)
                .stick_to_bottom(panel.autoscroll_log)
                .show(ui, |ui| {
                    let start = log_len.saturating_sub(200);
                    for (i, line) in host.execution_log.iter().enumerate().skip(start) {
                        ui.monospace(format!("{i:4} {line}"));
                    }
                });
        });
    panel.window_open = open;
    Ok(())
}
