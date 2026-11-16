//! Wave S → Wave C bridge — load incremental chunk bodies for streaming deserialize.
//!
//! **CONC-09** — When to make asynchronous requests (gea4v2 17.7.2 / 17.7.2.1): issue early, consume a frame later.
//! LEGACY: `async` / `job` / `streaming` hydrate (gea_v2_e4 p621-624).
//!
//! **IO-08** — This module only loads authoritative bodies. Derived caches are rebuilt on the
//! main-thread apply path (`io::save::apply::rebuild_derived_after_chunk_apply` /
//! `post_load_rebuild`), never persisted into streaming products.
//!
//! These fns are the **worker-side blocking body**: pure disk → [`SavedChunkBody`] products, no ECS
//! or scheduler access. Production callers must reach them through
//! [`super::task_pool::submit_stream_hydrate_work`] (request, early) and consume the products via
//! [`super::task_pool::apply_stream_io_completion`] (main thread, a frame later). The only direct
//! main-thread call is the opt-in `STREAMING_SYNC_HYDRATE` fallback in `streaming/mod.rs`.

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
    fn hydrate_request_issued_early_products_applied_a_frame_later() {
        use crate::io::streaming::task_pool::apply_stream_io_completion;
        use crate::io::streaming::{
            ChunkStreamIoDispatcher, ChunkStreamingScheduler, StreamHydrateDiagnostics,
            StreamIoWorkOrder,
        };
        // CONC-09: submit returns immediately with the scheduler untouched; products land
        // only when the main thread applies the polled completion.
        let dir = std::env::temp_dir().join("conc09_missing_bundle_never_created");
        let mut dispatcher = ChunkStreamIoDispatcher::default();
        let mut scheduler = ChunkStreamingScheduler::default();
        let mut diagnostics = StreamHydrateDiagnostics::default();
        assert!(dispatcher.submit(StreamIoWorkOrder {
            bundle_dir: dir,
            chunks: vec![IVec2::new(1, 1)],
        }));
        assert!(dispatcher.in_flight);
        assert!(scheduler.staged_chunk_bodies.is_empty());
        assert_eq!(diagnostics.main_thread_applies, 0);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let completion = loop {
            if let Some(done) = dispatcher.poll_completion() {
                break done;
            }
            assert!(std::time::Instant::now() < deadline, "hydrate worker never completed");
            std::thread::yield_now();
        };
        assert!(!dispatcher.in_flight);
        assert!(
            scheduler.staged_chunk_bodies.is_empty(),
            "poll_completion must not apply"
        );
        apply_stream_io_completion(completion, &mut scheduler, &mut diagnostics);
        assert!(scheduler.staged_chunk_bodies.is_empty()); // missing manifest -> Failed, nothing staged
        assert!(diagnostics.last_failure_message.is_some());
    }

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
