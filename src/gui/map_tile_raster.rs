//! Shared RGBA minimap / overworld raster from tile grid positions.

use bevy::math::IVec2;

use crate::gui::editor::world_preview::{preview_biome_rgba_for_tile, terrain_family_preview_rgba};
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
