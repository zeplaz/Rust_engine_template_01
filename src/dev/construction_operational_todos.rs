//! **CONSTRUCTION_OPERATIONAL_GREEN** — running-app gate (post Phase 2 P6–P8).
//!
//! Spec: [`super::construction_operational_gate.md`](super::construction_operational_gate.md)
//! **Not** Stage 5. Run before scaling Round 3 catalog/topology.

use bevy::prelude::{App, Resource};

use super::construction_live_todos::TodoStatus;
use super::stage5_live_todos::Stage5LiveTodo;

pub const CONSTRUCTION_OPERATIONAL_TODO_COUNT: usize = 8;

pub static CONSTRUCTION_OPERATIONAL_TODOS: &[Stage5LiveTodo] = &[
    Stage5LiveTodo {
        id: "CONSTRUCTION-OP-01",
        status: TodoStatus::Done,
        file: "src/construction/build_toolbox.rs",
        system: "ToolboxFunctional",
        goal: "Hierarchical toolbox; tool persists across commits (session).",
        runtime_check: "Sim: pick road → commit → still road tool without reopening panel.",
        failure_mode: "Tool resets to None every commit.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-OP-02",
        status: TodoStatus::Done,
        file: "src/construction/roads/",
        system: "RoadPathCommit",
        goal: "Road path LMB + Shift+LMB commit updates executed network / transport.",
        runtime_check: "Sim: 2+ points → commit → witness/proof shows segment enqueued+executed.",
        failure_mode: "Plans queue but world unchanged.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-OP-03",
        status: TodoStatus::Done,
        file: "src/construction/zones/",
        system: "ZonePaintFunctional",
        goal: "Zone drag paint + confirm spawns strategic Zone overlay.",
        runtime_check: "Sim: paint residential zone → confirm → Zone component exists.",
        failure_mode: "Zone paint queues CivilHousing sites.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-OP-04",
        status: TodoStatus::Done,
        file: "src/construction/residential_menu.rs",
        system: "BuildingPlacementFunctional",
        goal: "Structure pick → ghost → confirm → ConstructionSite entity.",
        runtime_check: "Sim: Duplex → confirm → site at footprint; intent panel operational fields only.",
        failure_mode: "Cannot place building end-to-end.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-OP-05",
        status: TodoStatus::Done,
        file: "src/construction/demolish.rs",
        system: "DemolishFunctional",
        goal: "Demolish pick → pending → confirm despawn.",
        runtime_check: "Sim: demolish existing site → entity gone.",
        failure_mode: "Demolish stub queues wrong pending type.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-OP-06",
        status: TodoStatus::Done,
        file: "src/construction/history.rs",
        system: "UndoFunctional",
        goal: "Ctrl+Z reverses last construction commit in sim.",
        runtime_check: "Sim: commit road → Ctrl+Z → segment/markers reverted.",
        failure_mode: "Undo no-op.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-OP-07",
        status: TodoStatus::Done,
        file: "debug_runs/construction_stage_live.json",
        system: "ConstructionProofJson",
        goal: "Running sim writes proof JSON with phase2 + round2 + operational boards.",
        runtime_check: "File exists after ~90 frames in sim; parseable JSON.",
        failure_mode: "Closure not machine-verifiable.",
    },
    Stage5LiveTodo {
        id: "CONSTRUCTION-OP-08",
        status: TodoStatus::Done,
        file: "src/construction/",
        system: "NoLegacyPaths",
        goal: "No `gui::build` placement, tile-road intent, or authority bypass in src/.",
        runtime_check: "`rg gui::build` / legacy road intent empty; invariants audit pass.",
        failure_mode: "Dual placement paths remain.",
    },
];

#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionOperationalWitness {
    pub toolbox: bool,
    pub road_commit: bool,
    pub zone_paint: bool,
    pub building_place: bool,
    pub demolish: bool,
    pub undo: bool,
    pub proof_json: bool,
    pub no_legacy: bool,
}

#[derive(Resource, Default)]
pub struct ConstructionOperationalTodoBoard {
    pub status: Vec<TodoStatus>,
}

impl ConstructionOperationalTodoBoard {
    pub fn sync_from_witness(&mut self, w: &ConstructionOperationalWitness) {
        let flags = [
            w.toolbox,
            w.road_commit,
            w.zone_paint,
            w.building_place,
            w.demolish,
            w.undo,
            w.proof_json,
            w.no_legacy,
        ];
        debug_assert_eq!(flags.len(), CONSTRUCTION_OPERATIONAL_TODO_COUNT);
        for (slot, ok) in self.status.iter_mut().zip(flags) {
            *slot = if ok { TodoStatus::Done } else { TodoStatus::Open };
        }
    }

    #[must_use]
    pub fn is_green(&self) -> bool {
        self.open_count() == 0
    }

    #[must_use]
    pub fn open_count(&self) -> usize {
        self.status.iter().filter(|s| **s == TodoStatus::Open).count()
    }
}

pub fn register_construction_operational_todo_hooks(app: &mut App) {
    app.init_resource::<ConstructionOperationalTodoBoard>()
        .init_resource::<ConstructionOperationalWitness>();
    let mut board = ConstructionOperationalTodoBoard::default();
    board.status = vec![TodoStatus::Open; CONSTRUCTION_OPERATIONAL_TODO_COUNT];
    app.insert_resource(board);
}

pub fn sync_construction_operational_board_from_witness(
    witness: &ConstructionOperationalWitness,
    board: &mut ConstructionOperationalTodoBoard,
) {
    board.sync_from_witness(witness);
}
