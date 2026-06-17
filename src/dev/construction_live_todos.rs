//! **Construction stage** live board — parallel to Stage 5 visual spine, not gated on FULL_APP.
//!
//! Registry: [`CONSTRUCTION_TODOS`]. Witness: [`crate::construction::ConstructionStageWitness`].
//! Spec: [`super::construction_recovery_todos.md`](super::construction_recovery_todos.md).

use bevy::log::info;
use bevy::prelude::{App, Resource, World};
use std::collections::HashSet;

use crate::construction::{ConstructionStageWitness, CONSTRUCTION_TODO_COUNT};

/// Reuse Stage 5 status enum for operator familiarity (not coupled to Stage 5 board).
pub use super::stage5_live_todos::TodoStatus;

/// One construction recovery row.
#[derive(Clone, Copy, Debug)]
pub struct ConstructionLiveTodo {
    pub id: &'static str,
    pub status: TodoStatus,
    pub file: &'static str,
    pub goal: &'static str,
    pub runtime_check: &'static str,
}

/// Authoritative construction lane registry (do **not** append to `STAGE5_TODOS`).
pub static CONSTRUCTION_TODOS: &[ConstructionLiveTodo] = &[
    ConstructionLiveTodo {
        id: "BUILD-P0-01",
        status: TodoStatus::Done,
        file: "src/construction/build_toolbox.rs",
        goal: "Persistent left build toolbox replaces `;` as primary tool UX.",
        runtime_check: "Simulation: Construction side panel visible; tool click sets active mode.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P0-02",
        status: TodoStatus::Done,
        file: "src/gui/input_bindings.rs",
        goal: "Semicolon cycle demoted; help points to toolbox.",
        runtime_check: "Default hints reference toolbox; `;` optional.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P0-03",
        status: TodoStatus::Done,
        file: "src/construction/build_tool_authority.rs",
        goal: "Single `ActiveBuildTool` resource; strip mirrors authority.",
        runtime_check: "Witness `active_build_tool` true; one write path from toolbox.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P0-04",
        status: TodoStatus::Done,
        file: "src/construction/build_mode.rs",
        goal: "BuildMode state machine; Escape cancels ghost/draft.",
        runtime_check: "Witness `build_mode` + escape clears tool/ghost.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P1-01",
        status: TodoStatus::Done,
        file: "src/construction/build_ghost.rs",
        goal: "Ghost systems do not commit gameplay state.",
        runtime_check: "Witness `ghost_commit_isolated`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P1-02",
        status: TodoStatus::Done,
        file: "src/construction/build_validation.rs",
        goal: "Shared GhostValid / placement preview for roads + buildings.",
        runtime_check: "Witness `shared_ghost_valid`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P1-03",
        status: TodoStatus::Done,
        file: "src/construction/residential_menu.rs",
        goal: "Residential submenu placeholder → building tool.",
        runtime_check: "Witness `residential_menu`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P2-01",
        status: TodoStatus::Done,
        file: "src/construction/roads/mod.rs",
        goal: "ActiveRoadPlacement control points resource.",
        runtime_check: "Witness `road_control_points`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P2-02",
        status: TodoStatus::Done,
        file: "src/construction/roads/input.rs",
        goal: "LMB add / RMB undo / Shift+LMB commit path input.",
        runtime_check: "Witness `road_input_model`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P2-03",
        status: TodoStatus::Done,
        file: "src/construction/roads/pathing.rs",
        goal: "Control points → segment previews.",
        runtime_check: "Witness `road_segment_preview`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P2-04",
        status: TodoStatus::Done,
        file: "src/construction/roads/ghost.rs",
        goal: "Road polyline ghost on map.",
        runtime_check: "Witness `road_ghost_draw`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P2-05",
        status: TodoStatus::Done,
        file: "src/construction/roads/popup.rs",
        goal: "Road tool popup Build/Cancel.",
        runtime_check: "Witness `road_popup`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P3-01",
        status: TodoStatus::Done,
        file: "src/construction/construction_pipeline.rs",
        goal: "Only commit systems mutate transport/sites.",
        runtime_check: "Witness `commit_funnel_audited`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P3-02",
        status: TodoStatus::Done,
        file: "src/construction/roads/commit.rs",
        goal: "Road commit from validated segments.",
        runtime_check: "Witness `road_commit_from_segments`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P3-03",
        status: TodoStatus::Done,
        file: "src/construction/",
        goal: "End-to-end road slice test.",
        runtime_check: "Witness `road_e2e_test`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P4-01",
        status: TodoStatus::Done,
        file: "src/construction/rail/",
        goal: "Rail path pipeline mirrors road.",
        runtime_check: "Witness `rail_pipeline`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P4-02",
        status: TodoStatus::Done,
        file: "src/construction/build_tool_authority.rs",
        goal: "Demolish tool stub.",
        runtime_check: "Witness `demolish_tool`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P4-03",
        status: TodoStatus::Done,
        file: "src/construction/zones/",
        goal: "Zone paint: LMB/Alt-drag tiles → Shift+LMB pending queue (no spawn).",
        runtime_check: "Witness `zone_paint`.",
    },
    ConstructionLiveTodo {
        id: "BUILD-P5-01",
        status: TodoStatus::Done,
        file: "src/construction/",
        goal: "Construction gameplay tree under src/construction/ (gui/build shim only).",
        runtime_check: "Witness `module_split`.",
    },
];

#[derive(Resource, Debug)]
pub struct ConstructionLiveTodoBoard {
    pub status: Vec<TodoStatus>,
}

impl Default for ConstructionLiveTodoBoard {
    fn default() -> Self {
        Self::from_template()
    }
}

impl ConstructionLiveTodoBoard {
    #[must_use]
    pub fn from_template() -> Self {
        Self {
            status: CONSTRUCTION_TODOS.iter().map(|t| t.status).collect(),
        }
    }

    pub fn sync_from_witness(&mut self, w: &ConstructionStageWitness) {
        let flags = [
            w.toolbox_panel,
            w.semicolon_demoted_in_help,
            w.active_build_tool,
            w.build_mode,
            w.ghost_commit_isolated,
            w.shared_ghost_valid,
            w.residential_menu,
            w.road_control_points,
            w.road_input_model,
            w.road_segment_preview,
            w.road_ghost_draw,
            w.road_popup,
            w.commit_funnel_audited,
            w.road_commit_from_segments,
            w.road_e2e_test,
            w.rail_pipeline,
            w.demolish_tool,
            w.zone_paint,
            w.module_split,
        ];
        debug_assert_eq!(flags.len(), CONSTRUCTION_TODO_COUNT);
        for (slot, ok) in self.status.iter_mut().zip(flags) {
            *slot = if ok { TodoStatus::Done } else { TodoStatus::Open };
        }
    }
}

/// Log each construction board "green" line once per session (witness sync runs every frame).
#[derive(Resource, Default)]
pub struct ConstructionBoardGreenLogGate {
    keys: HashSet<&'static str>,
}

impl ConstructionBoardGreenLogGate {
    pub fn log_once(&mut self, key: &'static str, message: &str) {
        if self.keys.insert(key) {
            info!("{message}");
        }
    }
}

/// Register construction board (parallel lane — **not** Stage 5).
pub fn register_construction_todo_runtime_hooks(app: &mut App) {
    app.init_resource::<ConstructionLiveTodoBoard>()
        .init_resource::<ConstructionStageWitness>()
        .init_resource::<ConstructionBoardGreenLogGate>();
    super::construction_p9_todos::register_construction_p9_todo_hooks(app);
}

/// Sync board from witness each simulation frame.
pub fn sync_construction_live_todo_board(world: &mut World) {
    let witness = world
        .get_resource::<ConstructionStageWitness>()
        .cloned()
        .unwrap_or_default();
    let Some(mut board) = world.get_resource_mut::<ConstructionLiveTodoBoard>() else {
        return;
    };
    board.sync_from_witness(&witness);
    let done = board.status.iter().filter(|s| **s == TodoStatus::Done).count();
    if done == CONSTRUCTION_TODOS.len() {
        if let Some(mut gate) = world.get_resource_mut::<ConstructionBoardGreenLogGate>() {
            gate.log_once(
                "construction_stage",
                &format!(
                    "CONSTRUCTION_STAGE_COMPLETE done={done}/{}",
                    CONSTRUCTION_TODOS.len()
                ),
            );
        }
    }
}
