//! Per-cell **surface fire** field on the chunk grid — CPU authority (Phase 1 fire runbook).
//! Moisture acts as a **barrier** to diffusion (river / wet cells). Scalar [`ChunkSurfaceFire`]
//! tracks chunk means for GPU uniforms; chunks without a matrix use scalar tick only.

use bevy::prelude::*;

use super::chunk_surface_fire::ChunkSurfaceFire;
use super::types::ChunkFireOverlay;
use crate::systems::chunk_environment_persist::{ChunkEnvironmentDirty, ChunkEnvironmentPersistHooks};
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::ecology::ChunkEcology;
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

pub(crate) fn spawn_chunk_fire_overlay_on_matrix(
    mut commands: Commands,
    q: Query<(Entity, &ChunkCellMatrix), (With<Chunk>, Without<ChunkFireOverlay>)>,
) {
    for (e, matrix) in &q {
        let n = (matrix.size.x * matrix.size.y) as usize;
        if n == 0 {
            continue;
        }
        commands.entity(e).insert((
            ChunkFireOverlay {
                heat: vec![0.0; n],
                fuel: vec![1.0; n],
            },
            ChunkEnvironmentDirty::default(),
        ));
    }
}

#[inline]
fn wet_diffusion_barrier(moisture: f32) -> f32 {
    let over = (moisture - 0.72).max(0.0) / 0.28;
    (1.0 - over * 0.88).clamp(0.08, 1.0)
}

pub fn chunk_fire_overlay_tick(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    mut hooks: ResMut<ChunkEnvironmentPersistHooks>,
    mut q: Query<(
        Entity,
        &ChunkCellMatrix,
        &ChunkWeather,
        Option<&ChunkSimLod>,
        Option<&ChunkEcology>,
        &mut ChunkFireOverlay,
        &mut ChunkSurfaceFire,
        &mut ChunkEnvironmentDirty,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }

    let mut scratch: Vec<f32> = Vec::new();

    for (entity, matrix, wx, lod, eco_opt, mut ovl, mut surf, mut dirty) in &mut q {
        let lod_s = lod.map(|l| l.dt_scale()).unwrap_or(1.0);
        let dt_e = dt * lod_s;

        let sx = matrix.size.x as usize;
        let sy = matrix.size.y as usize;
        let n = sx * sy;
        if matrix.moisture.len() != n || matrix.temperature.len() != n {
            continue;
        }
        if ovl.heat.len() != n || ovl.fuel.len() != n {
            ovl.heat.resize(n, 0.0);
            ovl.fuel.resize(n, 1.0f32);
        }

        let fire_risk = eco_opt.map(|e| e.fire_risk).unwrap_or(0.0);
        let eco_boost = 1.0 + 0.95 * fire_risk;

        let rain_suppress = (1.0 - wx.rain_intensity * 0.78).max(0.0);
        let wind_boost = 1.0 + wx.wind_speed * 0.6;

        scratch.clear();
        scratch.resize(n, 0.0);

        for y in 0..sy {
            for x in 0..sx {
                let i = y * sx + x;
                let m = matrix.moisture[i];
                let t = matrix.temperature[i];
                let dryness = (0.42 - m).max(0.0);
                let warmth = (t - 0.08).max(0.0);
                let spark =
                    (dryness * warmth * 3.5).min(0.08) * rain_suppress * wind_boost * eco_boost;

                let wet_line = (m * 0.12 + wx.rain_intensity * 0.18) * dt_e;

                let h = ovl.heat[i];
                let mut nh = (h + spark * dt_e * 8.0 - h * 0.35 * dt_e - wet_line
                    - h * wx.snow_depth * 0.08 * dt_e)
                    .clamp(0.0, 1.0);
                if nh < 0.02 {
                    nh = 0.0;
                }
                ovl.heat[i] = nh;

                let burn = nh * 0.01 * dt_e;
                ovl.fuel[i] = (ovl.fuel[i] - burn).clamp(0.0, 1.0);
                ovl.fuel[i] =
                    (ovl.fuel[i] + wx.rain_intensity * 0.002 * dt_e).clamp(0.0, 1.0);

                let mut sum = 0f32;
                let mut cnt = 0u32;
                if x > 0 {
                    sum += ovl.heat[i - 1];
                    cnt += 1;
                }
                if x + 1 < sx {
                    sum += ovl.heat[i + 1];
                    cnt += 1;
                }
                if y > 0 {
                    sum += ovl.heat[i - sx];
                    cnt += 1;
                }
                if y + 1 < sy {
                    sum += ovl.heat[i + sx];
                    cnt += 1;
                }
                if cnt == 0 {
                    continue;
                }
                let neigh_mean = sum / cnt as f32;
                let lap = neigh_mean - nh;
                let b_here = wet_diffusion_barrier(m);
                scratch[i] = lap * 0.65 * b_here * rain_suppress * dt_e * wind_boost.powf(0.35);
            }
        }

        for i in 0..n {
            ovl.heat[i] = (ovl.heat[i] + scratch[i]).clamp(0.0, 1.0);
        }

        let mean_h: f32 = ovl.heat.iter().sum::<f32>() / n as f32;
        let mean_f: f32 = ovl.fuel.iter().sum::<f32>() / n as f32;
        surf.heat = mean_h;
        surf.fuel = mean_f;

        if mean_h > 0.035 {
            if !dirty.fire_field {
                dirty.fire_field = true;
                hooks.notify_fire_field_dirty(entity);
            }
        } else if mean_h < 0.008 {
            dirty.fire_field = false;
        }
    }
}
