use bevy::prelude::*;

use super::authority::ViewProjectionAuthority;


use super::view_fire_isolation::ViewFireIsolationStats;
use super::per_view_policy::PerViewRepresentationPolicy;
use super::input_routing::{sync_view_input_routing_from_active_map, ViewInputRoutingState};
use super::trace::{advance_view_runtime_trace_frame, ViewRuntimeTrace};

pub struct ViewRuntimePlugin;

impl Plugin for ViewRuntimePlugin {
    fn build(&self, app: &mut App) {
        let audit = std::env::var("VIEW_RUNTIME_AUDIT")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        app.init_resource::<ViewProjectionAuthority>()
            .init_resource::<PerViewRepresentationPolicy>()
            .init_resource::<ViewFireIsolationStats>()
            .init_resource::<ViewInputRoutingState>()
            .insert_resource(ViewRuntimeTrace {
                enabled: audit,
                ..default()
            })
            .add_systems(
                PreUpdate,
                (
                    advance_view_runtime_trace_frame,
                    sync_view_input_routing_from_active_map,
                ),
            )
            ;
    }
}
