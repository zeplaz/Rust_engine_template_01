//! Runtime fit truth validation — expected vs painted map rects.

use std::time::Instant;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::MapDisplayTransform;
use crate::gui::MapViewInstanceId;
use crate::gui::MapPresentationDiagnostics;
use crate::render::{intra_update_stall_log, FrameUpdateAttrib};

#[derive(Clone, Debug)]
pub struct MapFitValidation {
    pub expected_image_rect: egui::Rect,
    pub actual_image_rect: egui::Rect,
    pub expected_viewport_rect: egui::Rect,
    pub actual_viewport_rect: egui::Rect,
    pub expected_uv_rect: egui::Rect,
    pub actual_uv_rect: egui::Rect,
    pub delta_pixels: f32,
    pub uv_delta: f32,
    pub mismatch: bool,
}

impl MapFitValidation {
    #[must_use]
    pub fn compare(
        expected_viewport: egui::Rect,
        actual_viewport: egui::Rect,
        expected_image: egui::Rect,
        actual_image: egui::Rect,
        expected_uv: egui::Rect,
        actual_uv: egui::Rect,
    ) -> Self {
        let delta_pixels = rect_delta_pixels(expected_image, actual_image)
            .max(rect_delta_pixels(expected_viewport, actual_viewport));
        let uv_delta = uv_delta_max(expected_uv, actual_uv);
        Self {
            expected_image_rect: expected_image,
            actual_image_rect: actual_image,
            expected_viewport_rect: expected_viewport,
            actual_viewport_rect: actual_viewport,
            expected_uv_rect: expected_uv,
            actual_uv_rect: actual_uv,
            delta_pixels,
            uv_delta,
            mismatch: delta_pixels > 1.0 || uv_delta > 0.0001,
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct MapFitValidationLog {
    pub world_preview: Option<MapFitValidation>,
    pub minimap: Option<MapFitValidation>,
    pub preview_scale: f32,
    pub minimap_scale: f32,
    pub expected_scale: f32,
    pub fit_mode_mismatch: bool,
    pub mismatch_frames: u64,
    preview_mismatch_logged: bool,
    minimap_mismatch_logged: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapFitConsumerTag(pub MapViewInstanceId);

pub fn validate_map_fit_system(
    query: Query<(&MapFitConsumerTag, &MapDisplayTransform)>,
    mut diagnostics: ResMut<MapPresentationDiagnostics>,
    mut log: ResMut<MapFitValidationLog>,
    update_attrib: Option<ResMut<FrameUpdateAttrib>>,
    dev_overlay: Option<Res<crate::gui::hud::HudDevOverlayState>>,
    spike_guard: Option<Res<crate::engine::UxFrameSpikeGuard>>,
) {
    if spike_guard.is_some_and(|g| g.suppress_optional_diagnostics) {
        return;
    }
    let t0 = Instant::now();
    let mut preview_scale = 0.0;
    let mut minimap_scale = 0.0;
    let mut expected_scale = 0.0;
    let mut fit_mode_mismatch = false;
    let mut mismatches = 0u32;

    for (tag, transform) in &query {
        let validation = MapFitValidation::compare(
            transform.expected_viewport_rect,
            transform.actual_viewport_rect,
            transform.expected_image_rect,
            transform.actual_image_rect,
            transform.expected_uv_rect,
            transform.uv_rect,
        );
        if validation.mismatch {
            mismatches = mismatches.saturating_add(1);
            let logged = match tag.0 {
                MapViewInstanceId::Minimap => &mut log.minimap_mismatch_logged,
                _ => &mut log.preview_mismatch_logged,
            };
            if !*logged {
                bevy::log::warn!(
                    "MAP FIT MISMATCH DETECTED ({:?}): delta {:.2}px · uv {:.5} · image {:?} vs {:?}",
                    tag.0,
                    validation.delta_pixels,
                    validation.uv_delta,
                    validation.expected_image_rect,
                    validation.actual_image_rect,
                );
                *logged = true;
            }
        } else {
            match tag.0 {
                MapViewInstanceId::Minimap => log.minimap_mismatch_logged = false,
                _ => log.preview_mismatch_logged = false,
            }
        }
        let slot = diagnostics.slot_mut(tag.0);
        slot.validation = Some(validation.clone());
        slot.fit_scale = transform.scale;
        slot.expected_fit_scale = transform.expected_scale;
        match tag.0 {
            MapViewInstanceId::Minimap => {
                log.minimap = Some(validation);
                minimap_scale = transform.scale;
            }
            _ => {
                log.world_preview = Some(validation);
                preview_scale = transform.scale;
            }
        }
        expected_scale = transform.expected_scale;
        fit_mode_mismatch |= transform.fit_mode != transform.expected_fit_mode;
    }

    log.preview_scale = preview_scale;
    log.minimap_scale = minimap_scale;
    log.expected_scale = expected_scale;
    log.fit_mode_mismatch = fit_mode_mismatch;
    if mismatches > 0 {
        log.mismatch_frames = log.mismatch_frames.wrapping_add(1);
    }

    if dev_overlay.is_some_and(|overlay| overlay.visible && overlay.show_map_transform) {
        let same_fit_mode = diagnostics.world_preview.fit_mode == diagnostics.minimap.fit_mode;
        if same_fit_mode && preview_scale > 0.0 && minimap_scale > 0.0 {
            debug_assert!(
                (preview_scale - minimap_scale).abs() < 0.01,
                "preview scale {preview_scale} diverged from minimap scale {minimap_scale}"
            );
        }
    }
    let ms = t0.elapsed().as_secs_f32() * 1000.0;
    if let Some(mut a) = update_attrib {
        a.map_fit_validate_ms = ms;
        intra_update_stall_log("egui_map_fit_validate", ms);
    }
}

#[must_use]
fn uv_delta_max(a: egui::Rect, b: egui::Rect) -> f32 {
    (a.min.x - b.min.x)
        .abs()
        .max((a.min.y - b.min.y).abs())
        .max((a.max.x - b.max.x).abs())
        .max((a.max.y - b.max.y).abs())
}

#[must_use]
fn rect_delta_pixels(a: egui::Rect, b: egui::Rect) -> f32 {
    let center = (a.center() - b.center()).length();
    let size = (a.size() - b.size()).length();
    center.max(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_drift_flags_mismatch() {
        let expected = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        let actual = egui::Rect::from_min_max(egui::pos2(0.1, 0.0), egui::pos2(1.0, 1.0));
        let panel = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
        let validation = MapFitValidation::compare(panel, panel, panel, panel, expected, actual);
        assert!(validation.mismatch);
        assert!(validation.uv_delta > 0.0);
    }
}
