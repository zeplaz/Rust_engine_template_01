//! Chunk-level **dirty hints** and **display policy** for strategic overlays (`chunk_scheduler_runbook_v1` direction).

use std::collections::HashSet;

use bevy::prelude::*;

/// Chunks whose [`super::ChunkStrategicOverlay`](super::ChunkStrategicOverlay) needs detailed coupling
/// (mobility / civilian / recon) this frame. Injectors and graph sync enqueue coords; coupling clears after work.
#[derive(Resource, Debug)]
pub struct StrategicOverlayCouplingScratch {
    pub dirty_chunks: HashSet<IVec2>,
    /// Every **N** frames, run full detail coupling on **Dormant** chunks too (cheap consistency pass).
    pub dormant_refresh_interval: u32,
    pub frame_counter: u32,
}

impl Default for StrategicOverlayCouplingScratch {
    fn default() -> Self {
        Self {
            dirty_chunks: HashSet::new(),
            dormant_refresh_interval: 24,
            frame_counter: 0,
        }
    }
}

impl StrategicOverlayCouplingScratch {
    #[inline]
    pub fn mark_dirty(&mut self, coord: IVec2) {
        self.dirty_chunks.insert(coord);
    }

    #[inline]
    pub fn dormant_global_refresh(&self) -> bool {
        self.dormant_refresh_interval > 0
            && self.frame_counter % self.dormant_refresh_interval == 0
    }
}

/// Runtime toggles for which transport-splat fields are written into overlays (gameplay / diagnostics).
#[derive(Resource, Clone, Debug)]
pub struct StrategicOverlayDisplayPolicy {
    pub apply_routing_congestion: bool,
    pub apply_ew_denial: bool,
}

impl Default for StrategicOverlayDisplayPolicy {
    fn default() -> Self {
        Self {
            apply_routing_congestion: true,
            apply_ew_denial: true,
        }
    }
}
