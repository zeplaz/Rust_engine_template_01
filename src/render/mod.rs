// Rendering systems
mod base_cam;
pub mod gpu_weather_fire_field;
mod light;
pub mod shaders;
mod tile_world_fallback;

#[cfg(feature = "bevy_tilemap_adapter")]
pub mod tilemap_adapter;

pub use tile_world_fallback::{SimMinimapUiState, TileWorldFallbackPlugin, TileWorldFallbackSprite};

// Public exports
pub use gpu_weather_fire_field::{
    GpuWeatherFireFieldPlugin, WeatherFireFieldDebugOverlay, WeatherFireFieldUniforms,
};
pub use light::*;

#[cfg(feature = "bevy_tilemap_adapter")]
pub use tilemap_adapter::{
    ChunkTilemaps, TilemapAdapterPlugin, TilemapLayerVisibility,
};