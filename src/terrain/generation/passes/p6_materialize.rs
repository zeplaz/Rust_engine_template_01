//! Pass 6 — [`resolve_material`](crate::terrain::material::resolve_material) per cell.

use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::material::{
    resolve_material, MaterialId, MaterialRegistry, RuleSet, TagRegistry,
};
#[cfg(feature = "dev_tools")]
use crate::terrain::material::resolve_material_with_rule_index;
use bevy::prelude::UVec2;

#[cfg(feature = "dev_tools")]
use crate::terrain::material::runtime::RuleTrace;

/// ECS wrapper lands in U5; this is the plain data for chunk material indices.
#[derive(Clone, Debug)]
pub struct MaterializedChunkData {
    pub size: UVec2,
    pub materials: Vec<MaterialId>,
}

pub fn materialize(
    matrix: &ChunkCellMatrix,
    rules: &RuleSet,
    registry: &MaterialRegistry,
    tag_registry: &TagRegistry,
) -> MaterializedChunkData {
    let n = matrix.elevation.len();
    let mut materials = Vec::with_capacity(n);
    for i in 0..n {
        materials.push(resolve_material(
            matrix.family[i],
            &matrix.weights[i],
            matrix.tags[i],
            rules,
            registry,
            tag_registry,
        ));
    }
    MaterializedChunkData {
        size: matrix.size,
        materials,
    }
}

#[cfg(feature = "dev_tools")]
pub fn materialize_traced(
    matrix: &ChunkCellMatrix,
    rules: &RuleSet,
    registry: &MaterialRegistry,
    tag_registry: &TagRegistry,
) -> (MaterializedChunkData, RuleTrace) {
    let n = matrix.elevation.len();
    let mut materials = Vec::with_capacity(n);
    let mut winners = Vec::with_capacity(n);
    for i in 0..n {
        let (id, w) = resolve_material_with_rule_index(
            matrix.family[i],
            &matrix.weights[i],
            matrix.tags[i],
            rules,
            registry,
            tag_registry,
        );
        materials.push(id);
        winners.push(w);
    }
    (
        MaterializedChunkData {
            size: matrix.size,
            materials,
        },
        RuleTrace { winners },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::biome::{BiomeWeights, TerrainClass};
    use crate::terrain::material::TagSet;
    use bevy::prelude::UVec2;

    #[cfg(feature = "dev_tools")]
    #[test]
    fn rule_trace_records_winning_index() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let registry = MaterialRegistry::load_from_json(
            root
                .join("assets/config/terrain/material_registry.example.json")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let tag_registry = TagRegistry::load_from_json(
            root
                .join("assets/config/terrain/tag_registry.example.json")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let rules = RuleSet::load_from_ron(
            root
                .join("assets/config/terrain/material_rules.example.ron")
                .to_str()
                .unwrap(),
        )
        .unwrap();

        let mut matrix = ChunkCellMatrix::new(UVec2::new(1, 1));
        matrix.family[0] = TerrainClass::Grassland;
        matrix.weights[0] = BiomeWeights::default();
        let mut t0 = TagSet::default();
        for name in ["wet", "lowland", "fertile"] {
            t0.insert(tag_registry.tag_id(name).unwrap());
        }
        matrix.tags[0] = t0;

        let (_data, trace) = materialize_traced(&matrix, &rules, &registry, &tag_registry);
        assert_eq!(trace.winners.len(), 1);
        assert_ne!(trace.winners[0], u32::MAX);
    }

    #[test]
    fn materialize_uses_resolver_e2e_small_chunk() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let registry = MaterialRegistry::load_from_json(
            root
                .join("assets/config/terrain/material_registry.example.json")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let tag_registry = TagRegistry::load_from_json(
            root
                .join("assets/config/terrain/tag_registry.example.json")
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let rules = RuleSet::load_from_ron(
            root
                .join("assets/config/terrain/material_rules.example.ron")
                .to_str()
                .unwrap(),
        )
        .unwrap();

        let mut matrix = ChunkCellMatrix::new(UVec2::new(2, 1));
        let loam = *registry.name_to_id.get("loam_wet").unwrap();
        let basalt = *registry.name_to_id.get("basalt_dense").unwrap();

        // Cell 0: Grassland + tags matching first rule (loam_wet)
        matrix.family[0] = TerrainClass::Grassland;
        matrix.weights[0] = BiomeWeights::default();
        let mut t0 = TagSet::default();
        for name in ["wet", "lowland", "fertile"] {
            t0.insert(tag_registry.tag_id(name).unwrap());
        }
        matrix.tags[0] = t0;

        // Cell 1: Stone + basalt rule tags
        matrix.family[1] = TerrainClass::Stone;
        matrix.weights[1] = BiomeWeights::default();
        let mut t1 = TagSet::default();
        for name in ["rock", "hard", "dry"] {
            t1.insert(tag_registry.tag_id(name).unwrap());
        }
        matrix.tags[1] = t1;

        let out = materialize(&matrix, &rules, &registry, &tag_registry);
        assert_eq!(out.size, UVec2::new(2, 1));
        assert_eq!(out.materials[0], loam);
        assert_eq!(out.materials[1], basalt);
    }
}
