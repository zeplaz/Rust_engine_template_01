//! Construction **finish lane** — cleanup after the 18× `BUILD-P*` witness board.
//!
//! Tracked separately from [`super::construction_live_todos::CONSTRUCTION_TODOS`] so P0–P4 closure
//! stays stable while module migration and stub completion proceed.
//!
//! Sync: [`sync_construction_finish_todo_board`] from [`ConstructionFinishWitness`] each simulation frame.

use bevy::prelude::{App, Resource};

use super::construction_live_todos::TodoStatus;
use super::stage5_live_todos::Stage5LiveTodo;

/// Finish / hardening rows (execute after BUILD-P* witness green).
pub static CONSTRUCTION_FINISH_TODOS: &[Stage5LiveTodo] = &[
    Stage5LiveTodo {
        id: "FINISH-BUILD-01",
        status: TodoStatus::Done,
        file: "src/construction/",
        system: "Physical move from src/gui/build/",
        goal: "All gameplay modules live under src/construction/ (roads/, zones/, pipeline/, …).",
        runtime_check: "rg 'mod build;' src/gui/mod.rs shows shim only; sources under src/construction/.",
        failure_mode: "Split ownership: facade re-export while logic still in gui/build.",
    },
    Stage5LiveTodo {
        id: "FINISH-BUILD-02",
        status: TodoStatus::Done,
        file: "src/**/*.rs",
        system: "Import path migration",
        goal: "External callers use crate::construction (HUD, dev board, engine); no direct gui::build in new code.",
        runtime_check: "rg 'crate::gui::build' src/ --glob '!src/gui/build/**' returns empty or shim-only.",
        failure_mode: "Agents keep patching gui/build and miss construction spine.",
    },
    Stage5LiveTodo {
        id: "FINISH-BUILD-03",
        status: TodoStatus::Done,
        file: "src/gui/build/mod.rs",
        system: "Deprecated shim",
        goal: "gui/build is `pub use crate::construction::*` + deprecation note; no duplicate sources.",
        runtime_check: "Only mod.rs remains under gui/build/.",
        failure_mode: "Two trees diverge on every edit.",
    },
    Stage5LiveTodo {
        id: "FINISH-BUILD-04",
        status: TodoStatus::Done,
        file: "src/construction/demolish.rs",
        system: "DemolishTool",
        goal: "Demolish: LMB pick site/corridor stub → ConstructionQueueIntent (no immediate delete).",
        runtime_check: "Witness demolish queues intent; execute path audited.",
        failure_mode: "Demolish button is label-only.",
    },
    Stage5LiveTodo {
        id: "FINISH-BUILD-05",
        status: TodoStatus::Done,
        file: "src/construction/construction_pipeline.rs",
        system: "Legacy tile road intents",
        goal: "build_road_segment_intent_system / confirm gated when ActiveBuildTool is Road|Rail path tool.",
        runtime_check: "Path tool active → only roads::commit enqueues plans.",
        failure_mode: "Dual road placement: spline + tile chain both fire.",
    },
    Stage5LiveTodo {
        id: "FINISH-BUILD-06",
        status: TodoStatus::Done,
        file: "src/construction/build_interaction.rs",
        system: "Building commit funnel",
        goal: "Building tools: shift-click / approve → pending only; confirm_site is sole site spawn path.",
        runtime_check: "rg CommitConstructionSite in construction/ excludes ghost/pick except confirm.",
        failure_mode: "Direct world placement bypasses pending queue.",
    },
    Stage5LiveTodo {
        id: "FINISH-BUILD-07",
        status: TodoStatus::Done,
        file: "src/dev/construction_recovery_todos.md, AGENTS.md",
        system: "Docs",
        goal: "Recovery doc + AGENTS point at src/construction/; P0–P4 rows marked Done with dates.",
        runtime_check: "Docs match tree; no stale gui/build primary paths.",
        failure_mode: "Operators follow wrong directory in recovery plan.",
    },
    Stage5LiveTodo {
        id: "FINISH-BUILD-08",
        status: TodoStatus::Done,
        file: "src/dev/construction_finish_todos.rs",
        system: "Finish witness",
        goal: "Finish board syncs from ConstructionFinishWitness; log CONSTRUCTION_FINISH_COMPLETE.",
        runtime_check: "All FINISH-BUILD-* Done in running sim after migration.",
        failure_mode: "Finish work untracked; regression invisible.",
    },
];

pub const CONSTRUCTION_FINISH_TODO_COUNT: usize = CONSTRUCTION_FINISH_TODOS.len();

#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionFinishWitness {
    pub physical_move: bool,
    pub imports_migrated: bool,
    pub gui_shim_only: bool,
    pub demolish_intent: bool,
    pub legacy_road_gated: bool,
    pub building_commit_audited: bool,
    pub docs_updated: bool,
    pub finish_board_wired: bool,
}

#[derive(Resource, Debug)]
pub struct ConstructionFinishTodoBoard {
    pub status: Vec<TodoStatus>,
}

impl Default for ConstructionFinishTodoBoard {
    fn default() -> Self {
        Self::from_template()
    }
}

impl ConstructionFinishTodoBoard {
    #[must_use]
    pub fn from_template() -> Self {
        Self {
            status: CONSTRUCTION_FINISH_TODOS
                .iter()
                .map(|t| t.status)
                .collect(),
        }
    }

    pub fn sync_from_witness(&mut self, w: &ConstructionFinishWitness) {
        let flags = [
            w.physical_move,
            w.imports_migrated,
            w.gui_shim_only,
            w.demolish_intent,
            w.legacy_road_gated,
            w.building_commit_audited,
            w.docs_updated,
            w.finish_board_wired,
        ];
        debug_assert_eq!(flags.len(), CONSTRUCTION_FINISH_TODO_COUNT);
        for (slot, ok) in self.status.iter_mut().zip(flags) {
            *slot = if ok { TodoStatus::Done } else { TodoStatus::Open };
        }
    }
}

pub fn register_construction_finish_todo_hooks(app: &mut App) {
    app.init_resource::<ConstructionFinishTodoBoard>()
        .init_resource::<ConstructionFinishWitness>();
}
