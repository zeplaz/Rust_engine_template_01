//! Live todo / proof boards — **harness only**.
//!
//! Registered from [`crate::engine::test_harness::TestHarnessPlugin`] (CLI `--test`),
//! never from the product [`crate::engine::EnginePlugin`] stack. Keeps closed-program
//! STAGE5/construction todo reopen loops out of normal play.

use bevy::prelude::*;

use crate::engine::EngineLaunchArgs;
use crate::render::{stage5_full_app_profile_requested, Stage5ReadinessProfile};

/// Inserts FULL_APP readiness (when requested) and all `*_live_todos` runtime hooks.
pub struct LiveTodoBoardsPlugin;

impl Plugin for LiveTodoBoardsPlugin {
    fn build(&self, app: &mut App) {
        let test_mode = app
            .world()
            .get_resource::<EngineLaunchArgs>()
            .is_some_and(|launch| launch.test_mode());
        if stage5_full_app_profile_requested(test_mode) {
            app.insert_resource(Stage5ReadinessProfile::FULL_APP);
            crate::dev::stage5_live_todos::register_stage5_todo_runtime_hooks(app);
        }
        crate::dev::visual_aidv2_live_todos::register_visual_aidv2_runtime_hooks(app);
        crate::dev::construction_live_todos::register_construction_todo_runtime_hooks(app);
        crate::dev::construction_finish_todos::register_construction_finish_todo_hooks(app);
        crate::dev::construction_phase2_todos::register_construction_phase2_todo_hooks(app);
        crate::dev::construction_round2_todos::register_construction_round2_todo_hooks(app);
        crate::dev::construction_round3_todos::register_construction_round3_todo_hooks(app);
        crate::dev::construction_operational_todos::register_construction_operational_todo_hooks(app);
        crate::dev::industrial_activation_todos::register_industrial_activation_todo_hooks(app);
        crate::dev::logistics_throughput_todos::register_logistics_throughput_todo_hooks(app);
        crate::dev::replay_editor_parity::register_replay_editor_parity_hooks(app);
    }
}
