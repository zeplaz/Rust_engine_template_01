//! Stage 5 **finish lane** — UX / world-gen / performance items beyond the 13 readiness closure rows.
//!
//! Tracked separately from [`super::stage5_live_todos::STAGE5_TODOS`] so FULL_APP predicate closure stays
//! scoped to visual-spine contracts. These rows are part of the same operator finish board when
//! [`Stage5FinishTodoBoard`] is initialized alongside [`super::stage5_live_todos::Stage5LiveTodoBoard`].
//!
//! [`sync_stage5_finish_todo_board`] runs each FULL_APP readiness eval (see [`super::stage5_live_todos::hook_post_readiness_evaluate`]):
//! **`FINISH-UX-06`** closes after a sustained streak of calm frames + map-fit agreement (predicate below).

use bevy::log::info;
use bevy::prelude::{Resource, World};

use crate::engine::UxFrameSpikeGuard;
use crate::gui::hud::frame_budget_diagnostics::FrameBudgetDiagnostics;
use crate::gui::MapFitValidationLog;

use super::stage5_live_todos::{Stage5LiveTodo, TodoStatus};

/// UX + world-gen + shell finish items (execute after / parallel to readiness P0–P4).
pub static STAGE5_FINISH_TODOS: &[Stage5LiveTodo] = &[
    Stage5LiveTodo {
        id: "FINISH-UX-01",
        status: TodoStatus::Done,
        file: "src/render/visual_agreement.rs, src/render/vt_ci_matrix.rs",
        system: "VisualAgreementFrame, VtCiMatrixLiveReport",
        goal: "VT-4 mismatch_count resets each frame; overlay hash vs full sim snapshot (not view-culled tactical).",
        runtime_check: "No monotonic VT-4 mismatch_count growth; READINESS_EVAL_END vt4=true in live app.",
        failure_mode: "False red readiness + lag from ever-growing mismatch_count (402+ per session).",
    },
    Stage5LiveTodo {
        id: "FINISH-UX-02",
        status: TodoStatus::Done,
        file: "src/terrain/generation/world_generator_enhanced.rs, src/gui/editor/world_gen_ui.rs",
        system: "WorldGenFlowState, GenerateWorldEvent",
        goal: "Preview/full generation works when operator opens generator (not silently ignored at Idle).",
        runtime_check: "Generate preview from NewWorldSetup or auto-promote Idle→NewWorldSetup on first generate click.",
        failure_mode: "Panel visible but preview never starts (warn: Ignored … use Main Menu).",
    },
    Stage5LiveTodo {
        id: "FINISH-UX-03",
        status: TodoStatus::Done,
        file: "src/gui/editor/world_gen_ui.rs, src/gui/editor/world_preview/",
        system: "OnEnter(FullReady), WorldGenUiState, WorldPreviewUiState",
        goal: "After full world generation, auto-dismiss world-gen panel + preview window until Escape/pause reopens.",
        runtime_check: "FullReady: world_gen visible=false, preview window_open=false; Enter world still available from pause.",
        failure_mode: "Chrome blocks map; operator thinks gen failed.",
    },
    Stage5LiveTodo {
        id: "FINISH-UX-04",
        status: TodoStatus::Done,
        file: "src/gui/in_game_pause_menu.rs (new), src/engine/states.rs",
        system: "InGameMenuState::Pause, Escape",
        goal: "Pause menu: Resume, Save, Load, World Generator, Main Menu, Quit.",
        runtime_check: "Escape in Simulation toggles pause overlay; items dispatch to correct BaseState/flow.",
        failure_mode: "No in-game escape path; only sim pause (P) conflated with UI pause.",
    },
    Stage5LiveTodo {
        id: "FINISH-UX-05",
        status: TodoStatus::Done,
        file: "src/gui/in_game_pause_menu.rs",
        system: "World Generator menu action + confirm modal",
        goal: "World Generator from pause warns and exits current world (Cancel / Exit); opens gen + preview on confirm.",
        runtime_check: "Confirm despawn + WorldGenFlowState::NewWorldSetup + panels visible.",
        failure_mode: "Silent world bleed or no confirm when leaving simulation.",
    },
    Stage5LiveTodo {
        id: "FINISH-UX-06",
        status: TodoStatus::Done,
        file: "src/gui/hud/frame_budget_diagnostics.rs, src/gui/map_view/",
        system: "FrameBudgetDiagnostics, map/preview widget layout",
        goal: "UI responsive under FULL_APP: sustained calm frames + map-fit agreement (see sync_stage5_finish_todo_board).",
        runtime_check: "readiness.live_finish_todo_board rows include FINISH-UX-06 Done after ~120 clean evals; stage5_full_app_live.json map_fit + frame_time under UxFrameSpikeGuard ceiling.",
        failure_mode: "Sluggish test/visual runs; preview/gen feel hung.",
    },
    Stage5LiveTodo {
        id: "FINISH-UX-07",
        status: TodoStatus::Done,
        file: "src/gui/editor/world_preview/preview_lifecycle.rs, render_raster.rs",
        system: "WorldPreviewLifecycle, ResolvedViewports",
        goal: "Preview raster only when lifecycle allows; valid viewport before GPU/CPU present.",
        runtime_check: "preview_readiness green before present; no raster when window closed after FullReady.",
        failure_mode: "Wasted GPU/CPU preview work while panels dismissed.",
    },
    Stage5LiveTodo {
        id: "FINISH-UX-08",
        status: TodoStatus::Done,
        file: "src/dev/stage5_live_todos.rs",
        system: "Stage5LiveTodoBoard regression reopen",
        goal: "Todo board reopens Done rows when readiness fails (no stale all_todos_done while vt4 red).",
        runtime_check: "STAGE5_TODO_BOARD_REGRESSION_REOPEN on passes=false; STAGE5_ACTIVE_TODO shows real next item.",
        failure_mode: "False green operator signal while violations active.",
    },
];

/// Consecutive readiness evaluations where [`finish_ux06_frame_predicate`] held (see [`FINISH_UX06_STREAK_DONE`]).
#[derive(Resource, Clone, Debug, Default)]
pub struct Stage5FinishUx06Streak {
    pub consecutive_good: u32,
    /// Last predicate failure (cleared when a good eval increments streak).
    pub last_blocker: Option<String>,
}

/// How many back-to-back **readiness evals** must see a clean UX-06 predicate before the row is [`TodoStatus::Done`].
pub const FINISH_UX06_STREAK_DONE: u32 = 120;

/// Optional board for FINISH-* rows (same length as [`STAGE5_FINISH_TODOS`]).
#[derive(Resource, Debug)]
pub struct Stage5FinishTodoBoard {
    pub status: Vec<TodoStatus>,
}

impl Default for Stage5FinishTodoBoard {
    fn default() -> Self {
        Self::from_template()
    }
}

impl Stage5FinishTodoBoard {
    #[must_use]
    pub fn from_template() -> Self {
        Self {
            status: STAGE5_FINISH_TODOS.iter().map(|t| t.status).collect(),
        }
    }

    pub fn init_open() -> Self {
        Self {
            status: vec![TodoStatus::Open; STAGE5_FINISH_TODOS.len()],
        }
    }
}

#[inline]
fn finish_ux06_index() -> Option<usize> {
    STAGE5_FINISH_TODOS.iter().position(|t| t.id == "FINISH-UX-06")
}

/// Live gate for **FINISH-UX-06**: no 250ms UX spike flag, map fit slots clean, frame time under UX ceiling.
#[must_use]
pub fn finish_ux06_frame_blocker(world: &World) -> Option<&'static str> {
    if world
        .get_resource::<UxFrameSpikeGuard>()
        .is_some_and(|g| g.spike_active)
    {
        return Some("ux_spike_active");
    }
    if let Some(log) = world.get_resource::<MapFitValidationLog>() {
        if log.fit_mode_mismatch {
            return Some("map_fit_mode_mismatch");
        }
        if log
            .world_preview
            .as_ref()
            .is_some_and(|v| v.mismatch)
        {
            return Some("world_preview_map_mismatch");
        }
        if log.minimap.as_ref().is_some_and(|v| v.mismatch) {
            return Some("minimap_map_mismatch");
        }
    }
    if let Some(b) = world.get_resource::<FrameBudgetDiagnostics>() {
        let ceiling = world
            .get_resource::<UxFrameSpikeGuard>()
            .map(|g| g.max_ms)
            .unwrap_or(250.0);
        if b.frame_time_ms > ceiling {
            return Some("frame_time_over_ceiling");
        }
    }
    None
}

#[must_use]
pub fn finish_ux06_frame_predicate(world: &World) -> bool {
    finish_ux06_frame_blocker(world).is_none()
}

/// Reconcile finish-lane rows from runtime witnesses (spine board unchanged).
pub fn sync_stage5_finish_todo_board(world: &mut World, readiness_passes: bool) {
    let Some(idx_06) = finish_ux06_index() else {
        return;
    };
    if !world.contains_resource::<Stage5FinishTodoBoard>()
        || !world.contains_resource::<Stage5FinishUx06Streak>()
    {
        return;
    }

    if !readiness_passes {
        {
            let mut streak = world.resource_mut::<Stage5FinishUx06Streak>();
            streak.consecutive_good = 0;
        }
        let mut board = world.resource_mut::<Stage5FinishTodoBoard>();
        if let Some(s) = board.status.get_mut(idx_06) {
            if *s == TodoStatus::Done {
                *s = TodoStatus::InProgress;
                info!(
                    target: "stage5_live_todos",
                    "STAGE5_FINISH_BOARD_REGRESSION FINISH-UX-06 readiness_fail → InProgress"
                );
            }
        }
        return;
    }

    let frame_blocker = finish_ux06_frame_blocker(world);
    let frame_ok = frame_blocker.is_none();

    let ux06_done = {
        let mut streak = world.resource_mut::<Stage5FinishUx06Streak>();
        if frame_ok {
            streak.consecutive_good = streak.consecutive_good.saturating_add(1);
            streak.last_blocker = None;
        } else {
            streak.consecutive_good = 0;
            streak.last_blocker = frame_blocker.map(str::to_string);
        }
        streak.consecutive_good >= FINISH_UX06_STREAK_DONE
    };
    let streak_n = world.resource::<Stage5FinishUx06Streak>().consecutive_good;

    let mut board = world.resource_mut::<Stage5FinishTodoBoard>();
    if let Some(s) = board.status.get_mut(idx_06) {
        let prev = *s;
        *s = if ux06_done {
            TodoStatus::Done
        } else {
            TodoStatus::InProgress
        };
        if prev != *s && *s == TodoStatus::Done {
            info!(
                target: "stage5_live_todos",
                "STAGE5_FINISH_BOARD FINISH-UX-06 → Done streak_evals={}",
                streak_n
            );
        }
    }
}

#[cfg(test)]
mod finish_board_tests {
    use super::*;

    #[test]
    fn ux06_done_after_streak_when_predicate_holds() {
        let mut world = World::new();
        world.insert_resource(Stage5FinishTodoBoard::from_template());
        world.insert_resource(Stage5FinishUx06Streak::default());
        world.insert_resource(UxFrameSpikeGuard::default());
        world.insert_resource(MapFitValidationLog::default());
        world.insert_resource(FrameBudgetDiagnostics::default());

        let idx = finish_ux06_index().unwrap();
        for _ in 0..FINISH_UX06_STREAK_DONE {
            sync_stage5_finish_todo_board(&mut world, true);
        }
        let board = world.resource::<Stage5FinishTodoBoard>();
        assert_eq!(board.status[idx], TodoStatus::Done);
    }

    #[test]
    fn readiness_fail_resets_ux06_streak() {
        let mut world = World::new();
        world.insert_resource(Stage5FinishTodoBoard::from_template());
        world.insert_resource(Stage5FinishUx06Streak {
            consecutive_good: FINISH_UX06_STREAK_DONE,
            last_blocker: None,
        });
        world.insert_resource(UxFrameSpikeGuard::default());
        world.insert_resource(MapFitValidationLog::default());
        world.insert_resource(FrameBudgetDiagnostics::default());

        let idx = finish_ux06_index().unwrap();
        world.get_resource_mut::<Stage5FinishTodoBoard>().unwrap().status[idx] = TodoStatus::Done;

        sync_stage5_finish_todo_board(&mut world, false);
        let streak = world.resource::<Stage5FinishUx06Streak>();
        assert_eq!(streak.consecutive_good, 0);
        let board = world.resource::<Stage5FinishTodoBoard>();
        assert_eq!(board.status[idx], TodoStatus::InProgress);
    }

    #[test]
    fn spike_guard_breaks_predicate() {
        let mut world = World::new();
        world.insert_resource(UxFrameSpikeGuard {
            spike_active: true,
            suppress_optional_diagnostics: true,
            ..Default::default()
        });
        world.insert_resource(MapFitValidationLog::default());
        world.insert_resource(FrameBudgetDiagnostics::default());
        assert!(!finish_ux06_frame_predicate(&world));
    }
}
