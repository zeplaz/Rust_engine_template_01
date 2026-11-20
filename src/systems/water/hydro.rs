//! Optional hydro generation from turbine setpoint + reservoir head (DF-6).
//! Any dam with a turbine setpoint can publish MW; DamFlood book mirrors it.

use bevy::prelude::*;

use super::surface_water::{hydro_power_mw, SurfaceWaterField, SurfaceWaterMode};

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

/// Sole writer of [`HydroGenerationState`]. Reads the field: each dam's turbine discharge is what
/// its own intake face really passed, under its own gross head (`P = η·ρ·g·Q·H`, summed over dams).
pub fn update_hydro_generation_system(
    field: Res<SurfaceWaterField>,
    mut hydro: ResMut<HydroGenerationState>,
) {
    if !field.wired {
        hydro.power_mw = 0.0;
        hydro.head_m = 0.0;
        hydro.q_m3s = 0.0;
        return;
    }
    let (mut q, mut power_mw, mut head) = (0.0_f32, 0.0_f32, 0.0_f32);
    let mut any_dam = false;
    for (_, flows) in field.dam_flows() {
        any_dam = true;
        head = head.max(flows.head_m);
        if flows.turbine_m3s > 0.0 && flows.head_m > 0.0 {
            q += flows.turbine_m3s;
            power_mw += hydro_power_mw(hydro.eta, flows.turbine_m3s, flows.head_m);
        }
    }
    if !any_dam {
        // No dam recorded: the reservoir's own stage is the only head there is.
        head = if field.mode == SurfaceWaterMode::LevelPool || field.pool.storage_m3 > 1.0 {
            (field.pool.stage_m - field.pool.bed_min_m).max(0.0)
        } else {
            field.depth_m.iter().copied().fold(0.0_f32, f32::max)
        };
    }
    hydro.head_m = head;
    hydro.q_m3s = q;
    hydro.power_mw = power_mw;
}
