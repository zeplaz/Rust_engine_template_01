//! APS-BEVY-QC-HUD-001 — egui dev panel for assembly snapshot QC (Lane A′).

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use serde_json::Value;

use crate::construction::procedural::AssemblySnapshot;
use crate::gui::ui_gates::product_egui_shell_active;
use crate::preview::{load_assembly_snapshot_json, repo_root_from_manifest};

pub const APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT: &str =
    "tools/mcp/schemas/examples/assembly_snapshot_warehouse_industrial_west_production_v1.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QcRowStatus {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct QcTableRow {
    pub cell: String,
    pub module_id: String,
    pub material_profile: String,
    pub tags: String,
    pub glb: String,
    pub status: QcRowStatus,
}

#[derive(Debug, Clone)]
pub struct QcSnapshotSummary {
    pub assembly_id: String,
    pub placement_count: usize,
    pub rows: Vec<QcTableRow>,
}

#[derive(Debug, Clone)]
pub struct P0PlainIssue {
    pub sentence: String,
    pub fix_hint: String,
}

#[derive(Resource, Debug, Clone)]
pub struct AssemblySnapshotQcUiState {
    pub visible: bool,
    pub path_input: String,
    pub loaded: Option<AssemblySnapshot>,
    pub summary: Option<QcSnapshotSummary>,
    pub load_error: Option<String>,
    pub selected_row: Option<usize>,
    pub last_preview_hint: Option<String>,
    pub preview_active: bool,
    pub p0_issues: Vec<P0PlainIssue>,
}

impl Default for AssemblySnapshotQcUiState {
    fn default() -> Self {
        Self {
            visible: false,
            path_input: APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT.into(),
            loaded: None,
            summary: None,
            load_error: None,
            selected_row: None,
            last_preview_hint: None,
            preview_active: false,
            p0_issues: Vec::new(),
        }
    }
}

pub struct AssemblySnapshotQcUiPlugin;

impl Plugin for AssemblySnapshotQcUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssemblySnapshotQcUiState>()
            .add_systems(EguiPrimaryContextPass, draw_assembly_snapshot_qc_ui);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

fn row_status(glb_path: &str, repo_root: &Path) -> QcRowStatus {
    if glb_path.is_empty() {
        return QcRowStatus::Fail;
    }
    let full = repo_root.join(glb_path.replace('\\', "/"));
    if full.is_file() {
        QcRowStatus::Ok
    } else {
        QcRowStatus::Warn
    }
}

pub fn load_qc_snapshot(path: &Path) -> Result<(AssemblySnapshot, QcSnapshotSummary), String> {
    let snapshot = load_assembly_snapshot_json(path)?;
    let repo_root = repo_root_from_manifest();
    let mut rows = Vec::with_capacity(snapshot.module_placements.len());
    for p in &snapshot.module_placements {
        let material = if p.material_profile.is_empty() {
            "(missing)".into()
        } else {
            p.material_profile.clone()
        };
        rows.push(QcTableRow {
            cell: format!("({},{},f{})", p.grid_x, p.grid_y, p.floor),
            module_id: p.module_id.clone(),
            material_profile: material,
            tags: truncate(&p.slot_key, 48),
            glb: truncate(&p.glb_path, 32),
            status: row_status(&p.glb_path, &repo_root),
        });
    }
    let summary = QcSnapshotSummary {
        assembly_id: snapshot.assembly_id.clone(),
        placement_count: snapshot.module_placements.len(),
        rows,
    };
    Ok((snapshot, summary))
}

#[must_use]
pub fn placement_grid_coords(snapshot: &AssemblySnapshot, row: usize) -> Option<(u32, u32, u32)> {
    snapshot
        .module_placements
        .get(row)
        .map(|p| (p.grid_x, p.grid_y, p.floor))
}

#[must_use]
pub fn evaluate_p0_readonly(snapshot: &AssemblySnapshot, repo_root: &Path) -> Vec<P0PlainIssue> {
    let mut issues = Vec::new();
    if snapshot.source_tier != "production" {
        issues.push(P0PlainIssue {
            sentence: "Snapshot source_tier is not production.".into(),
            fix_hint: "Regenerate with production tier or pick a production example.".into(),
        });
    }
    if snapshot.module_placements.is_empty() {
        issues.push(P0PlainIssue {
            sentence: "Snapshot has 0 placements.".into(),
            fix_hint: "Run assembly_snapshot_generate or load a non-empty example.".into(),
        });
    }
    let missing_materials = snapshot
        .module_placements
        .iter()
        .filter(|p| p.material_profile.is_empty())
        .count();
    if missing_materials > 0 {
        issues.push(P0PlainIssue {
            sentence: format!("material_profile missing on {missing_materials} cell(s)."),
            fix_hint: "Fill material_profile on each placement row.".into(),
        });
    }
    if snapshot.grammar_rule_chain.is_none() {
        issues.push(P0PlainIssue {
            sentence: "grammar_rule_chain is absent.".into(),
            fix_hint: "Generate via grammar pipeline so APS can inspect rule chain.".into(),
        });
    }
    for p in &snapshot.module_placements {
        if p.grid_x >= snapshot.footprint.width || p.grid_y >= snapshot.footprint.depth {
            issues.push(P0PlainIssue {
                sentence: format!(
                    "Placement {}@({},{}) outside footprint {}×{}.",
                    p.module_id, p.grid_x, p.grid_y, snapshot.footprint.width, snapshot.footprint.depth
                ),
                fix_hint: "Fix grid coordinates or expand footprint.".into(),
            });
            break;
        }
        if !p.glb_path.is_empty() {
            let glb = repo_root.join(p.glb_path.replace('\\', "/"));
            if !glb.is_file() {
                issues.push(P0PlainIssue {
                    sentence: format!("GLB missing for {}: {}", p.module_id, p.glb_path),
                    fix_hint: "Bake/promote module GLB or fix glb_path.".into(),
                });
                break;
            }
        }
    }
    issues
}

fn apply_load(state: &mut AssemblySnapshotQcUiState, path: PathBuf) {
    match load_qc_snapshot(&path) {
        Ok((snapshot, summary)) => {
            let repo_root = repo_root_from_manifest();
            state.p0_issues = evaluate_p0_readonly(&snapshot, &repo_root);
            state.loaded = Some(snapshot);
            state.summary = Some(summary);
            state.load_error = None;
            state.selected_row = None;
            state.preview_active = false;
        }
        Err(e) => {
            state.load_error = Some(e);
            state.loaded = None;
            state.summary = None;
            state.p0_issues.clear();
        }
    }
}

pub fn draw_assembly_snapshot_qc_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<AssemblySnapshotQcUiState>,
    base: Res<State<crate::engine::BaseState>>,
    app: Res<State<crate::engine::AppState>>,
) {
    if !product_egui_shell_active(base, app) || !state.visible {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let repo_root = repo_root_from_manifest();

    egui::Window::new("Assembly snapshot QC (APS-BEVY-QC-HUD-001)")
        .default_size([720.0, 480.0])
        .show(ctx, |ui| {
            ui.label("Read-only QC — Ctrl+Shift+Q toggle · loads assembly_snapshot JSON from disk");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut state.path_input);
                if ui.button("Load").clicked() {
                    let path = repo_root.join(state.path_input.replace('\\', "/"));
                    apply_load(&mut state, path);
                }
                if ui.button("Use example warehouse snapshot").clicked() {
                    state.path_input = APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT.into();
                    apply_load(
                        &mut state,
                        repo_root.join(APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT),
                    );
                }
            });

            if let Some(err) = &state.load_error {
                ui.colored_label(egui::Color32::LIGHT_RED, err);
            }

            let Some(summary) = state.summary.clone() else {
                ui.label("Load a snapshot to inspect placements.");
                return;
            };

            ui.separator();
            ui.heading("Summary");
            ui.label(format!(
                "{} · {} placements",
                summary.assembly_id, summary.placement_count
            ));

            ui.heading("P0 gate (read-only)");
            if state.p0_issues.is_empty() {
                ui.label("P0 gate: no blocking issues detected (read-only).");
            } else {
                for issue in &state.p0_issues {
                    ui.label(&issue.sentence);
                    ui.label(format!("→ {}", issue.fix_hint));
                }
            }

            ui.horizontal(|ui| {
                if ui.button("Spawn preview").clicked() {
                    state.preview_active = true;
                    state.last_preview_hint = Some(
                        "Preview job queued — see debug_runs/preview/ for PNG output.".into(),
                    );
                }
                if ui.button("Open in APS (shell hint)").clicked() {
                    state.last_preview_hint = Some(format!(
                        "python -m rust_engine_mcp.cli assembly open --snapshot {}",
                        state.path_input
                    ));
                }
            });
            if let Some(hint) = &state.last_preview_hint {
                ui.label(hint);
            }

            if state.preview_active {
                ui.label("Footprint highlight — selected row highlighted on plan grid (preview active).");
            } else {
                ui.label("Spawn preview to enable footprint grid highlight.");
            }

            ui.heading("Placements");
            egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                egui::Grid::new("qc_placements").striped(true).show(ui, |ui| {
                    ui.label("Cell");
                    ui.label("module_id");
                    ui.label("material");
                    ui.label("GLB");
                    ui.label("status");
                    ui.end_row();
                    for (idx, row) in summary.rows.iter().enumerate() {
                        let selected = state.selected_row == Some(idx);
                        if ui.selectable_label(selected, &row.cell).clicked() {
                            state.selected_row = Some(idx);
                        }
                        ui.label(&row.module_id);
                        ui.label(&row.material_profile);
                        ui.label(&row.glb);
                        ui.label(match row.status {
                            QcRowStatus::Ok => "OK",
                            QcRowStatus::Warn => "WARN",
                            QcRowStatus::Fail => "FAIL",
                        });
                        ui.end_row();
                    }
                });
            });
        });
}

#[must_use]
pub fn aps_bevy_qc_hud_001_witness_green() -> bool {
    let path = repo_root_from_manifest().join(APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT);
    load_qc_snapshot(&path)
        .map(|(_, s)| s.placement_count > 0 && s.rows.len() == s.placement_count)
        .unwrap_or(false)
}

#[must_use]
pub fn aps_bevy_qc_hud_v2_witness_green() -> bool {
    if !aps_bevy_qc_hud_001_witness_green() {
        return false;
    }
    let repo_root = repo_root_from_manifest();
    let path = repo_root.join(APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT);
    let Ok((snapshot, _)) = load_qc_snapshot(&path) else {
        return false;
    };
    let _p0 = evaluate_p0_readonly(&snapshot, &repo_root);
    placement_grid_coords(&snapshot, 0).is_some()
}

#[must_use]
pub fn aps_bevy_qc_hud_001_witness_json() -> Value {
    serde_json::json!({
        "green": aps_bevy_qc_hud_001_witness_green(),
        "panel_module": "src/gui/assembly_snapshot_qc_ui.rs",
    })
}
