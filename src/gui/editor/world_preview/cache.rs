//! Per-chunk CPU preview payloads (egui image); populated incrementally as chunk-diff preview matures.

use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::HashMap;

/// Cached RGBA for one chunk’s preview tile rect (Stage-2+); today raster still writes the global texture.
#[derive(Clone)]
pub struct ChunkPreviewCache {
    pub version: u64,
    pub image: egui::ColorImage,
    pub dirty: bool,
}

#[derive(Resource, Default)]
pub struct WorldPreviewChunkCaches {
    pub chunks: HashMap<IVec2, ChunkPreviewCache>,
}
