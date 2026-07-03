//! Shared RGBA minimap / overworld raster from tile grid positions.

use std::collections::HashMap;

use bevy::math::{IVec2, UVec2};

use crate::gui::editor::world_preview::{
    blend_fire_overlay, chunk_cell_key_for_world_tile, chunk_cell_layer_at_world_tile,
    preview_biome_rgba_for_tile, terrain_family_preview_rgba,
};
use crate::render::CHUNK_FIRE_OVERLAY_DISPLAY_MIN;
use crate::gui::editor::world_preview::{
    height_to_color, moisture_to_color, temperature_to_color, PreviewLayers,
};
use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry};
use crate::terrain::material::{MaterialId, MaterialRegistry};

pub const ROAD_TILE_RGBA: [u8; 4] = [48, 50, 58, 255];

/// Clears `data` then paints tiles and roads. `(tx, tz)` are column / row indices.
pub fn raster_tiles_and_roads_to_rgba(
    data: &mut Vec<u8>,
    tex_w: usize,
    tex_h: usize,
    tiles: impl Iterator<Item = (usize, usize, TerrainFamilyId)>,
    roads: impl Iterator<Item = (usize, usize)>,
    mat_slices: &[(IVec2, bevy::math::UVec2, &[MaterialId])],
    reg_opt: Option<&MaterialRegistry>,
    fam_opt: Option<&TerrainFamilyRegistry>,
) {
    raster_sim_minimap_layered_to_rgba(
        data,
        tex_w,
        tex_h,
        tiles.map(|(x, y, fam)| (x, y, fam, 0.0_f32, 0.0, 0.0)),
        roads,
        PreviewLayers::BIOME,
        mat_slices,
        reg_opt,
        fam_opt,
    );
}

/// Simulation minimap / overworld raster honoring preview base-layer selection.
pub fn raster_sim_minimap_layered_to_rgba(
    data: &mut Vec<u8>,
    tex_w: usize,
    tex_h: usize,
    tiles: impl Iterator<Item = (usize, usize, TerrainFamilyId, f32, f32, f32)>,
    roads: impl Iterator<Item = (usize, usize)>,
    layers: PreviewLayers,
    mat_slices: &[(IVec2, bevy::math::UVec2, &[MaterialId])],
    reg_opt: Option<&MaterialRegistry>,
    fam_opt: Option<&TerrainFamilyRegistry>,
) {
    let len = 4 * tex_w * tex_h;
    if data.len() != len {
        data.resize(len, 0);
    }
    raster_sim_minimap_layered_to_subregion(
        data,
        tex_w,
        tex_h,
        0,
        0,
        tex_w,
        tex_h,
        tiles,
        roads,
        layers,
        mat_slices,
        reg_opt,
        fam_opt,
    );
}

/// Clear `[x0..x1) × [y0..y1)` in an RGBA8 texture buffer (`tex_w` stride).
pub fn clear_rgba_subregion(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
) {
    let row_px = 4 * (x1 - x0);
    for y in y0..y1 {
        let row_start = 4 * (y * tex_w + x0);
        if row_start + row_px <= data.len() {
            data[row_start..row_start + row_px].fill(0);
        }
    }
}

/// Partial overworld raster — only clears and repaints the given tile-aligned sub-rectangle.
pub fn raster_sim_minimap_layered_to_subregion(
    data: &mut [u8],
    tex_w: usize,
    tex_h: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    tiles: impl Iterator<Item = (usize, usize, TerrainFamilyId, f32, f32, f32)>,
    roads: impl Iterator<Item = (usize, usize)>,
    layers: PreviewLayers,
    mat_slices: &[(IVec2, bevy::math::UVec2, &[MaterialId])],
    reg_opt: Option<&MaterialRegistry>,
    fam_opt: Option<&TerrainFamilyRegistry>,
) {
    if x0 >= x1 || y0 >= y1 || x1 > tex_w || y1 > tex_h {
        return;
    }
    clear_rgba_subregion(data, tex_w, x0, y0, x1, y1);

    let base = layers.base_bits();
    for (x, y, family, height, moisture, temperature) in tiles {
        if x < x0 || x >= x1 || y < y0 || y >= y1 {
            continue;
        }
        let pixel_index = 4 * (y * tex_w + x);
        if pixel_index + 3 >= data.len() {
            continue;
        }
        let color = if base.contains(PreviewLayers::HEIGHT) {
            height_to_color(height)
        } else if base.contains(PreviewLayers::MOISTURE) {
            moisture_to_color(moisture)
        } else if base.contains(PreviewLayers::TEMPERATURE) {
            temperature_to_color(temperature)
        } else if base.is_empty() {
            [0, 0, 0, 255]
        } else {
            match reg_opt {
                Some(reg) => preview_biome_rgba_for_tile(
                    x as u32,
                    y as u32,
                    family,
                    mat_slices,
                    reg,
                    fam_opt,
                ),
                None => terrain_family_preview_rgba(fam_opt, family),
            }
        };
        data[pixel_index] = color[0];
        data[pixel_index + 1] = color[1];
        data[pixel_index + 2] = color[2];
        data[pixel_index + 3] = color[3];
    }

    for (x, y) in roads {
        if x < x0 || x >= x1 || y < y0 || y >= y1 {
            continue;
        }
        let pixel_index = 4 * (y * tex_w + x);
        if pixel_index + 3 >= data.len() {
            continue;
        }
        data[pixel_index..pixel_index + 4].copy_from_slice(&ROAD_TILE_RGBA);
    }
}

/// Chunk-uniform fire heat from [`crate::render::SharedOverlayFieldBuffers`] (same source as world preview).
pub fn apply_shared_fire_heat_to_rgba(
    data: &mut [u8],
    tex_w: usize,
    tex_h: usize,
    chunk_geom: &[(IVec2, UVec2)],
    chunk_fire_heat: &HashMap<IVec2, f32>,
) {
    apply_shared_fire_heat_to_rgba_with_boost(data, tex_w, tex_h, chunk_geom, chunk_fire_heat, 1.0);
}

/// Same as [`apply_shared_fire_heat_to_rgba`] with extra tint strength when zoomed out (strategic view).
pub fn apply_shared_fire_heat_to_rgba_with_boost(
    data: &mut [u8],
    tex_w: usize,
    tex_h: usize,
    chunk_geom: &[(IVec2, UVec2)],
    chunk_fire_heat: &HashMap<IVec2, f32>,
    visibility_boost: f32,
) {
    if chunk_fire_heat.is_empty() || chunk_geom.is_empty() {
        return;
    }
    apply_shared_fire_heat_to_rgba_subregion(
        data,
        tex_w,
        0,
        0,
        tex_w,
        tex_h,
        chunk_geom,
        chunk_fire_heat,
        visibility_boost,
    );
}

/// Fire tint for a tile-aligned sub-rectangle only (used by chunked overworld raster).
pub fn apply_shared_fire_heat_to_rgba_subregion(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    chunk_geom: &[(IVec2, UVec2)],
    chunk_fire_heat: &HashMap<IVec2, f32>,
    visibility_boost: f32,
) {
    if chunk_fire_heat.is_empty() || chunk_geom.is_empty() {
        return;
    }
    let boost = visibility_boost.max(1.0);
    for y in y0..y1 {
        for x in x0..x1 {
            let Some(key) = chunk_cell_key_for_world_tile(x as u32, y as u32, chunk_geom) else {
                continue;
            };
            let heat = chunk_fire_heat.get(&key.chunk).copied().unwrap_or(0.0);
            if heat < CHUNK_FIRE_OVERLAY_DISPLAY_MIN {
                continue;
            }
            let i = 4 * (y * tex_w + x);
            if i + 3 >= data.len() {
                continue;
            }
            let base = [data[i], data[i + 1], data[i + 2], data[i + 3]];
            let tint = (heat * boost * 0.42).min(0.72);
            let out = blend_fire_overlay(base, tint, 0.0);
            data[i..i + 4].copy_from_slice(&out);
        }
    }
}

/// Per-cell fire tint — only hot cells inside chunk slabs are painted (not whole chunks).
pub fn apply_cell_fire_heat_to_rgba_subregion(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    cell_heat_layers: &[(IVec2, UVec2, &[f32])],
    visibility_boost: f32,
) {
    if cell_heat_layers.is_empty() {
        return;
    }
    let boost = visibility_boost.max(1.0);
    for y in y0..y1 {
        for x in x0..x1 {
            let heat = chunk_cell_layer_at_world_tile(x as u32, y as u32, cell_heat_layers)
                .unwrap_or(0.0);
            if heat < CHUNK_FIRE_OVERLAY_DISPLAY_MIN {
                continue;
            }
            let i = 4 * (y * tex_w + x);
            if i + 3 >= data.len() {
                continue;
            }
            let base = [data[i], data[i + 1], data[i + 2], data[i + 3]];
            let tint = (heat * boost * 0.42).min(0.72);
            let out = blend_fire_overlay(base, tint, 0.0);
            data[i..i + 4].copy_from_slice(&out);
        }
    }
}
