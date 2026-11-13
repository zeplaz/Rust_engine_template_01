// Input/Output systems.
//
// `templates.rs` retired (2026-04-26): the file referenced removed modules
// (`crate::io::deserialzers`, `crate::road_vehicles`) and an undefined
// `RoadVehicleConfigResource`. Active loading paths now live in
// `crate::io::serialization::deserializers`.
mod mouse;
pub mod serialization;
pub mod save;
pub mod snapshot;
pub mod streaming;

pub use mouse::*;
pub use save::{
    build_save_world_manifest, DirtyChunkSaveQueue, SaveWorldManifest, WorldSaveSpinePlugin,
};
pub use snapshot::{read_hybrid_world_snapshot_dev_v0, write_hybrid_world_snapshot_dev_v0, WorldSnapshotHeaderDevV0};
pub use streaming::{
    chunk_window_coords, ghost_band_seed_coords,
    hydrate_all_manifest_chunks, hydrate_stream_chunks_from_manifest, load_manifest_for_streaming,
    preview_coords_with_ghost_bands, ChunkCache, ChunkResidencyTable, ChunkStreamingPriority, ChunkStreamingScheduler,
    PendingStreamApplyQueue, StreamingSpinePlugin, TILE_STORAGE_DIFF_CONTRACT_BQ,
};
