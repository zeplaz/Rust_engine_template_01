//! Tilemap overlay scalar index — shared with `bevy_ecs_tilemap` adapter (feature-gated).

use super::layers::PreviewLayers;
use crate::terrain::family::TerrainFamilyId;
use crate::terrain::generation::ChunkCellMatrix;
use crate::terrain::material::TagSet;
use crate::terrain::mobility::MovementHint;

pub fn terrain_family_overlay_index(id: TerrainFamilyId) -> u32 {
    id.0 as u32
}

pub fn movement_hint_tile_index(hint: &MovementHint) -> u32 {
    if hint.blocked {
        255
    } else {
        let c = ((hint.cost_mul.clamp(1.0, 5.0) - 1.0) / 4.0 * 200.0
            + hint.stuck_risk.clamp(0.0, 1.0) * 54.0)
            .min(254.0);
        c as u32
    }
}

const BASE_MASK: PreviewLayers = PreviewLayers::HEIGHT
    .union(PreviewLayers::MOISTURE)
    .union(PreviewLayers::TEMPERATURE)
    .union(PreviewLayers::BIOME)
    .union(PreviewLayers::REGIONS);

/// One scalar per cell for the overlay tilemap — matches raster base/overlay priority.
pub fn tilemap_overlay_index_for_layers(
    matrix: &ChunkCellMatrix,
    x: u32,
    y: u32,
    layers: PreviewLayers,
    tag_pool: &TagSet,
    derived_slope: Option<f32>,
    mobility_hint: Option<&MovementHint>,
) -> u32 {
    if layers.contains(PreviewLayers::MOBILITY_OVERLAY) {
        if let Some(h) = mobility_hint {
            return movement_hint_tile_index(h);
        }
    }
    if layers.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY) {
        let s = derived_slope.unwrap_or(0.0);
        return (s.clamp(0.0, 1.0) * 255.0) as u32;
    }
    if layers.contains(PreviewLayers::TAG_OVERLAY) {
        let i = matrix.idx(x, y);
        return if matrix.tags[i].intersects(tag_pool) {
            240
        } else {
            0
        };
    }

    let base = layers & BASE_MASK;
    let i = matrix.idx(x, y);
    if base.contains(PreviewLayers::REGIONS) {
        return 0;
    }
    if base.contains(PreviewLayers::BIOME) {
        return terrain_family_overlay_index(matrix.family[i]);
    }
    if base.contains(PreviewLayers::HEIGHT) {
        return (matrix.elevation[i].clamp(0.0, 1.0) * 255.0) as u32;
    }
    if base.contains(PreviewLayers::MOISTURE) {
        return (matrix.moisture[i].clamp(0.0, 1.0) * 255.0) as u32;
    }
    if base.contains(PreviewLayers::TEMPERATURE) {
        return (matrix.temperature[i].clamp(0.0, 1.0) * 255.0) as u32;
    }

    if layers.contains(PreviewLayers::MOBILITY_OVERLAY) {
        return mobility_hint.map(movement_hint_tile_index).unwrap_or(0);
    }
    0
}
