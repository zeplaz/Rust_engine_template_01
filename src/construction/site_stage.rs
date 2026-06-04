//! **CON-P2-002** — staged site progress types (B-owned; **CON-P2-001** attaches on commit in `strategic/site/systems.rs`).

use bevy::prelude::*;

/// Per-phase progress in `[0, 1]`; advances via [`super::site_stage_tick::advance_site_construction_tick_system`].
#[derive(Component, Debug, Clone)]
pub struct SiteStageProgress {
    pub progress: f32,
    pub substep: Option<ClearingSubstep>,
}

impl Default for SiteStageProgress {
    fn default() -> Self {
        Self {
            progress: 0.0,
            substep: None,
        }
    }
}

/// Forest / obstructed clearing pipeline substeps (same `SiteConstructionPhase::Clearing`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearingSubstep {
    Trees,
    Stumps,
}
