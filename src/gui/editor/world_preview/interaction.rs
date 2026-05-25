//! Viewport pointer mapping: zoom-to-cursor and middle-mouse pan.

use bevy_egui::egui;

use crate::gui::map_view_projection::map_surface_screen_to_world;

use super::layers::PreviewLayers;
use super::viewport::EditorViewport;
use bevy::math::{UVec2, Vec2};

pub fn apply_zoom_toward(
    viewport: &mut EditorViewport,
    pointer_screen: egui::Pos2,
    viewport_center: egui::Pos2,
    zoom_factor: f32,
) {
    let z_old = viewport
        .zoom
        .clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
    let z_new = (z_old * zoom_factor).clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
    if (z_new - z_old).abs() < 1e-6 {
        return;
    }
    let rel = pointer_screen - viewport_center;
    let world_under_cursor = viewport.camera_center + Vec2::new(rel.x, rel.y) / z_old;
    viewport.zoom = z_new;
    viewport.camera_center = world_under_cursor - Vec2::new(rel.x, rel.y) / z_new;
}

pub fn apply_pan(viewport: &mut EditorViewport, delta_screen: Vec2) {
    let z = viewport
        .zoom
        .clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
    viewport.camera_center -= delta_screen / z;
}

pub fn update_hover_tile(
    viewport: &mut EditorViewport,
    pointer_screen: Option<egui::Pos2>,
    panel_rect: egui::Rect,
    image_rect: egui::Rect,
    tex_w: u32,
    tex_h: u32,
    zoom: f32,
    camera_center: Vec2,
) {
    let Some(p) = pointer_screen.filter(|p| panel_rect.contains(*p) && image_rect.contains(*p))
    else {
        viewport.hovered_tile = None;
        return;
    };
    let tw = tex_w.max(1) as f32;
    let th = tex_h.max(1) as f32;
    let w = map_surface_screen_to_world(p, image_rect, camera_center, zoom, tw, th);
    let tx = w.x.floor() as i32;
    let ty = w.y.floor() as i32;
    if tx >= 0 && ty >= 0 && (tx as u32) < tex_w && (ty as u32) < tex_h {
        viewport.hovered_tile = Some(UVec2::new(tx as u32, ty as u32));
    } else {
        viewport.hovered_tile = None;
    }
}
