//! Per-chunk **surface fire** proxy (heat / fuel) — CPU authoritative; feeds GPU field uniforms only.
//!
//! Does not replace full wildfire simulation; pairs with [`super::FirePlugin`] and
//! `GpuWeatherFireFieldPlugin` for visuals.

use bevy::prelude::*;

use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Smolder / active heat in `[0, 1]` and a simple **fuel** proxy for future spread rules.
#[derive(Component, Clone, Copy, Debug)]
pub struct ChunkSurfaceFire {
    pub heat: f32,
    pub fuel: f32,
}

impl Default for ChunkSurfaceFire {
    fn default() -> Self {
        Self { heat: 0.0, fuel: 1.0 }
    }
}

pub(crate) fn spawn_chunk_surface_fire_on_new_chunk(
    mut commands: Commands,
    q: Query<Entity, (Added<Chunk>, Without<ChunkSurfaceFire>)>,
) {
    for e in &q {
        commands.entity(e).insert(ChunkSurfaceFire::default());
    }
}

/// Dry + warm cells raise heat; [`ChunkWeather`] rain/wind and cell moisture pull or push it; scaled by [`ChunkSimLod`].
pub fn chunk_surface_fire_tick(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    mut query: Query<(
        &ChunkCellMatrix,
        &ChunkWeather,
        Option<&ChunkSimLod>,
        &mut ChunkSurfaceFire,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }

    for (matrix, wx, lod, mut fire) in &mut query {
        let lod_s = lod.map(|l| l.dt_scale()).unwrap_or(1.0);
        let dt_e = dt * lod_s;

        let n = (matrix.size.x * matrix.size.y) as usize;
        if matrix.moisture.len() != n || matrix.temperature.len() != n || n == 0 {
            continue;
        }
        let mut sum_m = 0f32;
        let mut sum_t = 0f32;
        for i in 0..n {
            sum_m += matrix.moisture[i];
            sum_t += matrix.temperature[i];
        }
        let c = n as f32;
        let mean_m = sum_m / c;
        let mean_t = sum_t / c;

        let dryness = (0.42 - mean_m).max(0.0);
        let warmth = (mean_t - 0.08).max(0.0);
        let rain_suppress = 1.0 - wx.rain_intensity * 0.78;
        let wind_boost = 1.0 + wx.wind_speed * 0.6;
        let spark = (dryness * warmth * 3.5).min(0.08) * rain_suppress.max(0.0) * wind_boost;

        let wet_line = (mean_m * 0.12 + wx.rain_intensity * 0.18) * dt_e;

        fire.heat = (fire.heat + spark * dt_e * 8.0
            - fire.heat * 0.35 * dt_e
            - wet_line
            - fire.heat * wx.snow_depth * 0.08 * dt_e)
            .clamp(0.0, 1.0);
        if fire.heat < 0.02 {
            fire.heat = 0.0;
        }
        let burn = fire.heat * 0.01 * dt_e;
        fire.fuel = (fire.fuel - burn).clamp(0.0, 1.0);
        // light rehydration proxy from rain (stub fuel recovery)
        fire.fuel = (fire.fuel + wx.rain_intensity * 0.002 * dt_e).clamp(0.0, 1.0);
    }
}
