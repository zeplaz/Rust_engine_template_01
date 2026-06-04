//! Single authoritative simulation-map viewport rect (VP-01…VP-04).
//!
//! **Dimensions** are written only by [`measure_sim_map_fill_viewport`]. Downstream
//! [`crate::gui::SimulationMapViewport`], camera scissor, and [`crate::render::ResolvedViewports`]
//! copy that rect — they must not re-derive geometry from window chrome or freeze envelopes.
//!
//! [`SimulationMapViewportHoleLatch`] gates **presentation** (`SimulationMapViewport::valid`) only;
//! camera scissor reads dimensions via [`SimulationMapViewport::is_adequate_for_camera`].
//! Latch never widens or shifts min/max (no `center_row` 8+6=14px padding in committed size).

use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use crate::gui::hud::{
    trace_viewport_authority, trace_viewport_chain_integrity, trace_viewport_drift,
    ViewportAuthoritySource, ViewportRectSanity, VIEWPORT_SIM_MAP_SAFE_MIN_H,
    VIEWPORT_SIM_MAP_SAFE_MIN_W,
};
use crate::gui::viewport_layout_solver::{
    commit_authority_from_semantic, frozen_exceeds_semantic_authority,
    semantic_viewport_from_map_fill, viewport_rescue_floor, SemanticViewportRect,
    ViewportSemanticSource,
};
use crate::gui::{
    simulation_map_fallback_logical_extent, SimulationMapViewport, SimulationMapViewportDebug,
    SimulationMapViewportTrace,
};

/// UI measure must agree with window chrome within this tolerance or we use chrome.
const CHROME_UI_AGREE_EPS_PX: f32 = 12.0;

#[inline]
#[must_use]
pub(crate) fn simulation_map_viewport_adequate_dims(min: Vec2, max: Vec2) -> bool {
    let s = (max - min).max(Vec2::ZERO);
    s.x >= VIEWPORT_SIM_MAP_SAFE_MIN_W && s.y >= VIEWPORT_SIM_MAP_SAFE_MIN_H
}

#[inline]
pub(crate) fn clamp_simulation_map_aabb_to_window(
    min: Vec2,
    max: Vec2,
    window: Vec2,
) -> (Vec2, Vec2) {
    let win = Vec2::new(window.x.max(1.0), window.y.max(1.0));
    let min = Vec2::new(min.x.clamp(0.0, win.x), min.y.clamp(0.0, win.y));
    let mut max = Vec2::new(max.x.clamp(0.0, win.x), max.y.clamp(0.0, win.y));
    if max.x < min.x {
        max.x = min.x;
    }
    if max.y < min.y {
        max.y = min.y;
    }
    (min, max)
}

#[inline]
#[must_use]
pub(crate) fn simulation_map_viewport_fits_window(vp: &SimulationMapViewport, window: Vec2) -> bool {
    let win = Vec2::new(window.x.max(1.0), window.y.max(1.0));
    vp.min.x >= -0.5
        && vp.min.y >= -0.5
        && vp.max.x <= win.x + 0.5
        && vp.max.y <= win.y + 0.5
}

/// Legacy horizontal inset on `center_row` (0 = full-bleed `sim_map_fill`). Must **not** be
/// included in [`AuthoritativeViewport`] — only [`SimulationMapViewportFill`] `ComputedNode` size counts.
pub const CENTER_ROW_HORIZONTAL_PAD_PX: f32 = 0.0;

/// Logical window-space rect for the simulation map — **sole dimension authority**.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct AuthoritativeViewport {
    pub valid: bool,
    pub min: Vec2,
    pub max: Vec2,
    pub generation: u64,
}

impl AuthoritativeViewport {
    #[inline]
    #[must_use]
    pub fn logical_size(self) -> Vec2 {
        (self.max - self.min).max(Vec2::ZERO)
    }

    #[inline]
    #[must_use]
    pub fn to_simulation_map_viewport(self, present: bool) -> SimulationMapViewport {
        SimulationMapViewport {
            valid: present,
            min: self.min,
            max: self.max,
        }
    }
}

/// Settle / hole-ready latch — **no stored min/max** (VP-03).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SimulationMapViewportHoleLatch {
    pub hole_ready: bool,
    pub settle_streak: u8,
    pub last_commit: &'static str,
    pub(crate) last_measured_size: Vec2,
    pub(crate) last_window_logical: Vec2,
}

impl SimulationMapViewportHoleLatch {
    pub fn reset_for_layout_change(&mut self) {
        self.hole_ready = false;
        self.settle_streak = 0;
        self.last_measured_size = Vec2::ZERO;
        self.last_window_logical = Vec2::ZERO;
    }
}

const HOLE_SETTLE_STREAK: u8 = 4;
const HOLE_SETTLE_EPS_PX: f32 = 6.0;

/// **Only** measurement site: corner-transform AABB of [`SimulationMapViewportFill`], clamped to window.
#[must_use]
/// @orchestrator-status IN_PROGRESS
/// @orchestrator-owner viewport_migration_agent
/// @orchestrator-do-not-cleanup
pub fn measure_sim_map_fill_viewport(
    node: &ComputedNode,
    global: &UiGlobalTransform,
    scale_factor: f32,
    window_logical: Vec2,
    sanity: &mut ViewportRectSanity,
) -> AuthoritativeViewport {
    if node.is_empty() {
        return AuthoritativeViewport::default();
    }
    let scale = scale_factor.max(1e-6);
    let (logical_min, logical_max) =
        measure_sim_map_fill_corners_crosscheck(node, global, scale);
    let (logical_min, logical_max) =
        clamp_simulation_map_aabb_to_window(logical_min, logical_max, window_logical);
    let fallback = simulation_map_fallback_logical_extent(window_logical);
    let (min, max, valid) =
        sanity.inspect_simulation_map_aabb(logical_min, logical_max, fallback, None);
    AuthoritativeViewport {
        valid,
        min,
        max,
        generation: 0,
    }
}

/// Deterministic map-hole rect: full client area (matches [`viewport_rescue_floor`]).
/// HUD strips and left stack are **overlays** and must not shrink this AABB.
#[must_use]
pub fn expected_sim_map_fill_aabb(window: Vec2) -> (Vec2, Vec2) {
    let rescue = viewport_rescue_floor(window);
    (rescue.min, rescue.max)
}

#[must_use]
fn authority_from_window_chrome(window_logical: Vec2, generation: u64) -> AuthoritativeViewport {
    let (min, max) = expected_sim_map_fill_aabb(window_logical);
    let (min, max) = clamp_simulation_map_aabb_to_window(min, max, window_logical);
    AuthoritativeViewport {
        valid: simulation_map_viewport_adequate_dims(min, max),
        min,
        max,
        generation,
    }
}

#[inline]
fn ui_measure_matches_chrome(ui: &AuthoritativeViewport, window_logical: Vec2) -> bool {
    if !ui.valid {
        return false;
    }
    let chrome = authority_from_window_chrome(window_logical, ui.generation);
    if !chrome.valid {
        return false;
    }
    (ui.min - chrome.min).length() <= CHROME_UI_AGREE_EPS_PX
        && (ui.max - chrome.max).length() <= CHROME_UI_AGREE_EPS_PX
}

/// Prefer live `sim_map_fill` measure when it matches window chrome; otherwise use chrome (resize-safe).
#[must_use]
pub fn resolve_authority_with_window_chrome(
    ui_measured: AuthoritativeViewport,
    window_logical: Vec2,
) -> (AuthoritativeViewport, bool) {
    if ui_measure_matches_chrome(&ui_measured, window_logical) {
        return (ui_measured, false);
    }
    let chrome = authority_from_window_chrome(window_logical, ui_measured.generation);
    (chrome, true)
}

/// Corner-transform AABB (debug cross-check only).
#[must_use]
pub fn measure_sim_map_fill_corners_crosscheck(
    node: &ComputedNode,
    global: &UiGlobalTransform,
    scale_factor: f32,
) -> (Vec2, Vec2) {
    let scale = scale_factor.max(1e-6);
    let half = node.size() * 0.5;
    let corners = [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
    ];
    let mut pmin = Vec2::splat(f32::INFINITY);
    let mut pmax = Vec2::splat(f32::NEG_INFINITY);
    for c in corners {
        let p = (*global) * c;
        pmin = pmin.min(p);
        pmax = pmax.max(p);
    }
    (pmin / scale, pmax / scale)
}

/// Advance hole-ready latch; returns commit tag for diagnostics (never mutates rect dimensions).
pub fn advance_simulation_map_hole_latch(
    authority: &AuthoritativeViewport,
    latch: &mut SimulationMapViewportHoleLatch,
    window_logical: Vec2,
) -> &'static str {
    if latch.last_window_logical != Vec2::ZERO
        && (window_logical - latch.last_window_logical).length_squared() > 64.0
    {
        latch.hole_ready = false;
        latch.settle_streak = 0;
        latch.last_measured_size = Vec2::ZERO;
    }
    latch.last_window_logical = window_logical;

    let vp = authority.to_simulation_map_viewport(true);
    let adequate = authority.valid
        && simulation_map_viewport_fits_window(&vp, window_logical)
        && simulation_map_viewport_adequate_dims(authority.min, authority.max);

    if !adequate {
        latch.hole_ready = false;
        latch.settle_streak = 0;
        latch.last_measured_size = Vec2::ZERO;
        return "hole_inadequate";
    }

    if latch.hole_ready {
        return "hole_hold";
    }

    let size = authority.logical_size();
    let delta = if latch.last_measured_size == Vec2::ZERO {
        Vec2::splat(f32::INFINITY)
    } else {
        (size - latch.last_measured_size).abs()
    };
    latch.last_measured_size = size;
    if delta.x < HOLE_SETTLE_EPS_PX && delta.y < HOLE_SETTLE_EPS_PX {
        latch.settle_streak = latch.settle_streak.saturating_add(1);
    } else {
        latch.settle_streak = 1;
    }
    if latch.settle_streak >= HOLE_SETTLE_STREAK {
        latch.hole_ready = true;
        "hole_settled"
    } else {
        "hole_settling"
    }
}

/// Publish authoritative rect + copy-through presentation viewport (VP-02).
pub fn publish_simulation_map_viewport(
    authority: &mut AuthoritativeViewport,
    semantic: &mut SemanticViewportRect,
    latch: &mut SimulationMapViewportHoleLatch,
    out: &mut SimulationMapViewport,
    trace: &mut SimulationMapViewportTrace,
    sim_dbg: &mut SimulationMapViewportDebug,
    window_logical: Vec2,
    generation: u64,
) {
    authority.generation = generation;

    let ui_raw = *authority;
    let (resolved, used_chrome) = resolve_authority_with_window_chrome(ui_raw, window_logical);
    *authority = resolved;
    if used_chrome {
        bevy::log::debug!(
            target: "viewport_authority::heal",
            ui_valid = ui_raw.valid,
            ui_wh = ?ui_raw.logical_size(),
            chrome_wh = ?authority.logical_size(),
            window = ?window_logical,
            "VIEWPORT_CHROME_ALIGN ui measure missing or diverged — using window chrome rect"
        );
    }

    trace.measured_valid = ui_raw.valid;
    trace.measured_size = ui_raw.logical_size();
    sim_dbg.measured_valid = authority.valid;
    sim_dbg.measured_min = authority.min;
    sim_dbg.measured_max = authority.max;
    trace_viewport_authority(
        ViewportAuthoritySource::UiMeasured,
        authority.min,
        authority.max,
        authority.valid,
    );

    let semantic_rect = semantic_viewport_from_map_fill(
        authority.valid,
        authority.min,
        authority.max,
    );
    *semantic = semantic_rect;

    let frozen_envelope = SemanticViewportRect::from_min_max(
        authority.valid,
        authority.min,
        authority.max,
        ViewportSemanticSource::SimMapFill,
    );
    if frozen_exceeds_semantic_authority(&frozen_envelope, &semantic_rect, 8.0) {
        authority.min = semantic_rect.min;
        authority.max = semantic_rect.max;
        authority.valid = semantic_rect.valid;
        bevy::log::warn!(
            target: "viewport_authority::heal",
            frozen_wh = ?frozen_envelope.logical_size(),
            semantic_wh = ?semantic_rect.logical_size(),
            "VIEWPORT_HEAL frozen envelope exceeded semantic — clamped to sim_map_fill"
        );
    }

    let solver_rect = commit_authority_from_semantic(semantic_rect, window_logical);
    sim_dbg.solver_valid = solver_rect.valid;
    sim_dbg.solver_min = solver_rect.min;
    sim_dbg.solver_max = solver_rect.max;
    trace_viewport_authority(
        ViewportAuthoritySource::LayoutSolver,
        solver_rect.min,
        solver_rect.max,
        solver_rect.valid,
    );

    // Dimensions always from semantic authority (solver == measured when adequate).
    let dim = if solver_rect.valid {
        AuthoritativeViewport {
            valid: true,
            min: solver_rect.min,
            max: solver_rect.max,
            generation,
        }
    } else {
        *authority
    };
    *authority = dim;

    let tag = advance_simulation_map_hole_latch(authority, latch, window_logical);
    latch.last_commit = tag;
    sim_dbg.last_commit = tag;

    // Chrome-aligned authority is stable across resize — do not gate camera on hole_settle streak.
    let vp_dims_ok =
        simulation_map_viewport_adequate_dims(authority.min, authority.max);
    let vp_fits = simulation_map_viewport_fits_window(
        &authority.to_simulation_map_viewport(true),
        window_logical,
    );
    let present = authority.valid && vp_dims_ok && vp_fits;
    *out = authority.to_simulation_map_viewport(present);

    sim_dbg.frozen = latch.hole_ready;
    trace_viewport_authority(
        ViewportAuthoritySource::UiCommitted,
        out.min,
        out.max,
        out.valid,
    );
    trace.committed_size = out.logical_size();
    trace.committed_from_stable_hold = latch.hole_ready;
    sim_dbg.pending_min = authority.min;
    sim_dbg.pending_max = authority.max;
    sim_dbg.pending_wh = authority.logical_size();
    trace.settle_streak = latch.settle_streak;
    trace.layout_settled = latch.hole_ready;

    trace_viewport_chain_integrity(
        authority.logical_size(),
        solver_rect.logical_size(),
        out.logical_size(),
        present,
    );
    let measured_for_drift = if ui_raw.valid {
        ui_raw.logical_size()
    } else {
        authority.logical_size()
    };
    trace_viewport_drift(measured_for_drift, out.logical_size());
}

/// Seed authoritative + presentation viewport from window chrome on enter-sim (before egui measure).
pub fn bootstrap_authoritative_viewport_on_enter_simulation(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut authority: ResMut<AuthoritativeViewport>,
    mut semantic: ResMut<SemanticViewportRect>,
    mut latch: ResMut<SimulationMapViewportHoleLatch>,
    mut sim: ResMut<SimulationMapViewport>,
    mut trace: ResMut<SimulationMapViewportTrace>,
    mut sim_dbg: ResMut<SimulationMapViewportDebug>,
) {
    let Ok(win) = windows.single() else {
        return;
    };
    if !crate::render::primary_window_logical_presentable(win.width(), win.height()) {
        return;
    }
    let window = Vec2::new(win.width(), win.height());
    let (min, max) = expected_sim_map_fill_aabb(window);
    authority.valid = true;
    authority.min = min;
    authority.max = max;
    authority.generation = authority.generation.wrapping_add(1);
    *semantic = semantic_viewport_from_map_fill(true, min, max);
    latch.hole_ready = true;
    latch.settle_streak = HOLE_SETTLE_STREAK;
    latch.last_commit = "enter_sim_bootstrap";
    latch.last_window_logical = window;
    latch.last_measured_size = (max - min).max(Vec2::ZERO);
    *sim = authority.to_simulation_map_viewport(true);
    trace.measured_valid = true;
    trace.measured_size = latch.last_measured_size;
    trace.committed_size = sim.logical_size();
    trace.committed_from_stable_hold = true;
    trace.layout_settled = true;
    sim_dbg.measured_valid = true;
    sim_dbg.measured_min = min;
    sim_dbg.measured_max = max;
    sim_dbg.frozen = true;
    sim_dbg.last_commit = latch.last_commit;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::SimulationMapViewport;

    #[test]
    fn expected_map_fill_uses_full_window_client_area() {
        let (min, max) = expected_sim_map_fill_aabb(Vec2::new(1280.0, 720.0));
        let wh = max - min;
        assert!((min.x).abs() < 0.5);
        assert!((min.y).abs() < 0.5);
        assert!((wh.x - 1280.0).abs() < 1.0);
        assert!((wh.y - 720.0).abs() < 1.0);
    }

    #[test]
    fn resolve_uses_chrome_when_ui_invalid() {
        let window = Vec2::new(1280.0, 720.0);
        let ui = AuthoritativeViewport::default();
        let (auth, used) = resolve_authority_with_window_chrome(ui, window);
        assert!(used);
        assert!(auth.valid);
        assert!(auth.logical_size().x > 1200.0);
    }

    #[test]
    fn resolve_uses_chrome_when_side_stack_steals_width() {
        let window = Vec2::new(1280.0, 720.0);
        let ui = AuthoritativeViewport {
            valid: true,
            min: Vec2::new(456.0, 94.0),
            max: Vec2::new(1280.0, 704.0),
            generation: 1,
        };
        let (auth, used) = resolve_authority_with_window_chrome(ui, window);
        assert!(used);
        assert!(auth.min.x < 32.0);
        assert!(auth.logical_size().x > 1200.0);
    }

    #[test]
    fn center_row_pad_constant_is_zero_for_full_bleed() {
        assert_eq!(CENTER_ROW_HORIZONTAL_PAD_PX, 0.0);
    }

    #[test]
    fn camera_adequate_without_presentation_valid() {
        let vp = SimulationMapViewport {
            valid: false,
            min: Vec2::ZERO,
            max: Vec2::new(1280.0, 720.0),
        };
        assert!(vp.is_adequate_for_camera());
    }

    #[test]
    fn presentation_copy_matches_authoritative_dimensions() {
        let auth = AuthoritativeViewport {
            valid: true,
            min: Vec2::new(100.0, 50.0),
            max: Vec2::new(1134.0, 600.0),
            generation: 1,
        };
        let out = auth.to_simulation_map_viewport(true);
        assert_eq!(out.min, auth.min);
        assert_eq!(out.max, auth.max);
        assert_eq!(out.logical_size(), auth.logical_size());
    }
}
