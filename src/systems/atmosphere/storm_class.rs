//! Storm classifier over [`AtmosphereFrame`] tile scalars (`plan_shared_atmosphere_weather_fire_v1.md` §3).
//!
//! Derived only — never a second weather/fire authority.
//! Priority: HurricaneClass > Thunderstorm > FirestormPlume > Developing > Clear.

use crate::systems::weather::{WeatherSample, WeatherTuning, BASELINE_PRESSURE_HPA};

/// Synoptic / convective class for one frame tile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum StormClass {
    #[default]
    Clear = 0,
    Developing = 1,
    Thunderstorm = 2,
    /// Vortex low + wind/rain/depth thresholds (W5).
    HurricaneClass = 3,
    /// Contiguous L0 heat+smoke+ember plume aggregate (W4).
    FirestormPlume = 4,
}

/// Thresholds for [`classify_storm`] (from weather tuning or compiled defaults).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StormClassThresholds {
    pub developing_cloud_min: f32,
    pub developing_humidity_min: f32,
    pub developing_low_depth_min_hpa: f32,
    pub developing_rain_max_mm_h: f32,
    pub thunder_lightning_min: f32,
    pub thunder_cloud_min: f32,
    pub thunder_rain_min_mm_h: f32,
    pub hurricane_wind_min_mps: f32,
    pub hurricane_low_depth_min_hpa: f32,
    pub hurricane_rain_min_mm_h: f32,
    pub hurricane_vortex_min: f32,
}

impl Default for StormClassThresholds {
    fn default() -> Self {
        Self {
            developing_cloud_min: 0.55,
            developing_humidity_min: 0.70,
            developing_low_depth_min_hpa: 4.0,
            developing_rain_max_mm_h: 2.0,
            thunder_lightning_min: 0.35,
            thunder_cloud_min: 0.85,
            thunder_rain_min_mm_h: 3.0,
            hurricane_wind_min_mps: 24.0,
            hurricane_low_depth_min_hpa: 12.0,
            hurricane_rain_min_mm_h: 1.5,
            hurricane_vortex_min: 0.12,
        }
    }
}

impl StormClassThresholds {
    /// Pull classifier knobs from [`WeatherTuning`] (W3/W5 RON fields).
    #[must_use]
    pub fn from_tuning(tuning: &WeatherTuning) -> Self {
        Self {
            developing_cloud_min: tuning.storm_developing_cloud_min,
            developing_humidity_min: tuning.storm_developing_humidity_min,
            developing_low_depth_min_hpa: tuning.storm_developing_low_depth_min_hpa,
            developing_rain_max_mm_h: tuning.storm_developing_rain_max_mm_h,
            thunder_lightning_min: tuning.storm_thunder_lightning_min,
            thunder_cloud_min: tuning.storm_thunder_cloud_min,
            thunder_rain_min_mm_h: tuning.storm_thunder_rain_min_mm_h,
            hurricane_wind_min_mps: tuning.storm_hurricane_wind_min_mps,
            hurricane_low_depth_min_hpa: tuning.storm_hurricane_low_depth_min_hpa,
            hurricane_rain_min_mm_h: tuning.storm_hurricane_rain_min_mm_h,
            hurricane_vortex_min: tuning.storm_hurricane_vortex_min,
        }
    }
}

/// Low-pressure depth proxy from a surface pressure sample (hPa below baseline).
#[inline]
#[must_use]
pub fn low_depth_hpa(pressure_hpa: f32) -> f32 {
    (BASELINE_PRESSURE_HPA - pressure_hpa).max(0.0)
}

/// Classify one weather column. Priority: Hurricane > Thunder > Firestorm > Developing > Clear.
#[must_use]
pub fn classify_storm(weather: &WeatherSample, thresholds: &StormClassThresholds) -> StormClass {
    classify_storm_with_context(weather, thresholds, false, 0.0)
}

/// Full classifier with L0 plume bit and pressure-system vortex influence.
#[must_use]
pub fn classify_storm_with_context(
    weather: &WeatherSample,
    thresholds: &StormClassThresholds,
    in_firestorm_plume: bool,
    vortex_influence: f32,
) -> StormClass {
    if !weather_valid_for_class(weather) {
        return StormClass::Clear;
    }
    if is_hurricane(weather, thresholds, vortex_influence) {
        return StormClass::HurricaneClass;
    }
    if is_thunderstorm(weather, thresholds) {
        return StormClass::Thunderstorm;
    }
    if in_firestorm_plume {
        return StormClass::FirestormPlume;
    }
    if is_developing(weather, thresholds) {
        return StormClass::Developing;
    }
    StormClass::Clear
}

fn weather_valid_for_class(w: &WeatherSample) -> bool {
    w.cloud.is_finite() && w.humidity.is_finite() && w.lightning.is_finite()
}

fn is_hurricane(w: &WeatherSample, t: &StormClassThresholds, vortex_influence: f32) -> bool {
    vortex_influence >= t.hurricane_vortex_min
        && w.wind_mps.length() >= t.hurricane_wind_min_mps
        && low_depth_hpa(w.pressure_hpa) >= t.hurricane_low_depth_min_hpa
        && w.rain_mm_h >= t.hurricane_rain_min_mm_h
}

fn is_thunderstorm(w: &WeatherSample, t: &StormClassThresholds) -> bool {
    w.lightning >= t.thunder_lightning_min
        && w.cloud >= t.thunder_cloud_min
        && w.rain_mm_h >= t.thunder_rain_min_mm_h
}

fn is_developing(w: &WeatherSample, t: &StormClassThresholds) -> bool {
    w.cloud >= t.developing_cloud_min
        && w.humidity >= t.developing_humidity_min
        && low_depth_hpa(w.pressure_hpa) >= t.developing_low_depth_min_hpa
        && w.rain_mm_h <= t.developing_rain_max_mm_h
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;

    fn sample(cloud: f32, humidity: f32, rain: f32, lightning: f32, pressure: f32) -> WeatherSample {
        WeatherSample {
            cloud,
            humidity,
            rain_mm_h: rain,
            lightning,
            pressure_hpa: pressure,
            ..Default::default()
        }
    }

    #[test]
    fn calm_is_clear() {
        let t = StormClassThresholds::default();
        assert_eq!(classify_storm(&WeatherSample::default(), &t), StormClass::Clear);
    }

    #[test]
    fn humid_low_cloud_is_developing() {
        let t = StormClassThresholds::default();
        let w = sample(0.6, 0.85, 0.5, 0.0, BASELINE_PRESSURE_HPA - 8.0);
        assert_eq!(classify_storm(&w, &t), StormClass::Developing);
    }

    #[test]
    fn lightning_rain_is_thunderstorm() {
        let t = StormClassThresholds::default();
        let w = sample(0.95, 0.9, 5.0, 0.5, BASELINE_PRESSURE_HPA - 12.0);
        assert_eq!(classify_storm(&w, &t), StormClass::Thunderstorm);
    }

    #[test]
    fn thunderstorm_outranks_developing_signals() {
        let t = StormClassThresholds::default();
        let w = sample(0.95, 0.9, 4.0, 0.4, BASELINE_PRESSURE_HPA - 10.0);
        assert_eq!(classify_storm(&w, &t), StormClass::Thunderstorm);
    }

    #[test]
    fn plume_bit_marks_firestorm() {
        let t = StormClassThresholds::default();
        let w = WeatherSample::default();
        assert_eq!(
            classify_storm_with_context(&w, &t, true, 0.0),
            StormClass::FirestormPlume
        );
        assert_eq!(
            classify_storm_with_context(&w, &t, false, 0.0),
            StormClass::Clear
        );
    }

    #[test]
    fn vortex_deep_wind_rain_is_hurricane() {
        let t = StormClassThresholds::default();
        let mut w = sample(0.9, 0.95, 4.0, 0.1, BASELINE_PRESSURE_HPA - 20.0);
        w.wind_mps = Vec2::new(30.0, 0.0);
        assert_eq!(
            classify_storm_with_context(&w, &t, false, 0.5),
            StormClass::HurricaneClass
        );
        assert_eq!(
            classify_storm_with_context(&w, &t, false, 0.0),
            StormClass::Clear,
            "no vortex → not hurricane"
        );
    }

    #[test]
    fn hurricane_outranks_thunder_when_vortex() {
        let t = StormClassThresholds::default();
        let mut w = sample(0.95, 0.95, 5.0, 0.5, BASELINE_PRESSURE_HPA - 20.0);
        w.wind_mps = Vec2::new(28.0, 0.0);
        assert_eq!(
            classify_storm_with_context(&w, &t, true, 0.4),
            StormClass::HurricaneClass
        );
    }
}
