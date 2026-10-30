//! Per-cell **surface fire** field on the chunk grid — CPU authority (Phase 1 fire runbook).
//! [`ChunkCellMatrix::moisture`] is atmospheric humidity / dryness; **standing water** (elevation +
//! `flooded` tag) suppresses burn and blocks spread — see [`super::surface_water::SurfaceWaterFireGate`].
//! Scalar [`ChunkSurfaceFire`] tracks chunk means for GPU uniforms.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::cell_spread::{
    admit_spark, apply_snow_to_hold, burn_fuel_step, lightning_spark, rain_blocks_neighbor_spread,
    scaled_neighbor_heat_gain, should_arm_burn_hold, spread_allowed, step_burn_hold,
    take_neighbor_deposit, FireCell, BURNED_HOLD_TICKS, NEIGHBOR_FUEL_DRAW, SPREAD_HEAT_MIN,
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
use crate::systems::ecology::{ChunkCellStand, ChunkEcology, StandBurnConfig, VegetationField};
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

/// Cardinal deposits are rated per second, not per tick: a hot neighbour heats a facing cell by
/// `NEIGHBOR_HEAT_GAIN * dt * DEPOSIT_RATE` (and draws `NEIGHBOR_FUEL_DRAW * dt * DEPOSIT_RATE` of its fuel). At a
/// per-tick gain the front crossed a chunk in about a second and drained cells to Burnt before they burned.
pub const DEPOSIT_RATE: f32 = 5.0;
/// Fuel a deposit pre-dries from the neighbour, per second (same scale as [`DEPOSIT_RATE`], kept milder).
const DEPOSIT_FUEL_RATE: f32 = 2.5;

/// Chance per second, per unit of spark strength, that a cell ignites on its own (spark is at most ~0.08).
const AMBIENT_IGNITION_PER_SPARK: f32 = 4.0e-6;
/// Heat an ambient ignition gives the cell (above the spread threshold so the burn takes hold).
const AMBIENT_IGNITION_HEAT: f32 = 0.3;

/// Deterministic roll in `[0, 1)` for one cell at one clock step.
#[inline]
fn ambient_roll(chunk_bits: u64, cell: u64, clock: u64) -> f32 {
    let mut x = chunk_bits
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ cell.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ clock.wrapping_mul(0x94D0_49BB_1331_11EB);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 40) as f32 / (1u64 << 24) as f32
}

/// Heat a lit cell burns at while it has fuel (before the moisture discount). A lit cell holds this steady
/// until its fuel is gone, so a front is a band of burning cells that advances, not a flash that decays.
const BURN_TARGET_HEAT: f32 = 0.78;
/// How fast a lit cell's heat climbs to its burn target (per second).
const BURN_HEAT_RISE: f32 = 1.2;

#[inline]
fn wet_diffusion_barrier(moisture: f32) -> f32 {
    let over = (moisture - 0.72).max(0.0) / 0.28;
    (1.0 - over * 0.88).clamp(0.08, 1.0)
}

pub fn chunk_fire_overlay_tick(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    water_gate: Res<SurfaceWaterFireGate>,
    stand_cfg: Option<Res<StandBurnConfig>>,
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
        Option<&ChunkCellStand>,
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

    let ember_clock = (time.elapsed_secs() * 60.0) as u64;
    let mut scratch: Vec<f32> = Vec::new();
    let default_cfg = StandBurnConfig::default();
    let stand_cfg = stand_cfg.as_deref().unwrap_or(&default_cfg);

    for (
        entity,
        matrix,
        wx,
        lod,
        eco_opt,
        veg_opt,
        prof_opt,
        fuel_opt,
        stand_opt,
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

        // The vegetation sets what each cell can carry: per cell from its standing stand when that is rated,
        // else the chunk profile's single ceiling.
        let stand = stand_opt.filter(|s| s.is_seeded() && s.fuel_cap.len() == n && s.kind.len() == n);
        match (stand, prof_opt) {
            (Some(s), _) => {
                for (f, cap) in ovl.fuel.iter_mut().zip(&s.fuel_cap) {
                    *f = (*f).clamp(0.0, *cap);
                }
            }
            (None, Some(prof)) => {
                let seed = profile_cell_fuel_seed(prof);
                for f in &mut ovl.fuel {
                    *f = (*f).clamp(0.0, seed);
                }
            }
            (None, None) => {}
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
                // What a steady burn here can consume: a ground fire (target under crown heat) leaves the canopy
                // share standing, a crown fire takes it all. A cell down to that floor has nothing left to burn.
                let target = BURN_TARGET_HEAT * (1.0 - m * 0.6).clamp(0.3, 1.0) * rain_suppress;
                let burn_floor = match stand {
                    Some(s) if target < stand_cfg.crown_heat => stand_cfg.crown(s.kind[i]) * s.fuel_cap[i],
                    _ => 0.0,
                };
                let exhausted = ovl.fuel[i] < MIN_CELL_FUEL_FOR_SPREAD || ovl.fuel[i] <= burn_floor + 0.005;
                // A stand with nothing left to burn goes out quickly (a stand without per-cell fuel
                // keeps the old slow cooling).
                let cooling = if stand.is_some() && exhausted { 0.9 } else { 0.14 };
                // An ambient spark is a rare ignition event, not a heat source: the spark rate is a chance per
                // second that sets a cell alight (the burn then sustains itself). Injecting spark*8 heat per
                // second lit every dry warm cell of the map within seconds.
                let ignite = if spark > 0.0 && h < SPREAD_HEAT_MIN {
                    let chance = spark * AMBIENT_IGNITION_PER_SPARK * dt_e;
                    if ambient_roll(entity.to_bits(), i as u64, ember_clock) < chance {
                        AMBIENT_IGNITION_HEAT
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let mut nh = (h + ignite
                    - h * cooling * dt_e
                    - wet_line
                    - rain_damp * h
                    - h * wx.snow_depth * 0.08 * dt_e)
                    .clamp(0.0, 1.0);
                // Combustion: a cell that is lit (past the spread threshold) with fuel left holds a steady burn.
                if nh >= SPREAD_HEAT_MIN && !exhausted {
                    if nh < target {
                        nh = (nh + (target - nh) * BURN_HEAT_RISE * dt_e).min(target);
                    }
                }
                // Dust floor only: deposits and diffusion are per-second rates, so a cell being heated gains a few
                // thousandths per tick and must be allowed to accumulate toward the spread threshold.
                if nh < 0.001 {
                    nh = 0.0;
                }
                ovl.heat[i] = nh;

                // Burning consumes this cell's fuel at its fuel kind's rate. A ground fire (cool cell)
                // stops at the canopy share; a hot cell burns the crown too. A cool cell refills toward
                // what the stand can carry (litter back, regrowth ramping up).
                let (rate, floor) = match stand {
                    Some(s) => {
                        let kind = s.kind[i];
                        if nh < 0.02 && ovl.fuel[i] < s.fuel_cap[i] {
                            ovl.fuel[i] = (ovl.fuel[i]
                                + s.fuel_cap[i] / stand_cfg.litter(kind).max(1.0) * dt_e)
                                .min(s.fuel_cap[i]);
                        }
                        let floor = if nh < stand_cfg.crown_heat {
                            stand_cfg.crown(kind) * s.fuel_cap[i]
                        } else {
                            0.0
                        };
                        (stand_cfg.rate(kind), floor)
                    }
                    None => (1.0, 0.0),
                };
                let stepped = burn_fuel_step(
                    ovl.fuel[i],
                    nh,
                    dt_e,
                    wx.rain_intensity,
                    wx.wind_speed,
                    rate,
                    floor,
                );
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
                let deposit_k = dt_e * DEPOSIT_RATE;
                let deposit_fuel_k = dt_e * DEPOSIT_FUEL_RATE;
                let mut deposit = |j: usize| {
                    if water_gate.cell_has_standing_water(matrix, j) {
                        return;
                    }
                    if !spread_allowed(wx.snow_depth, matrix.moisture[j]) {
                        return;
                    }
                    let fuel_before = ovl.fuel[j];
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
                        scaled_neighbor_heat_gain(wx.fog_density) * deposit_k,
                    ) {
                        taken[j] = true;
                        ovl.heat[j] = dst.heat;
                        ovl.fuel[j] = (fuel_before - NEIGHBOR_FUEL_DRAW * deposit_fuel_k).max(0.0);
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

/// Fire crosses chunk borders: a hot cell on a chunk edge deposits heat on the facing cell of the neighbouring
/// chunk under the same rules as the in-chunk cardinal deposit ([`take_neighbor_deposit`]). Without this each chunk
/// burns as an island: the front stalls at the seam, and a whole chunk lights at once when a spark lands across it.
///
/// Reads every chunk's edge cells as they stand after [`chunk_fire_overlay_tick`], then applies the deposits, so
/// the exchange is symmetric and order independent. Writes only `heat` / `fuel` of the receiving cell.
pub fn chunk_fire_edge_spread(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    water_gate: Res<SurfaceWaterFireGate>,
    mut q: Query<(
        Entity,
        &Chunk,
        &ChunkCellMatrix,
        &ChunkWeather,
        &mut ChunkFireOverlay,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let deposit_k = time.delta_secs() * ctrl.dt_scale() * DEPOSIT_RATE;
    let deposit_fuel_k = time.delta_secs() * ctrl.dt_scale() * DEPOSIT_FUEL_RATE;
    if deposit_k <= 0.0 {
        return;
    }
    let by_coord: HashMap<IVec2, Entity> = q.iter().map(|(e, c, ..)| (c.coord, e)).collect();
    let mut deposits: Vec<(Entity, usize, f32)> = Vec::new();
    let mut taken: HashSet<(Entity, usize)> = HashSet::new();
    for (_, chunk, matrix, wx, ovl) in q.iter() {
        let (sx, sy) = (matrix.size.x as usize, matrix.size.y as usize);
        let n = sx * sy;
        if n == 0 || ovl.heat.len() != n || ovl.fuel.len() != n || ovl.burn_hold.len() != n {
            continue;
        }
        if ovl.heat.iter().all(|h| *h < SPREAD_HEAT_MIN) || rain_blocks_neighbor_spread(wx.rain_intensity) {
            continue;
        }
        for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let Some(&ne) = by_coord.get(&(chunk.coord + IVec2::new(dx, dy))) else {
                continue;
            };
            let Ok((_, _, nm, nwx, novl)) = q.get(ne) else {
                continue;
            };
            let (nsx, nsy) = (nm.size.x as usize, nm.size.y as usize);
            if nsx != sx || nsy != sy || novl.heat.len() != n || novl.fuel.len() != n || novl.burn_hold.len() != n {
                continue;
            }
            let gain = scaled_neighbor_heat_gain(nwx.fog_density) * deposit_k;
            let along = if dx != 0 { sy } else { sx };
            for k in 0..along {
                let (si, di) = match (dx, dy) {
                    (1, 0) => (k * sx + (sx - 1), k * sx),
                    (-1, 0) => (k * sx, k * sx + (sx - 1)),
                    (0, 1) => ((sy - 1) * sx + k, k),
                    _ => (k, (sy - 1) * sx + k),
                };
                if water_gate.cell_has_standing_water(matrix, si)
                    || water_gate.cell_has_standing_water(nm, di)
                    || !spread_allowed(nwx.snow_depth, nm.moisture[di])
                {
                    continue;
                }
                let src = FireCell { heat: ovl.heat[si], fuel: ovl.fuel[si], burned_hold: ovl.burn_hold[si] };
                let mut dst = FireCell { heat: novl.heat[di], fuel: novl.fuel[di], burned_hold: novl.burn_hold[di] };
                if taken.contains(&(ne, di)) || !take_neighbor_deposit(false, &src, &mut dst, gain) {
                    continue;
                }
                taken.insert((ne, di));
                deposits.push((ne, di, gain));
            }
        }
    }
    for (e, di, gain) in deposits {
        if let Ok((_, _, _, _, mut ovl)) = q.get_mut(e) {
            ovl.heat[di] = (ovl.heat[di] + gain).min(1.0);
            ovl.fuel[di] = (ovl.fuel[di] - NEIGHBOR_FUEL_DRAW * deposit_fuel_k).max(0.0);
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
        // One deposit, rated per second: one 16 ms tick draws NEIGHBOR_FUEL_DRAW * dt * DEPOSIT_FUEL_RATE.
        let one_tick = NEIGHBOR_FUEL_DRAW * 0.016 * DEPOSIT_FUEL_RATE;
        assert!(
            (drawn - one_tick).abs() < one_tick * 0.5,
            "west fuel draw {drawn} is not one deposit ({one_tick})"
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
