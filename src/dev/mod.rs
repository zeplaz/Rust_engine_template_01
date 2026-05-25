//! Development-only runtime harnesses (Stage 5 live execution, etc.).
//!
//! This module may import the rest of the crate; other modules should avoid
//! importing `dev` except at explicit hook sites to prevent accidental coupling.

pub mod debug_run_envelope;
pub mod construction_live_todos;
pub mod construction_finish_todos;
pub mod construction_phase2_todos;
pub mod construction_p9_todos;
pub mod construction_round2_todos;
pub mod construction_round3_todos;
pub mod construction_operational_todos;
pub mod industrial_activation_todos;
pub mod logistics_throughput_todos;
pub mod orchestrator_health;
pub mod replay_editor_parity;
pub mod stage5_live_todos;
pub mod stage5_finish_todos;
pub mod visual_aidv2_live_todos;

pub use stage5_live_todos::{
    hook_post_readiness_evaluate, mark_stage5_todo, register_stage5_todo_runtime_hooks,
    emit_active_stage5_todo_context, Stage5LiveTodo, Stage5LiveTodoBoard, TodoStatus,
    STAGE5_ROOT_GATE_SEQUENCE, STAGE5_TODOS,
};
pub use orchestrator_health::{
    orchestrator_health_path, OrchestratorHealthPlugin, OrchestratorThreadHealthExport,
};
pub use debug_run_envelope::{
    debug_runs_dir, refresh_agent_debug_index, wrap_debug_run, write_debug_run_json,
    AGENT_DEBUG_INDEX_PATH, ENVELOPE_SCHEMA,
};
pub use stage5_finish_todos::{
    finish_ux06_frame_predicate, sync_stage5_finish_todo_board, Stage5FinishTodoBoard,
    Stage5FinishUx06Streak, FINISH_UX06_STREAK_DONE, STAGE5_FINISH_TODOS,
};
pub use construction_live_todos::{
    register_construction_todo_runtime_hooks, ConstructionLiveTodo, ConstructionLiveTodoBoard,
    CONSTRUCTION_TODOS,
};
pub use construction_finish_todos::{
    register_construction_finish_todo_hooks, ConstructionFinishTodoBoard, ConstructionFinishWitness,
    CONSTRUCTION_FINISH_TODOS,
};
pub use construction_phase2_todos::{
    register_construction_phase2_todo_hooks, ConstructionPhase2TodoBoard, ConstructionPhase2Witness,
    CONSTRUCTION_PHASE2_TODOS,
};
pub use construction_p9_todos::{
    con_e01_p9_acceptance_green, register_construction_p9_todo_hooks, ConstructionP9TodoBoard,
    ConstructionP9Witness, CONSTRUCTION_P9_TODOS,
};
pub use construction_round2_todos::{
    register_construction_round2_todo_hooks, ConstructionRound2TodoBoard, ConstructionRound2Witness,
    CONSTRUCTION_ROUND2_TODOS,
};
pub use construction_round3_todos::{
    register_construction_round3_todo_hooks, ConstructionRound3TodoBoard, ConstructionRound3Witness,
    CONSTRUCTION_ROUND3_TODOS,
};
pub use construction_operational_todos::{
    register_construction_operational_todo_hooks, ConstructionOperationalTodoBoard,
    ConstructionOperationalWitness, CONSTRUCTION_OPERATIONAL_TODOS,
};
pub use industrial_activation_todos::{
    register_industrial_activation_todo_hooks, IndustrialActivationTodoBoard,
    IndustrialActivationWitness, INDUSTRIAL_ACTIVATION_TODOS,
};
pub use logistics_throughput_todos::{
    register_logistics_throughput_todo_hooks, LogisticsThroughputTodoBoard,
    LogisticsThroughputWitness, LOGISTICS_THROUGHPUT_TODOS,
};
pub use visual_aidv2_live_todos::{
    hook_post_readiness_visual_aidv2, register_visual_aidv2_runtime_hooks,
    sync_visual_aidv2_todo_board_predicates, VisualAidV2LiveTodo, VisualAidV2LiveTodoBoard,
    VisualAidV2Witness, VISUAL_AID_V2_TODOS,
};
