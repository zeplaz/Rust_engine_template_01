//! Round 4 corridor phase overlay + tray legend (**R4-CORRIDOR-001** / **R4-MV-GHOST-001**).

use bevy::prelude::*;

use crate::strategic::{ConstructionPhase, CorridorConstructionBook};
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId};

use super::build_strip::{BuildStripState, ToolContext};
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

/// Retired egui corridor path paint — Bevy network mesh owns live strokes.
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn draw_corridor_phase_paths() {}

/// 48+52 footer legend (`construction_r4_tray_legend_v1.md`).
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn draw_r4_corridor_tray_legend() {}












#[must_use]
pub fn r4_corridor_legend_wired_witness_green() -> bool {
    r4_corridor_legend_self_check().is_ok()
}

/// **CON-R4-TAIL-001** — public alias for phase6 witness bundle.
#[must_use]
pub fn construction_r4_corridor_legend_wired_witness_green() -> bool {
    r4_corridor_legend_wired_witness_green()
}

fn r4_corridor_legend_self_check() -> Result<(), &'static str> {
    if !super::ghost_visual::corridor_phase_tokens_wired_green() {
        return Err("tokens");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    
    




}
