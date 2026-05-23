//! Per-chunk **surface fire** proxy (heat / fuel) — CPU authoritative; feeds GPU field uniforms only.
//!
//! Does not replace full wildfire simulation; pairs with [`super::FirePlugin`] and
//! `GpuWeatherFireFieldPlugin` for visuals.

use bevy::prelude::*;

use super::chunk_fuel_profile::ChunkFuelProfile;
use super::combustion::{
    crown_boost_for_old_growth, ecology_fire_risk_spark_factor, profile_spark_multiplier_gated,
    fuel_ignition_gate, DEFAULT_CELL_FUEL_SEED,
};
use crate::systems::ecology::VegetationField;
use super::types::ChunkFireOverlay;
use super::fire_fuel::FireFuelField;
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::ecology::ChunkEcology;
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use super::surface_water::SurfaceWaterFireGate;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Smolder / active heat in `[0, 1]` and a simple **fuel** proxy for future spread rules.
#[derive(Component, Clone, Copy, Debug)]
pub struct ChunkSurfaceFire {
    pub heat: f32,
    pub fuel: f32,
}

impl Default for ChunkSurfaceFire {
    fn default() -> Self {
        Self {
            heat: 0.0,
            fuel: DEFAULT_CELL_FUEL_SEED,
        }
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

/// Chunks **with** [`ChunkFireOverlay`] use per-cell sim; this path is **without** overlay only.
pub fn chunk_surface_fire_tick(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    water_gate: Res<SurfaceWaterFireGate>,
    mut query: Query<
        (
            &ChunkCellMatrix,
            &ChunkWeather,
            Option<&ChunkSimLod>,
            Option<&ChunkEcology>,
            Option<&VegetationField>,
            Option<&ChunkFuelProfile>,
            Option<&FireFuelField>,
            &mut ChunkSurfaceFire,
        ),
        Without<ChunkFireOverlay>,
    >,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }

    for (matrix, wx, lod, eco_opt, veg_opt, prof_opt, fuel_opt, mut fire) in &mut query {
        let lod_s = lod.map(|l| l.dt_scale()).unwrap_or(1.0);
        let dt_e = dt * lod_s;

        let n = (matrix.size.x * matrix.size.y) as usize;
        if matrix.moisture.len() != n || matrix.temperature.len() != n || n == 0 {
            continue;
        }
        let mut sum_m = 0f32;
        let mut sum_t = 0f32;
        let mut burnable = 0u32;
        for i in 0..n {
            if water_gate.cell_has_standing_water(matrix, i) {
                continue;
            }
            sum_m += matrix.moisture[i];
            sum_t += matrix.temperature[i];
            burnable += 1;
        }
        if burnable == 0 {
            fire.heat = 0.0;
            continue;
        }
        let c = burnable as f32;
        let mean_m = sum_m / c;
        let mean_t = sum_t / c;

        let dryness = SurfaceWaterFireGate::atmospheric_dryness(mean_m);
        let warmth = (mean_t - 0.08).max(0.0);
        let rain_suppress: f32 = 1.0 - wx.rain_intensity * 0.78;
        let wind_boost = 1.0 + wx.wind_speed * 0.6;
        let eco_boost = ecology_fire_risk_spark_factor(eco_opt.map(|e| e.fire_risk).unwrap_or(0.0));
        let old_growth = prof_opt
            .map(|p| p.old_growth)
            .or_else(|| veg_opt.map(|v| v.old_growth))
            .unwrap_or(0.0);
        let wildland_mass = prof_opt.map(|p| p.wildland_fuel_mass).unwrap_or(0.0);
        let profile_mult = prof_opt
            .map(profile_spark_multiplier_gated)
            .unwrap_or(0.0);
        let crown = fuel_opt
            .map(|f| crown_boost_for_old_growth(old_growth, f))
            .unwrap_or(1.0);
        let spark = if fuel_ignition_gate(wildland_mass) <= 0.0 {
            0.0
        } else {
            (dryness * warmth * 4.0).min(0.14)
                * rain_suppress.max(0.0)
                * wind_boost
                * eco_boost
                * profile_mult
                * crown
        };

        let wet_line = (mean_m * 0.10 + wx.rain_intensity * 0.22) * dt_e;

        fire.heat = (fire.heat + spark * dt_e * 6.0
            - fire.heat * 0.14 * dt_e
            - wet_line
            - fire.heat * wx.snow_depth * 0.06 * dt_e)
            .clamp(0.0, 1.0);
        if fire.fuel > 0.35 && dryness > 0.08 && fire.heat < 0.12 {
            fire.heat = fire.heat.max(0.08);
        }
        if fire.heat < 0.015 {
            fire.heat = 0.0;
        }
        let burn = fire.heat * 0.006 * dt_e;
        fire.fuel = (fire.fuel - burn).max(0.0);
        if let Some(prof) = prof_opt {
            let cap = super::combustion::profile_cell_fuel_seed(prof);
            fire.fuel = fire.fuel.min(cap);
        }
    }
}
