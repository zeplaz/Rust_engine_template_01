//! VM-A: publish [`ViewProjectionAuthority`] then rebuild [`ViewManager`] read model.

use bevy::math::{Rect, Vec2};
use bevy::prelude::*;

use crate::gui::{
    DebugFlags, MainWorldCamera, MapCameraDesired, MapViewInstanceId, MapViewInstances,
    MapViewPresentationStates, OverlayMask, ViewCameraState, ViewCameraTag, ViewFilterMask, ViewId,
    ViewInstance, ViewInteractionState, ViewManager, ViewRenderPolicy, ViewRenderTarget,
    WorldLodBand, WorldRepresentationFrame, VIEW_NO_ENTITY,
};
use crate::render::ResolvedViewports;

use super::authority::{ViewAuthorityWriter, ViewProjectionAuthority};
use super::ids::{default_isolation_group, ViewSurfaceId};
use super::layers::{RenderViewportContract, ViewRenderTargetDesc};
fn rect_from_resolved_logical(size: Vec2) -> Rect {
    Rect::from_center_size(Vec2::ZERO, size.max(Vec2::splat(1.0)))
}

fn rect_from_map_view_viewport(panel: Vec2, fallback: Rect) -> Rect {
    if panel.x > 0.0 && panel.y > 0.0 {
        Rect::from_center_size(panel * 0.5, panel)
    } else {
        fallback
    }
}

fn render_contract_from_resolved(
    logical_size: Vec2,
    physical_extent: UVec2,
    valid: bool,
) -> RenderViewportContract {
    RenderViewportContract {
        logical_size,
        physical_extent,
        valid,
        target: ViewRenderTargetDesc::None,
    }
}

fn world_main_camera_for_bridge(
    authority: &ViewProjectionAuthority,
    desired: &MapCameraDesired,
) -> ViewCameraState {
    authority
        .surface(ViewSurfaceId::WorldMain)
        .map(|s| s.camera)
        .unwrap_or_else(|| crate::gui::view_camera_state_from_map_camera_desired(desired))
}

/// Publish all surfaces into authority (VM-A bridge writer).
pub fn publish_view_surfaces_to_authority(
    authority: &mut ViewProjectionAuthority,
    resolved: &ResolvedViewports,
    lod_band: WorldLodBand,
    map_views: &MapViewInstances,
    map_presentation: &MapViewPresentationStates,
    desired: &MapCameraDesired,
    main_cam: Entity,
) {
    let cam_wm = world_main_camera_for_bridge(authority, desired);
    let sm = &resolved.simulation_map;
    let sim_map_viewport = if sm.valid {
        rect_from_resolved_logical(sm.logical_size)
    } else {
        rect_from_resolved_logical(resolved.primary_window.logical_size)
    };
    let sim_overlays = map_presentation
        .get(MapViewInstanceId::SimulationMap)
        .overlays;
    let world_main_overlays = OverlayMask {
        bits: sim_overlays,
    };
    let render_policy_wm = ViewRenderPolicy {
        lod_band,
        overlays: world_main_overlays,
        filter_mask: ViewFilterMask::default(),
        debug_flags: DebugFlags::default(),
    };

    let surfaces = [
        (
            ViewSurfaceId::WorldMain,
            ViewInstance {
                id: ViewId::WorldMain,
                camera_entity: main_cam,
                render_target: ViewRenderTarget::PrimaryWindow,
                camera: cam_wm,
                projection: cam_wm.to_projection(),
                interaction_state: ViewInteractionState::default(),
                viewport_rect: sim_map_viewport,
                render_policy: render_policy_wm.clone(),
            },
        ),
        (
            ViewSurfaceId::WorldPreview,
            {
                let cam_wp = ViewCameraState {
                    translation: map_views.world_preview.camera_center,
                    zoom: map_views.world_preview.zoom,
                    rotation: 0.0,
                };
                ViewInstance {
                    id: ViewId::WorldPreview,
                    camera_entity: VIEW_NO_ENTITY,
                    render_target: ViewRenderTarget::None,
                    camera: cam_wp,
                    projection: cam_wp.to_projection(),
                    interaction_state: ViewInteractionState {
                        pan_delta: Vec2::ZERO,
                        zoom_factor: 1.0,
                        hovered_tile: map_views.world_preview.hovered_tile,
                    },
                    viewport_rect: rect_from_map_view_viewport(
                        map_views.world_preview.viewport_size,
                        rect_from_resolved_logical(resolved.world_preview.logical_size),
                    ),
                    render_policy: ViewRenderPolicy {
                        lod_band,
                        overlays: OverlayMask {
                            bits: map_views.world_preview.overlays,
                        },
                        filter_mask: ViewFilterMask::default(),
                        debug_flags: DebugFlags::default(),
                    },
                }
            },
        ),
        (
            ViewSurfaceId::Minimap,
            {
                let cam_mm = ViewCameraState {
                    translation: map_views.minimap.camera_center,
                    zoom: map_views.minimap.zoom,
                    rotation: 0.0,
                };
                ViewInstance {
                    id: ViewId::Minimap,
                    camera_entity: VIEW_NO_ENTITY,
                    render_target: ViewRenderTarget::None,
                    camera: cam_mm,
                    projection: cam_mm.to_projection(),
                    interaction_state: ViewInteractionState::default(),
                    viewport_rect: rect_from_map_view_viewport(
                        map_views.minimap.viewport_size,
                        rect_from_resolved_logical(resolved.minimap_panel.logical_size),
                    ),
                    render_policy: ViewRenderPolicy {
                        lod_band,
                        overlays: OverlayMask {
                            bits: map_views.minimap.overlays,
                        },
                        filter_mask: ViewFilterMask::default(),
                        debug_flags: DebugFlags::default(),
                    },
                }
            },
        ),
        (
            ViewSurfaceId::SimulationMap,
            ViewInstance {
                id: ViewId::SimulationMap,
                camera_entity: main_cam,
                render_target: ViewRenderTarget::PrimaryWindow,
                camera: cam_wm,
                projection: cam_wm.to_projection(),
                interaction_state: ViewInteractionState::default(),
                viewport_rect: sim_map_viewport,
                render_policy: ViewRenderPolicy {
                    lod_band,
                    overlays: OverlayMask {
                        bits: sim_overlays,
                    },
                    filter_mask: ViewFilterMask::default(),
                    debug_flags: DebugFlags::default(),
                },
            },
        ),
    ];

    for (surface_id, inst) in surfaces {
        let wp = match surface_id {
            ViewSurfaceId::WorldPreview => &resolved.world_preview,
            ViewSurfaceId::Minimap => &resolved.minimap_panel,
            ViewSurfaceId::SimulationMap => &resolved.simulation_map,
            ViewSurfaceId::WorldMain => &resolved.simulation_map,
            _ => &resolved.primary_window,
        };
        let render = render_contract_from_resolved(wp.logical_size, wp.physical_extent, wp.valid);
        match surface_id {
            ViewSurfaceId::WorldMain | ViewSurfaceId::SimulationMap => {
                authority.commit_bridge_render_policy(
                    surface_id,
                    default_isolation_group(surface_id),
                    inst.camera_entity,
                    render,
                    inst.render_policy.clone(),
                    &inst,
                );
            }
            _ => {
                authority.upsert_from_view_instance(
                    surface_id,
                    default_isolation_group(surface_id),
                    &inst,
                    render,
                    ViewAuthorityWriter::BridgeCompat,
                );
            }
        }
    }
}

/// Rebuild [`ViewManager`] from authority (read model only).
pub fn rebuild_view_manager_from_authority(
    manager: &mut ViewManager,
    authority: &ViewProjectionAuthority,
) {
    manager.views.clear();
    for (surface_id, surface) in &authority.surfaces {
        let Some(view_id) = surface_id.to_view_id() else {
            continue;
        };
        manager.views.insert(
            view_id,
            ViewInstance {
                id: view_id,
                camera_entity: surface.camera_entity,
                render_target: match surface_id {
                    ViewSurfaceId::WorldMain | ViewSurfaceId::SimulationMap => {
                        ViewRenderTarget::PrimaryWindow
                    }
                    _ => ViewRenderTarget::None,
                },
                camera: surface.camera,
                projection: surface.camera.to_projection(),
                interaction_state: ViewInteractionState {
                    pan_delta: surface.interaction.pan_delta,
                    zoom_factor: surface.interaction.zoom_factor,
                    hovered_tile: None,
                },
                viewport_rect: Rect::from_center_size(
                    Vec2::ZERO,
                    surface.render.logical_size.max(Vec2::splat(1.0)),
                ),
                render_policy: surface.render_policy.clone(),
            },
        );
    }
}

/// VM-A entry: resolve main camera entity, publish authority, rebuild manager.
pub fn sync_view_authority_bridge(
    manager: &mut ViewManager,
    authority: &mut ViewProjectionAuthority,
    resolved: &ResolvedViewports,
    lod_frame: Option<&WorldRepresentationFrame>,
    map_views: &MapViewInstances,
    map_presentation: &MapViewPresentationStates,
    desired: &MapCameraDesired,
    main_cams: &Query<(Entity, Option<&ViewCameraTag>), With<MainWorldCamera>>,
) {
    let band = lod_frame
        .map(|f| f.global_band())
        .unwrap_or(WorldLodBand::Strategic);

    let mut main_cam = VIEW_NO_ENTITY;
    let mut first_main = None;
    for (entity, tag) in main_cams.iter() {
        if first_main.is_none() {
            first_main = Some(entity);
        }
        if let Some(t) = tag {
            if t.0 == ViewId::WorldMain {
                main_cam = entity;
                break;
            }
        }
    }
    if main_cam == VIEW_NO_ENTITY {
        main_cam = first_main.unwrap_or(VIEW_NO_ENTITY);
    }

    publish_view_surfaces_to_authority(
        authority,
        resolved,
        band,
        map_views,
        map_presentation,
        desired,
        main_cam,
    );

    rebuild_view_manager_from_authority(manager, authority);
}
