//! Wires [`crate::engine::NavSets`] **after** transport cost cache — damage / speed adjustments before motion.
//!
//! Step pack: `prompts/matrix/engine_bevy/runbook/s2_schedule_navigation_steps_v1.md`.

use crate::engine::NavSets;
use crate::systems::transport::TransportSchedule;
use bevy::prelude::*;

pub struct NavigationSchedulePlugin;

impl Plugin for NavigationSchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                NavSets::DamageSpeedAdjustment.after(TransportSchedule::CostCache),
                NavSets::MotionCalculation.after(NavSets::DamageSpeedAdjustment),
            ),
        )
        .add_systems(
            Update,
            nav_motion_stage_placeholder.in_set(NavSets::MotionCalculation),
        );
    }
}

/// Extension point for road-vehicle displacement / path followers (matrix R7).
/// Runs **after** [`NavSets::DamageSpeedAdjustment`] (e.g. `DamageSystem::apply_road_damage`).
/// **W3:** [`TransportNavExport`](crate::systems::transport::TransportNavExport) is refreshed in
/// `TransportSchedule::CostCache` (after cost cache); read that for coarse costs + `allowed_agents`.
fn nav_motion_stage_placeholder(
    _sim: Res<crate::systems::sim_control::SimControlState>,
    nav: Res<crate::systems::transport::TransportNavExport>,
) {
    let _ = nav.edges.len();
}
