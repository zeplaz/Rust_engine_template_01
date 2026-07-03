// Terrain generation systems
pub mod chunk;
pub mod chunk_worldgen_scheduler;
pub mod cell_matrix;
pub mod tile_chunk_map;
mod editor_chunk_tile_sync;
pub mod derived;
mod geo_plugin;
mod world_generator;
mod bevy_terrain_gen;
pub mod passes;
pub mod hydrology;
pub mod terrain_noise;
pub mod tuning_io;
pub mod polygon_world_semantics;
pub mod world_generator_enhanced;
mod world_gen_dense_cache;
pub mod world_gen_diagnostics;
mod world_generation_plugin;

pub use passes::{materialize, MaterializedChunkData};

// Public exports
pub use cell_matrix::ChunkCellMatrix;
pub use chunk_worldgen_scheduler::{
    dispatch_chunk_jobs, generate_chunk_cpu_height_moisture_temp, queue_mission_hint_jobs, queue_visible_chunks,
    ChunkGenCameraWindow, ChunkGenConfig, ChunkGenJob, ChunkGenMissionChunkHints, ChunkGenQueue, ChunkGenReason,
    ChunkTexturePatchQueue, ChunkWorldgenSchedulerPlugin, GpuChunkGenPipeline,
};
pub use tile_chunk_map::{
    brush_tile_inclusive_bounds, tile_rect_to_chunk_coords, tile_to_chunk_coord,
};
pub use editor_chunk_tile_sync::sync_tile_markers_into_affected_chunk_matrices;
pub use derived::{compute_slope_grade, stitch_all_chunk_slope_grades, stitch_chunk_slope_grades, ChunkDerivedMetrics};
pub use chunk::Chunk;
pub use geo_plugin::*;
pub use world_generator::*;
pub use bevy_terrain_gen::*;
pub use world_gen_diagnostics::WorldGenLastDebugReport;
pub use world_gen_dense_cache::{
    hydrate_chunk_matrices_from_dense_terrain, WorldGenDenseTerrainCache,
};
pub use world_generator_enhanced::*;
pub use world_generation_plugin::*;