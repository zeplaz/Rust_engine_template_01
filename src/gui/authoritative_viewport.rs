//! RTT-era simulation map fill helpers.
//!
//! **Canonical rect:** [`crate::gui::sim_map_rtt::SimulationMapFillRect`] ([`SimulationMapViewport`] alias),
//! updated each frame by [`crate::gui::sim_map_rtt::sync_simulation_map_fill_rect_system`].

use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use crate::gui::hud::{ViewportRectSanity, VIEWPORT_SIM_MAP_SAFE_MIN_H, VIEWPORT_SIM_MAP_SAFE_MIN_W};
use crate::gui::sim_map_rtt::SimulationMapFillRect;
use crate::gui::viewport_layout_solver::viewport_rescue_floor;
use crate::gui::{
    simulation_map_fallback_logical_extent, SimulationMapViewportDebug,
    SimulationMapViewportTrace,
};

/// Legacy horizontal inset on `center_row` (0 = full-bleed `sim_map_fill`).
pub const CENTER_ROW_HORIZONTAL_PAD_PX: f32 = 0.0;

/// Deprecated alias — use [`SimulationMapFillRect`] / [`SimulationMapViewport`].
pub type AuthoritativeViewport = SimulationMapFillRect;

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

/// Deterministic bootstrap rect: full client area (matches [`viewport_rescue_floor`]).
#[must_use]
pub fn expected_sim_map_fill_aabb(window: Vec2) -> (Vec2, Vec2) {
    let rescue = viewport_rescue_floor(window);
    (rescue.min, rescue.max)
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

/// Measure [`SimulationMapViewportFill`] corners — used by tests and layout audits.
#[must_use]
pub fn measure_sim_map_fill_viewport(
    node: &ComputedNode,
    global: &UiGlobalTransform,
    scale_factor: f32,
    window_logical: Vec2,
    sanity: &mut ViewportRectSanity,
) -> SimulationMapFillRect {
    if node.is_empty() {
        return SimulationMapFillRect::default();
    }
    let scale = scale_factor.max(1e-6);
    let (logical_min, logical_max) =
        measure_sim_map_fill_corners_crosscheck(node, global, scale);
    let (logical_min, logical_max) =
        clamp_simulation_map_aabb_to_window(logical_min, logical_max, window_logical);
    let fallback = simulation_map_fallback_logical_extent(window_logical);
    let (min, max, valid) =
        sanity.inspect_simulation_map_aabb(logical_min, logical_max, fallback, None);
    SimulationMapFillRect {
        valid,
        min,
        max,
        window_logical,
        ..Default::default()
    }
}

/// Keep debug trace slots aligned with the live RTT fill rect.
pub fn sync_simulation_map_fill_debug_trace(
    fill: &SimulationMapFillRect,
    trace: &mut SimulationMapViewportTrace,
    sim_dbg: &mut SimulationMapViewportDebug,
) {
    let size = fill.logical_size();
    trace.measured_valid = fill.valid;
    trace.measured_size = size;
    trace.committed_size = size;
    trace.committed_from_stable_hold = fill.valid;
    trace.settle_streak = u8::from(fill.valid) * 4;
    trace.layout_settled = fill.valid;
    sim_dbg.measured_valid = fill.valid;
    sim_dbg.measured_min = fill.min;
    sim_dbg.measured_max = fill.max;
    sim_dbg.solver_valid = fill.valid;
    sim_dbg.solver_min = fill.min;
    sim_dbg.solver_max = fill.max;
    sim_dbg.last_commit = "rtt_fill";
    sim_dbg.frozen = fill.valid;
    sim_dbg.pending_min = fill.min;
    sim_dbg.pending_max = fill.max;
    sim_dbg.pending_wh = size;
}

/// Seed fill rect from window chrome on enter-sim (before first UI layout measure).
pub fn bootstrap_authoritative_viewport_on_enter_simulation(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut fill: ResMut<SimulationMapFillRect>,
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
    fill.valid = simulation_map_viewport_adequate_dims(min, max);
    fill.min = min;
    fill.max = max;
    fill.window_logical = window;
    sync_simulation_map_fill_debug_trace(fill.as_ref(), trace.as_mut(), sim_dbg.as_mut());
    sim_dbg.last_commit = "enter_sim_bootstrap";
    trace.committed_from_stable_hold = true;
    trace.layout_settled = true;
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn center_row_pad_constant_is_zero_for_full_bleed() {
        assert_eq!(CENTER_ROW_HORIZONTAL_PAD_PX, 0.0);
    }

    #[test]
    fn fill_rect_adequate_when_dims_safe() {
        let fill = SimulationMapFillRect {
            valid: true,
            min: Vec2::ZERO,
            max: Vec2::new(1280.0, 720.0),
            window_logical: Vec2::new(1280.0, 720.0),
            ..Default::default()
        };
        assert!(fill.is_adequate_for_camera());
    }
}
