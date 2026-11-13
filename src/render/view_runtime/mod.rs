//! Multiview isolation runtime — **infrastructure lane** (VM-06…VM-11).
//!
//! Design: [`crate::dev::view_runtime_architecture_v1`](../../dev/view_runtime_architecture_v1.md).
//! VM-A: types + trace only; authority commit migrates incrementally from `gui::view_authority`.

mod authority;
mod bridge;
mod commit;
mod ids;
mod input_routing;
mod layers;
mod passes;
mod per_view_policy;
mod plugin;
mod surface;
mod trace;
mod view_fire_isolation;

#[cfg(test)]
mod isolation_tests;

pub use authority::{ViewAuthorityWriter, ViewProjectionAuthority};
pub use ids::{ViewEntity, ViewIsolationGroup, ViewSurfaceId};
pub use input_routing::{
    commit_deferred_map_view_poses_from_instances, commit_deferred_map_view_poses_to_authority,
    sync_view_input_routing_from_active_map, view_surface_from_map_instance, ViewInputRoutingState,
};
pub use layers::{
    InteractionViewportState, OverlayViewportPolicy, RenderViewportContract, SemanticViewportRect,
    ViewRenderTargetDesc,
};
pub use passes::{InteractionPass, OverlayPass, ViewCameraCommitPass};
pub use per_view_policy::PerViewRepresentationPolicy;
pub use view_fire_isolation::{
    overlay_masks_aligned_with_map_views, refresh_view_fire_isolation_witness,
    ViewFireIsolationWitness,
};
pub use bridge::{publish_view_surfaces_to_authority, sync_view_authority_bridge};
pub use commit::{
    apply_map_view_extents_from_authority, commit_resolved_viewports_to_authority,
    commit_simulation_map_hole_to_authority, render_contract_from_resolved,
    resolved_viewport_from_render_contract, resolved_viewport_from_surface,
    sync_resolved_viewports_from_authority,
};


pub use plugin::ViewRuntimePlugin;
pub use surface::ViewSurface;
pub use trace::{
    advance_view_runtime_trace_frame, ViewRuntimeTrace, ViewRuntimeTraceEntry, ViewViolationKind,
};

#[cfg(test)]
mod authority_pose_tests {
    
    use bevy::prelude::*;
    use crate::render::view_runtime::ViewProjectionAuthority;
    use crate::gui::{
        commit_map_camera_pose_to_view_authority, map_camera_desired_from_view_authority,
        MapCameraDesired,
    };
    use crate::render::view_runtime::ViewRuntimeTrace;

    /// **TRIAGE-VM-09-v2** — derive round-trip from authority WorldMain pose.
    #[test]
    fn triage_vm09_v2_derive_map_camera_desired_from_authority() {
        let mut authority = ViewProjectionAuthority::default();
        let mut trace = ViewRuntimeTrace::default();
        let pose = MapCameraDesired {
            translation: Vec3::new(42.0, 84.0, 999.0),
            scale: Vec3::splat(1.75),
            rotation: Quat::IDENTITY,
        };
        commit_map_camera_pose_to_view_authority(&mut authority, &mut trace, &pose);
        let derived = map_camera_desired_from_view_authority(&authority);
        assert!((derived.translation.x - 42.0).abs() < 1e-3);
        assert!((derived.translation.y - 84.0).abs() < 1e-3);
        assert!((derived.scale.x - 1.75).abs() < 1e-3);
    }

    /// **DEHACK-VIEW-001** — sim session commits pose via view authority (not raw MapCameraDesired).
    #[test]
    fn dehack_view_001_sim_default_uses_authority_commit() {
        let mut authority = ViewProjectionAuthority::default();
        let mut trace = ViewRuntimeTrace::default();
        let pose = MapCameraDesired {
            translation: Vec3::new(42.0, 84.0, 999.0),
            scale: Vec3::splat(1.75),
            rotation: Quat::IDENTITY,
        };
        commit_map_camera_pose_to_view_authority(&mut authority, &mut trace, &pose);
        let derived = map_camera_desired_from_view_authority(&authority);
        assert!((derived.translation.x - 42.0).abs() < 1e-3);
        assert!((derived.translation.y - 84.0).abs() < 1e-3);
        assert!((derived.scale.x - 1.75).abs() < 1e-3);
    }
}
