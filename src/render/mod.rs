// Rendering systems
mod base_cam;
pub mod extraction;
mod fire_smoke_shader_handles;
mod fx_burst_request;
pub mod gpu_weather_fire_field;
pub mod lighting;
mod light;
pub mod shaders;
pub mod sim_visual_extract;
mod tile_world_fallback;
mod overlay_field_buffers;

#[cfg(feature = "bevy_tilemap_adapter")]
pub mod tilemap_adapter;

pub use tile_world_fallback::{
    SimMinimapUiState, TileWorldFallbackAfterFireExtract, TileWorldFallbackPlugin,
    TileWorldFallbackRasterDirty, TileWorldFallbackSprite,
};

// Public exports
pub use fire_smoke_shader_handles::{
    FireSmokeShaderHandles, FIRE_PARTICLE_WGSL, SMOKE_VOLUME_WGSL,
};
pub use fx_burst_request::FxParticleBurstRequest;
pub use gpu_weather_fire_field::{
    FireVisualGpuInstanceStorage, GpuWeatherFireFieldPlugin, WeatherFireFieldDebugOverlay,
    WeatherFireFieldUniforms,
};
pub use sim_visual_extract::{
    ChunkFireHeat, ChunkSmokeGpu, ClimateVisualAggregate, FireEmitterGpu, FireVisualGpuInstance,
    SimChunkSmokeVisualExtract, SimFireEmitterVisualExtract,
};
pub use lighting::{
    build_fire_light_clusters, FireLightCluster, FireLightEmission as FireLightEmissionSample,
    FireLightType, CLUSTER_MERGE_RADIUS,
};
pub use extraction::{
    infer_combustion_class, infer_fire_emission_profile, material_id_at_chunk_center,
    terrain_family_at_chunk_center, CombustionClass, FireAtmosphereAggregate, FireEmissionProfile,
    FireVisualFrame, FireVisualFramePlugin, FireVisualFrameSet, FireVisualProxy,
};
pub use overlay_field_buffers::{
    SharedOverlayFieldBuffers, SharedOverlayFieldBuffersPlugin,
};
pub use light::*;

#[cfg(feature = "bevy_tilemap_adapter")]
pub use tilemap_adapter::{
    ChunkTilemaps, TilemapAdapterPlugin, TilemapLayerVisibility,
};