//! ECS â†’ save DTO snapshot builder (Wave S).

use bevy::math::IVec2;

use crate::io::save::dto::{SavedChunkBody, SavedTerrainCell};
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::material::{MaterialId, MaterializedChunk, MaterialRegistry, TagRegistry, TagSet};

/// Per-chunk inputs collected on the main thread before IO workers run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkSaveSnapshotInput {
    pub coord: IVec2,
    pub materials: Vec<MaterialId>,
    pub tags: Vec<TagSet>,
}

#[must_use]
pub fn tag_names_from_set(set: &TagSet, registry: &TagRegistry) -> Vec<String> {
    registry
        .tags
        .iter()
        .enumerate()
        .filter_map(|(index, tag)| {
            let id = crate::terrain::material::TagId(index as u16);
            if set.contains(id) {
                Some(tag.name.clone())
            } else {
                None
            }
        })
        .collect()
}

#[must_use]
pub fn build_chunk_save_snapshot_input(
    coord: IVec2,
    mat_chunk: &MaterializedChunk,
    cell_matrix: Option<&ChunkCellMatrix>,
) -> ChunkSaveSnapshotInput {
    let tags = cell_matrix
        .map(|matrix| matrix.tags.clone())
        .unwrap_or_else(|| vec![TagSet::default(); mat_chunk.materials.len()]);
    ChunkSaveSnapshotInput {
        coord,
        materials: mat_chunk.materials.clone(),
        tags,
    }
}

#[must_use]
pub fn build_saved_chunk_body(
    input: &ChunkSaveSnapshotInput,
    material_registry: &MaterialRegistry,
    tag_registry: &TagRegistry,
) -> SavedChunkBody {
    let cells = input
        .materials
        .iter()
        .zip(input.tags.iter())
        .map(|(material_id, tag_set)| SavedTerrainCell {
            material_name: material_registry
                .materials
                .get(material_id.0 as usize)
                .map(|material| material.name.clone())
                .unwrap_or_else(|| format!("unknown_material_{}", material_id.0)),
            tags: tag_names_from_set(tag_set, tag_registry),
        })
        .collect();
    SavedChunkBody {
        schema_version: super::dto::SAVED_CHUNK_BODY_SCHEMA_VERSION,
        chunk: [input.coord.x, input.coord.y],
        cells,
    }
}
