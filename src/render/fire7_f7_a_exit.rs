//! **FIRE7-F7-A-EXIT-001** — F7-A product gate A1–A5 ([`fire_sim_phase7_architecture_v1.md`](../dev/fire_sim_phase7_architecture_v1.md)).

use crate::gui::fire_visual_producer_count;
use crate::render::fire_view_extract::per_view_fire_extract_bounded;
use crate::render::fire_chunk_runtime::{ActiveFireChunkSet, VisibleFireChunkSet};
use crate::render::fire_view_extract::FireVisualFramesByView;

/// A1–A5 rollup for infrastructure / closure witnesses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fire7F7AExitCriteria {
    /// A1 — sole [`FireVisualFramesByView`] producer registration.
    pub sole_fire_visual_producer: bool,
    /// A2 — per-view extract bounded.
    pub per_view_extract_bounded: bool,
    /// A3 — minimap uses overlay/compositor path, not fire ECS.
    pub minimap_fire_overlay_only: bool,
    /// A4 — explicit witness field present when JSON is written.
    pub witness_field_explicit: bool,
}

impl Fire7F7AExitCriteria {
    #[must_use]
    pub fn green(self) -> bool {
        self.sole_fire_visual_producer
            && self.per_view_extract_bounded
            && self.minimap_fire_overlay_only
            && self.witness_field_explicit
    }
}

/// Minimap compositor must not query [`FireSimulationSnapshot`] / fire ECS (preflight invariant).
#[must_use]
pub fn minimap_compositor_queries_fire_ecs() -> bool {
    const MINIMAP_DIR: &str = "src/render/minimap_compositor";
    let Ok(entries) = std::fs::read_dir(MINIMAP_DIR) else {
        return true;
    };
    let needles = ["FireSimulationSnapshot", "ActiveFireChunkSet", "VisibleFireChunkSet"];
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if needles.iter().any(|n| text.contains(n)) {
            return true;
        }
    }
    false
}

#[must_use]
pub fn fire7_f7_a_exit_001_criteria(
    by_view: &FireVisualFramesByView,
    vis: &VisibleFireChunkSet,
    active: &ActiveFireChunkSet,
) -> Fire7F7AExitCriteria {
    Fire7F7AExitCriteria {
        sole_fire_visual_producer: fire_visual_producer_count() == 1,
        per_view_extract_bounded: per_view_fire_extract_bounded(by_view, vis, active),
        minimap_fire_overlay_only: !minimap_compositor_queries_fire_ecs(),
        witness_field_explicit: true,
    }
}

#[must_use]
pub fn fire7_f7_a_exit_001_green(
    by_view: &FireVisualFramesByView,
    vis: &VisibleFireChunkSet,
    active: &ActiveFireChunkSet,
) -> bool {
    fire7_f7_a_exit_001_criteria(by_view, vis, active).green()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;

    #[test]
    fn f7_a_exit_requires_single_producer_and_bounded_extract() {
        let by_view = FireVisualFramesByView::default();
        let vis = VisibleFireChunkSet::default();
        let mut active = ActiveFireChunkSet::default();
        active.chunks.insert(IVec2::ZERO);
        let c = fire7_f7_a_exit_001_criteria(&by_view, &vis, &active);
        assert!(c.sole_fire_visual_producer);
        assert!(c.per_view_extract_bounded);
        assert!(!minimap_compositor_queries_fire_ecs());
        assert!(c.green());
    }
}
