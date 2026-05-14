//! Wave S → Wave C bridge — load incremental chunk bodies for streaming deserialize.

use std::path::Path;

use bevy::prelude::IVec2;

use crate::io::save::{
    hydrate_chunk_bodies_from_manifest, load_chunk_body_for_coord, read_manifest_from_bundle,
    SavedChunkBody, SaveWorldManifest,
};

#[must_use]
pub fn load_manifest_for_streaming(bundle_dir: &Path) -> Option<SaveWorldManifest> {
    read_manifest_from_bundle(bundle_dir).ok()
}

#[must_use]
pub fn hydrate_stream_chunks_from_manifest(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
    chunks: &[IVec2],
) -> Vec<(IVec2, SavedChunkBody)> {
    chunks
        .iter()
        .filter_map(|chunk| {
            load_chunk_body_for_coord(bundle_dir, manifest, *chunk)
                .ok()
                .flatten()
                .map(|body| (*chunk, body))
        })
        .collect()
}

#[must_use]
pub fn hydrate_all_manifest_chunks(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
) -> Vec<(IVec2, SavedChunkBody)> {
    hydrate_chunk_bodies_from_manifest(bundle_dir, manifest)
        .ok()
        .map(|bodies| {
            bodies
                .into_iter()
                .filter_map(|body| {
                    let chunk = IVec2::new(body.chunk[0], body.chunk[1]);
                    Some((chunk, body))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::save::{
        flush_dirty_chunk_save_queue_sync, ChunkSaveSnapshotInput,
    };
    use crate::terrain::family::TerrainFamilyId;
    use crate::terrain::material::{MaterialDef, MaterialId, TagDef, TagId, TagRegistry, TagSet};

    #[test]
    fn stream_hydrate_reads_incremental_bundle_chunk() {
        let dir = std::env::temp_dir().join(format!(
            "wave_c_hydrate_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut registry = crate::terrain::material::MaterialRegistry {
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
        let mut tags = TagSet::default();
        tags.insert(TagId(0));
        let snapshot = ChunkSaveSnapshotInput {
            coord: IVec2::new(4, 5),
            materials: vec![MaterialId(0)],
            tags: vec![tags],
        };
        let (manifest, _) = flush_dirty_chunk_save_queue_sync(
            &dir,
            7,
            &[IVec2::new(4, 5)],
            &[snapshot],
            &registry,
            &tag_registry,
        )
        .unwrap();
        let loaded = hydrate_stream_chunks_from_manifest(&dir, &manifest, &[IVec2::new(4, 5)]);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1.cells[0].material_name, "grass");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
