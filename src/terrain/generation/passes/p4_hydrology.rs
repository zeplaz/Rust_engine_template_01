//! Pass 4 — hydrology / erosion tags (D8 flow + accumulation + depression fill).
//!
//! Designer context: impl Q **§55–58** (`implementation_questions_v1.md`).

use crate::terrain::biome::BiomeTuning;
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::generation::hydrology::{
    compute_hydrology_rect, HydrologyParams,
};
use crate::terrain::material::{TagRegistry, TagSet};

fn insert_tag_if(
    set: &mut TagSet,
    registry: &TagRegistry,
    name: &str,
    on: bool,
    tag_pool: &TagSet,
) {
    if !on {
        return;
    }
    let Some(id) = registry.tag_id(name) else {
        return;
    };
    if !tag_pool.contains(id) {
        return;
    }
    set.insert(id);
}

/// Merge hydrology-related tags into each cell's [`TagSet`] (preserves pass 2–3 tags).
pub fn apply_hydrology_with_params(
    matrix: &mut ChunkCellMatrix,
    tuning: &BiomeTuning,
    tag_registry: &TagRegistry,
    params: &HydrologyParams,
    tag_pool: &TagSet,
) {
    let w = matrix.size.x;
    let h = matrix.size.y;
    let hydro = compute_hydrology_rect(w, h, &matrix.elevation, params, 0, Some(&matrix.moisture));

    for y in 0..h {
        for x in 0..w {
            let i = matrix.idx(x, y);
            let mut tags = matrix.tags[i];

            let flooded = hydro.lake_mask[i as usize]
                || matrix.elevation[i] <= params.water_line
                || hydro.river_mask[i as usize];

            let mut max_slope = 0.0f32;
            for (_dx, nnx, nny) in [
                (1i32, x as i32 + 1, y as i32),
                (-1, x as i32 - 1, y as i32),
                (0, x as i32, y as i32 + 1),
                (0, x as i32, y as i32 - 1),
            ] {
                let nx = nnx;
                let ny = nny;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let j = matrix.idx(nx as u32, ny as u32);
                max_slope =
                    max_slope.max((matrix.elevation[i] - matrix.elevation[j]).abs());
            }

            let eroded = hydro.river_mask[i as usize]
                && max_slope >= params.erosion_slope_threshold;

            let silty = max_slope < params.erosion_slope_threshold * 0.5
                && matrix.moisture[i] >= params.silt_moisture_threshold
                && (hydro.river_mask[i as usize] || matrix.moisture[i] >= tuning.wetland_moist_threshold);

            insert_tag_if(&mut tags, tag_registry, "flooded", flooded, tag_pool);
            insert_tag_if(&mut tags, tag_registry, "eroded", eroded, tag_pool);
            insert_tag_if(&mut tags, tag_registry, "silted", silty, tag_pool);

            matrix.tags[i] = tags;
        }
    }
}

/// Default hydrology params derived from [`BiomeTuning::default`]; prefer
/// [`apply_hydrology_with_params`] with live tuning from [`WorldGenParams`](crate::terrain::generation::world_generator_enhanced::WorldGenParams).
pub fn apply_hydrology(matrix: &mut ChunkCellMatrix) {
    let tuning = BiomeTuning::default();
    let tag_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/config/terrain/tag_registry.example.json");
    let tag_registry =
        TagRegistry::load_from_json(tag_path.to_str().unwrap()).expect("example tag registry");
    let params = HydrologyParams::from_biome_tuning(&tuning);
    apply_hydrology_with_params(matrix, &tuning, &tag_registry, &params, &TagSet::ALL);
}

/// Chunk pipeline entry: uses world tuning + loaded tag registry (same files as material plugin).
pub fn apply_hydrology_chunk(
    matrix: &mut ChunkCellMatrix,
    tuning: &BiomeTuning,
    tag_registry: &TagRegistry,
    tag_pool: &TagSet,
) {
    let params = HydrologyParams::from_biome_tuning(tuning);
    apply_hydrology_with_params(matrix, tuning, tag_registry, &params, tag_pool);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;
    use crate::terrain::material::TagSet;
    use std::path::PathBuf;

    #[test]
    fn apply_hydrology_signature_compiles() {
        let mut matrix = ChunkCellMatrix::new(UVec2::new(4, 4));
        for i in 0..matrix.elevation.len() {
            matrix.elevation[i] = 0.5;
        }
        apply_hydrology(&mut matrix);
    }

    #[test]
    fn p4_hydrology_sets_flooded_tag() {
        let tuning = BiomeTuning::default();
        let tag_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/tag_registry.example.json");
        let tag_registry = TagRegistry::load_from_json(tag_path.to_str().unwrap()).unwrap();
        let flood_id = tag_registry.tag_id("flooded").expect("flooded tag in example registry");

        let mut matrix = ChunkCellMatrix::new(UVec2::new(6, 6));
        for y in 0..6u32 {
            for x in 0..6u32 {
                let i = matrix.idx(x, y);
                matrix.elevation[i] = if x == 2 && y == 2 {
                    tuning.deep_water_height_max * 0.5
                } else {
                    0.55
                };
                matrix.moisture[i] = 0.5;
            }
        }
        let params = HydrologyParams::from_biome_tuning(&tuning);
        apply_hydrology_with_params(&mut matrix, &tuning, &tag_registry, &params, &TagSet::ALL);
        assert!(matrix.tags[matrix.idx(2, 2)].contains(flood_id));
    }
}
