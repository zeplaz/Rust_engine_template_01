//! Simulation-map world → egui screen (construction ghosts / overlays).
//!
//! **VM-C:** Presentation reads [`ViewProjectionAuthority`] for [`ViewSurfaceId::SimulationMap`]
//! (then `WorldMain`), with [`MapCameraDesired`] as compat fallback only.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::{
    sim_map_cursor_world_xy, sim_map_world_vec3_to_egui, view_camera_state_from_map_camera_desired,
    MapCameraDesired, SimulationMapViewport, ViewCameraState,
};
use crate::render::view_runtime::{ViewProjectionAuthority, ViewSurfaceId};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Resolved tactical-map camera for construction presentation (authority-first).
pub struct ConstructionMapProjection<'a> {
    pub camera: ViewCameraState,
    map_vp: &'a SimulationMapViewport,
    world_w: f32,
    world_h: f32,
}

impl<'a> ConstructionMapProjection<'a> {
    #[must_use]
    pub fn resolve(
        authority: Option<&ViewProjectionAuthority>,
        desired: &MapCameraDesired,
        map_vp: &'a SimulationMapViewport,
        params: &WorldGenParams,
    ) -> Self {
        let camera = simulation_map_camera_from_authority(authority, desired);
        Self {
            camera,
            map_vp,
            world_w: params.width as f32,
            world_h: params.height as f32,
        }
    }

    #[inline]
    fn map_camera_compat(&self) -> MapCameraDesired {
        map_camera_desired_from_view_camera(self.camera)
    }

    #[must_use]
    pub fn zoom_screen_scale(&self) -> f32 {
        zoom_screen_scale_for_camera(self.camera)
    }

    #[must_use]
    pub fn world_to_egui(&self, world: Vec3) -> Option<egui::Pos2> {
        let compat = self.map_camera_compat();
        sim_map_world_vec3_to_egui(
            world,
            &compat,
            self.map_vp,
            self.world_w,
            self.world_h,
        )
    }

    #[must_use]
    pub fn cursor_world_xy(&self, cursor_logical: Vec2) -> Option<Vec2> {
        let compat = self.map_camera_compat();
        sim_map_cursor_world_xy(
            cursor_logical,
            &compat,
            self.map_vp,
            self.world_w,
            self.world_h,
        )
    }
}

#[inline]
#[must_use]
pub fn zoom_screen_scale_for_camera(camera: ViewCameraState) -> f32 {
    camera.zoom.abs().max(1e-3)
}

#[must_use]
pub fn simulation_map_camera_from_authority(
    authority: Option<&ViewProjectionAuthority>,
    desired: &MapCameraDesired,
) -> ViewCameraState {
    authority
        .and_then(|a| {
            a.surface(ViewSurfaceId::SimulationMap)
                .or_else(|| a.surface(ViewSurfaceId::WorldMain))
        })
        .map(|s| s.camera)
        .unwrap_or_else(|| view_camera_state_from_map_camera_desired(desired))
}

#[inline]
#[must_use]
pub fn map_camera_desired_from_view_camera(camera: ViewCameraState) -> MapCameraDesired {
    MapCameraDesired {
        translation: camera.translation.extend(0.0),
        scale: Vec3::splat(camera.zoom),
        ..Default::default()
    }
}

#[must_use]
pub fn world_to_sim_map_egui(
    world: Vec3,
    authority: Option<&ViewProjectionAuthority>,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    params: &WorldGenParams,
) -> Option<egui::Pos2> {
    ConstructionMapProjection::resolve(authority, desired, map_vp, params).world_to_egui(world)
}

#[must_use]
pub fn map_zoom_screen_scale(
    authority: Option<&ViewProjectionAuthority>,
    desired: &MapCameraDesired,
) -> f32 {
    zoom_screen_scale_for_camera(simulation_map_camera_from_authority(authority, desired))
}

/// Screen size of one world tile edge (for Syx-style occupation quads).
#[must_use]
pub fn tile_screen_extent(
    authority: Option<&ViewProjectionAuthority>,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    params: &WorldGenParams,
) -> f32 {
    let proj = ConstructionMapProjection::resolve(authority, desired, map_vp, params);
    let zoom = proj.zoom_screen_scale();
    let a = proj.world_to_egui(Vec3::new(0.5, 0.0, 0.5));
    let b = proj.world_to_egui(Vec3::new(1.5, 0.0, 0.5));
    match (a, b) {
        (Some(p0), Some(p1)) => (p1 - p0).length().max(4.0 * zoom),
        _ => 24.0 * zoom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::view_runtime::ViewAuthorityWriter;

    #[test]
    fn authority_simulation_map_overrides_compat_desired() {
        let desired = MapCameraDesired {
            translation: Vec3::new(100.0, 200.0, 0.0),
            scale: Vec3::splat(1.0),
            ..Default::default()
        };
        let mut authority = ViewProjectionAuthority::default();
        authority.commit_pose(
            ViewSurfaceId::SimulationMap,
            ViewCameraState {
                translation: Vec2::new(500.0, 600.0),
                zoom: 4.0,
                rotation: 0.0,
            },
            ViewAuthorityWriter::ViewportPipeline,
        );
        let cam = simulation_map_camera_from_authority(Some(&authority), &desired);
        assert!((cam.translation.x - 500.0).abs() < 1e-4);
        assert!((cam.zoom - 4.0).abs() < 1e-4);
    }

    /// PLAY-BUILD-06: zoom changes tile screen extent; same world tile maps at both zoom levels.
    #[test]
    fn tile_projection_stable_under_zoom() {
        use crate::gui::{MapCameraDesired, SimulationMapViewport};
        use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

        let map_vp = SimulationMapViewport {
            valid: true,
            min: Vec2::new(80.0, 60.0),
            max: Vec2::new(880.0, 540.0),
        };
        let params = WorldGenParams::default();
        let world = Vec3::new(64.5, 0.0, 64.5);

        let zoom_low = MapCameraDesired {
            translation: Vec3::new(64.0, 64.0, 0.0),
            scale: Vec3::splat(0.35),
            ..Default::default()
        };
        let zoom_high = MapCameraDesired {
            translation: Vec3::new(64.0, 64.0, 0.0),
            scale: Vec3::splat(6.0),
            ..Default::default()
        };

        let ext_low = tile_screen_extent(None, &zoom_low, &map_vp, &params);
        let ext_high = tile_screen_extent(None, &zoom_high, &map_vp, &params);
        assert!(
            ext_low != ext_high,
            "tile screen extent should change with zoom: low={ext_low} high={ext_high}"
        );

        let proj_low =
            ConstructionMapProjection::resolve(None, &zoom_low, &map_vp, &params);
        let proj_high =
            ConstructionMapProjection::resolve(None, &zoom_high, &map_vp, &params);
        let screen_low = proj_low.world_to_egui(world).expect("low");
        let screen_high = proj_high.world_to_egui(world).expect("high");
        assert!((screen_low - screen_high).length() > 1.0);

        let center = map_vp.min + (map_vp.max - map_vp.min) * 0.5;
        let back_low = proj_low.cursor_world_xy(center).expect("roundtrip low");
        let back_high = proj_high.cursor_world_xy(center).expect("roundtrip high");
        let world_xy = Vec2::new(world.x, world.z);
        assert!(back_low.distance(world_xy) < 80.0);
        assert!(back_high.distance(world_xy) < 80.0);
    }
}
