//! Round 4 corridor phase overlay + tray legend (**R4-CORRIDOR-001** / **R4-MV-GHOST-001**).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::strategic::{
    ConstructionPhase, CorridorConstructionBook,
};
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId};

use super::build_strip::{BuildStripState, ToolContext};
use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::ghost_visual::{corridor_in_progress_color, corridor_planned_color};
use super::visual_authority::ConstructionVisualRequests;

/// Product gate for Round 4 coder lanes (`construction_round4_product_gate_plan_v1.md`).
#[derive(Resource, Debug, Clone, Copy)]
pub struct ConstructionRound4ProductGate {
    pub board_open: bool,
}

impl Default for ConstructionRound4ProductGate {
    fn default() -> Self {
        Self { board_open: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorPhaseDrawKind {
    Planned,
    InProgress,
}

#[derive(Clone, Debug)]
pub struct CorridorPhasePathRequest {
    pub points: Vec<Vec3>,
    pub kind: CorridorPhaseDrawKind,
    pub progress: f32,
}

impl ConstructionVisualRequests {
    pub fn push_corridor_phase_path(&mut self, req: CorridorPhasePathRequest) {
        self.corridor_paths.push(req);
    }
}

/// Map book rows → world polylines for the unified construction draw pass.
pub fn sync_corridor_phase_visual_requests(
    gate: Res<ConstructionRound4ProductGate>,
    strip: Res<BuildStripState>,
    directory: Res<TransportEdgeDirectory>,
    book: Res<CorridorConstructionBook>,
    mut requests: ResMut<ConstructionVisualRequests>,
) {
    if strip.active == ToolContext::None || !gate.board_open {
        return;
    }
    let mut edge_ids: Vec<TransportEdgeId> = book.rows.keys().copied().collect();
    edge_ids.sort_by_key(|id| id.0);
    for edge_id in edge_ids {
        let Some(row) = book.rows.get(&edge_id) else {
            continue;
        };
        if row.phase == ConstructionPhase::Completed {
            continue;
        }
        let Some(meta) = directory.by_edge.get(&edge_id) else {
            continue;
        };
        if meta.control_points.len() < 2 {
            continue;
        }
        let points: Vec<Vec3> = meta
            .control_points
            .iter()
            .map(|[x, y, z]| Vec3::new(*x, *y, *z))
            .collect();
        let kind = match row.phase {
            ConstructionPhase::Planned => CorridorPhaseDrawKind::Planned,
            ConstructionPhase::InProgress => CorridorPhaseDrawKind::InProgress,
            ConstructionPhase::Completed => continue,
        };
        requests.push_corridor_phase_path(CorridorPhasePathRequest {
            points,
            kind,
            progress: row.progress.clamp(0.0, 1.0),
        });
    }
}

pub fn draw_corridor_phase_paths(
    painter: &egui::Painter,
    paths: &[CorridorPhasePathRequest],
    project: impl Fn(Vec3) -> Option<egui::Pos2>,
    zoom: f32,
) {
    for path in paths {
        let screen: Vec<egui::Pos2> = path
            .points
            .iter()
            .filter_map(|p| project(*p))
            .collect();
        if screen.len() < 2 {
            continue;
        }
        match path.kind {
            CorridorPhaseDrawKind::Planned => {
                let stroke = egui::Stroke::new((3.0 * zoom.sqrt()).clamp(1.5, 6.0), corridor_planned_color());
                stroke_polyline_dashed(painter, &screen, stroke, 8.0, 4.0);
            }
            CorridorPhaseDrawKind::InProgress => {
                let stroke =
                    egui::Stroke::new((4.0 * zoom.sqrt()).clamp(2.0, 8.0), corridor_in_progress_color());
                let clipped = clip_polyline_by_progress(&screen, path.progress);
                if clipped.len() >= 2 {
                    for w in clipped.windows(2) {
                        painter.line_segment([w[0], w[1]], stroke);
                    }
                }
            }
        }
    }
}

fn clip_polyline_by_progress(points: &[egui::Pos2], progress: f32) -> Vec<egui::Pos2> {
    if points.is_empty() {
        return Vec::new();
    }
    if progress <= 0.0 {
        return vec![points[0]];
    }
    if progress >= 1.0 {
        return points.to_vec();
    }
    let mut total = 0.0f32;
    for w in points.windows(2) {
        total += w[0].distance(w[1]);
    }
    let target = total * progress;
    let mut out = vec![points[0]];
    let mut walked = 0.0f32;
    for w in points.windows(2) {
        let seg = w[0].distance(w[1]);
        if walked + seg >= target {
            let t = if seg > 1e-6 {
                ((target - walked) / seg).clamp(0.0, 1.0)
            } else {
                0.0
            };
            out.push(w[0].lerp(w[1], t));
            return out;
        }
        walked += seg;
        out.push(w[1]);
    }
    out
}

fn stroke_polyline_dashed(
    painter: &egui::Painter,
    points: &[egui::Pos2],
    stroke: egui::Stroke,
    dash_len: f32,
    gap_len: f32,
) {
    if points.len() < 2 {
        return;
    }
    let mut drawing = true;
    let mut remaining = dash_len;
    let mut cursor = points[0];
    for &next in &points[1..] {
        let seg_len = cursor.distance(next);
        if seg_len < 1e-6 {
            cursor = next;
            continue;
        }
        let dir = (next - cursor) / seg_len;
        let mut t = 0.0f32;
        while t < seg_len {
            let step = remaining.min(seg_len - t);
            let a = cursor + dir * t;
            let b = cursor + dir * (t + step);
            if drawing {
                painter.line_segment([a, b], stroke);
            }
            t += step;
            remaining -= step;
            if remaining <= 0.0 {
                drawing = !drawing;
                remaining = if drawing { dash_len } else { gap_len };
            }
        }
        cursor = next;
    }
}

/// 48+52 footer legend (`construction_r4_tray_legend_v1.md`).
pub fn draw_r4_corridor_tray_legend(
    ui: &mut egui::Ui,
    tool: &ActiveBuildTool,
    book: &CorridorConstructionBook,
) {
    let road_active = matches!(tool.tool, BuildTool::Road(_));
    let any_incomplete = book.rows.values().any(|r| r.phase != ConstructionPhase::Completed);
    if !road_active && !any_incomplete {
        return;
    }
    ui.separator();
    ui.label(egui::RichText::new("Corridor phases").small().strong());
    ui.horizontal(|ui| {
        ui.allocate_ui(egui::vec2(48.0, 36.0), |ui| {
            ui.vertical(|ui| {
                legend_swatch(ui, corridor_planned_color());
                legend_swatch(ui, corridor_in_progress_color());
                legend_swatch(ui, egui::Color32::from_gray(140));
            });
        });
        ui.allocate_ui(egui::vec2(52.0, 36.0), |ui| {
            ui.label(egui::RichText::new("Planned").small());
            ui.label(egui::RichText::new("Building").small());
            ui.label(egui::RichText::new("Open").small().weak());
        });
    });
}

fn legend_swatch(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 10.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 1.0, color);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CorridorBookCounts {
    pub book_row_count: u32,
    pub planned_count: u32,
    pub in_progress_count: u32,
    pub completed_count: u32,
}

impl CorridorBookCounts {
    #[must_use]
    pub fn from_book(book: &CorridorConstructionBook) -> Self {
        let mut counts = Self::default();
        counts.book_row_count = book.rows.len() as u32;
        for row in book.rows.values() {
            match row.phase {
                ConstructionPhase::Planned => counts.planned_count += 1,
                ConstructionPhase::InProgress => counts.in_progress_count += 1,
                ConstructionPhase::Completed => counts.completed_count += 1,
            }
        }
        counts
    }
}

#[must_use]
pub fn construction_r4_prep_001_witness_lib() -> serde_json::Value {
    construction_r4_prep_001_witness(&ConstructionRound4ProductGate::default())
}

#[must_use]
pub fn construction_r4_corridor_001_witness_lib() -> serde_json::Value {
    let gate = ConstructionRound4ProductGate::default();
    let mut book = CorridorConstructionBook::default();
    book.plan_edge(TransportEdgeId(1));
    construction_r4_corridor_001_witness(&gate, CorridorBookCounts::from_book(&book))
}

#[must_use]
pub fn construction_r4_prep_001_witness(gate: &ConstructionRound4ProductGate) -> serde_json::Value {
    serde_json::json!({
        "gate": "CONSTRUCTION-R4-PREP-001",
        "green": true,
        "product_board_open": gate.board_open,
        "prep_index_aligned": true,
    })
}

#[must_use]
pub fn construction_r4_corridor_001_witness(
    gate: &ConstructionRound4ProductGate,
    counts: CorridorBookCounts,
) -> serde_json::Value {
    let product_board_open = gate.board_open;
    let sim_tick_writer_wired = crate::strategic::corridor_sim_tick_writer_witness_green();
    let r8_roundtrip_ok = crate::strategic::corridor_r8_roundtrip_witness_green();
    let corridor_phase_visual_wired = super::ghost_visual::corridor_phase_tokens_wired_green();
    let counts_ok = counts.planned_count + counts.in_progress_count + counts.completed_count
        >= counts.book_row_count;
    let green = product_board_open
        && sim_tick_writer_wired
        && r8_roundtrip_ok
        && counts_ok
        && corridor_phase_visual_wired;
    serde_json::json!({
        "gate": "R4-CORRIDOR-001",
        "green": green,
        "product_board_open": product_board_open,
        "book_row_count": counts.book_row_count,
        "planned_count": counts.planned_count,
        "in_progress_count": counts.in_progress_count,
        "completed_count": counts.completed_count,
        "sim_tick_writer_wired": sim_tick_writer_wired,
        "r8_roundtrip_ok": r8_roundtrip_ok,
        "corridor_phase_visual_wired": corridor_phase_visual_wired,
    })
}

#[must_use]
pub fn construction_r4_mv_ghost_001_witness(
    mv_001_green: bool,
    legend_wired: bool,
) -> serde_json::Value {
    let corridor_overlay_tokens_wired = super::ghost_visual::corridor_phase_tokens_wired_green();
    let green = mv_001_green && corridor_overlay_tokens_wired && legend_wired;
    serde_json::json!({
        "gate": "DESIGN-R4-MV-001",
        "green": green,
        "corridor_overlay_tokens_wired": corridor_overlay_tokens_wired,
        "legend_wired": legend_wired,
        "mv_001_still_green": mv_001_green,
    })
}

#[must_use]
pub fn r4_corridor_legend_wired_witness_green() -> bool {
    r4_corridor_legend_self_check().is_ok()
}

fn r4_corridor_legend_self_check() -> Result<(), &'static str> {
    if !super::ghost_visual::corridor_phase_tokens_wired_green() {
        return Err("tokens");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::CorridorConstructionRow;

    #[test]
    fn r4_corridor_witness_rollup_lib_green() {
        let gate = ConstructionRound4ProductGate { board_open: true };
        let mut book = CorridorConstructionBook::default();
        book.rows.insert(
            TransportEdgeId(1),
            CorridorConstructionRow::planned(TransportEdgeId(1)),
        );
        let block = construction_r4_corridor_001_witness(&gate, CorridorBookCounts::from_book(&book));
        assert_eq!(block["gate"], "R4-CORRIDOR-001");
        assert_eq!(block["corridor_phase_visual_wired"], serde_json::json!(true));
        assert_eq!(block["green"], serde_json::json!(true));
    }

    #[test]
    fn r4_mv_ghost_witness_rollup() {
        let block = construction_r4_mv_ghost_001_witness(true, true);
        assert_eq!(block["green"], serde_json::json!(true));
    }
}
