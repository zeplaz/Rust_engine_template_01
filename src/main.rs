use bevy::prelude::*;
use clap::Parser;
use proc_A_dine01::engine::{EngineLaunchArgs, EnginePlugin, TestWorldHarness};

/// Game / engine binary (Bevy).
#[derive(Parser)]
#[command(name = "proc_A_dine01")]
struct Cli {
    /// Load a small generated world and enter simulation with sim-debug defaults (`weather` | `fire`).
    #[arg(long, value_name = "MODE")]
    test: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let launch = EngineLaunchArgs::from_test_cli_flag(cli.test);
    let harness = if launch.test_mode() {
        TestWorldHarness {
            active: true,
            finished: false,
            phase: 0,
        }
    } else {
        TestWorldHarness::default()
    };

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .insert_resource(launch)
        .insert_resource(harness)
        .add_plugins(EnginePlugin)
        .run();
}
