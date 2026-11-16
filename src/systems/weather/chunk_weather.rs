//! Per-chunk weather scalars — a **projection** of [`crate::systems::atmosphere::AtmosphereFrame`]
//! (plan `plan_weather_wind_v1.md` §3, CQ-39).
//!
//! Written once per weather step (and once when a chunk first gets the component) by
//! [`project_chunk_weather`] from the frame's chunk-centre samples (never [`WeatherField::sample`]).
//! Existing readers (ecology, fire, LOD, sensors, VFX means) keep their 0..1 scalars; new readers
//! should read [`AtmosphereFrame`] directly.
//!
//! Does **not** mutate terrain ontology; consumers read this + [`DynamicTerrainOverlay`](crate::terrain::dynamic_overlay::DynamicTerrainOverlay).

use bevy::prelude::*;

use crate::systems::atmosphere::AtmosphereFrame;
use crate::terrain::generation::{chunk_world_center, Chunk, ChunkCellMatrix};
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

use super::field::WeatherField;

/// Wind speed that maps to `wind_speed == 1.0`.
pub const CHUNK_WIND_FULL_SCALE_MPS: f32 = 20.0;
/// Rain rate that maps to `rain_intensity == 1.0`.
pub const CHUNK_RAIN_FULL_SCALE_MM_H: f32 = 10.0;
/// Visibility that maps to `visibility_factor == 1.0`.
pub const CHUNK_VISIBILITY_FULL_SCALE_M: f32 = 10_000.0;

/// Local weather fields for one [`Chunk`] entity (rain, fog, snow depth, wind, lightning, visibility).
#[derive(Component, Debug, Clone, Copy)]
pub struct ChunkWeather {
    /// 0..1 (`rain mm/h / 10`), liquid precipitation only.
    pub rain_intensity: f32,
    /// Liquid precipitation rate at the chunk centre (mm/h). Zero when snowing.
    pub rain_mm_h: f32,
    pub fog_density: f32,
    /// 0..1 accumulated snow cover.
    pub snow_depth: f32,
    /// 0..1 derived as `|wind_mps| / 20 m/s` (kept for existing readers).
    pub wind_speed: f32,
    /// Wind vector at the chunk centre, m/s (x = east, y = tile row axis).
    pub wind_mps: Vec2,
    /// Gust magnitude (m/s) from the field sample (5 s hash bucket).
    pub gust_mps: f32,
    /// Cloud cover 0..1 at chunk centre (W3).
    pub cloud: f32,
    pub lightning_risk: f32,
    /// 1.0 = clear horizon (>= 10 km); lower = worse.
    pub visibility_factor: f32,
    /// Chunk-scale soil moisture **derived** from cell means + rain (coupling to ecology / hydrology).
    pub soil_moisture: f32,
}

impl Default for ChunkWeather {
    fn default() -> Self {
        Self {
            rain_intensity: 0.0,
            rain_mm_h: 0.0,
            fog_density: 0.0,
            snow_depth: 0.0,
            wind_speed: 0.0,
            wind_mps: Vec2::ZERO,
            gust_mps: 0.0,
            cloud: 0.0,
            lightning_risk: 0.0,
            visibility_factor: 1.0,
            soil_moisture: 0.45,
        }
    }
}

pub(crate) fn spawn_chunk_weather_on_new_chunk(
    mut commands: Commands,
    q: Query<Entity, (Added<Chunk>, Without<ChunkWeather>)>,
) {
    for entity in &q {
        commands.entity(entity).insert(ChunkWeather::default());
    }
}

fn matrix_mean_moisture(matrix: &ChunkCellMatrix) -> Option<f32> {
    let n = (matrix.size.x * matrix.size.y) as usize;
    if matrix.moisture.len() != n || n == 0 {
        return None;
    }
    Some(matrix.moisture.iter().sum::<f32>() / n as f32)
}

/// Project frame weather onto chunks: every chunk when the field revision changed, else only new components.
pub fn project_chunk_weather(
    field: Res<WeatherField>,
    frame: Res<AtmosphereFrame>,
    mut last_revision: Local<u64>,
    mut query: Query<(&Chunk, Option<&ChunkCellMatrix>, &mut ChunkWeather)>,
) {
    if !frame.is_weather_ready() || !field.is_ready() {
        return;
    }
    let weather_rev = frame.key().weather_revision;
    let changed = *last_revision != weather_rev;
    *last_revision = weather_rev;
    let step_h = field.step_secs as f32 / 3600.0;
    for (chunk, matrix, mut w) in &mut query {
        let added = w.is_added();
        if !changed && !added {
            continue;
        }
        let size = matrix.map_or(UVec2::splat(CHUNK_TILES_SIM), |m| m.size);
        let pos = chunk_world_center(chunk.coord, size) * TILE_METRES_GAMEPLAY;
        let Some(s) = frame
            .weather_at_chunk(chunk.coord)
            .or_else(|| frame.weather_at_pos_m(pos))
        else {
            continue;
        };
        let rain01 = (s.rain_mm_h / CHUNK_RAIN_FULL_SCALE_MM_H).clamp(0.0, 1.0);
        w.rain_intensity = if s.snow { 0.0 } else { rain01 };
        w.rain_mm_h = if s.snow { 0.0 } else { s.rain_mm_h.max(0.0) };
        w.fog_density = s.fog.clamp(0.0, 1.0);
        w.wind_mps = s.wind_mps;
        w.gust_mps = s.gust_mps.max(0.0);
        w.cloud = s.cloud.clamp(0.0, 1.0);
        w.wind_speed = (s.wind_mps.length() / CHUNK_WIND_FULL_SCALE_MPS).clamp(0.0, 1.0);
        w.lightning_risk = s.lightning.clamp(0.0, 1.0);
        w.visibility_factor = (s.visibility_m / CHUNK_VISIBILITY_FULL_SCALE_M).clamp(0.05, 1.0);
        let soil_target = matrix
            .and_then(matrix_mean_moisture)
            .map_or(0.38, |mm| (mm * 0.82 + rain01 * 0.26).clamp(0.0, 1.0));
        if added && !changed {
            w.soil_moisture = soil_target;
            continue;
        }
        // Accumulators advance once per weather step.
        w.soil_moisture += (soil_target - w.soil_moisture) * 0.3;
        if s.snow {
            w.snow_depth += rain01 * step_h * 2.0;
        } else if s.temp_c > 0.0 {
            w.snow_depth -= (0.05 + s.temp_c * 0.02) * step_h;
        }
        w.snow_depth = w.snow_depth.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, MinimalPlugins, Update};

    #[test]
    fn chunk_spawn_gets_chunk_weather() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, spawn_chunk_weather_on_new_chunk);

        let e = app
            .world_mut()
            .spawn(Chunk {
                coord: bevy::math::IVec2::new(1, -2),
            })
            .id();

        app.update();

        let w = app.world().entity(e).get::<ChunkWeather>().expect("weather");
        assert_eq!(w.rain_intensity, 0.0);
        assert!((w.visibility_factor - 1.0).abs() < f32::EPSILON);
    }
}
