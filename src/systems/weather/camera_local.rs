//! W3 — camera-local weather sample for precip / overlay / HUD (single VisualExtract writer).

use bevy::prelude::*;

use crate::render::extraction::extracted_camera_metrics::ExtractedCameraMetrics;
use crate::systems::sim_control::SimTimeMicros;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

use super::chunk_weather::{CHUNK_RAIN_FULL_SCALE_MM_H, CHUNK_VISIBILITY_FULL_SCALE_M};
use super::field::{WeatherField, WeatherSample};
use super::weather_visual::WeatherPrecipVisualSample;

/// Local weather at the main-camera focus — precip VFX + overlay authority (not world means).
#[derive(Resource, Clone, Debug)]
pub struct CameraLocalWeatherSample {
    pub sample: WeatherSample,
    pub rain01: f32,
    pub snow01: f32,
    pub fog: f32,
    pub pos_m: Vec2,
    pub ready: bool,
}

impl Default for CameraLocalWeatherSample {
    fn default() -> Self {
        Self {
            sample: WeatherSample::default(),
            rain01: 0.0,
            snow01: 0.0,
            fog: 0.0,
            pos_m: Vec2::ZERO,
            ready: false,
        }
    }
}

#[must_use]
pub fn rain_snow01_from_sample(s: &WeatherSample) -> (f32, f32) {
    let rate01 = (s.rain_mm_h / CHUNK_RAIN_FULL_SCALE_MM_H).clamp(0.0, 1.0);
    if s.snow {
        (0.0, rate01)
    } else {
        (rate01, 0.0)
    }
}

/// Sole writer of [`CameraLocalWeatherSample`]; also refreshes [`WeatherPrecipVisualSample`] for overlay/HUD.
pub fn publish_camera_local_weather_sample(
    field: Res<WeatherField>,
    sim_time: Option<Res<SimTimeMicros>>,
    cam: Res<ExtractedCameraMetrics>,
    mut out: ResMut<CameraLocalWeatherSample>,
    mut precip: ResMut<WeatherPrecipVisualSample>,
) {
    if !field.is_ready() {
        out.ready = false;
        return;
    }
    let t = sim_time
        .as_deref()
        .copied()
        .unwrap_or(SimTimeMicros(field.prev_micros));
    let pos_m = cam.translation * TILE_METRES_GAMEPLAY;
    let s = field.sample(pos_m, t);
    let (rain01, snow01) = rain_snow01_from_sample(&s);
    let fog = s.fog.clamp(0.0, 1.0);
    *out = CameraLocalWeatherSample {
        sample: s,
        rain01,
        snow01,
        fog,
        pos_m,
        ready: true,
    };
    *precip = WeatherPrecipVisualSample {
        rain: rain01,
        snow: snow01,
        fog,
        chunk_count: precip.chunk_count.max(1),
        visibility_factor: (s.visibility_m / CHUNK_VISIBILITY_FULL_SCALE_M).clamp(0.05, 1.0),
        wind_mps: s.wind_mps,
        gust_mps: s.gust_mps,
        cloud: s.cloud.clamp(0.0, 1.0),
        rain_mm_h: if s.snow { 0.0 } else { s.rain_mm_h.max(0.0) },
        visibility_m: s.visibility_m.max(0.0),
        local_ready: true,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rain_snow_split_from_sample() {
        let mut s = WeatherSample::default();
        s.rain_mm_h = 5.0;
        s.snow = false;
        let (r, sn) = rain_snow01_from_sample(&s);
        assert!((r - 0.5).abs() < 1e-4);
        assert_eq!(sn, 0.0);
        s.snow = true;
        let (r2, sn2) = rain_snow01_from_sample(&s);
        assert_eq!(r2, 0.0);
        assert!((sn2 - 0.5).abs() < 1e-4);
    }
}
