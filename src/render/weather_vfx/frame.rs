//! Weather precip GPU frame — climate-fed + zoom-banded streaks.

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

use crate::render::pipelines::gpu_instanced_quad::GpuInstancedQuadInstance;

/// Presentation frame for domain [`crate::render::ParticleDomainId::WeatherPrecip`].
///
/// **Single writer:** [`super::extract::fill_weather_precip_frame_from_climate`].
/// `gpu_authority` is set by [`super::draw::sync_weather_precip_draw_globals`].
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
pub struct WeatherPrecipFrame {
    pub mean_rain: f32,
    pub mean_snow: f32,
    pub mean_fog: f32,
    pub mean_wind: f32,
    pub weather_chunk_count: u32,
    pub climate_fed: bool,
    pub streaks: Vec<GpuInstancedQuadInstance>,
    pub instance_count: usize,
    pub gpu_authority: bool,
    pub camera_centered: bool,
    /// Snapshot of zoom used for ES-5-4 density.
    pub zoom_alpha: f32,
    pub tactical_band: bool,
}

impl WeatherPrecipFrame {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instance_count == 0
    }
}
