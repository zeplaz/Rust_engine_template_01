//! Resolved map-view frame: authoritative consumer output from the shared backend.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bevy::math::{Rect, UVec2};

use super::backend::MapTextureSource;

/// Authoritative per-consumer map view output (projection + texture + extent).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedMapViewFrame {
    pub projection_revision: u64,
    pub texture_source: MapTextureSource,
    pub viewport_extent: UVec2,
    pub overlay_revision: u64,
    pub world_bounds: Rect,
}

impl ResolvedMapViewFrame {
    /// Stable key for egui texture cache: changes when the **image handle**, pixel extent, overlay
    /// generation, or consumer projection token change — not redundant with unrelated viewport ticks.
    #[must_use]
    pub fn texture_revision_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.texture_source.handle().hash(&mut hasher);
        self.viewport_extent.hash(&mut hasher);
        self.overlay_revision.hash(&mut hasher);
        self.projection_revision.hash(&mut hasher);
        hasher.finish()
    }
}
