//! Semantic simulation-map viewport — geometry from **`sim_map_fill` only**.
//!
//! The solver does **not** derive viewport from `hud_root` or `window - chrome`.
//! It only applies layout floors and traces semantic ownership.

use bevy::math::Vec2;
use bevy::prelude::Resource;

use crate::gui::hud::viewport_authority_debug::{
    trace_viewport_authority, ViewportAuthoritySource,
};
use crate::gui::hud::{
    VIEWPORT_SIM_MAP_LAYOUT_MIN_H, VIEWPORT_SIM_MAP_LAYOUT_MIN_W, VIEWPORT_SIM_MAP_SAFE_MIN_H,
    VIEWPORT_SIM_MAP_SAFE_MIN_W,
};

/// UI node that owns viewport authority (for traces — catches hud_root vs sim_map_fill bugs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportAuthorityNode {
    HudRoot,
    CenterRow,
    SimMapFill,
    RescueFloor,
}

/// Legacy alias.
pub type ViewportSemanticNode = ViewportAuthorityNode;

/// Where the semantic viewport rect came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ViewportSemanticSource {
    #[default]
    None,
    SimMapFill,
    RescueFloor,
}

/// Authoritative viewport slot in logical window coordinates (`sim_map_fill` semantic).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SemanticViewportRect {
    pub valid: bool,
    pub min: Vec2,
    pub max: Vec2,
    pub source: ViewportSemanticSource,
}

impl SemanticViewportRect {
    #[must_use]
    pub fn from_min_max(valid: bool, min: Vec2, max: Vec2, source: ViewportSemanticSource) -> Self {
        Self {
            valid,
            min,
            max,
            source,
        }
    }

    #[must_use]
    pub fn logical_size(self) -> Vec2 {
        (self.max - self.min).max(Vec2::ZERO)
    }

    #[must_use]
    pub fn to_simulation_map_viewport(self) -> crate::gui::SimulationMapViewport {
        crate::gui::SimulationMapViewport {
            valid: self.valid,
            min: self.min,
            max: self.max,
            ..Default::default()
        }
    }
}

pub fn trace_viewport_solver_target(node: ViewportAuthorityNode, rect: SemanticViewportRect) {
    if !crate::gui::hud::viewport_authority_debug::viewport_authority_debug_enabled() {
        return;
    }
    let wh = rect.logical_size();
    bevy::log::info!(
        target: "viewport_authority::solver",
        ?node,
        valid = rect.valid,
        ?rect.source,
        w = wh.x,
        h = wh.y,
        "VIEWPORT_SOLVER_TARGET"
    );
}

/// Populate semantic viewport from measured `sim_map_fill` AABB (sole geometry read site).
///
/// @orchestrator-status IN_PROGRESS
/// @orchestrator-owner viewport_migration_agent
/// @orchestrator-do-not-cleanup
#[must_use]
pub fn semantic_viewport_from_map_fill(
    measured_valid: bool,
    measured_min: Vec2,
    measured_max: Vec2,
) -> SemanticViewportRect {
    let rect = SemanticViewportRect::from_min_max(
        measured_valid,
        measured_min,
        measured_max,
        ViewportSemanticSource::SimMapFill,
    );
    if rect.valid {
        trace_viewport_solver_target(ViewportAuthorityNode::SimMapFill, rect);
        trace_viewport_authority(
            ViewportAuthoritySource::LayoutSolver,
            rect.min,
            rect.max,
            true,
        );
    }
    rect
}

/// Floor-only stabilizer — never widens beyond measured; only raises inadequate axes to layout minimums.
#[must_use]
pub fn stabilize_viewport_floor(
    measured: SemanticViewportRect,
    layout_min: Vec2,
) -> SemanticViewportRect {
    if !measured.valid {
        return measured;
    }
    let wh = measured.logical_size();
    let min = measured.min;
    let mut max = measured.max;
    if wh.x < layout_min.x {
        max.x = min.x + layout_min.x;
    }
    if wh.y < layout_min.y {
        max.y = min.y + layout_min.y;
    }
    SemanticViewportRect::from_min_max(true, min, max, measured.source)
}

#[inline]
#[must_use]
fn measured_sim_map_adequate(wh: Vec2) -> bool {
    wh.x >= VIEWPORT_SIM_MAP_SAFE_MIN_W && wh.y >= VIEWPORT_SIM_MAP_SAFE_MIN_H
}

/// Commit authority: semantic `sim_map_fill` + optional rescue when measure invalid.
#[must_use]
pub fn commit_authority_from_semantic(
    semantic: SemanticViewportRect,
    window: Vec2,
) -> SemanticViewportRect {
    let layout_min = Vec2::new(
        VIEWPORT_SIM_MAP_LAYOUT_MIN_W,
        VIEWPORT_SIM_MAP_LAYOUT_MIN_H,
    );
    let stabilized = stabilize_viewport_floor(semantic, layout_min);

    if stabilized.valid && measured_sim_map_adequate(stabilized.logical_size()) {
        return stabilized;
    }

    if !semantic.valid {
        let rescue = viewport_rescue_floor(window);
        if rescue.valid {
            trace_viewport_solver_target(ViewportAuthorityNode::RescueFloor, rescue);
            return rescue;
        }
    }

    stabilized
}

/// Collapsed-layout rescue only — not used when `sim_map_fill` measure is valid.
///
/// Full client-area bleed: top HUD strips are **overlays**; they must not shrink the camera hole.
#[must_use]
pub fn viewport_rescue_floor(window: Vec2) -> SemanticViewportRect {
    let win = Vec2::new(window.x.max(1.0), window.y.max(1.0));
    let min_x = 0.0;
    let min_y = 0.0;
    let max_x = win.x.max(min_x + VIEWPORT_SIM_MAP_LAYOUT_MIN_W);
    let max_y = win.y.max(min_y + VIEWPORT_SIM_MAP_LAYOUT_MIN_H);
    SemanticViewportRect::from_min_max(
        max_x > min_x && max_y > min_y,
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, max_y),
        ViewportSemanticSource::RescueFloor,
    )
}

/// When frozen envelope exceeds adequate semantic measure, follow semantic (heal hud_root overshoot).
#[must_use]
#[allow(dead_code)] // hole publish path removed (RTT); retained for layout-solver tests + heal contract
pub fn frozen_exceeds_semantic_authority(
    frozen: &SemanticViewportRect,
    semantic: &SemanticViewportRect,
    eps_px: f32,
) -> bool {
    if !frozen.valid || !semantic.valid {
        return false;
    }
    if !measured_sim_map_adequate(semantic.logical_size()) {
        return false;
    }
    let frozen_wh = frozen.logical_size();
    let sem_wh = semantic.logical_size();
    frozen_wh.x > sem_wh.x + eps_px || frozen_wh.y > sem_wh.y + eps_px
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_matches_map_fill_measure() {
        let min = Vec2::new(18.0, 101.0);
        let max = Vec2::new(680.0, 618.0);
        let s = semantic_viewport_from_map_fill(true, min, max);
        assert_eq!(s.min, min);
        assert_eq!(s.max, max);
        assert_eq!(s.source, ViewportSemanticSource::SimMapFill);
    }

    #[test]
    fn stabilize_does_not_widen_to_hud_root() {
        let min = Vec2::new(18.0, 101.0);
        let max = Vec2::new(680.0, 618.0);
        let s = semantic_viewport_from_map_fill(true, min, max);
        let out = stabilize_viewport_floor(
            s,
            Vec2::new(VIEWPORT_SIM_MAP_LAYOUT_MIN_W, VIEWPORT_SIM_MAP_LAYOUT_MIN_H),
        );
        assert_eq!(out.min, min);
        assert_eq!(out.max, max);
    }

    #[test]
    fn frozen_oversize_semantic_detected() {
        let frozen = SemanticViewportRect::from_min_max(
            true,
            Vec2::ZERO,
            Vec2::new(676.0, 618.0),
            ViewportSemanticSource::SimMapFill,
        );
        let semantic = SemanticViewportRect::from_min_max(
            true,
            Vec2::new(18.0, 101.0),
            Vec2::new(680.0, 618.0),
            ViewportSemanticSource::SimMapFill,
        );
        assert!(frozen_exceeds_semantic_authority(&frozen, &semantic, 8.0));
    }
}
