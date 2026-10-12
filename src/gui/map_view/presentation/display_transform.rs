//! Canonical consumer-side map display transform (fit truth + painted rects).

use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;

use crate::gui::map_presentation_fit::{compute_map_fit_strict, MapFitMode};
use crate::gui::map_view_projection::{map_presentation_image_rect, map_texture_uv_rect};

#[derive(Component, Clone, Debug)]
pub struct MapDisplayTransform {
    pub expected_viewport_rect: bevy::prelude::Rect,
    pub actual_viewport_rect: bevy::prelude::Rect,
    pub expected_image_rect: bevy::prelude::Rect,
    pub actual_image_rect: bevy::prelude::Rect,
    pub uv_rect: bevy::prelude::Rect,
    pub expected_uv_rect: bevy::prelude::Rect,
    pub scale: f32,
    pub expected_scale: f32,
    pub fit_mode: MapFitMode,
    pub expected_fit_mode: MapFitMode,
    pub zoom: f32,
    pub pan_world: Vec2,
}

impl Default for MapDisplayTransform {
    fn default() -> Self {
        let rect = bevy::prelude::Rect::from_corners(Vec2::ZERO, Vec2::ZERO);
        Self {
            expected_viewport_rect: rect,
            actual_viewport_rect: rect,
            expected_image_rect: rect,
            actual_image_rect: rect,
            uv_rect: bevy::prelude::Rect::from_corners(bevy::prelude::Vec2::new(0.0, 0.0), bevy::prelude::Vec2::new(1.0, 1.0)),
            expected_uv_rect: bevy::prelude::Rect::from_corners(bevy::prelude::Vec2::new(0.0, 0.0), bevy::prelude::Vec2::new(1.0, 1.0)),
            scale: 1.0,
            expected_scale: 1.0,
            fit_mode: MapFitMode::Contain,
            expected_fit_mode: MapFitMode::Contain,
            zoom: 1.0,
            pan_world: Vec2::ZERO,
        }
    }
}

impl MapDisplayTransform {
    pub fn from_fit_truth(
        viewport_rect: bevy::prelude::Rect,
        texture_size: UVec2,
        fit_mode: MapFitMode,
        padding: f32,
        actual_image_rect: bevy::prelude::Rect,
        actual_uv_rect: bevy::prelude::Rect,
        camera_center: Vec2,
        zoom: f32,
        // GPU preview paints the strict contain-fit quad; pan/zoom live in the render target.
        contain_fit_authoritative: bool,
    ) -> Self {
        let inner = bevy::prelude::Rect::from_corners(
            viewport_rect.min + Vec2::splat(padding),
            viewport_rect.max - Vec2::splat(padding),
        );
        let fit = compute_map_fit_strict(inner, texture_size, fit_mode);
        let tex_w = texture_size.x.max(1) as f32;
        let tex_h = texture_size.y.max(1) as f32;
        let expected_image_rect = if contain_fit_authoritative {
            fit.image_rect
        } else {
            map_presentation_image_rect(fit.image_rect, camera_center, zoom, tex_w, tex_h)
        };
        let expected_uv_rect = if contain_fit_authoritative {
            fit.uv_rect
        } else {
            map_texture_uv_rect()
        };
        Self {
            expected_viewport_rect: inner,
            actual_viewport_rect: inner,
            expected_image_rect,
            actual_image_rect,
            uv_rect: actual_uv_rect,
            expected_uv_rect,
            scale: fit.scale,
            expected_scale: fit.scale,
            fit_mode,
            expected_fit_mode: fit_mode,
            zoom,
            pan_world: camera_center,
        }
    }
}

#[allow(dead_code)] // Editor map-view debug stroke; WorldGen preview no longer paints through egui.
pub fn paint_map_display_debug_outlines() {}
