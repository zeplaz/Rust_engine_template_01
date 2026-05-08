//! Pass 3 — fill `family` + `weights` using [`crate::terrain::family::classify_biome`]; add primary `BiomeId` tag when present in registry.

use crate::terrain::biome::{BiomeId, BiomeTuning};
use crate::terrain::family::{classify_biome, TerrainFamilyRegistry};
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::material::TagRegistry;

fn biome_primary_tag_name(id: BiomeId) -> &'static str {
    match id {
        BiomeId::Marine => "marine",
        BiomeId::Coastal => "coastal",
        BiomeId::Arid => "arid",
        BiomeId::Temperate => "temperate",
        BiomeId::Boreal => "boreal",
        BiomeId::Alpine => "alpine",
        BiomeId::Wetland => "wetland",
    }
}

/// Classifies each cell; **unions** primary-biome tag into existing `tags` when the name exists in `tag_registry`.
pub fn classify_cells(
    matrix: &mut ChunkCellMatrix,
    tuning: &BiomeTuning,
    tag_registry: &TagRegistry,
    families: &TerrainFamilyRegistry,
) {
    for i in 0..matrix.elevation.len() {
        let h = matrix.elevation[i];
        let m = matrix.moisture[i];
        let t = matrix.temperature[i];
        let classification = classify_biome(h, m, t, tuning, families);
        matrix.family[i] = classification.terrain_family;
        matrix.weights[i] = classification.biome_weights;
        let primary = classification.biome_weights.primary();
        let name = biome_primary_tag_name(primary);
        if let Some(tid) = tag_registry.tag_id(name) {
            let mut merged = matrix.tags[i];
            merged.insert(tid);
            matrix.tags[i] = merged;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::family::{classify_biome, TerrainFamilyRegistry};
    use bevy::prelude::UVec2;

    fn assert_weights_close(a: crate::terrain::biome::BiomeWeights, b: crate::terrain::biome::BiomeWeights) {
        let eps = 1e-5;
        assert!((a.marine - b.marine).abs() < eps);
        assert!((a.coastal - b.coastal).abs() < eps);
        assert!((a.arid - b.arid).abs() < eps);
        assert!((a.temperate - b.temperate).abs() < eps);
        assert!((a.boreal - b.boreal).abs() < eps);
        assert!((a.alpine - b.alpine).abs() < eps);
        assert!((a.wetland - b.wetland).abs() < eps);
    }

    #[test]
    fn pass3_uses_classify_biome_only() {
        let tuning = BiomeTuning::default();
        let tag_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/tag_registry.example.json");
        let tag_registry = TagRegistry::load_from_json(tag_path.to_str().unwrap()).unwrap();

        let fam_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/terrain_family_registry.example.json");
        let family_registry =
            TerrainFamilyRegistry::load_from_json(fam_path.to_str().unwrap()).unwrap();

        let h = 0.55;
        let m = 0.5;
        let t = 0.48;
        let expected = classify_biome(h, m, t, &tuning, &family_registry);

        let mut matrix = ChunkCellMatrix::new(UVec2::ONE);
        matrix.elevation[0] = h;
        matrix.moisture[0] = m;
        matrix.temperature[0] = t;

        classify_cells(&mut matrix, &tuning, &tag_registry, &family_registry);

        assert_eq!(matrix.family[0], expected.terrain_family);
        assert_weights_close(matrix.weights[0], expected.biome_weights);
    }
}
