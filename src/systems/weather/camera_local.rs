//! W3 — camera-local weather sample for precip / overlay / HUD (single VisualExtract writer).

use bevy::prelude::*;

use crate::render::extraction::extracted_camera_metrics::ExtractedCameraMetrics;
use crate::systems::atmosphere::AtmosphereFrame;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

use super::chunk_weather::{CHUNK_RAIN_FULL_SCALE_MM_H, CHUNK_VISIBILITY_FULL_SCALE_M};
use super::field::WeatherSample;
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
    /// [`AtmosphereFrame::revision`] this sample was read from (presentation ↔ sim revision parity).
    pub frame_revision: u64,
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
            frame_revision: 0,
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
///
/// W2: reads [`AtmosphereFrame`] only (bilinear over chunk-centre samples) — never `WeatherField`.
pub fn publish_camera_local_weather_sample(
    frame: Res<AtmosphereFrame>,
    cam: Res<ExtractedCameraMetrics>,
    mut out: ResMut<CameraLocalWeatherSample>,
    mut precip: ResMut<WeatherPrecipVisualSample>,
) {
    let pos_m = cam.translation * TILE_METRES_GAMEPLAY;
    let Some(s) = frame.weather_at_pos_m(pos_m) else {
        out.ready = false;
        return;
    };
    let (rain01, snow01) = rain_snow01_from_sample(&s);
    let fog = s.fog.clamp(0.0, 1.0);
    *out = CameraLocalWeatherSample {
        sample: s,
        rain01,
        snow01,
        fog,
        pos_m,
        ready: true,
        frame_revision: frame.revision(),
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

    use crate::systems::atmosphere::AtmosphereFrame;
    use crate::systems::weather::{WeatherField, WeatherTuning};
    use crate::terrain::world_scale_contract::CHUNK_TILES_SIM;

    #[test]
    fn camera_sample_comes_from_frame_not_field() {
        let extent = Vec2::splat(8.0 * CHUNK_TILES_SIM as f32 * TILE_METRES_GAMEPLAY);
        let field = WeatherField::new(77, extent, &WeatherTuning::default(), None, Default::default(), 0);
        let mut stack = crate::substrate::atmosphere::AtmosphereClipmapStack::default();
        stack.bump_revision();
        let mut frame = AtmosphereFrame::default();
        assert!(frame.rebuild(Some(&field), &stack, 4_000_000, &WeatherTuning::default()));
        let tile_pos = Vec2::new(100.0, 140.0);
        let want = frame
            .weather_at_pos_m(tile_pos * TILE_METRES_GAMEPLAY)
            .expect("camera inside field extent");
        let rev = frame.revision();

        // No `WeatherField` resource is inserted: the system must not need it.
        let mut app = App::new();
        app.insert_resource(frame)
            .insert_resource(ExtractedCameraMetrics {
                translation: tile_pos,
                ..Default::default()
            })
            .init_resource::<CameraLocalWeatherSample>()
            .init_resource::<WeatherPrecipVisualSample>()
            .add_systems(Update, publish_camera_local_weather_sample);
        app.update();

        let out = app.world().resource::<CameraLocalWeatherSample>();
        assert!(out.ready);
        assert_eq!(out.sample, want);
        assert_eq!(out.frame_revision, rev);
        let precip = app.world().resource::<WeatherPrecipVisualSample>();
        assert!(precip.local_ready);
        assert!((precip.rain_mm_h - if want.snow { 0.0 } else { want.rain_mm_h.max(0.0) }).abs() < 1e-5);
    }

    #[test]
    fn unbuilt_frame_leaves_camera_sample_not_ready() {
        let mut app = App::new();
        app.init_resource::<AtmosphereFrame>()
            .init_resource::<ExtractedCameraMetrics>()
            .init_resource::<CameraLocalWeatherSample>()
            .init_resource::<WeatherPrecipVisualSample>()
            .add_systems(Update, publish_camera_local_weather_sample);
        app.update();
        assert!(!app.world().resource::<CameraLocalWeatherSample>().ready);
    }

    /// W2 gate: this module reads the frame, never the field authority.
    #[test]
    fn publisher_does_not_read_weather_field() {
        let src = include_str!("camera_local.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap();
        assert!(!prod.contains("Res<WeatherField>"));
        assert!(!prod.contains("field.sample("));
    }

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
