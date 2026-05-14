//! Wave S — world persistence spine (manifest + incremental chunk saves).

mod apply;
mod async_io;
mod dirty_queue;
mod dto;
mod load;
mod manifest;
mod pipeline;
mod registry_snapshot;
mod snapshot_builder;

pub use apply::{apply_pending_save_pipeline_jobs, apply_saved_body_to_materialized_chunk};
pub use async_io::{poll_save_io_completions, SaveIoCompletion, SaveIoDispatcher, SaveIoWorkOrder};
pub use dirty_queue::{
    arm_save_flush_from_requests, enqueue_dirty_chunks_from_environment_hooks,
    enqueue_dirty_chunks_from_preview, DirtyChunkSaveQueue, RequestWorldSaveFlush,
};
pub use dto::{
    decode_chunk_body_ron, encode_chunk_body_ron, SavedChunkBody, SavedTerrainCell,
    SAVED_CHUNK_BODY_SCHEMA_VERSION,
};
pub use load::{
    hydrate_chunk_bodies_from_manifest, load_chunk_body_for_coord, load_chunk_body_from_artifact,
    material_ids_from_saved_body, read_manifest_from_bundle, tag_sets_from_saved_body,
    validate_manifest,
};
pub use manifest::{
    build_save_world_manifest, ChunkSetRef, OverlaySnapshotRef, RegistrySnapshotRef,
    SaveWorldManifest, SAVE_WORLD_MANIFEST_SCHEMA_VERSION,
};
pub use pipeline::{
    build_incremental_save_manifest, chunk_artifact_path, collect_chunk_save_snapshots,
    compress_payload, flush_dirty_chunk_save_queue, flush_dirty_chunk_save_queue_sync,
    stage_dirty_chunk_save_jobs, write_artifact_atomic, write_manifest_atomic,
    PendingSaveApplyQueue, SaveFlushRequested, SavePipelineJob, WorldSaveBundleSettings,
    WorldSaveSeed, WorldSaveSpinePlugin,
};
pub use registry_snapshot::{
    build_default_registry_snapshot_refs, write_registry_snapshot_artifacts,
};
pub use snapshot_builder::{
    build_chunk_save_snapshot_input, build_saved_chunk_body, tag_names_from_set,
    ChunkSaveSnapshotInput,
};

#[cfg(test)]
mod wave_s_governance {
    use super::*;

    #[test]
    fn wave_s_wire_format_uses_material_and_tag_names() {
        let body = SavedChunkBody {
            schema_version: SAVED_CHUNK_BODY_SCHEMA_VERSION,
            chunk: [0, 0],
            cells: vec![SavedTerrainCell {
                material_name: "grass".into(),
                tags: vec!["wet".into()],
            }],
        };
        let encoded = String::from_utf8(encode_chunk_body_ron(&body).unwrap()).unwrap();
        assert!(encoded.contains("material_name"));
        assert!(!encoded.contains("MaterialId"));
    }
}
