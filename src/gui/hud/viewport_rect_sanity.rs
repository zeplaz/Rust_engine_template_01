//! Validate and clamp UI layout rects before viewport suggestions are emitted.

use std::collections::HashMap;

use bevy::log::warn;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy_egui::egui;

use super::shell_diagnostics::ProductShellDiagnostics;

/// Width/height at or below this are treated as collapsed egui allocations.
pub const VIEWPORT_RECT_COLLAPSED_MAX: f32 = 4.0;
/// Minimum logical extent for preview/minimap suggestions after clamping.
pub const VIEWPORT_RECT_SAFE_MIN: f32 = 64.0;
/// Bevy UI `min_width` / `min_height` on [`SimulationMapViewportFill`] (flex collapse guard).
pub const VIEWPORT_SIM_MAP_LAYOUT_MIN_W: f32 = 400.0;
pub const VIEWPORT_SIM_MAP_LAYOUT_MIN_H: f32 = 300.0;
/// Minimum logical extent for the simulation map viewport after clamping / camera hole.
pub const VIEWPORT_SIM_MAP_SAFE_MIN_W: f32 = 128.0;
/// Minimum hole **height** before camera scissor + ortho fit (128px strips are layout defects).
pub const VIEWPORT_SIM_MAP_SAFE_MIN_H: f32 = 200.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ViewportRectSource {
    WorldPreviewCentralPanel,
    WorldPreviewEguiWindow,
    MinimapProductShellWindow,
    MinimapPanelSliders,
    SimulationMapViewportFill,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportRectIssueKind {
    NonFinite,
    Negative,
    Collapsed,
    BelowSafeMinimum,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewportRectIssue {
    pub source: ViewportRectSource,
    pub kind: ViewportRectIssueKind,
    pub width: f32,
    pub height: f32,
    pub clamped_width: f32,
    pub clamped_height: f32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct ViewportRectSanity {
    pub issues_total: u64,
    pub issues_by_source: HashMap<ViewportRectSource, u64>,
    pub last_issue: Option<ViewportRectIssue>,
    pub suppressed_logs: u64,
    last_logged: HashMap<ViewportRectSource, (ViewportRectIssueKind, Vec2)>,
}

impl ViewportRectSanity {
    pub fn inspect_egui_rect(
        &mut self,
        rect: egui::Rect,
        source: ViewportRectSource,
        fallback: Vec2,
        diag: Option<&mut ProductShellDiagnostics>,
    ) -> egui::Rect {
        let (size, kind) = classify_viewport_size(rect.width(), rect.height());
        let clamped = clamp_viewport_size(size, fallback, source);
        if let Some(kind) = kind {
            self.record_issue(source, kind, size, clamped, diag);
        }
        egui::Rect::from_min_size(rect.min, egui::vec2(clamped.x, clamped.y))
    }

    pub fn inspect_logical_size(
        &mut self,
        size: Vec2,
        source: ViewportRectSource,
        fallback: Vec2,
        diag: Option<&mut ProductShellDiagnostics>,
    ) -> Vec2 {
        let (measured, kind) = classify_viewport_size(size.x, size.y);
        let clamped = clamp_viewport_size(measured, fallback, source);
        if let Some(kind) = kind {
            self.record_issue(source, kind, measured, clamped, diag);
        }
        clamped
    }

    pub fn inspect_simulation_map_aabb(
        &mut self,
        min: Vec2,
        max: Vec2,
        _fallback_extent: Vec2,
        diag: Option<&mut ProductShellDiagnostics>,
    ) -> (Vec2, Vec2, bool) {
        let size = max - min;
        let (measured, kind) = classify_viewport_size(size.x, size.y);
        if kind.is_none() && measured.x >= VIEWPORT_SIM_MAP_SAFE_MIN_W
            && measured.y >= VIEWPORT_SIM_MAP_SAFE_MIN_H
        {
            return (min, max, true);
        }

        let issue_kind = kind.unwrap_or(ViewportRectIssueKind::BelowSafeMinimum);
        self.record_issue(
            ViewportRectSource::SimulationMapViewportFill,
            issue_kind,
            measured,
            measured,
            diag,
        );
        // Transitional Bevy UI layout (world-gen chrome): do **not** inflate to full-window fallback.
        // `min + fallback` produced wgpu scissor rects outside the swapchain (fatal validation error).
        (min, max, false)
    }

    fn record_issue(
        &mut self,
        source: ViewportRectSource,
        kind: ViewportRectIssueKind,
        size: Vec2,
        clamped: Vec2,
        diag: Option<&mut ProductShellDiagnostics>,
    ) {
        self.issues_total = self.issues_total.wrapping_add(1);
        *self.issues_by_source.entry(source).or_insert(0) += 1;
        self.last_issue = Some(ViewportRectIssue {
            source,
            kind,
            width: size.x,
            height: size.y,
            clamped_width: clamped.x,
            clamped_height: clamped.y,
        });
        if let Some(diag) = diag {
            diag.record_viewport_rect_issue(source, kind);
        }
        let signature = (kind, size);
        if self.last_logged.get(&source) == Some(&signature) {
            self.suppressed_logs = self.suppressed_logs.wrapping_add(1);
            return;
        }
        self.last_logged.insert(source, signature);
        warn!(
            target: "proc_A_dine01::gui::hud::viewport_rect",
            "invalid viewport rect source={source:?} kind={kind:?} size=({:.1},{:.1}) clamped=({:.1},{:.1})",
            size.x,
            size.y,
            clamped.x,
            clamped.y,
        );
    }
}

fn classify_viewport_size(width: f32, height: f32) -> (Vec2, Option<ViewportRectIssueKind>) {
    if !width.is_finite() || !height.is_finite() {
        return (Vec2::ZERO, Some(ViewportRectIssueKind::NonFinite));
    }
    if width < 0.0 || height < 0.0 {
        return (Vec2::new(width.max(0.0), height.max(0.0)), Some(ViewportRectIssueKind::Negative));
    }
    if width <= VIEWPORT_RECT_COLLAPSED_MAX || height <= VIEWPORT_RECT_COLLAPSED_MAX {
        return (Vec2::new(width, height), Some(ViewportRectIssueKind::Collapsed));
    }
    (Vec2::new(width, height), None)
}

fn clamp_viewport_size(size: Vec2, fallback: Vec2, source: ViewportRectSource) -> Vec2 {
    let (min_w, min_h) = match source {
        ViewportRectSource::SimulationMapViewportFill => {
            (VIEWPORT_SIM_MAP_SAFE_MIN_W, VIEWPORT_SIM_MAP_SAFE_MIN_H)
        }
        _ => (VIEWPORT_RECT_SAFE_MIN, VIEWPORT_RECT_SAFE_MIN),
    };
    let fallback_w = fallback.x.max(min_w);
    let fallback_h = fallback.y.max(min_h);
    Vec2::new(
        size.x.max(fallback_w).max(min_w),
        size.y.max(fallback_h).max(min_h),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_rect_clamps_to_fallback() {
        let mut sanity = ViewportRectSanity::default();
        let rect = sanity.inspect_egui_rect(
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(698.0, 1.0)),
            ViewportRectSource::WorldPreviewCentralPanel,
            Vec2::new(320.0, 240.0),
            None,
        );
        assert!(rect.height() >= VIEWPORT_RECT_SAFE_MIN);
    }

    #[test]
    fn simulation_map_aabb_rejects_transitional_tiny_layout() {
        let mut sanity = ViewportRectSanity::default();
        let (min, max, valid) = sanity.inspect_simulation_map_aabb(
            Vec2::new(576.0, 93.0),
            Vec2::new(692.0, 221.0),
            Vec2::new(1280.0, 634.0),
            None,
        );
        assert!(!valid);
        assert!((max.x - min.x) < VIEWPORT_SIM_MAP_SAFE_MIN_W);
        assert_eq!(min, Vec2::new(576.0, 93.0));
        assert_eq!(max, Vec2::new(692.0, 221.0));
    }

    #[test]
    fn simulation_map_aabb_accepts_stable_hole() {
        let mut sanity = ViewportRectSanity::default();
        let (min, max, valid) = sanity.inspect_simulation_map_aabb(
            Vec2::new(576.0, 94.0),
            Vec2::new(1274.0, 320.0),
            Vec2::new(1280.0, 634.0),
            None,
        );
        assert!(valid);
        assert!((max.x - min.x) >= VIEWPORT_SIM_MAP_SAFE_MIN_W);
        assert!((max.y - min.y) >= VIEWPORT_SIM_MAP_SAFE_MIN_H);
    }
}
