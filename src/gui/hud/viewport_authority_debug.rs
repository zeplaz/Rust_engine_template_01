//! Viewport authority tracing + typed debug rects (UI logical vs render physical).
//!
//! @orchestrator-status IN_PROGRESS
//! @orchestrator-owner viewport_migration_agent
//! @orchestrator-do-not-cleanup

use bevy::log::error;
use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub const VIEWPORT_AUTHORITY_TARGET: &str = "viewport_authority";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportAuthoritySource {
    UiMeasured,
    LayoutSolver,
    UiCommitted,
    ResolvedViewport,
    CameraLatch,
    CameraApplied,
}

/// Logical UI-space viewport (window coordinates).
#[derive(Clone, Copy, Debug, Default)]
pub struct UiViewportRect {
    pub logical_min: Vec2,
    pub logical_max: Vec2,
    pub valid: bool,
}

impl UiViewportRect {
    #[must_use]
    pub fn from_sim(sim: &crate::gui::SimulationMapViewport) -> Self {
        Self {
            logical_min: sim.min,
            logical_max: sim.max,
            valid: sim.valid,
        }
    }
}

impl std::fmt::Display for UiViewportRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.valid {
            return write!(f, "invalid");
        }
        let wh = (self.logical_max - self.logical_min).max(Vec2::ZERO);
        write!(
            f,
            "min=({:.0},{:.0}) max=({:.0},{:.0}) wh=({:.0},{:.0})",
            self.logical_min.x,
            self.logical_min.y,
            self.logical_max.x,
            self.logical_max.y,
            wh.x,
            wh.y
        )
    }
}

/// Physical camera scissor (GPU viewport — compare only after scale conversion).
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderViewportRect {
    pub physical_min: UVec2,
    pub physical_size: UVec2,
    pub valid: bool,
}

impl RenderViewportRect {
    #[must_use]
    pub fn to_logical(self, scale: f32) -> UiViewportRect {
        let s = scale.max(1e-6);
        let min = Vec2::new(self.physical_min.x as f32 / s, self.physical_min.y as f32 / s);
        let max = min
            + Vec2::new(
                self.physical_size.x as f32 / s,
                self.physical_size.y as f32 / s,
            );
        UiViewportRect {
            logical_min: min,
            logical_max: max,
            valid: self.valid,
        }
    }
}

impl std::fmt::Display for RenderViewportRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "phys pos=({}, {}) size=({}, {})",
            self.physical_min.x,
            self.physical_min.y,
            self.physical_size.x,
            self.physical_size.y
        )
    }
}

/// Orthographic world span — not comparable to pixel rects.
#[derive(Clone, Copy, Debug, Default)]
pub struct CameraProjectionInfo {
    pub world_width: f32,
    pub world_height: f32,
    pub view_pixels: Vec2,
    pub using_hole: bool,
}

impl std::fmt::Display for CameraProjectionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "world=({:.0},{:.0}) view_px=({:.0},{:.0}) render_hole={}",
            self.world_width,
            self.world_height,
            self.view_pixels.x,
            self.view_pixels.y,
            self.using_hole
        )
    }
}

#[inline]
pub fn viewport_authority_debug_enabled() -> bool {
    crate::dev::test_run_instrumentation::diagnostics_operator_trace_enabled(
        false,
        &[
            "VISUAL_DIAG",
            "VIEWPORT_DEBUG_OVERLAY",
            "VIEWPORT_AUTHORITY_DEBUG",
            "STAGE5_VERBOSE",
        ],
    )
}

#[inline]
pub fn viewport_debug_overlay_enabled() -> bool {
    viewport_authority_debug_enabled()
}

/// Trace a viewport rect from a single authority source.
pub fn trace_viewport_authority(source: ViewportAuthoritySource, min: Vec2, max: Vec2, valid: bool) {
    if !viewport_authority_debug_enabled() {
        return;
    }
    let wh = (max - min).max(Vec2::ZERO);
    info!(
        target: VIEWPORT_AUTHORITY_TARGET,
        ?source,
        valid,
        ?min,
        ?max,
        w = wh.x,
        h = wh.y,
        "VIEWPORT_AUTHORITY"
    );
}

/// Fail loud when measured → solver → committed diverge (adequate presentation only).
pub fn trace_viewport_chain_integrity(
    measured: Vec2,
    solver: Vec2,
    committed: Vec2,
    present: bool,
) {
    if !viewport_authority_debug_enabled() {
        return;
    }
    let dm = (measured - solver).abs();
    let dc = (measured - committed).abs();
    if dm.length() > 0.5 {
        error!(
            target: "viewport_authority::integrity",
            ?measured,
            ?solver,
            ?dm,
            "VIEWPORT_CHAIN solver != measured"
        );
    }
    if present && dc.length() > 0.5 {
        error!(
            target: "viewport_authority::integrity",
            ?measured,
            ?committed,
            ?dc,
            "VIEWPORT_CHAIN committed != measured (dimensions must copy authoritative)"
        );
    }
}

/// Warn when raw UI measure diverges from committed authority (layout drift).
pub fn trace_viewport_drift(measured: Vec2, committed: Vec2) {
    if !viewport_authority_debug_enabled() {
        return;
    }
    let delta = measured - committed;
    if delta.length() > 1.0 {
        let hint = if delta.x.abs() > 8.0 && delta.y.abs() < 2.0 {
            "likely stale freeze width (center_row pad 8+6=14?) or frozen_shrink_ignored"
        } else if delta.y.abs() > 8.0 && delta.x.abs() < 2.0 {
            "likely stale freeze height or chrome strip"
        } else {
            "check AuthoritativeViewport vs SimulationMapViewport copy-through"
        };
        warn!(
            target: "viewport_authority::drift",
            ?delta,
            ?measured,
            ?committed,
            hint,
            "VIEWPORT_DRIFT measured vs committed"
        );
    }
}

/// Debug-only: assert authoritative vs presentation vs camera scissor (VP-06).
pub fn assert_viewport_integrity(
    authority: &crate::gui::AuthoritativeViewport,
    sim: &crate::gui::SimulationMapViewport,
    cam_logical: Option<UiViewportRect>,
) {
    if !viewport_authority_debug_enabled() {
        return;
    }
    if !authority.valid {
        return;
    }
    let auth_wh = authority.logical_size();
    let sim_wh = sim.logical_size();
    let d_sim = (auth_wh - sim_wh).abs();
    if d_sim.length() > 0.5 {
        panic!(
            "viewport integrity: SimulationMapViewport dimensions != AuthoritativeViewport \
             auth={auth_wh:?} sim={sim_wh:?} delta={d_sim:?}"
        );
    }
    if let Some(cam) = cam_logical {
        if cam.valid {
            let cam_wh = (cam.logical_max - cam.logical_min).max(Vec2::ZERO);
            let d_cam = (auth_wh - cam_wh).abs();
            if sim.valid && d_cam.length() > 1.0 {
                panic!(
                    "viewport integrity: camera scissor != authoritative \
                     auth={auth_wh:?} cam={cam_wh:?} delta={d_cam:?}"
                );
            }
        }
    }
}

pub struct ViewportIntegrityAssertPlugin;

impl Plugin for ViewportIntegrityAssertPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            run_viewport_integrity_assert
                .after(crate::gui::SimulationViewportSyncSet::ApplyCameraScissor)
                .run_if(viewport_authority_debug_enabled),
        );
    }
}

fn run_viewport_integrity_assert(
    authority: Res<crate::gui::AuthoritativeViewport>,
    sim: Res<crate::gui::SimulationMapViewport>,
    cam: Query<&Camera, With<crate::gui::MainWorldCamera>>,
    win: Query<&Window, With<PrimaryWindow>>,
) {
    let cam_logical = cam.single().ok().and_then(|c| {
        let scale = win.single().ok()?.scale_factor();
        c.viewport.as_ref().map(|vp| {
            let s = scale.max(1e-6);
            let min = Vec2::new(vp.physical_position.x as f32 / s, vp.physical_position.y as f32 / s);
            let max = min
                + Vec2::new(vp.physical_size.x as f32 / s, vp.physical_size.y as f32 / s);
            UiViewportRect {
                logical_min: min,
                logical_max: max,
                valid: true,
            }
        })
    });
    assert_viewport_integrity(authority.as_ref(), sim.as_ref(), cam_logical);
}

pub fn stroke_viewport_debug_rect(
    painter: &bevy_egui::egui::Painter,
    min: Vec2,
    max: Vec2,
    color: bevy_egui::egui::Color32,
    label: &str,
) {
    if max.x <= min.x || max.y <= min.y {
        return;
    }
    let wh = max - min;
    let text = format!("{label} {:.0}x{:.0}", wh.x, wh.y);
    let r = bevy_egui::egui::Rect::from_min_max(
        bevy_egui::egui::pos2(min.x, min.y),
        bevy_egui::egui::pos2(max.x, max.y),
    );
    painter.rect_stroke(
        r,
        0.0,
        bevy_egui::egui::Stroke::new(3.0, color),
        bevy_egui::egui::StrokeKind::Outside,
    );
    painter.text(
        r.left_top() + bevy_egui::egui::vec2(4.0, 4.0),
        bevy_egui::egui::Align2::LEFT_TOP,
        text,
        bevy_egui::egui::FontId::monospace(12.0),
        color,
    );
}

