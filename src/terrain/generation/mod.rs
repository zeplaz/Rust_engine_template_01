// Terrain generation systems
pub mod chunk;
pub mod cell_matrix;
mod geo_plugin;
mod world_generator;
mod bevy_terrain_gen;
pub mod passes;
pub mod hydrology;
pub mod terrain_noise;
pub mod tuning_io;
pub mod world_generator_enhanced;
pub mod world_gen_diagnostics;
mod world_generation_plugin;

pub use passes::{materialize, MaterializedChunkData};

// Public exports
pub use cell_matrix::ChunkCellMatrix;
pub use chunk::Chunk;
pub use geo_plugin::*;
pub use world_generator::*;
pub use bevy_terrain_gen::*;
pub use world_gen_diagnostics::WorldGenLastDebugReport;
pub use world_generator_enhanced::*;
pub use world_generation_plugin::*;