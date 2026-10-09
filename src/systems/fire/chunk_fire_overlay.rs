//! Per-cell **surface fire** field on the chunk grid — CPU authority (Phase 1 fire runbook).
//! [`ChunkCellMatrix::moisture`] is atmospheric humidity / dryness; **standing water** (elevation +
//! `flooded` tag) suppresses burn and blocks spread — see [`super::surface_water::SurfaceWaterFireGate`].
//! Scalar [`ChunkSurfaceFire`] tracks chunk means for GPU uniforms.

use bevy::prelude::*;

use super::cell_spread::{
    admit_spark, apply_snow_to_hold, lightning_spark, rain_blocks_neighbor_spread,
    scaled_neighbor_heat_gain, should_arm_burn_hold, spread_allowed, step_burn_hold,
    fuel_step_in_wind, take_neighbor_deposit, FireCell, BURNED_HOLD_TICKS,
};
use super::chunk_fuel_profile::ChunkFuelProfile;
use super::chunk_surface_fire::ChunkSurfaceFire;
use super::combustion::{
    crown_boost_for_old_growth, ecology_fire_risk_spark_factor, fuel_ignition_gate,
    profile_cell_fuel_seed, profile_spark_multiplier_gated, profile_weighted_smoke_toxic_explosion,
    DEFAULT_CELL_FUEL_SEED, MIN_CELL_FUEL_FOR_SPREAD,
};
use super::fire_fuel::FireFuelField;
use super::surface_water::SurfaceWaterFireGate;
use super::types::ChunkFireOverlay;
use super::witness_collectors::FireEcologyWitness;
use crate::systems::chunk_environment_persist::{
    ChunkEnvironmentDirty, ChunkEnvironmentPersistHooks,
};
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::ecology::{ChunkEcology, VegetationField};
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
                fuel: vec![DEFAULT_CELL_FUEL_SEED; n],
                smoke: vec![0.0; n],
                toxic: vec![0.0; n],
                burn_hold: vec![0; n],
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
    water_gate: Res<SurfaceWaterFireGate>,
    mut hooks: ResMut<ChunkEnvironmentPersistHooks>,
    mut witness: ResMut<FireEcologyWitness>,
    mut q: Query<(
        Entity,
        &ChunkCellMatrix,
        &ChunkWeather,
        Option<&ChunkSimLod>,
        Option<&ChunkEcology>,
        Option<&VegetationField>,
        Option<&ChunkFuelProfile>,
        Option<&FireFuelField>,
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

    for (
        entity,
        matrix,
        wx,
        lod,
        eco_opt,
        veg_opt,
        prof_opt,
        fuel_opt,
        mut ovl,
        mut surf,
        mut dirty,
    ) in &mut q
    {
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
            ovl.fuel.resize(n, DEFAULT_CELL_FUEL_SEED);
        }
        if ovl.smoke.len() != n {
            ovl.smoke.resize(n, 0.0);
        }
        if ovl.toxic.len() != n {
            ovl.toxic.resize(n, 0.0);
        }
        if ovl.burn_hold.len() != n {
            ovl.burn_hold.resize(n, 0);
        }
        let heat_open = ovl.heat.clone();

        let fire_risk = eco_opt.map(|e| e.fire_risk).unwrap_or(0.0);
        let eco_boost = ecology_fire_risk_spark_factor(fire_risk);
        let old_growth = prof_opt
            .map(|p| p.old_growth)
            .or_else(|| veg_opt.map(|v| v.old_growth))
            .unwrap_or(0.0);
        let wildland_mass = prof_opt.map(|p| p.wildland_fuel_mass).unwrap_or(0.0);
        let fuel_gate = fuel_ignition_gate(wildland_mass);
        let profile_mult = prof_opt.map(profile_spark_multiplier_gated).unwrap_or(0.0);
        let crown = fuel_opt
            .map(|f| crown_boost_for_old_growth(old_growth, f))
            .unwrap_or(1.0);
        let (smoke_k, toxic_k, _) = prof_opt
            .map(profile_weighted_smoke_toxic_explosion)
            .unwrap_or((0.5, 0.12, 0.0));

        if let Some(prof) = prof_opt {
            let seed = profile_cell_fuel_seed(prof);
            for f in &mut ovl.fuel {
                *f = (*f).clamp(0.0, seed);
            }
        }

        witness.chunks_sampled = witness.chunks_sampled.saturating_add(1);
        if fuel_gate <= 0.0 {
            witness.chunks_fuel_gated = witness.chunks_fuel_gated.saturating_add(1);
        }

        let rain_suppress = (1.0 - wx.rain_intensity * 0.78).max(0.0);
        let wind_boost = 1.0 + wx.wind_speed * 0.6;

        scratch.clear();
        scratch.resize(n, 0.0);

        for y in 0..sy {
            for x in 0..sx {
                let i = y * sx + x;
                if water_gate.cell_has_standing_water(matrix, i) {
                    ovl.heat[i] = 0.0;
                    ovl.smoke[i] = (ovl.smoke[i] * (1.0 - dt_e * 2.5)).max(0.0);
                    ovl.toxic[i] = (ovl.toxic[i] * (1.0 - dt_e * 2.0)).max(0.0);
                    scratch[i] = 0.0;
                    continue;
                }

                let m = matrix.moisture[i];
                let t = matrix.temperature[i];
                let dryness = SurfaceWaterFireGate::atmospheric_dryness(m);
                let warmth = (t - 0.08).max(0.0);
                let raw_spark = (dryness * warmth * 3.5).min(0.08)
                    * rain_suppress
                    * wind_boost
                    * eco_boost
                    + lightning_spark(wildland_mass, ovl.fuel[i], wx.lightning_risk);
                let admitted = admit_spark(wildland_mass, ovl.fuel[i], raw_spark);
                let spark = if admitted <= 0.0 || profile_mult <= 0.0 {
                    witness.fuel_gated_spark_cells =
                        witness.fuel_gated_spark_cells.saturating_add(1);
                    0.0
                } else {
                    if raw_spark > 0.001 {
                        witness.ungated_spark_cells = witness.ungated_spark_cells.saturating_add(1);
                    }
                    admitted * profile_mult * crown
                };

                let wet_line = (m * 0.12 + wx.rain_intensity * 0.18) * dt_e;
                let rain_damp = wx.rain_intensity * 0.28 * dt_e;

                let h = ovl.heat[i];
                let mut nh = (h + spark * dt_e * 8.0
                    - h * 0.14 * dt_e
                    - wet_line
                    - rain_damp * h
                    - h * wx.snow_depth * 0.08 * dt_e)
                    .clamp(0.0, 1.0);
                if nh < 0.02 {
                    nh = 0.0;
                }
                ovl.heat[i] = nh;

                let stepped = fuel_step_in_wind(ovl.fuel[i], nh, dt_e, wx.rain_intensity, wx.wind_speed);
                ovl.fuel[i] = stepped.fuel;
                if stepped.crossed_below_spread {
                    witness.fuel_depleted_cells = witness.fuel_depleted_cells.saturating_add(1);
                }
                if should_arm_burn_hold(stepped.arm_hold, wx.snow_depth) {
                    ovl.burn_hold[i] = BURNED_HOLD_TICKS;
                }

                ovl.smoke[i] = (ovl.smoke[i] * (1.0 - dt_e * 1.15) + nh * smoke_k * 1.2 * dt_e
                    - wx.rain_intensity * 0.18 * dt_e)
                    .clamp(0.0, 1.0);
                ovl.toxic[i] = (ovl.toxic[i] * (1.0 - dt_e * 0.95) + nh * toxic_k * 1.35 * dt_e
                    - wx.rain_intensity * 0.12 * dt_e)
                    .clamp(0.0, 1.0);

                let mut sum = 0f32;
                let mut cnt = 0u32;
                let mut add = |ni: usize| {
                    if water_gate.cell_has_standing_water(matrix, ni) {
                        return;
                    }
                    sum += ovl.heat[ni];
                    cnt += 1;
                };
                if x > 0 {
                    add(i - 1);
                }
                if x + 1 < sx {
                    add(i + 1);
                }
                if y > 0 {
                    add(i - sx);
                }
                if y + 1 < sy {
                    add(i + sx);
                }
                if cnt == 0 {
                    continue;
                }
                let neigh_mean = sum / cnt as f32;
                let lap = neigh_mean - nh;
                let b_here = wet_diffusion_barrier(m);
                scratch[i] = if ovl.fuel[i] < MIN_CELL_FUEL_FOR_SPREAD || fuel_gate <= 0.0 {
                    0.0
                } else {
                    let spread = lap * 0.65 * b_here * rain_suppress * dt_e * wind_boost.powf(0.35);
                    if spread.abs() > 1e-8 {
                        witness.neighbor_spread_cells =
                            witness.neighbor_spread_cells.saturating_add(1);
                    }
                    spread
                };
            }
        }

        for i in 0..n {
            if water_gate.cell_has_standing_water(matrix, i) {
                ovl.heat[i] = 0.0;
                ovl.burn_hold[i] = 0;
                continue;
            }
            ovl.heat[i] = (ovl.heat[i] + scratch[i]).clamp(0.0, 1.0);
        }

        for i in 0..n {
            if water_gate.cell_has_standing_water(matrix, i) {
                ovl.burn_hold[i] = 0;
                continue;
            }
            let mut cell = FireCell {
                heat: ovl.heat[i],
                fuel: ovl.fuel[i],
                burned_hold: ovl.burn_hold[i],
            };
            step_burn_hold(&mut cell);
            apply_snow_to_hold(&mut cell, wx.snow_depth);
            ovl.heat[i] = cell.heat;
            ovl.fuel[i] = cell.fuel;
            ovl.burn_hold[i] = cell.burned_hold;
        }

        let heat_src = ovl.heat.clone();
        let fuel_src = ovl.fuel.clone();
        let hold_src = ovl.burn_hold.clone();
        let rain_closed = rain_blocks_neighbor_spread(wx.rain_intensity);
        let mut taken = vec![false; n];
        for y in 0..sy {
            for x in 0..sx {
                let i = y * sx + x;
                if rain_closed || water_gate.cell_has_standing_water(matrix, i) {
                    continue;
                }
                let src = FireCell {
                    heat: heat_src[i],
                    fuel: fuel_src[i],
                    burned_hold: hold_src[i],
                };
                let mut deposit = |j: usize| {
                    if water_gate.cell_has_standing_water(matrix, j) {
                        return;
                    }
                    if !spread_allowed(wx.snow_depth, matrix.moisture[j]) {
                        return;
                    }
                    let mut dst = FireCell {
                        heat: if heat_open[j] >= 1.0 {
                            heat_open[j]
                        } else {
                            ovl.heat[j]
                        },
                        fuel: ovl.fuel[j],
                        burned_hold: ovl.burn_hold[j],
                    };
                    if take_neighbor_deposit(
                        taken[j],
                        &src,
                        &mut dst,
                        scaled_neighbor_heat_gain(wx.fog_density),
                    ) {
                        taken[j] = true;
                        ovl.heat[j] = dst.heat;
                        ovl.fuel[j] = dst.fuel;
                        witness.neighbor_spread_cells =
                            witness.neighbor_spread_cells.saturating_add(1);
                    }
                };
                if x > 0 {
                    deposit(i - 1);
                }
                if x + 1 < sx {
                    deposit(i + 1);
                }
                if y > 0 {
                    deposit(i - sx);
                }
                if y + 1 < sy {
                    deposit(i + sx);
                }
            }
        }

        let mut sum_h = 0f32;
        let mut burnable = 0u32;
        for i in 0..n {
            if water_gate.cell_has_standing_water(matrix, i) {
                continue;
            }
            sum_h += ovl.heat[i];
            burnable += 1;
        }
        let mean_h = if burnable > 0 {
            sum_h / burnable as f32
        } else {
            0.0
        };
        let mean_f: f32 = ovl.fuel.iter().sum::<f32>() / n as f32;
        surf.heat = mean_h;
        surf.fuel = mean_f;

        witness.accumulate_chunk(mean_h, mean_f, old_growth, fuel_gate > 0.0);

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

#[cfg(test)]
mod link_tests {
    use super::super::cell_spread::NEIGHBOR_FUEL_DRAW;
    use super::*;
    use crate::strategic::disruption_after_cell_heat;
    use crate::systems::chunk_environment_persist::{
        ChunkEnvironmentDirty, ChunkEnvironmentPersistHooks,
    };
    use crate::systems::fire::{
        ChunkFireOverlay, ChunkSurfaceFire, FireEcologyWitness, SurfaceWaterFireGate,
    };
    use crate::systems::sim_control::SimControlState;
    use crate::systems::weather::ChunkWeather;
    use crate::terrain::generation::ChunkCellMatrix;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn grid() -> (ChunkCellMatrix, ChunkFireOverlay) {
        let mut matrix = ChunkCellMatrix::new(UVec2::new(3, 3));
        for elev in &mut matrix.elevation {
            *elev = 1.0;
        }
        let n = 9usize;
        let overlay = ChunkFireOverlay {
            heat: vec![0.0; n],
            fuel: vec![0.4; n],
            smoke: vec![0.0; n],
            toxic: vec![0.0; n],
            burn_hold: vec![0; n],
        };
        (matrix, overlay)
    }

    fn run_one(matrix: ChunkCellMatrix, overlay: ChunkFireOverlay, paused: bool) -> ChunkFireOverlay {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(16)))
            .insert_resource(SimControlState {
                paused,
                steps_remaining: 0,
                speed: 1.0,
            })
            .init_resource::<SurfaceWaterFireGate>()
            .init_resource::<ChunkEnvironmentPersistHooks>()
            .init_resource::<FireEcologyWitness>()
            .add_systems(Update, chunk_fire_overlay_tick);
        app.world_mut().spawn((
            matrix,
            ChunkWeather::default(),
            overlay,
            ChunkSurfaceFire::default(),
            ChunkEnvironmentDirty::default(),
        ));
        // Real time reports zero on the first update. The second update is the tick.
        app.update();
        app.update();
        let mut q = app.world_mut().query::<&ChunkFireOverlay>();
        q.iter(app.world()).next().expect("overlay").clone()
    }

    #[test]
    fn link_west_and_north_receive_the_deposit_a_diagonal_does_not() {
        let (matrix, mut overlay) = grid();
        let center = 4usize;
        overlay.heat[center] = 0.6;
        overlay.fuel[center] = 0.8;
        let west = 3usize;
        let north = 1usize;
        let before_west = overlay.fuel[west];
        let out = run_one(matrix, overlay, false);
        assert!(out.heat[west] > 0.0, "west {}", out.heat[west]);
        assert!(out.heat[north] > 0.0, "north {}", out.heat[north]);
        for diagonal in [0usize, 2, 6, 8] {
            assert_eq!(out.heat[diagonal], 0.0, "diagonal {diagonal}");
        }
        let drawn = before_west - out.fuel[west];
        assert!(
            (drawn - NEIGHBOR_FUEL_DRAW).abs() < 0.005,
            "west fuel draw {drawn} is not one deposit"
        );
    }

    #[test]
    fn link_saturated_neighbor_does_not_give_up_more_fuel() {
        let (matrix, mut overlay) = grid();
        overlay.heat[4] = 0.6;
        overlay.fuel[4] = 0.8;
        let west = 3usize;
        overlay.heat[west] = 1.0;
        overlay.fuel[west] = 0.5;
        let before = overlay.fuel[west];
        let out = run_one(matrix, overlay, false);
        let drawn = before - out.fuel[west];
        assert!(
            drawn < NEIGHBOR_FUEL_DRAW * 0.5,
            "saturated neighbor gave up {drawn}"
        );
    }

    #[test]
    fn link_paused_fire_tick_does_not_raise_disruption() {
        let (matrix, mut overlay) = grid();
        overlay.heat[4] = 0.4;
        let before = overlay.heat[4];
        let ctrl = SimControlState {
            paused: true,
            steps_remaining: 0,
            speed: 1.0,
        };
        assert!(!ctrl.should_tick());
        let disruption_before = disruption_after_cell_heat(0.0, before);
        let out = run_one(matrix, overlay, true);
        assert!((out.heat[4] - before).abs() < 1e-6);
        assert!((disruption_after_cell_heat(0.0, out.heat[4]) - disruption_before).abs() < 1e-6);
    }
}
