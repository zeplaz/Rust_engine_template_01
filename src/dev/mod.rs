//! Development-only runtime harnesses (Stage 5 live execution, etc.).
//!
//! This module may import the rest of the crate; other modules should avoid
//! importing `dev` except at explicit hook sites to prevent accidental coupling.

pub mod diagnostics;
pub mod mig_a_adoption;
pub mod schedule_cycle_probe;
pub mod perf_scope_frame_log;
pub mod test_run_instrumentation;
pub mod sim_run_dump;
pub mod sim_run_dump_atmosphere;
pub mod sim_run_dump_fidelity;
pub mod sim_run_dump_weather;
pub mod sim_run_dump_massing;
pub mod sim_run_dump_occupation;
pub mod sim_run_dump_quadrants;
pub mod sim_run_dump_dam_flood;
pub mod sim_run_dump_reporters;
pub mod orchestrator_health;
pub mod city_faction_progress;


pub use orchestrator_health::{
    orchestrator_health_path, OrchestratorHealthPlugin, OrchestratorThreadHealthExport,
};
pub use test_run_instrumentation::{
    instrumentation_active, instrumentation_flush_secs, instrumentation_frame_jsonl,
    instrumentation_quiet_terminal, instrumentation_stall_spans, TestRunInstrumentation,
    TestRunInstrumentationPlugin,
};
pub use sim_run_dump::{
    SimDumpArm, SimDumpSet, SimRunDump, SimRunDumpPlugin, SIM_RUN_DUMP_JSON, SIM_RUN_DUMP_SCHEMA,
};
pub use mig_a_adoption::{
    mig_a_static_scene_enabled, mig_a4_render_recovery_enabled, MigAAdoptionPlugin,
    MigAAdoptionState, MigAStaticScenePlugin,
};
