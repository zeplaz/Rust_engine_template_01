//! Main-thread ECS apply for Wave S saved chunk bodies (IO-07 chunk coord remap).
//!
//! **IO-08** (gea4v1 7.2.2.10 *Post-Load Initialization*): after authoritative bodies land,
//! rebuild **derived** surfaces ([`rebuild_derived_after_chunk_apply`]) — preview invalidation,
//! portal attachment map, logistics route dirty keys. Derived caches are never written into
//! the save body.

use bevy::prelude::*;

use crate::economy::logistics::types::{LogisticsRouteDirtyTiles, PortalAttachmentMap, RouteCache};
use crate::io::save::dto::{decode_chunk_body_ron, SavedChunkBody};
use crate::io::save::load::{material_ids_from_saved_body, tag_sets_from_saved_body};
use crate::io::save::pipeline::PendingSaveApplyQueue;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::systems::transport::TransportEdgeDirectory;
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::generation::Chunk;
use crate::terrain::material::{
    invalidate_world, InvalidationReason, MaterializedChunk, MaterialRegistry, TagRegistry,
    WorldPreviewState,
};

/// Counters for post-load derived rebuilds (diagnostics / Accept).
#[derive(Resource, Clone, Debug, Default)]
pub struct PostLoadDerivedBook {
    pub rebuild_passes: u64,
    pub chunks_touched: u64,
    pub portal_maps_cleared: u64,
    pub route_dirty_keys_marked: u64,
}

/// Optional derived-cache sinks for IO-08 (all Option so minimal harnesses still compile).
pub struct PostLoadDerivedSinks<'a> {
    pub preview: Option<&'a mut WorldPreviewState>,
    pub book: Option<&'a mut PostLoadDerivedBook>,
    pub portal_map: Option<&'a mut PortalAttachmentMap>,
    pub route_dirty: Option<&'a mut LogisticsRouteDirtyTiles>,
    pub route_cache: Option<&'a mut RouteCache>,
    pub directory: Option<&'a TransportEdgeDirectory>,
}

/// Rebuild derived caches after authoritative chunk bodies / overlays were applied (IO-08).
///
/// - Preview: invalidate touched chunk coords.
/// - Portals: clear [`PortalAttachmentMap`] so `rebuild_portal_attachment_map_system` rebinds.
/// - Routes: mark all live directory endpoints dirty and clear [`RouteCache`] (stale paths).
pub fn rebuild_derived_after_chunk_apply(sinks: PostLoadDerivedSinks<'_>, touched: &[IVec2]) {
    if touched.is_empty() {
        return;
    }
    let PostLoadDerivedSinks {
        preview,
        book,
        portal_map,
        route_dirty,
        route_cache,
        directory,
    } = sinks;

    if let Some(preview) = preview {
        invalidate_world(InvalidationReason::Registry, preview, touched.iter().copied());
    }

    let mut portal_cleared = 0_u64;
    if let Some(map) = portal_map {
        map.facility_to_graph.clear();
        map.revision = 0;
        portal_cleared = 1;
    }

    let mut marked = 0_u64;
    if let Some(dirty) = route_dirty {
        if let Some(directory) = directory {
            for meta in directory.by_edge.values() {
                if dirty.keys.insert(meta.head_key.clone()) {
                    marked = marked.saturating_add(1);
                }
                if dirty.keys.insert(meta.tail_key.clone()) {
                    marked = marked.saturating_add(1);
                }
            }
        }
        // Chunk apply with no directory still needs a refresh signal.
        for coord in touched {
            let key = format!("t{}_{}", coord.x, coord.y);
            if dirty.keys.insert(key) {
                marked = marked.saturating_add(1);
            }
        }
    }
    if let Some(cache) = route_cache {
        cache.routes.clear();
    }
    if let Some(book) = book {
        book.rebuild_passes = book.rebuild_passes.saturating_add(1);
        book.chunks_touched = book
            .chunks_touched
            .saturating_add(touched.len() as u64);
        book.portal_maps_cleared = book.portal_maps_cleared.saturating_add(portal_cleared);
        book.route_dirty_keys_marked = book.route_dirty_keys_marked.saturating_add(marked);
    }
}

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
    mut post_load: Option<ResMut<PostLoadDerivedBook>>,
    mut portal_map: Option<ResMut<PortalAttachmentMap>>,
    mut route_dirty: Option<ResMut<LogisticsRouteDirtyTiles>>,
    mut route_cache: Option<ResMut<RouteCache>>,
    directory: Option<Res<TransportEdgeDirectory>>,
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
    rebuild_derived_after_chunk_apply(
        PostLoadDerivedSinks {
            preview: preview.as_deref_mut(),
            book: post_load.as_deref_mut(),
            portal_map: portal_map.as_deref_mut(),
            route_dirty: route_dirty.as_deref_mut(),
            route_cache: route_cache.as_deref_mut(),
            directory: directory.as_deref(),
        },
        &touched,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::save::dto::{
        encode_chunk_body_ron, SavedChunkBody, SavedTerrainCell, SAVED_CHUNK_BODY_SCHEMA_VERSION,
    };
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
            collision: Vec::new(),
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

    #[test]
    fn e7_chunk_body_name_round_trip_matches_runtime_ids() {
        let mut registry = MaterialRegistry {
            schema_version: 1,
            materials: vec![
                MaterialDef {
                    name: "grass".into(),
                    family: TerrainFamilyId(0),
                    tags: Vec::new(),
                    properties: serde_json::json!({}),
                    preview_color: [0, 128, 0, 255],
                },
                MaterialDef {
                    name: "stone".into(),
                    family: TerrainFamilyId(0),
                    tags: Vec::new(),
                    properties: serde_json::json!({}),
                    preview_color: [128, 128, 128, 255],
                },
            ],
            name_to_id: Default::default(),
            collision: Vec::new(),
        };
        registry.name_to_id.insert("grass".into(), MaterialId(0));
        registry.name_to_id.insert("stone".into(), MaterialId(1));
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
            chunk: [3, 2],
            cells: vec![
                SavedTerrainCell {
                    material_name: "grass".into(),
                    tags: vec!["wet".into()],
                },
                SavedTerrainCell {
                    material_name: "stone".into(),
                    tags: vec![],
                },
            ],
        };
        let applied = apply_saved_body_to_chunk_state(&body, &registry, &tag_registry);
        assert_eq!(applied.materials, vec![MaterialId(0), MaterialId(1)]);
        let ron = encode_chunk_body_ron(&body).unwrap();
        let wire = String::from_utf8(ron).unwrap();
        assert!(!wire.contains("MaterialId"));
    }

    #[test]
    fn io08_post_load_rebuild_clears_portals_and_marks_route_dirty() {
        let mut book = PostLoadDerivedBook::default();
        let mut portals = PortalAttachmentMap {
            revision: 9,
            facility_to_graph: Default::default(),
        };
        // Seed a fake facility mapping so clear is observable.
        portals
            .facility_to_graph
            .insert(Entity::from_bits(1), crate::strategic::LogisticsNodeId(0));
        let mut dirty = LogisticsRouteDirtyTiles::default();
        let mut cache = RouteCache::default();
        cache.next_id = 3;

        rebuild_derived_after_chunk_apply(
            PostLoadDerivedSinks {
                preview: None,
                book: Some(&mut book),
                portal_map: Some(&mut portals),
                route_dirty: Some(&mut dirty),
                route_cache: Some(&mut cache),
                directory: None,
            },
            &[IVec2::new(1, 2)],
        );
        assert_eq!(book.rebuild_passes, 1);
        assert_eq!(book.chunks_touched, 1);
        assert_eq!(book.portal_maps_cleared, 1);
        assert!(portals.facility_to_graph.is_empty());
        assert_eq!(portals.revision, 0);
        assert!(dirty.keys.contains("t1_2"));
        assert!(cache.routes.is_empty());

        rebuild_derived_after_chunk_apply(
            PostLoadDerivedSinks {
                preview: None,
                book: Some(&mut book),
                portal_map: None,
                route_dirty: None,
                route_cache: None,
                directory: None,
            },
            &[],
        );
        assert_eq!(book.rebuild_passes, 1, "empty touch must not count");
    }
}
