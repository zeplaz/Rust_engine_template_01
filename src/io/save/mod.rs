//! Wave S — world persistence spine (manifest + incremental chunk saves).

mod autosave;
mod apply;
mod async_io;
mod dirty_queue;
mod dto;
mod load;
mod manifest;
mod pipeline;
mod registry_snapshot;
mod snapshot_builder;
mod settlement_overlay;
mod fire_overlay;
mod atmosphere_clipmap_overlay;
mod transport_overlay;
mod wave_s_artifacts;
mod wire_format;

pub use autosave::{tick_world_save_autosave, WorldSaveAutosaveSettings};
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
    flush_dirty_chunk_save_queue, flush_dirty_chunk_save_queue_sync,
    stage_dirty_chunk_save_jobs, write_artifact_atomic, write_manifest_atomic,
    PendingSaveApplyQueue, SaveFlushRequested, SavePipelineJob, WorldSaveBundleSettings,
    WorldSaveSeed, WorldSaveSpinePlugin,
};
pub use registry_snapshot::{
    build_default_registry_snapshot_refs, write_registry_snapshot_artifacts,
};
pub use settlement_overlay::{
    build_settlement_overlay_refs, econ_og_save_001_witness_green,
    settlement_books_manifest_roundtrip_witness_green,
    settlement_books_save_roundtrip_witness_green, try_hydrate_settlement_books_from_manifest,
    try_hydrate_settlement_books_on_bundle_dir, write_settlement_overlay_on_save_flush,
    write_settlement_overlay_to_bundle, SettlementBooksHydrateState, SETTLEMENT_BOOKS_REL_PATH,
    SETTLEMENT_OVERLAY_NAME,
};
pub use fire_overlay::{
    apply_chunk_fire_state, capture_chunk_fire_state, capture_chunk_fire_state_from_rows,
    default_fire_overlay_ref, fire_overlay_ref, fire_save_roundtrip_lib_green,
    hash_saved_fire_state, hash_surface_fire_rows, read_chunk_fire_state_ron,
    refresh_fire_save_roundtrip_live_witness, write_chunk_fire_state_ron, ChunkFireStateSnapshot,
    SavedFireChunkRow, FIRE_OVERLAY_NAME, FIRE_SAVE_ROUNDTRIP_JSON, FIRE_STATE_REL_PATH,
    FIRE_STATE_SCHEMA_VERSION,
};
pub use atmosphere_clipmap_overlay::{
    atmosphere_clipmap_manifest_roundtrip_green, atmosphere_clipmap_save_roundtrip_green,
    build_atmos_clipmap_overlay_refs, capture_atmosphere_clipmap_snapshot,
    default_atmos_clipmap_overlay_ref, hydrate_atmosphere_clipmap_snapshot,
    read_atmosphere_clipmap_snapshot, refresh_atmosphere_clipmap_save_roundtrip_witness,
    try_hydrate_atmosphere_clipmap_from_manifest, write_atmosphere_clipmap_overlay_to_bundle,
    write_atmosphere_clipmap_snapshot, AtmosphereClipmapSnapshot, ATMOS_CLIPMAP_OVERLAY_NAME,
    ATMOS_CLIPMAP_SAVE_ROUNDTRIP_JSON, ATMOS_CLIPMAP_SCHEMA_VERSION, ATMOS_CLIPMAP_STATE_REL_PATH,
};
pub use transport_overlay::{
    read_transport_snapshot_ron, transport_overlay_ref, write_transport_snapshot_ron,
    TRANSPORT_OVERLAY_NAME,
};
pub use wave_s_artifacts::{
    apply_wave_s_shell_capture_requests, apply_wave_s_shell_restore_requests,
    hydrate_wave_s_artifacts_from_bundle, product_shell_bundle_exists, read_blueprint_presets,
    read_product_shell_bundle, try_autoload_wave_s_on_bundle_dir, wave_s_autoload_shell_enabled,
    write_blueprint_presets, write_product_shell_bundle, WaveSImportedBlueprints,
    WaveSShellCapturePending, WaveSShellHydrateState, WaveSShellHydrateWitness,
    WaveSShellRestorePending, WAVE_S_BLUEPRINT_PRESETS_REL_PATH, WAVE_S_PRODUCT_SHELL_REL_PATH,
};
pub use crate::dev::runtime_witness::wave_s::{
    build_wave_s_hydrate_proof_payload, write_wave_s_hydrate_live_proof_system, WaveSLiveProofState,
    WAVE_S_HYDRATE_JSON,
};
pub use wire_format::{
    active_chunk_artifact_body_kind, active_save_payload_compression, active_save_wire_format,
    compress_payload, decompress_payload, unwrap_chunk_artifact_body, wrap_chunk_artifact_body,
    SaveArtifactBodyKind, SavePayloadCompression, SaveWireFormat, SAVE_ARTIFACT_ENVELOPE_VERSION,
    SAVE_ARTIFACT_MAGIC, SAVE_BINARY_BULK_DEFERRED,
};
pub use snapshot_builder::{
    build_chunk_save_snapshot_input, build_saved_chunk_body, tag_names_from_set,
    ChunkSaveSnapshotInput,
};

#[cfg(test)]
mod wave_s_governance {
    use super::*;

    #[test]
    fn wave_s_chunk_artifact_envelope_wraps_ron_identity() {
        let body = SavedChunkBody {
            schema_version: SAVED_CHUNK_BODY_SCHEMA_VERSION,
            chunk: [0, 0],
            cells: vec![SavedTerrainCell {
                material_name: "grass".into(),
                tags: vec!["wet".into()],
            }],
        };
        let encoded = encode_chunk_body_ron(&body).unwrap();
        let wrapped = compress_payload(&encoded);
        assert_eq!(&wrapped[..4], SAVE_ARTIFACT_MAGIC);
        assert_eq!(active_save_payload_compression(), SavePayloadCompression::Identity);
        assert_eq!(active_chunk_artifact_body_kind(), SaveArtifactBodyKind::RonChunkTextual);
        let decoded = decode_chunk_body_ron(unwrap_chunk_artifact_body(&wrapped).unwrap()).unwrap();
        assert_eq!(decoded, body);
    }

    #[test]
    fn wave_s_wire_format_uses_material_and_tag_names() {
        use crate::io::save::active_save_wire_format;
        use crate::io::save::SaveWireFormat;

        assert_eq!(active_save_wire_format(), SaveWireFormat::RonTextual);
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
