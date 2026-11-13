//! Weather field tuning table (`assets/configs/weather/weather_v1.ron`).
//!
//! The compiled-in copy is the default; [`load_weather_tuning`] re-reads the file at runtime so the
//! table can be tuned without a rebuild. Metric units throughout.

use std::path::{Path, PathBuf};

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Repo-relative path of the tuning table.
pub const WEATHER_TUNING_RON: &str = "assets/configs/weather/weather_v1.ron";

const EMBEDDED_WEATHER_TUNING: &str = include_str!("../../../assets/configs/weather/weather_v1.ron");

/// Scenario / test weather override.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeatherPresetId {
    Calm,
    Windy,
    StormFront,
    FireWeather,
}

/// One preset: replaces the seeded climate and pressure systems.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WeatherPreset {
    pub id: WeatherPresetId,
    pub prevailing_mps: f32,
    pub base_temp_offset_c: f32,
    pub humidity: f32,
    pub systems: u32,
    pub low_fraction: f32,
    pub strength_hpa: (f32, f32),
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WeatherTuning {
    pub schema_version: u32,
    pub cell_size_m: f32,
    pub weather_step_secs: u32,
    /// Solves run at field creation so humidity / cloud / rain start developed.
    #[serde(default)]
    pub spinup_steps: u32,
    pub climate_step_secs: u32,
    pub season_length_secs: f32,
    pub base_temp_mean_c: f32,
    pub base_temp_amplitude_c: f32,
    pub elevation_range_m: f32,
    pub lapse_rate_c_per_km: f32,
    pub prevailing_speed_mps: (f32, f32),
    pub prevailing_veer_rad: f32,
    pub systems_count: (u32, u32),
    pub systems_km2_per_extra: f32,
    pub system_strength_hpa: (f32, f32),
    pub system_radius_frac: (f32, f32),
    pub system_radius_m: (f32, f32),
    pub system_lifetime_secs: (f32, f32),
    pub system_steering_frac: f32,
    pub system_cross_drift_mps: f32,
    pub low_fraction: f32,
    pub gradient_mps_per_hpa_km: f32,
    pub friction_turn_deg: f32,
    pub max_wind_mps: f32,
    pub ridge_factor: f32,
    pub valley_factor: f32,
    pub exposure_span: f32,
    pub rough_factor: f32,
    pub channel_max: f32,
    pub humidity_base: f32,
    pub humidity_relax: f32,
    pub water_evaporation: f32,
    pub low_humidity_load: f32,
    pub high_drying: f32,
    pub rain_sink_per_mm_h: f32,
    pub cloud_humidity_min: f32,
    pub cloud_humidity_span: f32,
    pub low_lift_per_hpa: f32,
    pub orographic_lift: f32,
    pub rain_cloud_min: f32,
    pub rain_max_mm_h: f32,
    pub visibility_clear_m: f32,
    pub visibility_fog_k: f32,
    pub visibility_rain_k: f32,
    pub gust_bucket_secs: u32,
    pub gust_ratio: (f32, f32),
    #[serde(default)]
    pub weather_preset: Option<WeatherPresetId>,
    #[serde(default)]
    pub presets: Vec<WeatherPreset>,
}

impl Default for WeatherTuning {
    fn default() -> Self {
        parse_weather_tuning(EMBEDDED_WEATHER_TUNING)
            .expect("embedded weather_v1.ron must parse")
    }
}

impl WeatherTuning {
    #[must_use]
    pub fn preset(&self, id: WeatherPresetId) -> Option<&WeatherPreset> {
        self.presets.iter().find(|p| p.id == id)
    }

    /// Problems that would make the solve meaningless (empty when valid).
    #[must_use]
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if !(self.cell_size_m >= 100.0) {
            issues.push(format!("cell_size_m {} < 100", self.cell_size_m));
        }
        if self.weather_step_secs == 0 {
            issues.push("weather_step_secs must be > 0".into());
        }
        if self.systems_count.0 == 0 || self.systems_count.0 > self.systems_count.1 {
            issues.push(format!("systems_count {:?} invalid", self.systems_count));
        }
        if self.gust_bucket_secs == 0 {
            issues.push("gust_bucket_secs must be > 0".into());
        }
        issues
    }
}

pub fn parse_weather_tuning(text: &str) -> Result<WeatherTuning, String> {
    ron::from_str(text).map_err(|e| format!("weather tuning RON parse: {e}"))
}

fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

pub fn load_weather_tuning_from_path(path: &Path) -> Result<WeatherTuning, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let t = parse_weather_tuning(&text)?;
    let issues = t.validate();
    if issues.is_empty() {
        Ok(t)
    } else {
        Err(format!("{}: {}", path.display(), issues.join("; ")))
    }
}

/// Runtime load: disk table when present and valid, else the compiled-in default (with the error).
#[must_use]
pub fn load_weather_tuning() -> (WeatherTuning, Option<String>) {
    match load_weather_tuning_from_path(&repo_asset_path(WEATHER_TUNING_RON)) {
        Ok(t) => (t, None),
        Err(err) => (WeatherTuning::default(), Some(err)),
    }
}
