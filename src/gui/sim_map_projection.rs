//! Sim-map projection helpers for construction placement (BUILD-READ-REWIRE-001).
//!
//! **MATH-06** ★ Solving 3D problems in 2D — gea4v1 5.1 (p350–351) ·
//! **LEGACY** `math_3d` — gea_v1_e4 p350 (prefer over gea_e3 p378).
//!
//! Contract (locked): **solve on the 2D map grid, lift for presentation**.
//! - Dimension reduction: world [`Vec3`] → map-plane XY (pick, footprint, logistics).
//! - Lift: map XY (+ presentation Z) → [`Vec3`] for ghosts / overlays / camera plane.
//! - Manual in-frame math uses [`SimMapProjectionFrame::visible_w`]/[`SimMapProjectionFrame::visible_h`]
//!   (ortho fixed world span). Never invent a 3D solve for grid placement.

use bevy::prelude::*;

use super::tactical::map_camera::{
    map_plane_horizontal_xy, sim_map_image_rect, sim_map_world_vec3_to_screen,
};
use super::{MainWorldCameraOrthoTrace, MapCameraDesired, SimulationMapViewport};

/// Default presentation Z when lifting a 2D map solve onto the tactical plane.
pub const MAP_PRESENTATION_Z: f32 = 0.0;

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
///
/// `visible_w`/`visible_h` are **world units** across [`Self::screen_rect`] (ortho fixed span).
/// Manual screen↔world math must use these — not screen pixel counts alone.
#[derive(Debug, Clone, Copy)]
pub struct SimMapProjectionFrame {
    pub screen_rect: bevy::prelude::Rect,
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
    let screen_rect = if super::tactical::map_camera::tactical_map_full_window_render()
        && map_vp.window_logical.x > 1.0
        && map_vp.window_logical.y > 1.0
    {
        bevy::prelude::Rect::from_corners(
            bevy::prelude::Vec2::new(0.0, 0.0),
            bevy::prelude::Vec2::new(0.0, 0.0)
                + bevy::prelude::Vec2::new(map_vp.window_logical.x, map_vp.window_logical.y),
        )
    } else {
        sim_map_image_rect(map_vp)
    };
    // Ortho fixed_* = world span across the map hole (MATH-06 solve plane extents).
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

/// MATH-06: reduce a 3D world point to 2D map-plane solve space.
///
/// Path-tool `Vec3(x, 0, row)` maps row → Y; otherwise XY is kept. Height / presentation Z
/// is discarded — grid solve never depends on it.
#[inline]
#[must_use]
pub fn reduce_world_to_map_xy(world: Vec3) -> Vec2 {
    map_plane_horizontal_xy(world)
}

/// MATH-06: lift a 2D map solve result into presentation space.
#[inline]
#[must_use]
pub fn lift_map_xy_for_presentation(xy: Vec2, presentation_z: f32) -> Vec3 {
    Vec3::new(xy.x, xy.y, presentation_z)
}

/// Screen → world XY on the frame's 2D solve plane (manual egui / debug path).
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

/// Inverse of [`sim_map_screen_to_world_xy_in_frame`] — world XY → screen on the same frame.
#[must_use]
pub fn sim_map_world_xy_to_screen_in_frame(
    world_xy: Vec2,
    pose: &MapCameraPresentationPose,
    frame: &SimMapProjectionFrame,
) -> Vec2 {
    let rect = frame.screen_rect;
    let cam = pose.translation.truncate();
    let sx = rect.center().x + (world_xy.x - cam.x) * rect.width().max(1.0) / frame.visible_w.max(1e-6);
    let sy =
        rect.center().y - (world_xy.y - cam.y) * rect.height().max(1.0) / frame.visible_h.max(1e-6);
    Vec2::new(sx, sy)
}

/// [`Camera::world_to_viewport`] after MATH-06 reduce — logical px (matches [`Window::cursor_position`]).
#[must_use]
pub fn camera_map_plane_vec3_to_logical_screen(
    camera: &Camera,
    xf: &GlobalTransform,
    world: Vec3,
) -> Option<Vec2> {
    let xy = reduce_world_to_map_xy(world);
    camera
        .world_to_viewport(xf, Vec3::new(xy.x, xy.y, world.z))
        .ok()
}

/// Project a world [`Vec3`] to screen after MATH-06 reduce (via [`sim_map_world_vec3_to_screen`]).
#[must_use]
pub fn sim_map_world_vec3_to_screen_rendered(
    world: Vec3,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    _camera: &Camera,
    _xf: &GlobalTransform,
    _window: &Window,
    _ortho: Option<&MainWorldCameraOrthoTrace>,
) -> Option<bevy::prelude::Vec2> {
    sim_map_world_vec3_to_screen(
        world,
        desired,
        map_vp,
        map_vp.window_logical.x.max(map_vp.max.x).max(1.0),
        map_vp.window_logical.y.max(map_vp.max.y).max(1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_frame() -> (MapCameraPresentationPose, SimMapProjectionFrame) {
        let pose = MapCameraPresentationPose {
            translation: Vec3::new(256.0, 256.0, 0.0),
            zoom: 2.0,
            rotation: Quat::IDENTITY,
        };
        let frame = SimMapProjectionFrame {
            screen_rect: Rect::from_corners(Vec2::new(100.0, 50.0), Vec2::new(900.0, 550.0)),
            visible_w: 400.0,
            visible_h: 250.0,
            camera_authoritative: false,
        };
        (pose, frame)
    }

    #[test]
    fn math06_reduce_drops_height_keeps_grid_xy() {
        let p = Vec3::new(12.0, 7.5, 3.0);
        let xy = reduce_world_to_map_xy(p);
        assert!((xy.x - 12.0).abs() < 1e-5);
        assert!((xy.y - 7.5).abs() < 1e-5);
    }

    #[test]
    fn math06_reduce_path_tool_z_row_maps_to_y() {
        let p = Vec3::new(10.0, 0.0, 42.0);
        let xy = reduce_world_to_map_xy(p);
        assert!((xy.x - 10.0).abs() < 1e-5);
        assert!((xy.y - 42.0).abs() < 1e-5);
    }

    #[test]
    fn math06_lift_preserves_xy_sets_presentation_z() {
        let xy = Vec2::new(3.0, 9.0);
        let lifted = lift_map_xy_for_presentation(xy, 1.25);
        assert!((lifted.x - 3.0).abs() < 1e-5);
        assert!((lifted.y - 9.0).abs() < 1e-5);
        assert!((lifted.z - 1.25).abs() < 1e-5);
    }

    #[test]
    fn math06_dim_reduction_roundtrip_ignores_presentation_z() {
        let xy = Vec2::new(100.0, 200.0);
        let lifted = lift_map_xy_for_presentation(xy, 99.0);
        let back = reduce_world_to_map_xy(lifted);
        assert!((back.x - xy.x).abs() < 1e-5);
        assert!((back.y - xy.y).abs() < 1e-5);
        // Re-reduce after path-tool encoding still lands on the same grid solve.
        let path_encoded = Vec3::new(xy.x, 0.0, xy.y);
        let from_path = reduce_world_to_map_xy(path_encoded);
        assert!((from_path.x - xy.x).abs() < 1e-5);
        assert!((from_path.y - xy.y).abs() < 1e-5);
    }

    #[test]
    fn math06_screen_world_roundtrip_in_frame() {
        let (pose, frame) = sample_frame();
        let world = Vec2::new(280.0, 310.0);
        let screen = sim_map_world_xy_to_screen_in_frame(world, &pose, &frame);
        let back = sim_map_screen_to_world_xy_in_frame(screen, &pose, &frame);
        assert!(
            (back.x - world.x).abs() < 0.05,
            "x round-trip drift {}",
            (back.x - world.x).abs()
        );
        assert!(
            (back.y - world.y).abs() < 0.05,
            "y round-trip drift {}",
            (back.y - world.y).abs()
        );
    }

    #[test]
    fn math06_north_projects_above_screen_center() {
        let (pose, frame) = sample_frame();
        let center = pose.translation.truncate();
        let north = Vec2::new(center.x, center.y + 40.0);
        let south = Vec2::new(center.x, center.y - 40.0);
        let n = sim_map_world_xy_to_screen_in_frame(north, &pose, &frame);
        let s = sim_map_world_xy_to_screen_in_frame(south, &pose, &frame);
        assert!(
            n.y < s.y,
            "world +Y (north) must map to smaller screen Y; n={} s={}",
            n.y,
            s.y
        );
    }

    #[test]
    fn math06_visible_span_scales_screen_delta() {
        let (pose, mut frame) = sample_frame();
        let world = pose.translation.truncate() + Vec2::new(50.0, 0.0);
        let a = sim_map_world_xy_to_screen_in_frame(world, &pose, &frame);
        frame.visible_w *= 2.0;
        let b = sim_map_world_xy_to_screen_in_frame(world, &pose, &frame);
        let mid = frame.screen_rect.center().x;
        // Doubling visible world span halves screen offset from center.
        assert!(((a.x - mid).abs() - 2.0 * (b.x - mid).abs()).abs() < 0.5);
    }
}
