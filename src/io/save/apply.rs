//! Main-thread ECS apply for Wave S saved chunk bodies.

use bevy::prelude::*;

use crate::io::save::dto::{decode_chunk_body_ron, SavedChunkBody};
use crate::io::save::load::{material_ids_from_saved_body, tag_sets_from_saved_body};
use crate::io::save::pipeline::PendingSaveApplyQueue;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::generation::Chunk;
use crate::terrain::material::{
    invalidate_world, InvalidationReason, MaterializedChunk, MaterialRegistry, TagRegistry,
    WorldPreviewState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedChunkState {
    pub materials: Vec<crate::terrain::material::MaterialId>,
    pub tags: Vec<crate::terrain::material::TagSet>,
}

#[must_use]
pub fn apply_saved_body_to_chunk_state(
    body: &SavedChunkBody,
    materials: &MaterialRegistry,
    tags: &TagRegistry,
) -> AppliedChunkState {
    AppliedChunkState {
        materials: material_ids_from_saved_body(body, materials),
        tags: tag_sets_from_saved_body(body, tags),
    }
}

pub fn apply_saved_body_to_materialized_chunk(
    mat_chunk: &mut MaterializedChunk,
    cell_matrix: Option<&mut ChunkCellMatrix>,
    body: &SavedChunkBody,
    materials: &MaterialRegistry,
    tags: &TagRegistry,
) -> bool {
    if body.cells.is_empty() {
        return false;
    }
    let applied = apply_saved_body_to_chunk_state(body, materials, tags);
    if applied.materials.is_empty() {
        return false;
    }
    let cell_count = applied.materials.len() as u32;
    if mat_chunk.size.x > 0
        && mat_chunk.size.y > 0
        && (mat_chunk.size.x * mat_chunk.size.y) as usize == applied.materials.len()
    {
        mat_chunk.materials = applied.materials;
    } else {
        let width = mat_chunk.size.x.max(1);
        let height = (cell_count + width - 1) / width;
        mat_chunk.size = UVec2::new(width, height.max(1));
        mat_chunk.materials = applied.materials;
    }
    if let Some(matrix) = cell_matrix {
        if matrix.tags.len() == applied.tags.len() {
            matrix.tags = applied.tags;
        }
    }
    true
}

pub fn apply_pending_save_pipeline_jobs(
    mut pending: ResMut<PendingSaveApplyQueue>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    tags: Res<Assets<TagRegistry>>,
    mut chunks: Query<(Entity, &Chunk, &mut MaterializedChunk, Option<&mut ChunkCellMatrix>)>,
    mut preview: Option<ResMut<WorldPreviewState>>,
) {
    if pending.jobs.is_empty() {
        return;
    }
    let Some(material_registry) = materials.get(&handles.material_registry) else {
        return;
    };
    let Some(tag_registry) = tags.get(&handles.tag_registry) else {
        return;
    };
    let jobs = std::mem::take(&mut pending.jobs);
    let mut touched = Vec::new();
    for job in jobs {
        let Ok(body) = decode_chunk_body_ron(&job.body_bytes) else {
            continue;
        };
        for (_entity, chunk, mut mat_chunk, mut cell_matrix) in chunks.iter_mut() {
            if chunk.coord != job.chunk {
                continue;
            }
            if apply_saved_body_to_materialized_chunk(
                &mut mat_chunk,
                cell_matrix.as_deref_mut(),
                &body,
                material_registry,
                tag_registry,
            ) {
                touched.push(chunk.coord);
            }
            break;
        }
    }
    if let Some(preview) = preview.as_mut() {
        if !touched.is_empty() {
            invalidate_world(InvalidationReason::Registry, preview, touched.iter().copied());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::save::dto::{SavedChunkBody, SavedTerrainCell, SAVED_CHUNK_BODY_SCHEMA_VERSION};
    use crate::terrain::family::TerrainFamilyId;
    use crate::terrain::material::{MaterialDef, MaterialId, TagDef, TagId};

    #[test]
    fn apply_saved_body_updates_materialized_chunk_cells() {
        let mut registry = MaterialRegistry {
            schema_version: 1,
            materials: vec![MaterialDef {
                name: "grass".into(),
                family: TerrainFamilyId(0),
                tags: Vec::new(),
                properties: serde_json::json!({}),
                preview_color: [0, 128, 0, 255],
            }],
            name_to_id: Default::default(),
        };
        registry.name_to_id.insert("grass".into(), MaterialId(0));
        let tag_registry = TagRegistry {
            schema_version: 1,
            tags: vec![TagDef {
                name: "wet".into(),
                category: "moisture".into(),
            }],
            name_to_id: [("wet".into(), TagId(0))].into_iter().collect(),
        };
        let body = SavedChunkBody {
            schema_version: SAVED_CHUNK_BODY_SCHEMA_VERSION,
            chunk: [0, 0],
            cells: vec![SavedTerrainCell {
                material_name: "grass".into(),
                tags: vec!["wet".into()],
            }],
        };
        let mut mat_chunk = MaterializedChunk {
            size: UVec2::ONE,
            materials: vec![MaterialId(0)],
        };
        assert!(apply_saved_body_to_materialized_chunk(
            &mut mat_chunk,
            None,
            &body,
            &registry,
            &tag_registry,
        ));
        assert_eq!(mat_chunk.materials, vec![MaterialId(0)]);
    }
}
