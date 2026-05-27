//! Main-thread ECS apply for Wave C streamed chunk bodies.
//!
//! **Invariant (S6-22):** `PendingStreamApplyQueue::ready_bodies` is drained only here on the
//! main thread. TaskPool workers must not mutate ECS; they enqueue into the queue only.
//! Schedule: `reconstruct_staged_chunks_into_cache` → `apply_pending_stream_chunk_bodies` →
//! `clear_async_domain_apply_labels_after_stream_apply` (see `StreamingSpinePlugin`).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::io::save::apply_saved_body_to_materialized_chunk;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::generation::Chunk;
use crate::terrain::material::{
    invalidate_world, InvalidationReason, MaterializedChunk, MaterialRegistry, TagRegistry,
    WorldPreviewState,
};

use super::PendingStreamApplyQueue;

pub fn apply_pending_stream_chunk_bodies(
    mut apply_queue: ResMut<PendingStreamApplyQueue>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    tags: Res<Assets<TagRegistry>>,
    mut chunks: Query<(Entity, &Chunk, &mut MaterializedChunk, Option<&mut ChunkCellMatrix>)>,
    mut preview: Option<ResMut<WorldPreviewState>>,
) {
    if apply_queue.ready_bodies.is_empty() {
        return;
    }
    let Some(material_registry) = materials.get(&handles.material_registry) else {
        return;
    };
    let Some(tag_registry) = tags.get(&handles.tag_registry) else {
        return;
    };
    let bodies = std::mem::take(&mut apply_queue.ready_bodies);
    let mut coord_to_entity: HashMap<IVec2, Entity> = HashMap::with_capacity(chunks.iter().len());
    for (entity, chunk, _, _) in chunks.iter() {
        coord_to_entity.insert(chunk.coord, entity);
    }
    let mut touched = Vec::new();
    for (coord, body) in bodies {
        let Some(&entity) = coord_to_entity.get(&coord) else {
            continue;
        };
        let Ok((_entity, _chunk, mut mat_chunk, mut cell_matrix)) = chunks.get_mut(entity) else {
            continue;
        };
        if apply_saved_body_to_materialized_chunk(
            &mut mat_chunk,
            cell_matrix.as_deref_mut(),
            &body,
            material_registry,
            tag_registry,
        ) {
            touched.push(coord);
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

    #[test]
    fn pending_stream_apply_queue_defaults_empty() {
        let queue = PendingStreamApplyQueue::default();
        assert!(queue.ready_bodies.is_empty());
    }
}
