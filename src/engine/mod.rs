// Core engine functionality
#[cfg(feature = "legacy_engine")]
mod engine;
mod engine_with_worldgen; // New engine implementation with world generation
pub mod worldgen_chrome_debug;
pub mod ux_orchestration;
pub mod ux_states;
pub mod debug_maneuver;
pub mod launch_args;
pub mod states;
pub mod play_scenario;
/// CLI/menu test worlds — import `engine::test_harness::*` internally; not re-exported at `engine::*` root.
pub mod test_harness;
mod transitions;
mod sets;

// Logic models — heavy optional deps; see `research_lmodels` feature in Cargo.toml.
#[cfg(feature = "research_lmodels")]
pub mod lmodels;

// Public exports
pub use engine_with_worldgen::*; // Use the world generation version
pub use ux_orchestration::{
    legacy_flow_for_worldgen_generating, legacy_flow_for_worldgen_preview, ux_begin_world_gen_from_menu,
    ux_enter_world_from_world_gen, ux_pause_confirm_exit_to_shutdown, ux_pause_resume,
    ux_return_to_main_menu, UxBridgeSet, UxOrchestrationPlugin,
};
pub use ux_states::{
    worldgen_lifecycle_active, worldgen_preview_systems_enabled, AppState, PauseState,
    UxFrameSpikeGuard, WorldGenChromeLatch, WorldGenState,
};
pub use debug_maneuver::{
    DebugCaptureFrameGate, DebugManeuver, DebugManeuverPlugin, FrameLayoutDebugSession,
    UnittestWorldFixture, FULL_CAPTURE_MIN_FRAMES_DEFAULT, GRACEFUL_EXIT_FRAMES_AFTER_PROOF,
};
pub use launch_args::{EngineLaunchArgs, TestScene};
pub use states::*;
pub use play_scenario::{
    active_play_truth_env_seeds, default_play_blocked_by_env_seeds, ActivePlayScenario,
    DefaultIndustrialPlayState, PlayScenarioId, PlayScenarioPlugin,
    DEFAULT_INDUSTRIAL_LOGISTICS_CHAIN_TILES, DEFAULT_INDUSTRIAL_MIN_WORLD_TILES,
    DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN, PLAY_SCENARIO_LIVE_JSON, PLAY_TRUTH_FORBIDDEN_ENV_SEEDS,
};
/// Active only while a CLI `--test` world is in sim (zoom/fire defaults). Not harness control state.
pub use test_harness::ActiveTestScene;
pub use transitions::*;
pub use sets::*;