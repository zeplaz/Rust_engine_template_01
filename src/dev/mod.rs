//! Development-only runtime harnesses (Stage 5 live execution, etc.).
//!
//! This module may import the rest of the crate; other modules should avoid
//! importing `dev` except at explicit hook sites to prevent accidental coupling.

pub mod debug_run_envelope;
pub mod diagnostic_events;
pub mod diagnostics;
pub mod mig_a_adoption;
pub mod mig_a_audit;
pub mod stage7_play_witness;
pub mod stage7_behavioral_witness;
pub mod runtime_witness;
pub mod schedule_cycle_probe;
pub mod perf_scope_frame_log;
pub mod sim_spectrum_analytics;
pub mod schedule_sync_witness;
pub mod triage_perf_vfx_fix_live_proof;
pub mod test_run_instrumentation;
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
pub mod engine_deep_debug;
pub mod tactical_map_debug;
pub mod proof_grade;
pub mod replay_editor_parity;
pub mod stage5_live_todos;
pub mod stage5_finish_todos;
pub mod visual_aidv2_live_todos;
pub mod live_todo_boards_plugin;
pub mod spectator_parity_live_proof;
pub mod world_scale_contract_live_proof;
pub mod landscape_grammar_live_proof;
pub mod landscape_grammar_sim_harness;
pub mod veg_runtime_proof_live;
pub mod fire_ecology_lib_harness;
pub mod cod_deployable_recipe_live_proof;
pub mod sim_effect_spine_live_proof;
pub mod product_fire_vfx_live_proof;
pub mod city_faction_progress;
pub mod vegetation_snapshot_roundtrip_live_proof;
pub mod minimap_topology_legend_live_proof;
pub mod landscape_grammar_burn_live_proof;
pub mod landscape_grammar_fire_harvest_wire_live_proof;
pub mod landscape_grammar_visual_smoke_live_proof;

pub use stage5_live_todos::{
    hook_post_readiness_evaluate, mark_stage5_todo, register_stage5_todo_runtime_hooks,
    emit_active_stage5_todo_context, Stage5LiveTodo, Stage5LiveTodoBoard, TodoStatus,
    STAGE5_ROOT_GATE_SEQUENCE, STAGE5_TODOS,
};
pub use live_todo_boards_plugin::LiveTodoBoardsPlugin;
pub use orchestrator_health::{
    orchestrator_health_path, OrchestratorHealthPlugin, OrchestratorThreadHealthExport,
};
pub use debug_run_envelope::{
    assert_witness_honesty_before_write, debug_runs_dir, refresh_agent_debug_index,
    wrap_debug_run, write_debug_run_json, AGENT_DEBUG_INDEX_PATH, ENVELOPE_SCHEMA,
    WITNESS_HONESTY_ENFORCE_ENV, WITNESS_HONESTY_SKIP_ENV,
};
pub use runtime_witness::{
    witness_gate_snapshot, witness_writes_enabled, write_enveloped_witness,
    write_enveloped_witness_unchecked, LiveProofCadence, MIGRATION_SHIM_PATHS,
    ENV_RUNTIME_WITNESS_WRITES, ENV_RUNTIME_WITNESS_WRITES_FORCE_OFF,
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
    sync_visual_aidv2_todo_board_predicates, visual_aidv2_live_board_json,
    write_visual_aidv2_live_proof, VisualAidV2HarnessState, VisualAidV2LiveTodo,
    VisualAidV2LiveTodoBoard, VisualAidV2Witness, refresh_visual_aidv2_harness_lib_witness,
    VISUAL_AID_V2_LIVE_JSON, VISUAL_AID_V2_TODOS,
};
pub use stage7_behavioral_witness::{
    build_stage7_behavioral_witness_payload, commit_stage7_behavioral_witness,
    refresh_s7b_m3_steward_remedy_001_live_witness, refresh_s7b_m4_play_remedy_001_live_witness,
    refresh_s7b_steward_001_live_witness,
    write_stage7_behavioral_witness_system, Stage7BehavioralLiveProofState,
    STAGE7_BEHAVIORAL_LIVE_JSON,
};
pub use stage7_play_witness::{
    build_stage7_play_witness_payload, write_stage7_play_witness_system,
    Stage7PlayLiveProofState, STAGE7_PLAY_LIVE_JSON,
};
pub use sim_spectrum_analytics::{
    sim_spectrum_analytics_enabled, sim_spectrum_analytics_quiet_terminal,
    SimSpectrumAnalyticsPlugin, SIM_SPECTRUM_LIVE_JSON,
};
pub use test_run_instrumentation::{
    instrumentation_active, instrumentation_flush_secs, instrumentation_frame_jsonl,
    instrumentation_quiet_terminal, instrumentation_stall_spans, TestRunInstrumentation,
    TestRunInstrumentationPlugin,
};
pub use mig_a_adoption::{
    build_mig_a_rollup_json, mig_a_static_scene_enabled, mig_a4_render_recovery_enabled,
    MigAAdoptionPlugin, MigAAdoptionState, MigAStaticScenePlugin, MIG_A1_A2_A16_JSON,
    MIG_A_ROLLUP_JSON,
};
