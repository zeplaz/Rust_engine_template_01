//! Pass 2 — threshold tags from scalar fields using [`BiomeTuning`] bands (names → ids via [`TagRegistry`](crate::terrain::material::TagRegistry)).

use crate::terrain::biome::BiomeTuning;
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::material::{TagRegistry, TagSet};

fn insert_named(
    set: &mut TagSet,
    registry: &TagRegistry,
    name: &str,
    on: bool,
) {
    if !on {
        return;
    }
    if let Some(id) = registry.tag_id(name) {
        set.insert(id);
    }
}

/// Writes **baseline** threshold tags into `matrix.tags` (replaces prior contents per cell).
pub fn apply_threshold_tags(
    matrix: &mut ChunkCellMatrix,
    tuning: &BiomeTuning,
    tag_registry: &TagRegistry,
) {
    let n = &tuning.threshold_tag_names;
    for i in 0..matrix.elevation.len() {
        let h = matrix.elevation[i];
        let m = matrix.moisture[i];
        let t = matrix.temperature[i];
        let mut tags = TagSet::default();
        let lowland = h >= tuning.beach_height_max && h < tuning.mountain_height_min;
        insert_named(&mut tags, tag_registry, &n.lowland, lowland);
        insert_named(
            &mut tags,
            tag_registry,
            &n.highland,
            h >= tuning.mountain_height_min,
        );
        insert_named(
            &mut tags,
            tag_registry,
            &n.wet,
            m >= tuning.wetland_moist_threshold,
        );
        insert_named(
            &mut tags,
            tag_registry,
            &n.dry,
            m <= tuning.desert_moisture_max,
        );
        insert_named(
            &mut tags,
            tag_registry,
            &n.hot,
            t >= tuning.hot_lowlands_temperature_min,
        );
        insert_named(
            &mut tags,
            tag_registry,
            &n.cold,
            t <= tuning.tundra_temperature_max,
        );
        matrix.tags[i] = tags;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;

    #[test]
    fn pass2_threshold_tags_lowland_wet() {
        let tuning = BiomeTuning::default();
        let tag_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/tag_registry.example.json");
        let tag_registry = TagRegistry::load_from_json(tag_path.to_str().unwrap()).unwrap();

        let mut matrix = ChunkCellMatrix::new(UVec2::ONE);
        // Above beach, below mountain → lowland; moisture at wetland threshold → wet
        matrix.elevation[0] = 0.5;
        matrix.moisture[0] = tuning.wetland_moist_threshold + 0.01;
        matrix.temperature[0] = 0.5;

        apply_threshold_tags(&mut matrix, &tuning, &tag_registry);

        let low_id = tag_registry.tag_id("lowland").unwrap();
        let wet_id = tag_registry.tag_id("wet").unwrap();
        assert!(matrix.tags[0].contains(low_id));
        assert!(matrix.tags[0].contains(wet_id));
    }
}
