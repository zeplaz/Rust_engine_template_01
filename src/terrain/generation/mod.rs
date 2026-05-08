// Terrain generation systems
pub mod chunk;
pub mod cell_matrix;
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
pub mod world_gen_diagnostics;
mod world_generation_plugin;

pub use passes::{materialize, MaterializedChunkData};

// Public exports
pub use cell_matrix::ChunkCellMatrix;
pub use derived::{compute_slope_grade, stitch_all_chunk_slope_grades, stitch_chunk_slope_grades, ChunkDerivedMetrics};
pub use chunk::Chunk;
pub use geo_plugin::*;
pub use world_generator::*;
pub use bevy_terrain_gen::*;
pub use world_gen_diagnostics::WorldGenLastDebugReport;
pub use world_generator_enhanced::*;
pub use world_generation_plugin::*;