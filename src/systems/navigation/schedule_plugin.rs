//! Wires [`crate::engine::NavSets`] **after** transport cost cache — damage / speed adjustments before motion.
//!
//! Step pack: `prompts/matrix/engine_bevy/runbook/s2_schedule_navigation_steps_v1.md`.

use crate::engine::NavSets;
use crate::systems::transport::TransportSchedule;
use bevy::prelude::*;

pub struct NavigationSchedulePlugin;

impl Plugin for NavigationSchedulePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::systems::navigation::LogisticsGridPathScratch>()
            .configure_sets(
                Update,
                (
                    NavSets::DamageSpeedAdjustment.after(TransportSchedule::CostCache),
                    NavSets::MotionCalculation.after(NavSets::DamageSpeedAdjustment),
                ),
            )
            .add_systems(
                Startup,
                crate::systems::navigation::load_logistics_path_concerns_on_startup,
            )
            .add_systems(
                Update,
                (
                    crate::systems::navigation::sync_truck_motion_waypoints_system,
                    crate::systems::navigation::potential_field_navigation_system
                        .after(crate::systems::navigation::sync_truck_motion_waypoints_system),
                )
                    .in_set(NavSets::MotionCalculation),
            );
    }
}
