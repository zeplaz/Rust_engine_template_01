//! Shared world-tile → egui projection for preview + minimap consumers.

use bevy::math::Vec2;
use bevy_egui::egui;

use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::map_view::MapViewState as EditorViewport;

/// Map world-tile coordinates to an egui rect inside a clipped viewport.
#[must_use]
pub fn map_display_rect(
    viewport_center: egui::Pos2,
    camera_center: Vec2,
    zoom: f32,
    tex_w: f32,
    tex_h: f32,
) -> egui::Rect {
    let z = zoom.clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
    let rect_min = egui::pos2(
        viewport_center.x - camera_center.x * z,
        viewport_center.y - camera_center.y * z,
    );
    egui::Rect::from_min_max(rect_min, rect_min + egui::vec2(tex_w * z, tex_h * z))
}

/// Apply presentation zoom/pan after a strict fit rect.
#[must_use]
pub fn map_presentation_image_rect(
    fitted_view: egui::Rect,
    camera_center: Vec2,
    zoom: f32,
    tex_w: f32,
    tex_h: f32,
) -> egui::Rect {
    map_display_rect(fitted_view.center(), camera_center, zoom, tex_w, tex_h)
}

/// Full-texture UVs for a world-sized raster (CPU or GPU render target).
#[must_use]
pub fn map_texture_uv_rect() -> egui::Rect {
    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
}

#[must_use]
pub fn map_surface_world_to_screen(
    world_tile: Vec2,
    image_rect: egui::Rect,
    camera_center: Vec2,
    zoom: f32,
    _tex_w: f32,
    _tex_h: f32,
) -> egui::Pos2 {
    let z = zoom.clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
    let rel = (world_tile - camera_center) * z;
    let c = image_rect.center();
    egui::pos2(c.x + rel.x, c.y + rel.y)
}

#[must_use]
pub fn map_surface_screen_to_world(
    screen: egui::Pos2,
    image_rect: egui::Rect,
    camera_center: Vec2,
    zoom: f32,
    _tex_w: f32,
    _tex_h: f32,
) -> Vec2 {
    let z = zoom.clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX).max(1e-6);
    let c = image_rect.center();
    let rel = Vec2::new(screen.x - c.x, screen.y - c.y);
    camera_center + rel / z
}

pub fn ensure_viewport_camera_initialized(
    viewport: &mut EditorViewport,
    tex_w: f32,
    tex_h: f32,
) {
    if !viewport.camera_initialized && tex_w > 0.0 && tex_h > 0.0 {
        viewport.reset_camera_for_map(tex_w, tex_h);
    }
}
