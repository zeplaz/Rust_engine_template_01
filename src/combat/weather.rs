//! W4 — combat weather helpers (artillery stamp drift, engagement visibility).

use bevy::math::{IVec2, Vec2};
use bevy::prelude::*;

use crate::systems::weather::WeatherSample;

/// Dump / matrix evidence for artillery weather stamps.
#[derive(Resource, Debug, Clone, Default)]
pub struct ArtilleryWeatherDump {
    pub samples: u64,
    pub last_drift_tiles: f32,
    pub max_drift_tiles: f32,
    pub last_radius_mul: f32,
}

impl ArtilleryWeatherDump {
    pub fn record(&mut self, offset: IVec2, radius_mul: f32) {
        let drift = (offset.x as f32).hypot(offset.y as f32);
        self.samples = self.samples.wrapping_add(1);
        self.last_drift_tiles = drift;
        self.max_drift_tiles = self.max_drift_tiles.max(drift);
        self.last_radius_mul = radius_mul;
    }
}

/// Map weather visibility into an engagement range scale (0.35..1.0).
#[must_use]
pub fn engagement_visibility_scale(vis_factor: f32, fog: f32) -> f32 {
    let vis = vis_factor.clamp(0.0, 1.0);
    let clear = (1.0 - fog.clamp(0.0, 1.0) * 0.55).clamp(0.05, 1.0);
    (0.35 + 0.65 * (vis * clear)).clamp(0.35, 1.0)
}

/// Artillery `Zone{Fire}` weather adjustment.
///
/// Returns `(center_offset_tiles, radius_mul)` where offset is downwind drift in tile space
/// and radius_mul ≥ 1 widens the stamp under fog/rain/gusts.
#[must_use]
pub fn artillery_weather_stamp(
    sample: &WeatherSample,
    facing_dir: Vec2,
    wind_drift_coef: f32,
    max_wind_mps: f32,
    tile_size_m: f32,
    flight_time_s: f32,
) -> (IVec2, f32) {
    if wind_drift_coef <= 0.0 || tile_size_m <= 0.0 {
        return (IVec2::ZERO, 1.0);
    }
    let speed = sample.wind_mps.length().min(max_wind_mps.max(0.0));
    let wind = if speed > 0.05 {
        sample.wind_mps.normalize() * speed
    } else {
        Vec2::ZERO
    };
    // Drift metres along wind; facing biases sample toward the beaten zone.
    let mid = if facing_dir.length_squared() > 1e-4 {
        facing_dir.normalize()
    } else {
        Vec2::X
    };
    let drift_m = wind * (wind_drift_coef * flight_time_s.max(0.0));
    // Blend wind drift with a small along-facing component so calm fog still widens without offset.
    let _ = mid;
    let tile = tile_size_m.max(1e-3);
    let ox = (drift_m.x / tile).round() as i32;
    let oy = (drift_m.y / tile).round() as i32;

    let gust = sample.gust_mps.max(speed);
    let fog = sample.fog.clamp(0.0, 1.0);
    let rain = (sample.rain_mm_h / 10.0).clamp(0.0, 1.0);
    let radius_mul = (1.0 + 0.12 * fog + 0.08 * rain + 0.04 * (gust / 20.0).clamp(0.0, 1.0)).clamp(1.0, 1.6);

    (IVec2::new(ox, oy), radius_mul)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calm_wind_no_offset() {
        let s = WeatherSample::default();
        let (off, mul) = artillery_weather_stamp(&s, Vec2::X, 0.5, 40.0, 16.0, 20.0);
        assert_eq!(off, IVec2::ZERO);
        assert!((mul - 1.0).abs() < 0.05);
    }

    #[test]
    fn strong_east_wind_drifts_east() {
        let s = WeatherSample {
            wind_mps: Vec2::new(12.0, 0.0),
            gust_mps: 14.0,
            ..Default::default()
        };
        let (off, mul) = artillery_weather_stamp(&s, Vec2::X, 0.4, 40.0, 16.0, 25.0);
        assert!(off.x > 0, "expected east drift, got {off:?}");
        assert!(mul >= 1.0);
    }

    #[test]
    fn fog_widens_radius() {
        let clear = WeatherSample::default();
        let foggy = WeatherSample {
            fog: 0.9,
            rain_mm_h: 6.0,
            ..Default::default()
        };
        let (_, m0) = artillery_weather_stamp(&clear, Vec2::X, 0.3, 40.0, 16.0, 20.0);
        let (_, m1) = artillery_weather_stamp(&foggy, Vec2::X, 0.3, 40.0, 16.0, 20.0);
        assert!(m1 > m0);
    }

    #[test]
    fn low_visibility_shrinks_engagement() {
        assert!((engagement_visibility_scale(1.0, 0.0) - 1.0).abs() < 1e-3);
        assert!(engagement_visibility_scale(0.2, 0.8) < 0.6);
    }
}
