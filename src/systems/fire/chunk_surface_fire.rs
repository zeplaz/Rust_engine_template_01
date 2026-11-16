//! Per-chunk **surface fire** proxy (heat / fuel) — CPU authoritative; feeds GPU field uniforms only.
//!
//! Does not replace full wildfire simulation; pairs with [`super::FirePlugin`] and
//! `GpuWeatherFireFieldPlugin` for visuals.
//!
//! ## RND-15 — Decals / scorchmark layering
//!
//! NEW: **RND-15** · gea4v2 **11.6.6** *Decals* · LEGACY `geometry_visual_effects`
//! (gea_v2_e4 p141). Long burn scars are **decal-like**: a projected overlay on terrain
//! (tint / z-biased surface layer), not mesh mutation and not fire particle cards
//! (RND-14 / gea4v2 11.6.4). RND-15 scorch tint is render-only via
//! [`crate::render::fire_vfx::overlay_heat_scorch_decal01`]. Do not retarget RND-04
//! `TRANSPARENT_ORDER_*` or RND-14 card half-edges from this module.

use bevy::prelude::*;

use super::chunk_fuel_profile::ChunkFuelProfile;
use super::combustion::{
    crown_boost_for_old_growth, ecology_fire_risk_spark_factor, fuel_ignition_gate,
    profile_spark_multiplier_gated, DEFAULT_CELL_FUEL_SEED,
};
use super::fire_fuel::FireFuelField;
use super::surface_water::SurfaceWaterFireGate;
use super::types::ChunkFireOverlay;
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::ecology::ChunkEcology;
use crate::systems::ecology::VegetationField;
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Extinction floor in [`chunk_surface_fire_tick`] — heat below this clamps to zero.
pub const SURFACE_FIRE_EXTINCTION_HEAT: f32 = 0.015;

/// Upper bound of the **scorchmark / decal residual** band (RND-15).
///
/// Heat in `[SURFACE_FIRE_EXTINCTION_HEAT, SURFACE_FIRE_SCORCH_HEAT_MAX)` is terrain-projected
/// scar overlay, not active particle-card fuel. Matches
/// [`crate::render::CHUNK_FIRE_OVERLAY_DISPLAY_MIN`] / fire_vfx particle seed floor.
pub const SURFACE_FIRE_SCORCH_HEAT_MAX: f32 = 0.12;

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
        let profile_mult = prof_opt.map(profile_spark_multiplier_gated).unwrap_or(0.0);
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
        if fire.heat < SURFACE_FIRE_EXTINCTION_HEAT {
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

#[cfg(test)]
mod rnd15_decal_tests {
    //! RND-15 tripwires: surface-fire scorchmark = decal-like terrain overlay (gea4v2 11.6.6).

    use super::*;

    #[test]
    fn rnd15_docs_dual_cite_decal_path() {
        let src = include_str!("chunk_surface_fire.rs");
        assert!(
            src.contains("RND-15") && src.contains("11.6.6"),
            "chunk_surface_fire must dual-cite RND-15 / gea4v2 11.6.6"
        );
        assert!(
            src.contains("geometry_visual_effects"),
            "chunk_surface_fire must cite LEGACY geometry_visual_effects"
        );
        let low = src.to_ascii_lowercase();
        assert!(
            low.contains("decal") && (low.contains("scorch") || low.contains("scar")),
            "chunk_surface_fire must name decal + scorchmark/scar stance"
        );
    }

    #[test]
    fn rnd15_extinction_floor_matches_tick_contract() {
        assert_eq!(SURFACE_FIRE_EXTINCTION_HEAT, 0.015);
        assert_eq!(SURFACE_FIRE_SCORCH_HEAT_MAX, 0.12);
        assert!(SURFACE_FIRE_EXTINCTION_HEAT < SURFACE_FIRE_SCORCH_HEAT_MAX);
    }
}
