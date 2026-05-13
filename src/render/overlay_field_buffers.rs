//! Shared overlay field buffers for **minimap + world preview** sampling (`base_visual_dev01` P1-G).
//!
//! Chunk heat in [`SharedOverlayFieldBuffers`] is **derived only** from [`FireVisualFrame::chunk_heat`](crate::render::extraction::FireVisualFrame) each frame;
//! `revision` bumps when that map changes (rounded compare) so previews can invalidate cheaply.

use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default, Debug)]
pub struct SharedOverlayFieldBuffers {
    pub revision: u64,
    /// Chunk-grid visual surface heat (0..1), max per chunk — from fire visual extract, not sim ECS.
    pub chunk_fire_heat: HashMap<IVec2, f32>,
}

/// True when `next` differs meaningfully from `prev` (integer millidegree heat per chunk).
#[must_use]
pub fn chunk_fire_heat_maps_differ(
    prev: &HashMap<IVec2, f32>,
    next: &HashMap<IVec2, f32>,
) -> bool {
    if prev.len() != next.len() {
        return true;
    }
    for (k, v) in next {
        let a = (v * 1000.0).round() as i32;
        let b = (prev.get(k).copied().unwrap_or(-1.0) * 1000.0).round() as i32;
        if a != b {
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
