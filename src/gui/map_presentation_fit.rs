//! Map image fit policy for egui consumers (presentation only).

use bevy::math::{UVec2, Vec2};
use bevy_egui::egui;

use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::map_view::MapViewState as EditorViewport;
use crate::gui::map_view::MapViewInstanceId;

/// How a map texture is fitted into an egui panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MapFitMode {
    #[default]
    Contain,
    Cover,
    Stretch,
    PixelPerfect,
}

pub type MapAspectMode = MapFitMode;

/// Shared inner inset for preview and minimap map panels.
pub const MAP_PANEL_INSET_PX: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapDisplayResult {
    pub image_rect: egui::Rect,
    pub uv_rect: egui::Rect,
    pub scale: f32,
}

impl MapFitMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Contain => "Contain",
            Self::Cover => "Cover",
            Self::Stretch => "Stretch",
            Self::PixelPerfect => "PixelPerfect",
        }
    }
}

#[must_use]
pub const fn default_fit_mode_for(id: MapViewInstanceId) -> MapFitMode {
    match id {
        MapViewInstanceId::Minimap => MapFitMode::Cover,
        _ => MapFitMode::Contain,
    }
}

/// Canonical strict fit — all map consumers must use this at the render boundary.
#[must_use]
pub fn compute_map_fit_strict(
    viewport: egui::Rect,
    texture_size: UVec2,
    mode: MapAspectMode,
) -> MapDisplayResult {
    let texture_size = egui::vec2(texture_size.x.max(1) as f32, texture_size.y.max(1) as f32);
    if viewport.width() <= 0.0
        || viewport.height() <= 0.0
        || texture_size.x <= 0.0
        || texture_size.y <= 0.0
    {
        return MapDisplayResult {
            image_rect: viewport,
            uv_rect: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            scale: 1.0,
        };
    }

    match mode {
        MapFitMode::Stretch => MapDisplayResult {
            image_rect: viewport,
            uv_rect: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            scale: (viewport.width() / texture_size.x).max(viewport.height() / texture_size.y),
        },
        MapFitMode::PixelPerfect => {
            let scale = (viewport.width() / texture_size.x)
                .min(viewport.height() / texture_size.y)
                .min(1.0);
            let size = texture_size * scale;
            MapDisplayResult {
                image_rect: egui::Rect::from_center_size(viewport.center(), size),
                uv_rect: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                scale,
            }
        }
        MapFitMode::Contain => {
            let scale = (viewport.width() / texture_size.x).min(viewport.height() / texture_size.y);
            let size = texture_size * scale;
            MapDisplayResult {
                image_rect: egui::Rect::from_center_size(viewport.center(), size),
                uv_rect: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                scale,
            }
        }
        MapFitMode::Cover => {
            let tex_aspect = texture_size.x / texture_size.y;
            let panel_aspect = viewport.width() / viewport.height();
            let scale = (viewport.width() / texture_size.x).max(viewport.height() / texture_size.y);
            if tex_aspect > panel_aspect {
                let uv_w = panel_aspect / tex_aspect;
                let u0 = (1.0 - uv_w) * 0.5;
                MapDisplayResult {
                    image_rect: viewport,
                    uv_rect: egui::Rect::from_min_max(egui::pos2(u0, 0.0), egui::pos2(u0 + uv_w, 1.0)),
                    scale,
                }
            } else {
                let uv_h = tex_aspect / panel_aspect;
                let v0 = (1.0 - uv_h) * 0.5;
                MapDisplayResult {
                    image_rect: viewport,
                    uv_rect: egui::Rect::from_min_max(egui::pos2(0.0, v0), egui::pos2(1.0, v0 + uv_h)),
                    scale,
                }
            }
        }
    }
}

/// Letterbox / crop / stretch a texture into a panel rect.
#[must_use]
pub fn map_fit_rect(
    panel: egui::Rect,
    texture_size: egui::Vec2,
    fit: MapFitMode,
    padding: f32,
) -> (egui::Rect, egui::Rect) {
    let fit = compute_map_fit_strict(
        panel.shrink(padding),
        UVec2::new(texture_size.x.max(1.0) as u32, texture_size.y.max(1.0) as u32),
        fit,
    );
    (fit.image_rect, fit.uv_rect)
}

/// Initial zoom so the full map fits inside the panel with a small margin.
#[must_use]
pub fn map_fit_zoom_for_panel(panel: Vec2, tex_w: f32, tex_h: f32, margin: f32) -> f32 {
    if tex_w <= 0.0 || tex_h <= 0.0 || panel.x <= 0.0 || panel.y <= 0.0 {
        return 1.0;
    }
    let scale = margin * (panel.x / tex_w).min(panel.y / tex_h);
    scale.clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX)
}

pub fn fit_viewport_to_map(viewport: &mut EditorViewport, panel: Vec2, tex_w: f32, tex_h: f32) {
    viewport.reset_camera_for_map(tex_w, tex_h);
    viewport.zoom = map_fit_zoom_for_panel(panel, tex_w, tex_h, 0.92);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contain_preserves_aspect_and_stays_inside_panel() {
        let panel = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(400.0, 200.0));
        let (image, uv) = map_fit_rect(panel, egui::vec2(320.0, 320.0), MapFitMode::Contain, 0.0);
        assert!(panel.contains_rect(image));
        assert_eq!(uv, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)));
        assert!((image.width() / image.height() - 1.0).abs() < 0.01);
    }

    #[test]
    fn cover_crops_uv_when_panel_is_wider() {
        let panel = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(400.0, 200.0));
        let (_, uv) = map_fit_rect(panel, egui::vec2(320.0, 320.0), MapFitMode::Cover, 0.0);
        assert!(uv.height() < 1.0);
        assert_eq!(uv.width(), 1.0);
    }
}
