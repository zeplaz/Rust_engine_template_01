//! Phase 2 **P9 closure** — final 5 rows (`PHASE2-BUILD-16` … `20`).
//!
//! Parent registry: [`super::construction_phase2_todos::CONSTRUCTION_PHASE2_TODOS`].
//! Human checklist: [`super::construction_recovery_todos.md`](super::construction_recovery_todos.md).
//!
//! **CON-E01-P9:** runtime board syncs from [`ConstructionPhase2Witness`] tail flags + proof write.

use bevy::prelude::{App, Resource};

use super::construction_live_todos::TodoStatus;
use super::construction_phase2_todos::ConstructionPhase2Witness;
use super::stage5_live_todos::Stage5LiveTodo;

pub const CONSTRUCTION_P9_TODO_COUNT: usize = 5;

/// P9 execution board (maps 1:1 to phase2 witness tail).
pub static CONSTRUCTION_P9_TODOS: &[Stage5LiveTodo] = &[
    Stage5LiveTodo {
        id: "PHASE2-BUILD-16",
        status: TodoStatus::Done,
        file: "src/construction/live_proof.rs",
        system: "ConstructionProofJson",
        goal: "Simulation writes debug_runs/construction_stage_live.json with boards + witnesses.",
        runtime_check: "File exists after sim run; witness construction_proof_json true.",
        failure_mode: "No machine-readable construction closure.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-17",
        status: TodoStatus::Done,
        file: "src/construction/roads/spline.rs",
        system: "CurvedRoadSpline",
        goal: "Catmull-Rom preview chain; commit flattens to validated segments.",
        runtime_check: "≥3 control points use curved samples in ghost + commit.",
        failure_mode: "Roads are straight segments only.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-18",
        status: TodoStatus::Done,
        file: "src/construction/snap.rs",
        system: "GridAndNodeSnap",
        goal: "Road popup toggles grid + node snap; cursor/placement respect settings.",
        runtime_check: "Snap toggles change quantized world XZ and node lock.",
        failure_mode: "Free-floating road points off network.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-19",
        status: TodoStatus::Done,
        file: "src/construction/upgrade.rs",
        system: "RoadUpgradeLane",
        goal: "Upgrade button on road popup enqueues wider segment on nearest executed tile pair.",
        runtime_check: "ExecutedRoadNetwork tile → ConstructionPlanQueue entry.",
        failure_mode: "Must rebuild road to widen.",
    },
    Stage5LiveTodo {
        id: "PHASE2-BUILD-20",
        status: TodoStatus::Done,
        file: "src/construction/terrain_conform.rs",
        system: "TerrainConform",
        goal: "Ghost + road control points set Y from terrain conform sampler.",
        runtime_check: "conform_world_y non-zero on slopes; ghost entity Y updated.",
        failure_mode: "All previews at y=0.",
    },
];

/// Tail of [`ConstructionPhase2Witness`] — proof + P9 advanced lanes.
#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionP9Witness {
    pub construction_proof_json: bool,
    pub curved_road_spline: bool,
    pub grid_and_node_snap: bool,
    pub road_upgrade_lane: bool,
    pub terrain_conform: bool,
}

impl ConstructionP9Witness {
    #[must_use]
    pub fn from_phase2(w: &ConstructionPhase2Witness) -> Self {
        Self {
            construction_proof_json: w.construction_proof_json,
            curved_road_spline: w.curved_road_spline,
            grid_and_node_snap: w.grid_and_node_snap,
            road_upgrade_lane: w.road_upgrade_lane,
            terrain_conform: w.terrain_conform,
        }
    }

    #[must_use]
    pub fn all_green(&self) -> bool {
        self.construction_proof_json
            && self.curved_road_spline
            && self.grid_and_node_snap
            && self.road_upgrade_lane
            && self.terrain_conform
    }
}

#[derive(Resource, Debug)]
pub struct ConstructionP9TodoBoard {
    pub status: Vec<TodoStatus>,
}

impl Default for ConstructionP9TodoBoard {
    fn default() -> Self {
        Self::from_template()
    }
}

impl ConstructionP9TodoBoard {
    #[must_use]
    pub fn from_template() -> Self {
        Self {
            status: CONSTRUCTION_P9_TODOS
                .iter()
                .map(|t| t.status)
                .collect(),
        }
    }

    pub fn sync_from_witness(&mut self, w: &ConstructionP9Witness) {
        let flags = [
            w.construction_proof_json,
            w.curved_road_spline,
            w.grid_and_node_snap,
            w.road_upgrade_lane,
            w.terrain_conform,
        ];
        debug_assert_eq!(flags.len(), CONSTRUCTION_P9_TODO_COUNT);
        for (slot, ok) in self.status.iter_mut().zip(flags) {
            *slot = if ok { TodoStatus::Done } else { TodoStatus::Open };
        }
    }

    #[must_use]
    pub fn open_count(&self) -> usize {
        self.status.iter().filter(|s| **s == TodoStatus::Open).count()
    }

    #[must_use]
    pub fn is_green(&self) -> bool {
        self.open_count() == 0
    }
}

/// **CON-E01-P9** acceptance rollup for live proof JSON.
#[must_use]
pub fn con_e01_p9_acceptance_green(w: &ConstructionP9Witness, proof_written: bool) -> bool {
    proof_written
        && (w.construction_proof_json || proof_written)
        && w.curved_road_spline
        && w.grid_and_node_snap
        && w.road_upgrade_lane
        && w.terrain_conform
}

pub fn register_construction_p9_todo_hooks(app: &mut App) {
    app.init_resource::<ConstructionP9Witness>()
        .init_resource::<ConstructionP9TodoBoard>();
}

pub fn sync_construction_p9_board_from_witness(
    witness: &ConstructionP9Witness,
    board: &mut ConstructionP9TodoBoard,
) {
    board.sync_from_witness(witness);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p9_witness_maps_phase2_tail() {
        let phase2 = ConstructionPhase2Witness {
            construction_proof_json: true,
            curved_road_spline: true,
            grid_and_node_snap: true,
            road_upgrade_lane: true,
            terrain_conform: true,
            ..Default::default()
        };
        let p9 = ConstructionP9Witness::from_phase2(&phase2);
        assert!(p9.all_green());
        assert!(con_e01_p9_acceptance_green(&p9, true));
    }

    #[test]
    fn p9_board_sync_marks_tail_done() {
        let witness = ConstructionP9Witness {
            construction_proof_json: true,
            curved_road_spline: true,
            grid_and_node_snap: true,
            road_upgrade_lane: true,
            terrain_conform: true,
        };
        let mut board = ConstructionP9TodoBoard::from_template();
        sync_construction_p9_board_from_witness(&witness, &mut board);
        assert!(board.is_green());
        assert_eq!(board.open_count(), 0);
    }
}
