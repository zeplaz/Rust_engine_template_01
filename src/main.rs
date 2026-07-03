use bevy::prelude::*;
use clap::Parser;
use proc_A_dine01::engine::{EngineLaunchArgs, EnginePlugin};
use proc_A_dine01::engine::test_harness::TestWorldHarness;
use proc_A_dine01::render::DebugRenderTraceConfig;

/// Game / engine binary (Bevy).
#[derive(Parser)]
#[command(name = "proc_A_dine01")]
struct Cli {
    /// Debug / VFX test boot: `frame` (layout), `visual`/`capture` (proof+exit), `demo` (stay open),
    /// `unittest` (fixture world), or `weather` | `fire` | `atmosphere`.
    #[arg(long, value_name = "MODE")]
    test: Option<String>,
    /// Override unittest fixture path (default `assets/fixtures/unittest_world.ron`).
    #[arg(long, value_name = "PATH")]
    unittest_world: Option<String>,
    #[arg(long)]
    debug_viewport_trace: bool,
    #[arg(long)]
    debug_camera_sync: bool,
    #[arg(long)]
    debug_render_routing: bool,
    /// Log window / sim-map hole / camera scissor / ortho fit edges (`sim_view_sync` target).
    #[arg(long)]
    debug_sim_view_sync: bool,
    /// Consolidated visual / viewport / render-spine diagnostics (`visual_diag` target).
    #[arg(long)]
    debug_visual_diag: bool,
    /// ENGINE-DEEP-DEBUG-001: intrusive minimap/GPU/schedule witnesses (see runbook).
    #[arg(long)]
    deep_debug: bool,
    /// With `--test visual` / `capture`: keep the window open after proof (no auto `AppExit`).
    #[arg(long)]
    stay_open: bool,
}

fn main() {
    let cli = Cli::parse();
    if cli.deep_debug {
        proc_A_dine01::dev::engine_deep_debug::arm_deep_debug_from_cli();
    }
    let launch = EngineLaunchArgs::from_cli(cli.test, cli.stay_open, cli.unittest_world);
    if launch.maneuver == proc_A_dine01::engine::DebugManeuver::FrameScreen {
        std::env::set_var("UI_LAYOUT_DEBUG", "1");
    }
    // Debug builds default to vsync; `--test` harness runs need uncapped present for honest perf.
    if launch.test_mode()
        && std::env::var("PERF_NO_VSYNC").ok().is_none()
        && std::env::var("PERF_VSYNC").ok().is_none()
    {
        std::env::set_var("PERF_NO_VSYNC", "1");
    }
    let debug_trace = DebugRenderTraceConfig::from_cli_flags(
        cli.debug_viewport_trace,
        cli.debug_camera_sync,
        cli.debug_render_routing,
        cli.debug_sim_view_sync,
        cli.debug_visual_diag,
    );
    let test_mode = launch.test_mode();
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .insert_resource(launch)
        .insert_resource(debug_trace);
    if test_mode {
        app.insert_resource(TestWorldHarness {
            active: true,
            finished: false,
            phase: 0,
            defaults_applied: false,
            logistics_visual_seeded: false,
            concrete_chain_e2e_seeded: false,
            concrete_chain_seed_phase: 0,
            s7p_logistics_throughput_seeded: false,
            s7p_logistics_finalize_pending: false,
            s7p_logistics_seed_phase: 0,
            s7p_logistics_seed_ticks: 0,
            minimap_m2_overlay_seeded: false,
            post_enter_sim_frame: 0,
        });
    }
    app.add_plugins(EnginePlugin).run();
}
