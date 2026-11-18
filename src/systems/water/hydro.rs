//! Optional hydro generation from turbine setpoint + reservoir head (DF-6).
//! Any dam with a turbine setpoint can publish MW; DamFlood book mirrors it.

use bevy::prelude::*;

use super::surface_water::{hydro_power_mw, SurfaceWaterField, SurfaceWaterMode};
use super::BarrierControlIntent;

/// Live hydro output derived from general water + barrier control (not scene-private).
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct HydroGenerationState {
    pub eta: f32,
    pub power_mw: f32,
    pub head_m: f32,
    pub q_m3s: f32,
}

impl Default for HydroGenerationState {
    fn default() -> Self {
        Self {
            eta: 0.90,
            power_mw: 0.0,
            head_m: 0.0,
            q_m3s: 0.0,
        }
    }
}

/// Sole writer of [`HydroGenerationState`]. Reads field + barrier intent.
pub fn update_hydro_generation_system(
    field: Res<SurfaceWaterField>,
    control: Res<BarrierControlIntent>,
    mut hydro: ResMut<HydroGenerationState>,
) {
    if !field.wired {
        hydro.power_mw = 0.0;
        hydro.head_m = 0.0;
        hydro.q_m3s = 0.0;
        return;
    }
    let has_crest = field.hydraulics.barrier_crest_m.iter().any(|c| *c > 0.5);
    let head = if field.mode == SurfaceWaterMode::LevelPool || field.pool.storage_m3 > 1.0 {
        (field.pool.stage_m - field.pool.bed_min_m).max(0.0)
    } else {
        field
            .depth_m
            .iter()
            .copied()
            .fold(0.0_f32, f32::max)
            .max((field.pool.stage_m - field.pool.bed_min_m).max(0.0))
    };
    // Turbine draws up to setpoint while a crest + head exist (DF-6). Do not require outlet_open —
    // closed-outlet impound with a turbine setpoint still reports available head/power.
    let q = if has_crest && head > 0.5 {
        control
            .effective_turbine_m3s()
            .clamp(0.0, field.hydraulics.turbine_design_m3s.max(1.0))
    } else {
        0.0
    };
    hydro.head_m = head;
    hydro.q_m3s = q;
    hydro.power_mw = if q > 0.0 && head > 0.0 {
        hydro_power_mw(hydro.eta, q, head)
    } else {
        0.0
    };
}
