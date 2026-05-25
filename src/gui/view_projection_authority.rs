//! Per-view projection read surface (Phase 1). Authoritative snapshots live in [`crate::gui::ViewManager`]
//! after [`crate::gui::ViewAuthoritySystemSet::SyncViewManager`] (which runs after
//! [`crate::gui::map_camera::MapCameraSystemSet::ApplyInput`] so [`MapCameraDesired`] is current). Prefer these helpers over ad-hoc
//! [`crate::gui::MapCameraDesired`] reads when you have a [`crate::gui::ViewId`].
//!
//! **vm-06:** [`view_surface_world_to_screen`] / [`view_surface_screen_to_world`] route through
//! [`view_instance`] so callers do not mix main vs minimap vs preview camera state.
//!
//! **proj-viewport-authority:** for the main tactical map use [`ViewId::WorldMain`] with
//! [`camera_translation`] / [`camera_zoom`] (fall back to [`crate::gui::MapCameraDesired`] only when the
//! bridge has not populated [`ViewManager`] yet).

use bevy::math::{Rect, Vec2};
use bevy_egui::egui;

use crate::gui::map_view_projection::{map_surface_screen_to_world, map_surface_world_to_screen};
use crate::gui::{ViewId, ViewInstance, ViewManager};

#[inline]
#[must_use]
pub fn view_instance(manager: &ViewManager, id: ViewId) -> Option<&ViewInstance> {
    manager.view(id)
}

#[inline]
#[must_use]
pub fn view_visible_world_rect(manager: &ViewManager, id: ViewId) -> Option<Rect> {
    manager.view(id).map(ViewInstance::visible_world_rect)
}

#[inline]
#[must_use]
pub fn camera_translation(manager: &ViewManager, id: ViewId) -> Option<Vec2> {
    manager.view(id).map(|v| v.camera.translation)
}

#[inline]
#[must_use]
pub fn camera_zoom(manager: &ViewManager, id: ViewId) -> Option<f32> {
    manager.view(id).map(|v| v.camera.zoom)
}

#[inline]
#[must_use]
pub fn view_surface_world_to_screen(
    manager: &ViewManager,
    id: ViewId,
    world_tile: Vec2,
    image_rect: egui::Rect,
    tex_w: f32,
    tex_h: f32,
) -> Option<egui::Pos2> {
    let v = view_instance(manager, id)?;
    Some(map_surface_world_to_screen(
        world_tile,
        image_rect,
        v.camera.translation,
        v.camera.zoom,
        tex_w,
        tex_h,
    ))
}

#[inline]
#[must_use]
pub fn view_surface_screen_to_world(
    manager: &ViewManager,
    id: ViewId,
    screen: egui::Pos2,
    image_rect: egui::Rect,
    tex_w: f32,
    tex_h: f32,
) -> Option<Vec2> {
    let v = view_instance(manager, id)?;
    Some(map_surface_screen_to_world(
        screen,
        image_rect,
        v.camera.translation,
        v.camera.zoom,
        tex_w,
        tex_h,
    ))
}
