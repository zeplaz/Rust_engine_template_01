//! APS-BEVY-QC-HUD-001 — Bevy UI panel for assembly snapshot QC (Lane A′).

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use serde_json::Value;

use crate::construction::procedural::AssemblySnapshot;
use crate::gui::panel_presence::PanelPresence;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::ui_gates::in_simulation_or_editor;
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
        app.init_resource::<AssemblySnapshotQcUiState>().add_systems(
            Update,
            (
                sync_assembly_snapshot_qc_bevy,
                assembly_snapshot_qc_bevy_actions.after(sync_assembly_snapshot_qc_bevy),
            )
                .run_if(in_simulation_or_editor),
        );
    }
}

#[derive(Component)]
pub struct AssemblySnapshotQcBevyRoot;

#[derive(Component)]
struct AssemblySnapshotQcBevyBodyText;

#[derive(Component)]
struct AssemblySnapshotQcLoadBtn;

#[derive(Default)]
struct AssemblyQcStamp {
    key: u64,
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

fn qc_panel_body(state: &AssemblySnapshotQcUiState) -> String {
    let mut lines = vec![
        "Assembly snapshot QC".to_string(),
        format!("path: {}", state.path_input),
    ];
    if let Some(err) = &state.load_error {
        lines.push(format!("error: {err}"));
    }
    if let Some(sum) = &state.summary {
        lines.push(format!(
            "assembly {} · {} placements",
            sum.assembly_id, sum.placement_count
        ));
        for (i, row) in sum.rows.iter().take(16).enumerate() {
            let st = match row.status {
                QcRowStatus::Ok => "OK",
                QcRowStatus::Warn => "Warn",
                QcRowStatus::Fail => "Fail",
            };
            lines.push(format!(
                "{i}: {cell} {mod_id} [{st}] {mat}",
                cell = row.cell,
                mod_id = row.module_id,
                mat = row.material_profile,
            ));
        }
        if sum.rows.len() > 16 {
            lines.push(format!("… +{} more", sum.rows.len() - 16));
        }
    } else {
        lines.push("No snapshot loaded — press Load.".into());
    }
    if !state.p0_issues.is_empty() {
        lines.push("── P0 ──".into());
        for issue in state.p0_issues.iter().take(6) {
            lines.push(format!("· {}", issue.sentence));
        }
    }
    lines.join("\n")
}

fn sync_assembly_snapshot_qc_bevy(
    mut commands: Commands,
    state: Res<AssemblySnapshotQcUiState>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<AssemblySnapshotQcBevyRoot>>,
    mut body: Query<&mut Text, With<AssemblySnapshotQcBevyBodyText>>,
    mut stamp: Local<AssemblyQcStamp>,
) {
    presence.assembly_snapshot_qc_root = !roots.is_empty();
    if !state.visible {
        for e in &roots {
            commands.entity(e).despawn();
        }
        presence.assembly_snapshot_qc_root = false;
        stamp.key = 0;
        return;
    }

    let text = qc_panel_body(state.as_ref());
    let mut key = text.len() as u64;
    for b in text.bytes() {
        key = key.wrapping_mul(0x9E37_79B9).wrapping_add(b as u64);
    }

    if roots.is_empty() {
        let tf = TextFont::from_font_size(11.0).with_font(mono.0.clone());
        commands
            .spawn((
                AssemblySnapshotQcBevyRoot,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(48.0),
                    left: Val::Px(16.0),
                    width: Val::Px(420.0),
                    max_height: Val::Px(360.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::ZERO,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(palette.bevy_hud_panel_fill()),
                BorderColor::all(palette.bevy_wire_magenta()),
                FocusPolicy::Block,
                ZIndex(953),
            ))
            .with_children(|p| {
                p.spawn((
                    AssemblySnapshotQcBevyBodyText,
                    Text::new(text.clone()),
                    tf.clone(),
                    TextColor(palette.bevy_primary_text()),
                ));
                p.spawn((
                    AssemblySnapshotQcLoadBtn,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(palette.bevy_button_idle()),
                    BorderColor::all(palette.bevy_wire_magenta()),
                    FocusPolicy::Block,
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("Load"),
                        tf,
                        TextColor(palette.bevy_primary_text()),
                    ));
                });
            });
        stamp.key = key;
        presence.assembly_snapshot_qc_root = true;
        return;
    }

    if stamp.key != key {
        for mut t in &mut body {
            *t = Text::new(text.clone());
        }
        stamp.key = key;
    }
    presence.assembly_snapshot_qc_root = true;
}

fn assembly_snapshot_qc_bevy_actions(
    q: Query<&Interaction, (Changed<Interaction>, With<AssemblySnapshotQcLoadBtn>)>,
    mut state: ResMut<AssemblySnapshotQcUiState>,
) {
    for interaction in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let path = repo_root_from_manifest().join(state.path_input.replace('\\', "/"));
        apply_load(state.as_mut(), path);
    }
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
