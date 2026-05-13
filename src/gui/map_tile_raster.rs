//! Shared RGBA minimap / overworld raster from tile grid positions.

use std::collections::HashMap;

use bevy::math::{IVec2, UVec2};

use crate::gui::editor::world_preview::{
    blend_fire_overlay, chunk_cell_key_for_world_tile, preview_biome_rgba_for_tile,
    terrain_family_preview_rgba,
};
use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry};
use crate::terrain::material::{MaterialId, MaterialRegistry};

pub const ROAD_TILE_RGBA: [u8; 4] = [255, 120, 0, 255];

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
    let len = 4 * tex_w * tex_h;
    if data.len() != len {
        data.resize(len, 0);
    }
    data.fill(0);

    for (x, y, family) in tiles {
        if x >= tex_w || y >= tex_h {
            continue;
        }
        let pixel_index = 4 * (y * tex_w + x);
        if pixel_index + 3 >= data.len() {
            continue;
        }
        let color = match reg_opt {
            Some(reg) => preview_biome_rgba_for_tile(
                x as u32,
                y as u32,
                family,
                mat_slices,
                reg,
                fam_opt,
            ),
            None => terrain_family_preview_rgba(fam_opt, family),
        };
        data[pixel_index] = color[0];
        data[pixel_index + 1] = color[1];
        data[pixel_index + 2] = color[2];
        data[pixel_index + 3] = color[3];
    }

    for (x, y) in roads {
        if x >= tex_w || y >= tex_h {
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
    if chunk_fire_heat.is_empty() || chunk_geom.is_empty() {
        return;
    }
    for y in 0..tex_h {
        for x in 0..tex_w {
            let Some(key) = chunk_cell_key_for_world_tile(x as u32, y as u32, chunk_geom) else {
                continue;
            };
            let heat = chunk_fire_heat.get(&key.chunk).copied().unwrap_or(0.0);
            if heat <= 0.002 {
                continue;
            }
            let i = 4 * (y * tex_w + x);
            if i + 3 >= data.len() {
                continue;
            }
            let base = [data[i], data[i + 1], data[i + 2], data[i + 3]];
            let out = blend_fire_overlay(base, heat, 0.0);
            data[i..i + 4].copy_from_slice(&out);
        }
    }
}
