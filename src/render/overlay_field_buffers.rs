//! Shared overlay field buffers for **minimap + world preview** sampling (`base_visual_dev01` P1-G).
//!
//! Chunk heat in [`SharedOverlayFieldBuffers`] is **derived only** from the full sim snapshot
//! [`crate::render::fire_chunk_runtime::FireSimulationSnapshot::chunk_heat`] each frame (global truth, not view-culled).
//! `revision` bumps when that map changes (rounded compare) so previews can invalidate cheaply.
//! [`crate::gui::OverlayFieldFrame::fire_heat_overlay_revision`](crate::gui::OverlayFieldFrame) mirrors this revision for the overlay matrix (T3-C) without duplicating the map.
//! **Invariant:** do not add a second ECS fire scan here — extend the View Representation / [`OverlayFieldFrame`](crate::gui::OverlayFieldFrame) matrix instead.

use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default, Debug, Clone)]
pub struct SharedOverlayFieldBuffers {
    pub stamp: crate::systems::sim_control::SimStepStamp,
    pub revision: u64,
    /// Chunk-grid visual surface heat (0..1), max per chunk — from full [`crate::render::fire_chunk_runtime::FireSimulationSnapshot`], not sim ECS.
    pub chunk_fire_heat: HashMap<IVec2, f32>,
}

/// MAP-BLINK-001 — cold-start overlay ramp length (frames).
pub const OVERLAY_WARMUP_BLEND_FRAMES: u32 = 8;

/// Minimum per-chunk heat delta before overlay revision bumps (reduces minimap/world raster churn).
pub const CHUNK_FIRE_HEAT_OVERLAY_EPS: f32 = 0.028;

/// Chunk heat at or above this value is written to the shared overlay / CPU fire tint.
pub const CHUNK_FIRE_OVERLAY_DISPLAY_MIN: f32 = 0.12;

/// True when `next` differs meaningfully from `prev` (integer millidegree heat per chunk).
#[must_use]
pub fn chunk_fire_heat_maps_differ(
    prev: &HashMap<IVec2, f32>,
    next: &HashMap<IVec2, f32>,
) -> bool {
    chunk_fire_heat_maps_differ_eps(prev, next, CHUNK_FIRE_HEAT_OVERLAY_EPS)
}

/// Like [`chunk_fire_heat_maps_differ`] with a custom epsilon (decay passes use a looser threshold).
#[must_use]
pub fn chunk_fire_heat_maps_differ_eps(
    prev: &HashMap<IVec2, f32>,
    next: &HashMap<IVec2, f32>,
    eps: f32,
) -> bool {
    if prev.len() != next.len() {
        return true;
    }
    for (k, v) in next {
        let old = prev.get(k).copied().unwrap_or(-1.0);
        if (v - old).abs() > eps {
            return true;
        }
    }
    for (k, v) in prev {
        if next.get(k).is_none() && *v > eps {
            return true;
        }
    }
    false
}

impl SharedOverlayFieldBuffers {
    #[inline]
    pub fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    #[inline]
    #[must_use]
    pub fn fire_surface_heat_at(&self, chunk_coord: IVec2) -> f32 {
        self.chunk_fire_heat
            .get(&chunk_coord)
            .copied()
            .unwrap_or(0.0)
    }
}

pub struct SharedOverlayFieldBuffersPlugin;

impl Plugin for SharedOverlayFieldBuffersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SharedOverlayFieldBuffers>();
    }
}
