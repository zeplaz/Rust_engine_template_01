//! Tactical map viewport indicator on the simulation minimap (MINIMAP-UX-002).
//!
//! Draws a frame on the minimap raster showing where [`crate::gui::ViewId::WorldMain`]
//! (main map camera) is looking at the current zoom level.

use bevy::math::{Rect, Vec2};
use bevy_egui::egui;

use crate::gui::map_camera::sim_map_visible_world_span;
use crate::gui::style::UiPalette;
use crate::gui::{MapCameraDesired, SimulationMapViewport, ViewId, ViewManager};

/// Map world-tile coordinates into minimap image space (after strict fit + UV crop).
#[must_use]
pub fn world_tile_to_minimap_screen(
    world_xy: Vec2,
    tex_w: f32,
    tex_h: f32,
    image_rect: egui::Rect,
    sample_uv: egui::Rect,
) -> egui::Pos2 {
    let u = (world_xy.x / tex_w.max(1.0)).clamp(0.0, 1.0);
    let v = (world_xy.y / tex_h.max(1.0)).clamp(0.0, 1.0);
    let uv = egui::pos2(u, v);
    let uv_w = sample_uv.width().max(1e-6);
    let uv_h = sample_uv.height().max(1e-6);
    let rel_u = (uv.x - sample_uv.min.x) / uv_w;
    let rel_v = (uv.y - sample_uv.min.y) / uv_h;
    egui::pos2(
        image_rect.min.x + rel_u * image_rect.width(),
        image_rect.min.y + rel_v * image_rect.height(),
    )
}

/// Visible world-tile AABB for the tactical (WorldMain) camera.
///
/// Uses the same [`sim_map_visible_world_span`] contract as the main map ortho hole so the
/// minimap frame tracks pan/zoom when the measured sim-map viewport is valid.
#[must_use]
pub fn tactical_visible_world_rect(
    manager: &ViewManager,
    desired: &MapCameraDesired,
    sim_viewport: &SimulationMapViewport,
    tex_w: f32,
    tex_h: f32,
) -> Option<Rect> {
    let (cam, zoom) = manager
        .view(ViewId::WorldMain)
        .map(|v| (v.camera.translation, v.camera.zoom))
        .unwrap_or_else(|| (desired.translation.truncate(), desired.scale.x));

    if sim_viewport.is_adequate_for_camera() {
        let zoom = zoom.max(1e-4);
        let (fw, fh) = sim_map_visible_world_span(sim_viewport, zoom, tex_w, tex_h);
        if fw > 1e-3 && fh > 1e-3 {
            return Some(clamp_world_rect_to_map(
                Rect::from_center_half_size(cam, Vec2::new(fw * 0.5, fh * 0.5)),
                tex_w,
                tex_h,
            ));
        }
    }

    // Early boot: ViewManager rect before sim viewport measure is ready.
    if let Some(view) = manager.view(ViewId::WorldMain) {
        let r = view.visible_world_rect();
        if r.width() > 1e-3 && r.height() > 1e-3 {
            return Some(clamp_world_rect_to_map(r, tex_w, tex_h));
        }
    }
    None
}

#[inline]
fn clamp_world_rect_to_map(rect: Rect, tex_w: f32, tex_h: f32) -> Rect {
    let max = Vec2::new(tex_w.max(1.0), tex_h.max(1.0));
    Rect::from_corners(
        Vec2::new(rect.min.x.clamp(0.0, max.x), rect.min.y.clamp(0.0, max.y)),
        Vec2::new(rect.max.x.clamp(0.0, max.x), rect.max.y.clamp(0.0, max.y)),
    )
}

/// Screen-space rectangle for the tactical viewport on the minimap image.
#[must_use]
pub fn tactical_viewport_screen_rect(
    world_rect: Rect,
    tex_w: f32,
    tex_h: f32,
    image_rect: egui::Rect,
    sample_uv: egui::Rect,
) -> egui::Rect {
    let corners = [
        Vec2::new(world_rect.min.x, world_rect.min.y),
        Vec2::new(world_rect.max.x, world_rect.min.y),
        Vec2::new(world_rect.max.x, world_rect.max.y),
        Vec2::new(world_rect.min.x, world_rect.max.y),
    ];
    let mut min = egui::pos2(f32::INFINITY, f32::INFINITY);
    let mut max = egui::pos2(f32::NEG_INFINITY, f32::NEG_INFINITY);
    for c in corners {
        let p = world_tile_to_minimap_screen(c, tex_w, tex_h, image_rect, sample_uv);
        min = min.min(p);
        max = max.max(p);
    }
    egui::Rect::from_min_max(min, max)
}

/// Gold wire frame + light fill: "you are here / this is your zoom window" on the minimap.
pub fn paint_tactical_viewport_frame_on_minimap(
    painter: &egui::Painter,
    palette: &UiPalette,
    manager: &ViewManager,
    desired: &MapCameraDesired,
    sim_viewport: &SimulationMapViewport,
    tex_w: f32,
    tex_h: f32,
    image_rect: egui::Rect,
    sample_uv: egui::Rect,
) {
    let Some(world_rect) = tactical_visible_world_rect(manager, desired, sim_viewport, tex_w, tex_h)
    else {
        return;
    };
    let mut frame = tactical_viewport_screen_rect(world_rect, tex_w, tex_h, image_rect, sample_uv);
    if frame.width() < 0.5 || frame.height() < 0.5 {
        return;
    }
    // Only bump sub-pixel frames so zoom/pan tracking stays faithful when zoomed in.
    const MIN_VISIBLE_PX: f32 = 2.0;
    if frame.width() < MIN_VISIBLE_PX || frame.height() < MIN_VISIBLE_PX {
        let cx = frame.center().x;
        let cy = frame.center().y;
        let scale = (MIN_VISIBLE_PX / frame.width().max(frame.height())).max(1.0);
        let w = (frame.width() * scale).min(image_rect.width());
        let h = (frame.height() * scale).min(image_rect.height());
        frame = egui::Rect::from_center_size(egui::pos2(cx, cy), egui::vec2(w, h));
    }
    let frame = frame.intersect(image_rect);
    if frame.width() < 2.0 || frame.height() < 2.0 {
        return;
    }
    let fill = egui::Color32::from_rgba_unmultiplied(255, 220, 100, 48);
    let stroke = egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 240, 160));
    let _ = palette;
    painter.rect_filled(frame, 0.0, fill);
    painter.rect_stroke(frame, 0.0, stroke, egui::StrokeKind::Outside);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_tile_corners_map_inside_image_rect() {
        let image = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(200.0, 100.0));
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        let p0 = world_tile_to_minimap_screen(Vec2::ZERO, 320.0, 320.0, image, uv);
        let p1 = world_tile_to_minimap_screen(Vec2::new(320.0, 320.0), 320.0, 320.0, image, uv);
        assert!((p0.x - 10.0).abs() < 0.5);
        assert!((p0.y - 20.0).abs() < 0.5);
        assert!((p1.x - 210.0).abs() < 0.5);
        assert!((p1.y - 120.0).abs() < 0.5);
    }
}
