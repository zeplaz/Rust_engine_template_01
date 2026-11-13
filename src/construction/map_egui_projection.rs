//! Simulation-map world → egui screen (construction ghosts / overlays).
//!
//! **VM-C:** Presentation reads [`ViewProjectionAuthority`] for [`ViewSurfaceId::SimulationMap`]
//! (then `WorldMain`), with [`MapCameraDesired`] as compat fallback only.

use bevy::prelude::*;

use crate::gui::{
    sim_map_screen_to_world_xy, sim_map_world_vec3_to_screen, sim_map_world_xy_to_screen_with_ortho,
    view_camera_state_from_map_camera_desired, MainWorldCameraOrthoTrace, MapCameraDesired,
    SimulationMapViewport, ViewCameraState,
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
    pub fn world_to_screen(&self, world: Vec3) -> Option<bevy::prelude::Vec2> {
        let compat = self.map_camera_compat();
        sim_map_world_vec3_to_screen(world, &compat, self.map_vp, self.world_w, self.world_h)
    }

    /// Prefer [`MainWorldCameraOrthoTrace`] — matches RTT stretch + tactical map camera.
    #[must_use]
    pub fn world_to_screen_with_ortho(
        &self,
        world: Vec3,
        ortho: &MainWorldCameraOrthoTrace,
    ) -> Option<bevy::prelude::Vec2> {
        sim_map_world_xy_to_screen_with_ortho(Vec2::new(world.x, world.z), self.map_vp, ortho)
    }

    #[must_use]
    pub fn world_to_screen_rendered(
        &self,
        world: Vec3,
        ortho: Option<&MainWorldCameraOrthoTrace>,
    ) -> Option<bevy::prelude::Vec2> {
        if let Some(ortho) = ortho.filter(|o| o.fixed_width > 0.0 && o.fixed_height > 0.0) {
            return self.world_to_screen_with_ortho(world, ortho);
        }
        self.world_to_screen(world)
    }

    #[must_use]
    pub fn cursor_world_xy(&self, cursor_logical: Vec2) -> Option<Vec2> {
        let compat = self.map_camera_compat();
        sim_map_screen_to_world_xy(
            cursor_logical,
            &compat,
            self.map_vp,
            self.world_w,
            self.world_h,
        )
    }

    #[must_use]
    pub fn cursor_world_xy_with_ortho(
        &self,
        cursor_logical: Vec2,
        ortho: &crate::gui::MainWorldCameraOrthoTrace,
    ) -> Option<Vec2> {
        crate::gui::sim_map_screen_to_world_xy_with_ortho(cursor_logical, self.map_vp, ortho)
    }

    #[must_use]
    pub fn cursor_world_xy_rendered(
        &self,
        cursor_logical: Vec2,
        camera: &Camera,
        xf: &GlobalTransform,
        window: &Window,
        ortho: Option<&crate::gui::MainWorldCameraOrthoTrace>,
    ) -> Option<Vec2> {
        let _ = (camera, xf, window);
        if let Some(ortho) = ortho {
            return self.cursor_world_xy_with_ortho(cursor_logical, ortho);
        }
        if let Some(frame) = crate::gui::sim_map_projection_frame(camera, self.map_vp, window, None)
        {
            let pose = crate::gui::MapCameraPresentationPose {
                translation: self.map_camera_compat().translation,
                zoom: self.camera.zoom,
                rotation: self.map_camera_compat().rotation,
            };
            return Some(crate::gui::sim_map_screen_to_world_xy_in_frame(
                cursor_logical,
                &pose,
                &frame,
            ));
        }
        self.cursor_world_xy(cursor_logical)
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
pub fn world_to_sim_map_screen(
    world: Vec3,
    authority: Option<&ViewProjectionAuthority>,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    params: &WorldGenParams,
) -> Option<bevy::prelude::Vec2> {
    ConstructionMapProjection::resolve(authority, desired, map_vp, params).world_to_screen(world)
}

#[must_use]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn world_to_sim_map_screen_rendered(
    world: Vec3,
    ortho: Option<&MainWorldCameraOrthoTrace>,
    authority: Option<&ViewProjectionAuthority>,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    params: &WorldGenParams,
) -> Option<bevy::prelude::Vec2> {
    ConstructionMapProjection::resolve(authority, desired, map_vp, params)
        .world_to_screen_rendered(world, ortho)
}

#[must_use]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn map_zoom_screen_scale(
    authority: Option<&ViewProjectionAuthority>,
    desired: &MapCameraDesired,
) -> f32 {
    zoom_screen_scale_for_camera(simulation_map_camera_from_authority(authority, desired))
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

}

pub use world_to_sim_map_screen as world_to_sim_map_egui;
