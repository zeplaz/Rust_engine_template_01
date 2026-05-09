//! Per-chunk fast weather scalars (local layer; see weather runbook §4.3).
//!
//! Does **not** mutate terrain ontology; consumers read this + [`DynamicTerrainOverlay`](crate::terrain::dynamic_overlay::DynamicTerrainOverlay).

use bevy::prelude::*;

use crate::systems::sim_control::SimControlState;
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Local weather fields for one [`Chunk`] entity (rain, fog, snow depth, wind, lightning, visibility).
#[derive(Component, Debug, Clone)]
pub struct ChunkWeather {
    pub rain_intensity: f32,
    pub fog_density: f32,
    pub snow_depth: f32,
    pub wind_speed: f32,
    pub lightning_risk: f32,
    /// 1.0 = clear horizon; lower = worse (for combat / sensors stubs).
    pub visibility_factor: f32,
    /// Chunk-scale soil moisture **derived** from cell means + rain (runbook coupling to ecology / hydrology).
    pub soil_moisture: f32,
}

impl Default for ChunkWeather {
    fn default() -> Self {
        Self {
            rain_intensity: 0.0,
            fog_density: 0.0,
            snow_depth: 0.0,
            wind_speed: 0.0,
            lightning_risk: 0.0,
            visibility_factor: 1.0,
            soil_moisture: 0.45,
        }
    }
}

/// Debug/diagnostic: increments when [`weather_chunk_tick`] advances under [`SimControlState::should_tick`].
#[derive(Resource, Debug, Default)]
pub struct WeatherSimDiagnostics {
    pub ticks_advanced: u64,
}

/// World-aggregated scalars for variable renewable derate (wind / solar). Updated when the sim ticks.
#[derive(Resource, Debug, Clone, Copy)]
pub struct GlobalRenewableWeatherFactors {
    /// ~0.05–1.2 from mean chunk wind.
    pub wind_capacity_factor: f32,
    /// ~0.05–1.0 from mean cloud proxy (fog + rain).
    pub solar_capacity_factor: f32,
}

impl Default for GlobalRenewableWeatherFactors {
    fn default() -> Self {
        Self {
            wind_capacity_factor: 1.0,
            solar_capacity_factor: 1.0,
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
    let sum: f32 = matrix.moisture.iter().sum();
    Some(sum / n as f32)
}

fn targets_from_cell_matrix(matrix: &ChunkCellMatrix) -> Option<(f32, f32, f32, f32, f32)> {
    let n = (matrix.size.x * matrix.size.y) as usize;
    if matrix.moisture.len() != n || matrix.temperature.len() != n || n == 0 {
        return None;
    }
    let mut sum_m = 0f32;
    let mut sum_t = 0f32;
    let mut min_t = f32::MAX;
    let mut max_t = f32::MIN;
    for i in 0..n {
        let m = matrix.moisture[i];
        let t = matrix.temperature[i];
        sum_m += m;
        sum_t += t;
        min_t = min_t.min(t);
        max_t = max_t.max(t);
    }
    let c = n as f32;
    let mean_m = sum_m / c;
    let mean_t = sum_t / c;
    let temp_spread = (max_t - min_t).max(0.0);

    let rain = (mean_m * (mean_t + 0.22).max(0.0) * 1.15).clamp(0.0, 1.0);
    let snow = ((0.38 - mean_t).max(0.0) * mean_m * 2.2).clamp(0.0, 1.0);
    let fog = (mean_m * mean_m * (1.0 - (temp_spread * 2.2).min(1.0))).clamp(0.0, 1.0);
    let wind = ((temp_spread * 3.2) + mean_m * 0.35).clamp(0.0, 1.0);
    let lightning = (mean_m * (mean_t + 0.12).max(0.0) * 0.75).clamp(0.0, 1.0);
    Some((rain, fog, snow, wind, lightning))
}

fn update_visibility(w: &mut ChunkWeather) {
    w.visibility_factor = (w.fog_density * 0.4 + w.rain_intensity * 0.2)
        .min(1.0)
        .mul_add(-1.0, 1.0)
        .clamp(0.05, 1.0);
}

/// Lerp toward targets from [`ChunkCellMatrix`] when present; otherwise decay toward calm.
pub fn weather_chunk_tick(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    mut diag: ResMut<WeatherSimDiagnostics>,
    mut query: Query<(Option<&ChunkCellMatrix>, Option<&ChunkSimLod>, &mut ChunkWeather)>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }

    diag.ticks_advanced = diag.ticks_advanced.wrapping_add(1);

    for (matrix_opt, lod_opt, mut w) in &mut query {
        let lod_s = lod_opt.map(|l| l.dt_scale()).unwrap_or(1.0);
        let dt_l = dt * lod_s;
        let k_drive = (0.22 * dt_l).clamp(0.0, 0.38);
        let k_decay = (0.14 * dt_l).clamp(0.0, 0.22);

        match matrix_opt {
            Some(matrix) => {
                if let Some((tr, tf, ts, tw, tl)) = targets_from_cell_matrix(matrix) {
                    let lerp = |a: f32, b: f32| a * (1.0 - k_drive) + b * k_drive;
                    w.rain_intensity = lerp(w.rain_intensity, tr);
                    w.fog_density = lerp(w.fog_density, tf);
                    w.snow_depth = lerp(w.snow_depth, ts);
                    w.wind_speed = lerp(w.wind_speed, tw);
                    w.lightning_risk = lerp(w.lightning_risk, tl);
                }
                if let Some(mm) = matrix_mean_moisture(matrix) {
                    let lerp = |a: f32, b: f32| a * (1.0 - k_drive) + b * k_drive;
                    let soil_target =
                        (mm * 0.82 + w.rain_intensity * 0.26).clamp(0.0, 1.0);
                    w.soil_moisture = lerp(w.soil_moisture, soil_target);
                }
            }
            None => {
                w.rain_intensity *= 1.0 - k_decay;
                w.fog_density *= 1.0 - k_decay;
                w.snow_depth *= 1.0 - k_decay * 0.5;
                w.wind_speed *= 1.0 - k_decay * 0.7;
                w.lightning_risk *= 1.0 - k_decay;
                let lerp = |a: f32, b: f32| a * (1.0 - k_decay) + b * k_decay;
                w.soil_moisture = lerp(w.soil_moisture, 0.38);
            }
        }
        w.rain_intensity = w.rain_intensity.max(0.0);
        w.fog_density = w.fog_density.max(0.0);
        w.snow_depth = w.snow_depth.max(0.0);
        w.wind_speed = w.wind_speed.max(0.0);
        w.lightning_risk = w.lightning_risk.max(0.0);
        update_visibility(&mut w);
    }
}

pub(crate) fn update_global_renewable_weather_factors(
    ctrl: Res<SimControlState>,
    mut out: ResMut<GlobalRenewableWeatherFactors>,
    q: Query<&ChunkWeather>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let mut n = 0u32;
    let mut sum_wind = 0f32;
    let mut sum_cloud = 0f32;
    for w in &q {
        n += 1;
        sum_wind += w.wind_speed;
        let cloud = (w.fog_density * 0.55 + w.rain_intensity * 0.45).clamp(0.0, 1.0);
        sum_cloud += cloud;
    }
    if n == 0 {
        *out = GlobalRenewableWeatherFactors::default();
        return;
    }
    let nf = n as f32;
    let avg_wind = sum_wind / nf;
    let avg_cloud = sum_cloud / nf;
    out.wind_capacity_factor = (0.12 + avg_wind * 0.98).clamp(0.05, 1.2);
    out.solar_capacity_factor = (1.0 - avg_cloud * 0.88).clamp(0.05, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{App, MinimalPlugins, Update, UVec2};

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

    #[test]
    fn cell_matrix_targets_rain_for_moist_warm_chunk() {
        let mut matrix = ChunkCellMatrix::new(UVec2::new(2, 2));
        for m in matrix.moisture.iter_mut() {
            *m = 0.9;
        }
        for t in matrix.temperature.iter_mut() {
            *t = 0.2;
        }
        let (rain, fog, snow, wind, lightning) = targets_from_cell_matrix(&matrix).unwrap();
        assert!(rain > 0.15, "rain={rain}");
        assert!(fog >= 0.0 && snow >= 0.0);
        assert!(wind >= 0.0 && lightning >= 0.0);
    }

    #[test]
    fn global_renewable_factors_track_chunk_weather() {
        let mut app = App::new();
        app.init_resource::<SimControlState>()
            .init_resource::<GlobalRenewableWeatherFactors>()
            .add_plugins(MinimalPlugins)
            .add_systems(Update, update_global_renewable_weather_factors);

        app.world_mut().spawn(ChunkWeather {
            rain_intensity: 0.0,
            fog_density: 1.0,
            snow_depth: 0.0,
            wind_speed: 0.5,
            lightning_risk: 0.0,
            visibility_factor: 0.5,
            soil_moisture: 0.45,
        });

        app.update();

        let g = app.world().resource::<GlobalRenewableWeatherFactors>();
        assert!(g.solar_capacity_factor < 0.6);
        assert!(g.wind_capacity_factor > 0.2);
    }
}
