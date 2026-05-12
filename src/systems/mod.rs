// Game systems
pub mod agents;
pub mod atmosphere;
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
pub use atmosphere::{
    advect_atmosphere_field,
    atmosphere_field_blend_fire_overlay_sources,
    atmosphere_field_fill_from_chunks,
    atmosphere_overlay_rgba,
    configure_atmosphere_pipeline_sets,
    merge_atmosphere_into_logistics_sample,
    visibility_between,
    ATMOSPHERE_ASHFALL_WGSL, ATMOSPHERE_GROUND_HAZE_WGSL, ATMOSPHERE_HEAT_DISTORTION_WGSL,
    ATMOSPHERE_PARTICLE_INSTANCING_WGSL, ATMOSPHERE_SMOKE_COLUMN_WGSL, WEATHER_FIRE_FIELD_WGSL,
    ATMOSPHERE_VALIDATION_LAYOUT_V1,
    AtmosphereCell, AtmosphereDiagnostics, AtmosphereField, AtmosphereParticle,
    AtmosphereParticleBudget, AtmosphereParticleKind, AtmospherePerfThresholds,
    AtmospherePipelineSet, AtmospherePlugin, AtmosphereRenderLayers,
    AtmosphereValidationRegion, ChunkSmokeGpu, FireEmitter, FireEmitterGpu,
    GlobalWind, OverlayMode, SimChunkSmokeVisualExtract, SimFireEmitterVisualExtract,
    fire_emitter_from_heat_fuel,
    tile_in_any_validation_region,
};
pub use damage::*;
pub use chunk_environment_persist::{
    ChunkEnvironmentDirty, ChunkEnvironmentPersistHooks, ChunkEnvironmentPersistPlugin,
};
pub use chunk_environment_set::configure_chunk_environment_sets;
pub use chunk_sim_lod::{ChunkSimLod, ChunkSimLodPlugin};
pub use ecology::{
    chunk_ecology_tick, derive_vegetation_structure, integrate_vegetation_field_step,
    succession_stage_from_vegetation, ChunkEcology, EcologicalSuccessionStage, EcologyPlugin,
    VegetationField, VegetationStructure,
};
pub use fire::*;
pub use navigation::*;
pub use production::*;
pub use sim_control::*;
pub use terrain::*;
pub use transport::*;
pub use weather::*;