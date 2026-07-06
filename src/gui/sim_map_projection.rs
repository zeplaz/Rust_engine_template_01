//! Sim-map projection helpers for construction placement debug (BUILD-READ-REWIRE-001).

use bevy::prelude::*;
use bevy_egui::egui;

use super::{
    map_camera::{map_plane_horizontal_xy, sim_map_image_rect, sim_map_world_vec3_to_egui},
    MainWorldCameraOrthoTrace, MapCameraDesired, SimulationMapViewport,
};

/// Tactical camera pose for pick / ghost alignment (transform + desired zoom).
#[derive(Debug, Clone, Copy)]
pub struct MapCameraPresentationPose {
    pub translation: Vec3,
    pub zoom: f32,
    pub rotation: Quat,
}

#[must_use]
pub fn map_camera_pose_for_presentation(
    _xf: &GlobalTransform,
    desired: &MapCameraDesired,
) -> MapCameraPresentationPose {
    MapCameraPresentationPose {
        translation: desired.translation,
        zoom: desired.scale.x,
        rotation: desired.rotation,
    }
}

#[must_use]
pub fn map_camera_desired_from_presentation(pose: &MapCameraPresentationPose) -> MapCameraDesired {
    MapCameraDesired {
        translation: pose.translation,
        scale: Vec3::splat(pose.zoom),
        rotation: pose.rotation,
    }
}

/// Healed sim-map hole + ortho span used by placement debug probes.
#[derive(Debug, Clone, Copy)]
pub struct SimMapProjectionFrame {
    pub screen_rect: egui::Rect,
    pub visible_w: f32,
    pub visible_h: f32,
    pub camera_authoritative: bool,
}

#[must_use]
pub fn sim_map_projection_frame(
    camera: &Camera,
    map_vp: &SimulationMapViewport,
    _window: &Window,
    ortho: Option<&MainWorldCameraOrthoTrace>,
) -> Option<SimMapProjectionFrame> {
    if !map_vp.is_adequate_for_camera() {
        return None;
    }
    let screen_rect = if super::map_camera::tactical_map_full_window_render()
        && map_vp.window_logical.x > 1.0
        && map_vp.window_logical.y > 1.0
    {
        egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(map_vp.window_logical.x, map_vp.window_logical.y),
        )
    } else {
        sim_map_image_rect(map_vp)
    };
    let (visible_w, visible_h) = ortho
        .map(|o| (o.fixed_width, o.fixed_height))
        .unwrap_or((screen_rect.width(), screen_rect.height()));
    Some(SimMapProjectionFrame {
        screen_rect,
        visible_w,
        visible_h,
        camera_authoritative: camera.viewport.is_some(),
    })
}

#[must_use]
pub fn sim_map_screen_to_world_xy_in_frame(
    cursor: Vec2,
    pose: &MapCameraPresentationPose,
    frame: &SimMapProjectionFrame,
) -> Vec2 {
    let rect = frame.screen_rect;
    let cam = pose.translation.truncate();
    let dx = (cursor.x - rect.center().x) * frame.visible_w / rect.width().max(1.0);
    // World +Y is north; screen +Y is down — must match [`sim_map_screen_to_world_xy`].
    let dy = -(cursor.y - rect.center().y) * frame.visible_h / rect.height().max(1.0);
    Vec2::new(cam.x + dx, cam.y + dy)
}

/// [`Camera::world_to_viewport`] — already logical px (matches [`Window::cursor_position`]).
#[must_use]
pub fn camera_map_plane_vec3_to_logical_screen(
    camera: &Camera,
    xf: &GlobalTransform,
    world: Vec3,
) -> Option<Vec2> {
    let xy = map_plane_horizontal_xy(world);
    camera
        .world_to_viewport(xf, Vec3::new(xy.x, xy.y, world.z))
        .ok()
}

#[must_use]
pub fn sim_map_world_vec3_to_egui_rendered(
    world: Vec3,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    _camera: &Camera,
    _xf: &GlobalTransform,
    _window: &Window,
    _ortho: Option<&MainWorldCameraOrthoTrace>,
) -> Option<egui::Pos2> {
    sim_map_world_vec3_to_egui(
        world,
        desired,
        map_vp,
        map_vp.window_logical.x.max(map_vp.max.x).max(1.0),
        map_vp.window_logical.y.max(map_vp.max.y).max(1.0),
    )
}
