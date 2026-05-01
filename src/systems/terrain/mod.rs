//! Terrain-facing systems (material unification, future streaming).

pub mod material_plugin;

pub use material_plugin::{materialize_chunks, MaterialUnificationPlugin, TerrainRegistriesHandles};
