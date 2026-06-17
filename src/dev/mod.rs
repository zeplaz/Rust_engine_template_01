//! Development-only runtime harnesses (Stage 5 live execution, etc.).
//!
//! This module may import the rest of the crate; other modules should avoid
//! importing `dev` except at explicit hook sites to prevent accidental coupling.

pub mod debug_run_envelope;
pub mod stage7_play_witness;
pub mod stage7_behavioral_witness;
pub mod runtime_witness;
pub mod schedule_cycle_probe;
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
pub mod proof_grade;
pub mod replay_editor_parity;
pub mod stage5_live_todos;
pub mod stage5_finish_todos;
pub mod visual_aidv2_live_todos;
// Lib witness refresh bundles — compiled only for `cargo test --lib` (not runtime).
#[cfg(test)]
pub mod steward_witness_sync_proof;
#[cfg(test)]
pub mod steward_spark_vfx_proof;
#[cfg(test)]
pub mod steward_s7b_preflight_proof;
#[cfg(test)]
pub mod steward_ui_oh_gate_proof;
#[cfg(test)]
pub mod steward_w3_gate_proof;
#[cfg(test)]
pub mod coder_a_ui_five_lane_proof;
#[cfg(test)]
pub mod coder_a_ui_w3_p4_m3_proof;
#[cfg(test)]
pub mod coder_a_dual_queue_closure_v1;
#[cfg(test)]
pub mod coder_a_wave3_closure_v1;
#[cfg(test)]
pub mod coder_a_infra_stress_closure_v1;
pub mod compile_hygiene_live;
pub mod f2_smoke_pipeline_debug;
#[cfg(test)]
pub mod coder_b_ui_five_lane_proof;
#[cfg(test)]
pub mod coder_b_ui_shell_tail_closure_v1;
#[cfg(test)]
pub mod coder_b_ui_w3_witness_proof;
#[cfg(test)]
pub mod coder_b_ui_w3_p6_proof;
#[cfg(test)]
pub mod triage_vm09_v2_proof;
#[cfg(test)]
pub mod s7b_m2_m3_coder_proof;
#[cfg(test)]
pub mod coder_b_s7p_construction_mv_proof;
pub mod transport_network_live_proof;
// Wired for lib tests — BUILD-READ-REWIRE-003 (map zoom witness).
pub mod map_zoom_coherence_live_proof;
pub mod design_minimap_widget_live_proof;
pub mod design_build_toolbox_hud_live_proof;
pub mod design_fire_play_visibility_live_proof;
pub mod build_read_world_002_live_proof;
pub mod build_read_visual_001_live_proof;
pub mod build_read_debug_live_proof;
pub mod construction_placement_live_proof;
pub mod vfx_fire_test_highlight_live_proof;
pub mod landscape_grammar_live_proof;
pub mod landscape_grammar_sim_harness;
pub mod veg_runtime_proof_live;
pub mod fire_ecology_lib_harness;
pub mod product_verify_live_proof;
pub mod sim_effect_spine_live_proof;
pub mod design_event_log_ui_live_proof;
pub mod pilot_catalog_parity_live_proof;
pub mod build_read_grammar_v003_live_proof;
pub mod aps_bevy_qc_hud_live_proof;
#[cfg(test)]
pub mod coder_b_queue_bundle_proof;
#[cfg(test)]
pub mod coder_b_wave3_bundle_proof;
pub mod sim_steward_combined_regression;
pub mod vegetation_snapshot_roundtrip_live_proof;
pub mod minimap_topology_legend_live_proof;
pub mod infra_e0_profile_catalog_live_proof;
pub mod infra_utility_overlay_live_proof;
pub mod utility_network_live_proof;
pub mod infra_overlay_live_proof;
pub mod landscape_grammar_burn_live_proof;
pub mod landscape_grammar_fire_harvest_wire_live_proof;
pub mod landscape_grammar_visual_smoke_live_proof;
pub mod aps_dna_consumer_live_proof;
pub mod landscape_map_stamp_contract_live_proof;
pub mod wit_hon_phase6_reconcile_live_proof;
pub mod ind_play_witness_live_proof;
pub mod coder_b_parallel_wave_live_proof;
pub mod veg_resolver_parity_live_proof;
pub mod coder_b_e5_resolver_gate_live_proof;
#[cfg(test)]
pub mod phase6_coder_queue_bundle_proof;

pub use stage5_live_todos::{
    hook_post_readiness_evaluate, mark_stage5_todo, register_stage5_todo_runtime_hooks,
    emit_active_stage5_todo_context, Stage5LiveTodo, Stage5LiveTodoBoard, TodoStatus,
    STAGE5_ROOT_GATE_SEQUENCE, STAGE5_TODOS,
};
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
    sync_visual_aidv2_todo_board_predicates, VisualAidV2LiveTodo, VisualAidV2LiveTodoBoard,
    VisualAidV2Witness, VISUAL_AID_V2_TODOS,
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
#[cfg(test)]
pub use coder_b_queue_bundle_proof::refresh_coder_b_queue_bundle_live_witnesses;
