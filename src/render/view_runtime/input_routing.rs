//! VM-07: map UI input routes through [`ViewProjectionAuthority`] (not direct [`ViewManager`] writes).

use bevy::prelude::*;

use crate::gui::{ActiveMapViewInput, MapViewInstanceId, MapViewInstances, ViewCameraState};

use super::authority::{ViewAuthorityWriter, ViewProjectionAuthority};
use super::ids::ViewSurfaceId;
use super::trace::ViewRuntimeTrace;

/// Per-frame input routing witness (infrastructure JSON + HUD).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewInputRoutingState {
    pub active_surface: Option<ViewSurfaceId>,
    pub blocks_world_main: bool,
}

#[must_use]
pub fn view_surface_from_map_instance(id: MapViewInstanceId) -> ViewSurfaceId {
    match id {
        MapViewInstanceId::WorldPreview => ViewSurfaceId::WorldPreview,
        MapViewInstanceId::Minimap => ViewSurfaceId::Minimap,
        MapViewInstanceId::SimulationMap
        | MapViewInstanceId::TacticalMap
        | MapViewInstanceId::FullscreenMap
        | MapViewInstanceId::CommanderMap
        | MapViewInstanceId::Stage7IntelMap => ViewSurfaceId::SimulationMap,
    }
}

/// Mirror [`ActiveMapViewInput`] into view-runtime routing state (read by witnesses).
pub fn sync_view_input_routing_from_active_map(
    active: Res<ActiveMapViewInput>,
    mut routing: ResMut<ViewInputRoutingState>,
) {
    routing.active_surface = active.0.map(view_surface_from_map_instance);
    routing.blocks_world_main = active.blocks_main_world_map_camera_input();
}

/// After deferred [`MapViewInteractionByView`] applies to [`MapViewInstances`], commit poses to authority.
pub fn commit_deferred_map_view_poses_from_instances(
    views: &MapViewInstances,
    authority: &mut ViewProjectionAuthority,
    trace: &mut ViewRuntimeTrace,
) {
    let preview_cam = ViewCameraState {
        translation: views.world_preview.camera_center,
        zoom: views.world_preview.zoom,
        rotation: 0.0,
    };
    let minimap_cam = ViewCameraState {
        translation: views.minimap.camera_center,
        zoom: views.minimap.zoom,
        rotation: 0.0,
    };

    if trace.enabled {
        authority.commit_pose_traced(
            ViewSurfaceId::WorldPreview,
            preview_cam,
            ViewAuthorityWriter::PreviewPanel,
            Some(trace),
        );
        authority.commit_pose_traced(
            ViewSurfaceId::Minimap,
            minimap_cam,
            ViewAuthorityWriter::MinimapShell,
            Some(trace),
        );
    } else {
        authority.commit_pose(
            ViewSurfaceId::WorldPreview,
            preview_cam,
            ViewAuthorityWriter::PreviewPanel,
        );
        authority.commit_pose(
            ViewSurfaceId::Minimap,
            minimap_cam,
            ViewAuthorityWriter::MinimapShell,
        );
    }
}

pub fn commit_deferred_map_view_poses_to_authority(
    views: Res<MapViewInstances>,
    mut authority: ResMut<ViewProjectionAuthority>,
    mut trace: ResMut<ViewRuntimeTrace>,
) {
    commit_deferred_map_view_poses_from_instances(
        views.as_ref(),
        authority.as_mut(),
        trace.as_mut(),
    );
}
