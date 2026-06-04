use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::ViewAuthoritySystemSet;

use super::authority::ViewProjectionAuthority;
use super::witness_state::{
    clear_minimap_map_camera_write_flag, refresh_view_runtime_witness, ViewRuntimeWitness,
};
use crate::dev::runtime_witness::view_runtime::{
    write_view_runtime_live_proof_system, ViewRuntimeLiveProofState,
};
use super::view_fire_isolation::{
    refresh_view_fire_isolation_witness, ViewFireIsolationWitness,
};
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
            .init_resource::<ViewRuntimeWitness>()
            .init_resource::<ViewRuntimeLiveProofState>()
            .init_resource::<ViewFireIsolationWitness>()
            .init_resource::<ViewInputRoutingState>()
            .insert_resource(ViewRuntimeTrace {
                enabled: audit,
                ..default()
            })
            .add_systems(
                PreUpdate,
                (
                    clear_minimap_map_camera_write_flag,
                    advance_view_runtime_trace_frame,
                    sync_view_input_routing_from_active_map,
                ),
            )
            .add_systems(
                Update,
                (
                    refresh_view_fire_isolation_witness,
                    refresh_view_runtime_witness,
                    write_view_runtime_live_proof_system,
                )
                    .chain()
                    .after(crate::render::FireVisualFrameSet::BuildProfiles)
                    .after(ViewAuthoritySystemSet::SyncViewManager)
                    .run_if(in_state(BaseState::Simulation)),
            );
    }
}
