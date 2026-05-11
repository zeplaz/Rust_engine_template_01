//! Terrain / material preview colors and scalar → RGBA mapping.

use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry};
use crate::terrain::material::{
    family_default_material_def, MaterialId, MaterialRegistry,
};
use bevy::math::{IVec2, UVec2};

/// RGBA for editor / minimap when material registry is missing or has no row for `family`.
pub fn terrain_family_preview_rgba(
    families: Option<&TerrainFamilyRegistry>,
    id: TerrainFamilyId,
) -> [u8; 4] {
    fn by_name(name: &str) -> [u8; 4] {
        match name {
            "DeepWater" => [0, 0, 128, 255],
            "ShallowWater" => [0, 0, 255, 255],
            "Beach" => [240, 240, 64, 255],
            "Desert" => [255, 255, 128, 255],
            "Grassland" => [0, 255, 0, 255],
            "Forest" => [0, 128, 0, 255],
            "DenseForest" => [0, 64, 0, 255],
            "Mountain" => [128, 128, 128, 255],
            "SnowCappedMountain" => [255, 255, 255, 255],
            "Tundra" => [192, 192, 255, 255],
            "Swamp" => [64, 64, 0, 255],
            "Cliff" => [90, 90, 90, 255],
            "Concrete" => [170, 170, 170, 255],
            "Dirt" => [139, 69, 19, 255],
            "Snow" => [250, 250, 250, 255],
            "Stone" => [120, 120, 120, 255],
            _ => [128, 0, 128, 255],
        }
    }
    if let Some(reg) = families {
        if let Some(d) = reg.def(id) {
            return by_name(&d.name);
        }
    }
    let u = (id.0 as u32)
        .wrapping_mul(1103515245)
        .wrapping_add(12345);
    [
        (u & 0xff) as u8,
        ((u >> 8) & 0xff) as u8,
        ((u >> 16) & 0xff) as u8,
        255,
    ]
}

pub fn preview_biome_rgba_for_tile(
    tx: u32,
    ty: u32,
    terrain_family: TerrainFamilyId,
    chunks: &[(IVec2, UVec2, &[MaterialId])],
    registry: &MaterialRegistry,
    families: Option<&TerrainFamilyRegistry>,
) -> [u8; 4] {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size, materials) in chunks {
        let sx = size.x as i32;
        let sy = size.y as i32;
        let wx0 = coord.x * sx;
        let wy0 = coord.y * sy;
        if tx_i < wx0 || ty_i < wy0 {
            continue;
        }
        let lx = tx_i - wx0;
        let ly = ty_i - wy0;
        if lx < 0 || ly < 0 || lx >= sx || ly >= sy {
            continue;
        }
        let idx = (ly * sx + lx) as usize;
        if idx < materials.len() {
            let mid = materials[idx];
            return registry.materials[mid.0 as usize].preview_color;
        }
    }
    if let Some(def) = family_default_material_def(registry, terrain_family) {
        return def.preview_color;
    }
    terrain_family_preview_rgba(families, terrain_family)
}

/// `sim.traction_mod` from the resolved material at world tile `(tx, ty)`, else **1.0**.
pub fn material_traction_mod_for_world_tile(
    tx: u32,
    ty: u32,
    terrain_family: TerrainFamilyId,
    chunks: &[(IVec2, UVec2, &[MaterialId])],
    registry: &MaterialRegistry,
) -> f32 {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size, materials) in chunks {
        let sx = size.x as i32;
        let sy = size.y as i32;
        let wx0 = coord.x * sx;
        let wy0 = coord.y * sy;
        if tx_i < wx0 || ty_i < wy0 {
            continue;
        }
        let lx = tx_i - wx0;
        let ly = ty_i - wy0;
        if lx < 0 || ly < 0 || lx >= sx || ly >= sy {
            continue;
        }
        let idx = (ly * sx + lx) as usize;
        if idx < materials.len() {
            let mid = materials[idx];
            return registry.materials[mid.0 as usize]
                .sim_f32("traction_mod")
                .unwrap_or(1.0);
        }
    }
    if let Some(def) = family_default_material_def(registry, terrain_family) {
        return def.sim_f32("traction_mod").unwrap_or(1.0);
    }
    1.0
}

pub fn height_to_color(height: f32) -> [u8; 4] {
    let h = (height * 255.0) as u8;
    [h, h, h, 255]
}

pub fn moisture_to_color(moisture: f32) -> [u8; 4] {
    let m = (moisture * 255.0) as u8;
    [0, 0, m, 255]
}

pub fn temperature_to_color(temperature: f32) -> [u8; 4] {
    let t = (temperature * 255.0) as u8;
    [t, 0, 0, 255]
}
