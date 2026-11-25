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
            .init_resource::<crate::economy::logistics::orders::SiteLegRouteBook>()
            .init_resource::<crate::economy::logistics::road_flow::RoadFlowBook>()
            .init_resource::<crate::systems::navigation::offroad_speed::OffroadSpeedField>()
            .init_resource::<crate::economy::logistics::track_wear::TrackWearBook>()
            // Worn tracks are placed through the construction queue (no-op when construction added it).
            .init_resource::<crate::construction::ConstructionPlanQueue>()
            .add_systems(
                Startup,
                crate::systems::navigation::load_logistics_path_concerns_on_startup,
            )
            .add_systems(
                Update,
                (
                    // RD-4: traffic from last frame's legs feeds this frame's planning.
                    crate::economy::logistics::road_flow::publish_road_flow_system
                        .before(crate::systems::navigation::plan_site_leg_routes_system),
                    // RD-2: off-road speed per tile, read by the leg planner (and formations).
                    crate::systems::navigation::offroad_speed::publish_offroad_speed_field_system
                        .before(crate::systems::navigation::plan_site_leg_routes_system),
                    crate::systems::navigation::plan_site_leg_routes_system,
                    crate::systems::navigation::sync_truck_motion_waypoints_system
                        .after(crate::systems::navigation::plan_site_leg_routes_system),
                    // RD-5: traffic wears tracks; well-worn runs along legs become Track roads.
                    (
                        crate::economy::logistics::track_wear::accumulate_track_wear_system,
                        crate::economy::logistics::track_wear::form_worn_tracks_system,
                    )
                        .chain()
                        .after(crate::systems::navigation::plan_site_leg_routes_system),
                    crate::systems::navigation::potential_field_navigation_system
                        .after(crate::systems::navigation::sync_truck_motion_waypoints_system),
                )
                    .in_set(NavSets::MotionCalculation),
            );
    }
}
