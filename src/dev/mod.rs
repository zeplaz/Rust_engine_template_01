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
pub mod stage7_play_live_proof;
pub mod stage7_behavioral_live_proof;
pub mod steward_witness_sync_proof;
pub mod steward_spark_vfx_proof;
pub mod steward_s7b_preflight_proof;
pub mod steward_ui_oh_gate_proof;
pub mod steward_w3_gate_proof;
pub mod coder_a_ui_five_lane_proof;
pub mod coder_a_ui_w3_p4_m3_proof;
pub mod coder_a_dual_queue_closure_v1;
pub mod coder_a_wave3_closure_v1;
pub mod compile_hygiene_live;
pub mod coder_b_ui_five_lane_proof;
pub mod coder_b_ui_w3_witness_proof;
pub mod coder_b_ui_w3_p6_proof;
pub mod triage_vm09_v2_proof;
pub mod s7b_m2_m3_coder_proof;
pub mod coder_b_s7p_construction_mv_proof;
pub mod coder_b_queue_bundle_proof;
pub mod coder_b_wave3_bundle_proof;

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
pub use stage7_behavioral_live_proof::{
    build_stage7_behavioral_live_proof_payload, commit_stage7_behavioral_live_proof,
    write_stage7_behavioral_live_proof_system, Stage7BehavioralLiveProofState,
    STAGE7_BEHAVIORAL_LIVE_JSON,
};
pub use stage7_play_live_proof::{
    build_stage7_play_live_proof_payload, write_stage7_play_live_proof_system,
    Stage7PlayLiveProofState, STAGE7_PLAY_LIVE_JSON,
};
pub use coder_b_queue_bundle_proof::refresh_coder_b_queue_bundle_live_witnesses;
