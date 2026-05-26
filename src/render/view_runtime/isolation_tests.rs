//! VM-C C6 — view runtime isolation guards (unit-level).

use bevy::prelude::*;

use bevy_egui::egui;

use crate::gui::{
    commit_map_camera_pose_to_view_authority, sync_view_manager_world_main_from_authority,
    view_surface_screen_to_world, MapCameraDesired, ViewCameraState, ViewId, ViewInstance,
    ViewInteractionState, ViewManager, ViewRenderPolicy, ViewRenderTarget,
    VIEW_NO_ENTITY,
};
use crate::render::view_runtime::{ViewAuthorityWriter, ViewProjectionAuthority, ViewRuntimeTrace};

#[test]
fn map_camera_input_commits_authority_before_view_manager_sync() {
    let mut authority = ViewProjectionAuthority::default();
    let mut trace = ViewRuntimeTrace::default();
    let mut manager = ViewManager::default();
    let desired = MapCameraDesired {
        translation: Vec3::new(42.0, 17.0, 0.0),
        scale: Vec3::splat(2.5),
        ..Default::default()
    };

    commit_map_camera_pose_to_view_authority(&mut authority, &mut trace, &desired);
    let wm = authority
        .surface(crate::render::view_runtime::ViewSurfaceId::WorldMain)
        .expect("WorldMain committed");
    assert!((wm.camera.translation.x - 42.0).abs() < 1e-4);
    assert!((wm.camera.zoom - 2.5).abs() < 1e-4);
    assert_eq!(
        authority
            .last_pose_writer
            .get(&crate::render::view_runtime::ViewSurfaceId::WorldMain)
            .copied(),
        Some(ViewAuthorityWriter::MapCameraInput)
    );

    sync_view_manager_world_main_from_authority(&mut manager, &authority);
    let inst = manager
        .views
        .get(&ViewId::WorldMain)
        .expect("WorldMain instance");
    assert!((inst.camera.translation.x - 42.0).abs() < 1e-4);
}

#[test]
fn simulation_map_pose_does_not_overwrite_world_main() {
    let mut authority = ViewProjectionAuthority::default();

    authority.commit_pose(
        crate::render::view_runtime::ViewSurfaceId::WorldMain,
        crate::gui::ViewCameraState {
            translation: Vec2::new(100.0, 0.0),
            zoom: 4.0,
            rotation: 0.0,
        },
        ViewAuthorityWriter::MapCameraInput,
    );

    authority.commit_pose(
        crate::render::view_runtime::ViewSurfaceId::SimulationMap,
        crate::gui::ViewCameraState {
            translation: Vec2::new(7.0, 9.0),
            zoom: 0.5,
            rotation: 0.0,
        },
        ViewAuthorityWriter::PreviewPanel,
    );
    let wm_after = authority
        .surface(crate::render::view_runtime::ViewSurfaceId::WorldMain)
        .expect("WorldMain unchanged");
    assert!((wm_after.camera.translation.x - 100.0).abs() < 1e-4);
    let sim = authority
        .surface(crate::render::view_runtime::ViewSurfaceId::SimulationMap)
        .expect("SimulationMap");
    assert!((sim.camera.translation.x - 7.0).abs() < 1e-4);
}

#[test]
fn preview_panel_writer_tagged_separately_from_map_input() {
    let mut authority = ViewProjectionAuthority::default();
    let mut trace = ViewRuntimeTrace::default();

    commit_map_camera_pose_to_view_authority(
        &mut authority,
        &mut trace,
        &MapCameraDesired {
            translation: Vec3::new(1.0, 2.0, 0.0),
            scale: Vec3::ONE,
            ..Default::default()
        },
    );
    authority.commit_pose(
        crate::render::view_runtime::ViewSurfaceId::WorldPreview,
        crate::gui::ViewCameraState {
            translation: Vec2::new(50.0, 60.0),
            zoom: 2.0,
            rotation: 0.0,
        },
        ViewAuthorityWriter::PreviewPanel,
    );

    assert_eq!(
        authority
            .last_pose_writer
            .get(&crate::render::view_runtime::ViewSurfaceId::WorldMain)
            .copied(),
        Some(ViewAuthorityWriter::MapCameraInput)
    );
    assert_eq!(
        authority
            .last_pose_writer
            .get(&crate::render::view_runtime::ViewSurfaceId::WorldPreview)
            .copied(),
        Some(ViewAuthorityWriter::PreviewPanel)
    );
}

#[test]
fn bridge_upsert_preserves_map_camera_pose_writer_tag() {
    use crate::gui::ViewCameraState;
    use crate::render::view_runtime::layers::RenderViewportContract;
    use crate::render::view_runtime::{ViewIsolationGroup, ViewSurfaceId};
    use crate::gui::{ViewInstance, ViewRenderTarget};

    let mut authority = ViewProjectionAuthority::default();
    authority.commit_pose(
        ViewSurfaceId::WorldMain,
        ViewCameraState {
            translation: Vec2::new(10.0, 20.0),
            zoom: 3.0,
            rotation: 0.0,
        },
        ViewAuthorityWriter::MapCameraInput,
    );
    let inst = ViewInstance {
        id: ViewId::WorldMain,
        camera_entity: Entity::PLACEHOLDER,
        render_target: ViewRenderTarget::PrimaryWindow,
        camera: ViewCameraState {
            translation: Vec2::new(10.0, 20.0),
            zoom: 3.0,
            rotation: 0.0,
        },
        projection: Default::default(),
        interaction_state: Default::default(),
        viewport_rect: bevy::math::Rect::from_corners(Vec2::ZERO, Vec2::new(800.0, 600.0)),
        render_policy: Default::default(),
    };
    authority.upsert_from_view_instance(
        ViewSurfaceId::WorldMain,
        ViewIsolationGroup::WorldSimulation,
        &inst,
        RenderViewportContract::default(),
        ViewAuthorityWriter::BridgeCompat,
    );
    assert_eq!(
        authority
            .last_pose_writer
            .get(&ViewSurfaceId::WorldMain)
            .copied(),
        Some(ViewAuthorityWriter::MapCameraInput)
    );
}

#[test]
fn deferred_preview_commits_preview_panel_writer() {
    use crate::gui::MapViewInstances;
    use crate::render::view_runtime::ViewSurfaceId;
    use crate::render::view_runtime::commit_deferred_map_view_poses_from_instances;

    let mut views = MapViewInstances::default();
    views.world_preview.camera_center = Vec2::new(33.0, 44.0);
    views.world_preview.zoom = 1.25;

    let mut authority = ViewProjectionAuthority::default();
    let mut trace = ViewRuntimeTrace::default();
    commit_deferred_map_view_poses_from_instances(&views, &mut authority, &mut trace);

    let wp = authority
        .surface(ViewSurfaceId::WorldPreview)
        .expect("preview surface");
    assert!((wp.camera.translation.x - 33.0).abs() < 1e-4);
    assert_eq!(
        authority
            .last_pose_writer
            .get(&ViewSurfaceId::WorldPreview)
            .copied(),
        Some(ViewAuthorityWriter::PreviewPanel)
    );
}

/// **INFRA-PROJ2-001** — per-view screen→world uses each view's camera (not WorldMain).
#[test]
fn infra_proj2_view_surface_screen_to_world_isolates_minimap_and_preview() {
    let mut manager = ViewManager::default();
    let image_rect = egui::Rect::from_min_max(egui::pos2(10.0, 20.0), egui::pos2(110.0, 120.0));
    let screen = egui::pos2(60.0, 70.0);
    let tex = 128.0;

    let minimap_cam = ViewCameraState {
        translation: Vec2::new(64.0, 64.0),
        zoom: 2.0,
        rotation: 0.0,
    };
    let preview_cam = ViewCameraState {
        translation: Vec2::new(10.0, 20.0),
        zoom: 0.5,
        rotation: 0.0,
    };

    manager.views.insert(
        ViewId::Minimap,
        ViewInstance {
            id: ViewId::Minimap,
            camera_entity: VIEW_NO_ENTITY,
            render_target: ViewRenderTarget::None,
            camera: minimap_cam,
            projection: minimap_cam.to_projection(),
            interaction_state: ViewInteractionState::default(),
            viewport_rect: Rect::from_corners(Vec2::ZERO, Vec2::splat(200.0)),
            render_policy: ViewRenderPolicy::default(),
        },
    );
    manager.views.insert(
        ViewId::WorldPreview,
        ViewInstance {
            id: ViewId::WorldPreview,
            camera_entity: VIEW_NO_ENTITY,
            render_target: ViewRenderTarget::None,
            camera: preview_cam,
            projection: preview_cam.to_projection(),
            interaction_state: ViewInteractionState::default(),
            viewport_rect: Rect::from_corners(Vec2::ZERO, Vec2::splat(200.0)),
            render_policy: ViewRenderPolicy::default(),
        },
    );

    let mm = view_surface_screen_to_world(
        &manager,
        ViewId::Minimap,
        screen,
        image_rect,
        tex,
        tex,
    )
    .expect("minimap");
    let wp = view_surface_screen_to_world(
        &manager,
        ViewId::WorldPreview,
        screen,
        image_rect,
        tex,
        tex,
    )
    .expect("preview");
    assert!(
        (mm - wp).length() > 1.0,
        "PROJ-2: minimap vs preview world hit must differ (mm={mm:?} wp={wp:?})"
    );
}
