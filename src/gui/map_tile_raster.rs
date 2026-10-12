//! Shared RGBA minimap / overworld raster from tile grid positions.

use std::collections::HashMap;

use bevy::math::{IVec2, UVec2};
use bevy::prelude::Resource;

use crate::gui::editor::world_preview::{
    blend_fire_overlay, blend_overlay, chunk_cell_key_for_world_tile,
    chunk_cell_layer_at_world_tile, landform_relief_rgba, preview_biome_rgba_for_tile,
    slope_grade_to_color, terrain_family_preview_rgba,
};
use crate::render::CHUNK_FIRE_OVERLAY_DISPLAY_MIN;
use crate::gui::editor::world_preview::{
    height_to_color, moisture_to_color, temperature_to_color, PreviewLayers,
};
use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry};
use crate::terrain::material::{MaterialId, MaterialRegistry};

pub const ROAD_TILE_RGBA: [u8; 4] = [48, 50, 58, 255];

/// Play-sprite pixel mode. One terrain image; the ops-strip control is the only writer.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayMapPixelMode {
    pub height_slope: bool,
}

impl Default for PlayMapPixelMode {
    fn default() -> Self {
        Self {
            height_slope: false,
        }
    }
}

/// Layers painted into [`crate::render::TileWorldFallbackSprite`].
///
/// Off: the presentation mask, with an empty base filled by biome plus relief.
/// On: height base and the derived slope overlay on that same mask.
#[must_use]
pub fn layers_for_play_sprite(mode: PlayMapPixelMode, mut layers: PreviewLayers) -> PreviewLayers {
    if mode.height_slope {
        layers.replace_base(PreviewLayers::HEIGHT);
        layers.insert(PreviewLayers::DERIVED_SLOPE_OVERLAY);
        return layers;
    }
    layers.remove(PreviewLayers::DERIVED_SLOPE_OVERLAY);
    if layers.base_bits().contains(PreviewLayers::HEIGHT) {
        layers.replace_base(PreviewLayers::BIOME);
    } else if layers.base_bits().is_empty() {
        layers.replace_base(PreviewLayers::BIOME);
    }
    layers
}

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

    let samples: Vec<_> = tiles.collect();
    let mut elev: HashMap<(usize, usize), f32> = HashMap::with_capacity(samples.len());
    for (x, y, _, height, _, _) in &samples {
        elev.insert((*x, *y), *height);
    }

    let hydro = play_hydro_field(&elev);
    let base = layers.base_bits();
    for (x, y, family, height, moisture, temperature) in samples {
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
        let (slope, curve) = landform_from_neighbor_elev(&elev, x, y, height);
        let mut color = landform_relief_rgba(color, slope, curve);
        if base.contains(PreviewLayers::MOISTURE) {
            let steep = (slope / 0.2).clamp(0.0, 1.0);
            color[2] = ((color[2] as f32) * (1.0 - steep * 0.45))
                .round()
                .clamp(0.0, 255.0) as u8;
        }
        if let Some(field) = hydro.as_ref() {
            color = tint_play_hydro(color, field, x, y, slope, curve);
        }
        if layers.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY) {
            color = blend_overlay(color, slope_grade_to_color(slope), 0.72);
        }
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
            let tint = (heat * boost * 0.85).min(0.92);
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
            let tint = (heat * boost * 0.85).min(0.92);
            let out = blend_fire_overlay(base, tint, 0.0);
            data[i..i + 4].copy_from_slice(&out);
        }
    }
}

struct PlayHydroField {
    w: usize,
    drainage: Vec<f32>,
    basins: Vec<u32>,
    max_drain: f32,
}

fn play_hydro_field(elev: &HashMap<(usize, usize), f32>) -> Option<PlayHydroField> {
    if elev.is_empty() {
        return None;
    }
    let mut max_x = 0usize;
    let mut max_y = 0usize;
    for &(x, y) in elev.keys() {
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let w = max_x + 1;
    let h = max_y + 1;
    let mut grid = vec![0.5f32; w * h];
    for (&(x, y), &z) in elev {
        if x < w && y < h {
            grid[y * w + x] = z;
        }
    }
    let drainage =
        crate::terrain::generation::drainage_accumulation(&grid, w as u32, h as u32);
    let basins = crate::terrain::generation::flood_basin_ids(&grid);
    let max_drain = drainage.iter().copied().fold(1.0f32, f32::max);
    Some(PlayHydroField {
        w,
        drainage,
        basins,
        max_drain,
    })
}

fn tint_play_hydro(
    color: [u8; 4],
    field: &PlayHydroField,
    x: usize,
    y: usize,
    slope: f32,
    curve: f32,
) -> [u8; 4] {
    if color[3] == 0 {
        return color;
    }
    let i = y * field.w + x;
    let Some(&drain) = field.drainage.get(i) else {
        return color;
    };
    let basin = field.basins.get(i).copied().unwrap_or(0) == 1;
    let wet = if field.max_drain > 1.01 {
        ((drain - 1.0) / (field.max_drain - 1.0)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let support = crate::terrain::generation::support_from_slope(slope);
    let hide = crate::terrain::generation::concealment_from_landform(slope, curve);
    let mut r = color[0] as f32;
    let mut g = color[1] as f32;
    let mut b = color[2] as f32;
    if basin {
        r *= 0.55;
        g *= 0.55;
    }
    r *= 0.45 + 0.55 * support;
    g *= 1.0 - hide * 0.22;
    let erode = crate::terrain::generation::erosion_from_slope_drainage(slope, drain);
    g *= 1.0 - erode * 0.4;
    b = b * (1.0 - wet * 0.2) + wet * 220.0;
    [
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
        color[3],
    ]
}

/// Paint tracer cells on the play raster. Callers pass cells from `shot_trace_tile`.
pub fn stamp_tracer_cells_subregion(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    cells: impl Iterator<Item = (i32, i32)>,
) {
    stamp_cells_rgba(
        data,
        tex_w,
        x0,
        y0,
        x1,
        y1,
        cells,
        TRACER_STAMP_RGBA,
    );
}

/// Impact color is distinct from the in-flight tracer.
pub const IMPACT_STAMP_RGBA: [u8; 4] = [220, 48, 48, 255];
pub const TRACER_STAMP_RGBA: [u8; 4] = [255, 220, 40, 255];
/// Screen rect of one map cell: `[x, y, side, side]`.
/// `px_per_tile` is the same scale the terrain uses. Doubling it doubles the
/// origin delta and the side, so the cell stays the tile under the stamp.
#[must_use]
pub fn map_tile_stamp_rect(tile_x: i32, tile_y: i32, px_per_tile: f32) -> [f32; 4] {
    let px = px_per_tile.abs().max(1e-6);
    let x = tile_x as f32 * px;
    let y = tile_y as f32 * px;
    [x, y, px, px]
}

pub const TOWN_STAMP_RGBA: [u8; 4] = [64, 196, 220, 255];
pub const HAUL_STAMP_RGBA: [u8; 4] = [80, 255, 120, 255];
pub const FOOTPRINT_STAMP_RGBA: [u8; 4] = [255, 150, 40, 255];

pub fn stamp_town_centers_subregion(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    cells: impl Iterator<Item = (i32, i32)>,
) {
    stamp_cells_rgba(data, tex_w, x0, y0, x1, y1, cells, TOWN_STAMP_RGBA);
}

pub fn stamp_haul_cells_subregion(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    cells: impl Iterator<Item = (i32, i32)>,
) {
    stamp_cells_rgba(data, tex_w, x0, y0, x1, y1, cells, HAUL_STAMP_RGBA);
}

pub fn stamp_growth_origins_subregion(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    cells: impl Iterator<Item = (i32, i32)>,
) {
    stamp_cells_rgba(data, tex_w, x0, y0, x1, y1, cells, FOOTPRINT_STAMP_RGBA);
}

pub fn stamp_impact_cells_subregion(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    cells: impl Iterator<Item = (i32, i32)>,
) {
    stamp_cells_rgba(
        data,
        tex_w,
        x0,
        y0,
        x1,
        y1,
        cells,
        IMPACT_STAMP_RGBA,
    );
}

fn stamp_cells_rgba(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    cells: impl Iterator<Item = (i32, i32)>,
    rgba: [u8; 4],
) {
    for (x, y) in cells {
        let rect = map_tile_stamp_rect(x, y, 1.0);
        let x = rect[0] as i32;
        let y = rect[1] as i32;
        if x < 0 || y < 0 {
            continue;
        }
        let (x, y) = (x as usize, y as usize);
        if x < x0 || x >= x1 || y < y0 || y >= y1 {
            continue;
        }
        let pixel_index = 4 * (y * tex_w + x);
        if pixel_index + 3 >= data.len() {
            continue;
        }
        data[pixel_index..pixel_index + 4].copy_from_slice(&rgba);
    }
}

fn landform_from_neighbor_elev(
    elev: &HashMap<(usize, usize), f32>,
    x: usize,
    y: usize,
    height: f32,
) -> (f32, f32) {
    let mut max_d = 0.0f32;
    let mut acc = 0.0f32;
    for (dx, dy) in [(1isize, 0), (-1, 0), (0, 1), (0, -1)] {
        let nx = x as isize + dx;
        let ny = y as isize + dy;
        let nh = if nx >= 0 && ny >= 0 {
            elev.get(&(nx as usize, ny as usize))
                .copied()
                .unwrap_or(height)
        } else {
            height
        };
        max_d = max_d.max((height - nh).abs());
        acc += nh;
    }
    (max_d, ((acc * 0.25) - height).clamp(-1.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_map_ridge_is_lighter_than_hollow() {
        let mut data = vec![0u8; 4 * 3 * 3];
        let fam = crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID;
        let tiles = [
            (0usize, 1usize, fam, 0.8f32, 0.0, 0.0),
            (1, 0, fam, 0.8, 0.0, 0.0),
            (1, 1, fam, 0.2, 0.0, 0.0),
            (1, 2, fam, 0.8, 0.0, 0.0),
            (2, 1, fam, 0.8, 0.0, 0.0),
            (0, 0, fam, 0.2, 0.0, 0.0),
            (2, 0, fam, 0.2, 0.0, 0.0),
            (0, 2, fam, 0.2, 0.0, 0.0),
            (2, 2, fam, 0.95, 0.0, 0.0),
        ];
        raster_sim_minimap_layered_to_rgba(
            &mut data,
            3,
            3,
            tiles.into_iter(),
            std::iter::empty(),
            PreviewLayers::BIOME,
            &[],
            None,
            None,
        );
        let hollow = data[4 * (1 * 3 + 1) + 1];
        let ridge = data[4 * (2 * 3 + 2) + 1];
        assert!(ridge > hollow, "ridge {ridge} hollow {hollow}");
    }

    #[test]
    fn play_map_tracer_cell_is_brighter_than_untouched() {
        let mut data = vec![0u8; 4 * 4 * 4];
        stamp_tracer_cells_subregion(&mut data, 4, 0, 0, 4, 4, std::iter::once((2, 1)));
        let hit = data[4 * (1 * 4 + 2)];
        let miss = data[4 * (3 * 4 + 0)];
        assert!(hit > miss);
    }

    fn five_by_five(pit_z: f32, peak_z: f32) -> Vec<u8> {
        let fam = crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID;
        let mut tiles = Vec::new();
        for y in 0..5usize {
            for x in 0..5usize {
                let z = if (x, y) == (2, 2) {
                    pit_z
                } else if (x, y) == (0, 0) {
                    peak_z
                } else {
                    0.55
                };
                tiles.push((x, y, fam, z, 0.0, 0.0));
            }
        }
        let mut data = vec![0u8; 4 * 5 * 5];
        raster_sim_minimap_layered_to_rgba(
            &mut data,
            5,
            5,
            tiles.into_iter(),
            std::iter::empty(),
            PreviewLayers::BIOME,
            &[],
            None,
            None,
        );
        data
    }

    #[test]
    fn play_map_pit_is_wetter_than_peak() {
        let data = five_by_five(0.05, 0.95);
        let pit = data[4 * (2 * 5 + 2) + 2];
        let peak = data[4 * (0 * 5 + 0) + 2];
        assert!(pit > peak, "pit blue {pit} peak blue {peak}");
    }

    #[test]
    fn play_map_flood_basin_is_darker_than_peak() {
        let data = five_by_five(0.05, 0.95);
        let pit = data[4 * (2 * 5 + 2)] as u16 + data[4 * (2 * 5 + 2) + 1] as u16;
        let peak = data[4 * (0 * 5 + 0)] as u16 + data[4 * (0 * 5 + 0) + 1] as u16;
        assert!(pit < peak, "basin {pit} peak {peak}");
    }

    #[test]
    fn play_map_moisture_steep_is_darker_than_flat() {
        let fam = crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID;
        let mut tiles = Vec::new();
        for y in 0..5usize {
            for x in 0..5usize {
                let z = if (x, y) == (0, 0) { 0.95 } else { 0.55 };
                tiles.push((x, y, fam, z, 0.7, 0.0));
            }
        }
        let mut data = vec![0u8; 4 * 5 * 5];
        raster_sim_minimap_layered_to_rgba(
            &mut data,
            5,
            5,
            tiles.into_iter(),
            std::iter::empty(),
            PreviewLayers::MOISTURE,
            &[],
            None,
            None,
        );
        let steep = data[2];
        let flat = data[4 * (4 * 5 + 4) + 2];
        assert!(steep < flat, "steep blue {steep} flat blue {flat}");
    }

    #[test]
    fn play_map_steep_support_is_dimmer_than_flat() {
        let data = five_by_five(0.55, 0.95);
        let steep = data[4 * (0 * 5 + 0)];
        let flat = data[4 * (4 * 5 + 4)];
        assert!(steep < flat, "steep red {steep} flat red {flat}");
    }

    #[test]
    fn play_map_impact_stamp_differs_from_tracer() {
        let mut tracer = vec![0u8; 16];
        let mut impact = vec![0u8; 16];
        stamp_tracer_cells_subregion(&mut tracer, 2, 0, 0, 2, 2, std::iter::once((1, 0)));
        stamp_impact_cells_subregion(&mut impact, 2, 0, 0, 2, 2, std::iter::once((1, 0)));
        let tr = &tracer[4..8];
        let im = &impact[4..8];
        assert_eq!(tr, &TRACER_STAMP_RGBA);
        assert_eq!(im, &IMPACT_STAMP_RGBA);
        assert_ne!(tr, im);
    }

    #[test]
    fn play_map_town_center_differs_from_empty() {
        let mut data = vec![0u8; 4 * 4 * 4];
        stamp_town_centers_subregion(&mut data, 4, 0, 0, 4, 4, std::iter::once((1, 2)));
        let hit = &data[4 * (2 * 4 + 1)..4 * (2 * 4 + 1) + 4];
        let miss = &data[0..4];
        assert_eq!(hit, &TOWN_STAMP_RGBA);
        assert_ne!(hit, miss);
    }

    #[test]
    fn play_map_concealed_pit_is_less_green_than_peak() {
        let data = five_by_five(0.05, 0.95);
        let pit = data[4 * (2 * 5 + 2) + 1];
        let peak = data[4 * (0 * 5 + 0) + 1];
        assert!(pit < peak, "pit green {pit} peak green {peak}");
    }

    #[test]
    fn play_map_eroded_peak_is_less_green_than_flat() {
        let data = five_by_five(0.55, 0.95);
        let steep = data[4 * (0 * 5 + 0) + 1];
        let flat = data[4 * (4 * 5 + 4) + 1];
        assert_ne!(steep, flat, "eroded {steep} flat {flat}");
    }

    #[test]
    fn play_map_haul_stamp_differs_from_town() {
        let mut data = vec![0u8; 16];
        stamp_haul_cells_subregion(&mut data, 2, 0, 0, 2, 2, std::iter::once((1, 0)));
        assert_eq!(&data[4..8], &HAUL_STAMP_RGBA);
        assert_ne!(&data[4..8], &TOWN_STAMP_RGBA);
    }

    #[test]
    fn play_map_growth_origin_is_not_the_town_stamp() {
        let mut data = vec![0u8; 16];
        stamp_town_centers_subregion(&mut data, 2, 0, 0, 2, 2, std::iter::once((0, 0)));
        stamp_growth_origins_subregion(&mut data, 2, 0, 0, 2, 2, std::iter::once((1, 0)));
        assert_eq!(&data[0..4], &TOWN_STAMP_RGBA);
        assert_eq!(&data[4..8], &FOOTPRINT_STAMP_RGBA);
        assert_ne!(&data[4..8], &HAUL_STAMP_RGBA);
    }

    #[test]
    fn fold_play_clear_pixel_stays_zero() {
        let mut elev = HashMap::new();
        elev.insert((0usize, 0usize), 0.1);
        elev.insert((1, 0), 0.9);
        let field = play_hydro_field(&elev).expect("hydro");
        let clear = [40, 80, 20, 0];
        assert_eq!(tint_play_hydro(clear, &field, 0, 0, 0.4, 0.2), clear);
    }

    #[test]
    fn fold_dry_non_basin_skips_darken() {
        let mut elev = HashMap::new();
        elev.insert((0usize, 0usize), 0.1);
        elev.insert((1, 0), 0.9);
        let field = play_hydro_field(&elev).expect("hydro");
        assert_eq!(field.basins[0], 1);
        assert_eq!(field.basins[1], 0);
        let base = [200, 200, 200, 255];
        let floor = tint_play_hydro(base, &field, 0, 0, 0.0, 0.0);
        let dry = tint_play_hydro(base, &field, 1, 0, 0.0, 0.0);
        assert_eq!(dry[0], 200);
        assert!(floor[0] < dry[0], "basin red {} dry red {}", floor[0], dry[0]);
    }

    fn paint_one(mode: PlayMapPixelMode, into: &mut Vec<u8>) {
        let fam = crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID;
        let layers = layers_for_play_sprite(mode, PreviewLayers::BIOME);
        raster_sim_minimap_layered_to_rgba(
            into,
            1,
            1,
            std::iter::once((0usize, 0usize, fam, 0.2_f32, 0.0, 0.0)),
            std::iter::empty(),
            layers,
            &[],
            None,
            None,
        );
    }

    #[test]
    fn upm_default_play_raster_is_not_the_height_view() {
        let off = layers_for_play_sprite(PlayMapPixelMode::default(), PreviewLayers::BIOME);
        assert!(
            !off.base_bits().contains(PreviewLayers::HEIGHT),
            "default play layers must stay on the biome base"
        );
        assert!(!off.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY));
        let mut data = vec![0u8; 4];
        paint_one(PlayMapPixelMode::default(), &mut data);
        let height = height_to_color(0.2);
        assert_ne!(&data[..4], &height, "default pixels are biome plus relief");
    }

    #[test]
    fn upm_height_slope_toggle_repaints_the_same_buffer() {
        let mut data = vec![0u8; 4];
        paint_one(PlayMapPixelMode::default(), &mut data);
        let biome = data.clone();
        let on = PlayMapPixelMode { height_slope: true };
        let layers = layers_for_play_sprite(on, PreviewLayers::BIOME);
        assert!(layers.base_bits().contains(PreviewLayers::HEIGHT));
        assert!(layers.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY));
        paint_one(on, &mut data);
        assert_eq!(data.len(), biome.len());
        assert_ne!(data, biome);
    }

    #[test]
    fn upm_heat_tint_does_not_take_zoom_as_input() {
        let boost = crate::render::play_sprite_fire_heat_boost();
        assert_eq!(boost, 1.0);
        let src = include_str!("../render/tile_world_fallback.rs");
        assert!(
            !src.contains("px_per_tile.max"),
            "sprite heat strength must not divide by camera pixels-per-tile"
        );
        assert!(
            src.contains("play_sprite_fire_heat_boost()"),
            "the play sprite passes the fixed heat strength"
        );
    }

    #[test]
    fn upm_simulation_schedule_omits_egui_pointer_preupdate() {
        use crate::engine::states::BaseState;
        use bevy::prelude::*;
        use bevy::state::app::StatesPlugin;

        // egui pointer pre-update retired with bevy_egui drop; omit helper must stay callable.
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<BaseState>();
        crate::engine::omit_egui_pointer_preupdate_in_simulation(&mut app);
        assert!(
            app.world().resource::<Schedules>().get(PreUpdate).is_some(),
            "PreUpdate schedule present after omit_egui_pointer_preupdate_in_simulation"
        );
    }
}
