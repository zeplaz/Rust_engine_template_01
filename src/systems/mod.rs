// Game systems
pub mod agents;
pub mod chunk_environment_persist;
pub mod chunk_environment_set;
pub mod chunk_sim_lod;
pub mod collision;
pub mod damage;
pub mod ecology;
pub mod fire;
pub mod navigation;
pub mod production;
pub mod sim_control;
pub mod terrain;
pub mod transport;
pub mod weather;

// Public exports
pub use agents::*;
pub use damage::*;
pub use chunk_environment_persist::{
    ChunkEnvironmentDirty, ChunkEnvironmentPersistHooks, ChunkEnvironmentPersistPlugin,
};
pub use chunk_environment_set::configure_chunk_environment_sets;
pub use chunk_sim_lod::{ChunkSimLod, ChunkSimLodPlugin};
pub use ecology::{ChunkEcology, EcologyPlugin};
pub use fire::*;
pub use navigation::*;
pub use production::*;
pub use sim_control::*;
pub use terrain::*;
pub use transport::*;
pub use weather::*;