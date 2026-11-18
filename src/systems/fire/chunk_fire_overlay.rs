//! Per-cell **surface fire** field on the chunk grid — CPU authority (Phase 1 fire runbook).
//! [`ChunkCellMatrix::moisture`] is atmospheric humidity / dryness; **standing water** (elevation +
//! `flooded` tag) suppresses burn and blocks spread — see [`super::surface_water::SurfaceWaterFireGate`].
//! Scalar [`ChunkSurfaceFire`] tracks chunk means for GPU uniforms.
//!
//! **ECS-08** — Game object updates as jobs / async updates (gea4v2 17.7.4.1–2 · 17.7.4.6).
//! **CONC-01** — Job systems (gea4v1 8.6.4). LEGACY: chunk-parallel game-object update (gea_v2_e4 p626).
//!
//! Chunk-parallel shape:
//! 1. Tick-start snapshots = job inputs (immutable for the tick).
//! 2. Per-chunk cell update in [`chunk_fire_overlay_tick`] = sole writer of that chunk's
//!    [`ChunkFireOverlay`] during the tick (no dual writer across systems).
//! 3. Cross-chunk edge deposits collect into a job-result list, then
//!    [`apply_edge_fire_deposits`] applies on the main thread only (ordered after the tick).

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::cell_spread::{
    admit_spark, apply_snow_to_hold, burn_fuel_step, directional_heat_gain, fire_cell_hash01,
    fire_front_deposit_jitter, lightning_spark, neighbor_step_dist,
    rain_mm_h_blocks_neighbor_spread, scaled_neighbor_heat_gain, should_arm_burn_hold, slope_rise,
    spread_allowed, step_burn_hold, take_neighbor_deposit, FireCell, BURNED_HOLD_TICKS, NEIGHBOR8,
    NEIGHBOR_FUEL_DRAW, SPREAD_HEAT_MIN,
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
use super::ecology_metrics::FireEcologyStats;
use crate::systems::chunk_environment_persist::{
    ChunkEnvironmentDirty, ChunkEnvironmentPersistHooks,
};
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::ecology::{
    ChunkCellFuelMoisture, ChunkCellStand, ChunkEcology, StandBurnConfig, VegetationField,
};
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Per-chunk overlay at tick start (previous tick’s end state) for PHYS-06 neighbour wake.
#[derive(Clone)]
struct ChunkFireTickSnap {
    sx: usize,
    sy: usize,
    heat: Vec<f32>,
    fuel: Vec<f32>,
    burn_hold: Vec<u16>,
    wind_mps: Vec2,
}

#[inline]
fn overlay_sleeping(ovl: &ChunkFireOverlay) -> bool {
    ovl.heat.iter().all(|v| *v == 0.0)
        && ovl.smoke.iter().all(|v| *v == 0.0)
        && ovl.toxic.iter().all(|v| *v == 0.0)
        && ovl.burn_hold.iter().all(|v| *v == 0)
}

#[inline]
fn snap_chunk_sleeping(snap: &ChunkFireTickSnap) -> bool {
    snap.heat.iter().all(|&v| v == 0.0) && snap.burn_hold.iter().all(|&v| v == 0)
}

#[inline]
fn chunk_has_spread_source(heat: &[f32], fuel: &[f32], hold: &[u16]) -> bool {
    heat.iter().enumerate().any(|(i, &h)| {
        can_spread(&FireCell {
            heat: h,
            fuel: fuel[i],
            burned_hold: hold[i],
        })
    })
}

/// Visit cells on the source chunk’s edge that face the neighbour at offset `(toward_dx, toward_dy)`.
/// Avoids allocating a per-neighbour `Vec` on the hot path (CQ-60).
fn for_each_source_edge_cell(
    sx: usize,
    sy: usize,
    toward_dx: i32,
    toward_dy: i32,
    mut visit: impl FnMut(usize),
) {
    match (toward_dx, toward_dy) {
        (1, 0) => {
            for y in 0..sy {
                visit(y * sx + (sx - 1));
            }
        }
        (-1, 0) => {
            for y in 0..sy {
                visit(y * sx);
            }
        }
        (0, 1) => {
            for x in 0..sx {
                visit((sy - 1) * sx + x);
            }
        }
        (0, -1) => {
            for x in 0..sx {
                visit(x);
            }
        }
        (1, 1) => visit((sy - 1) * sx + (sx - 1)),
        (1, -1) => visit(sx - 1),
        (-1, 1) => visit((sy - 1) * sx),
        (-1, -1) => visit(0),
        _ => {}
    }
}

/// Sleeping chunks to run the full tick: only neighbours of the active-chunk set (spread sources).
fn sleeping_chunks_to_wake(
    snaps: &HashMap<IVec2, ChunkFireTickSnap>,
    active: &HashSet<IVec2>,
) -> HashSet<IVec2> {
    let mut wake = HashSet::new();
    for &acoord in active {
        let Some(src) = snaps.get(&acoord) else {
            continue;
        };
        for (dx, dy) in NEIGHBOR8 {
            let ncoord = acoord + IVec2::new(dx, dy);
            let Some(dst) = snaps.get(&ncoord) else {
                continue;
            };
            if !snap_chunk_sleeping(dst) {
                continue;
            }
            let mut edge_hot = false;
            for_each_source_edge_cell(src.sx, src.sy, dx, dy, |i| {
                if edge_hot {
                    return;
                }
                if can_spread(&FireCell {
                    heat: src.heat[i],
                    fuel: src.fuel[i],
                    burned_hold: src.burn_hold[i],
                }) {
                    edge_hot = true;
                }
            });
            let wind_wake = {
                let sp = src.wind_mps.length();
                if sp > 0.45 {
                    let dir = Vec2::new(dx as f32, dy as f32).normalize();
                    src.wind_mps.normalize().dot(dir) > 0.35
                        && chunk_has_spread_source(&src.heat, &src.fuel, &src.burn_hold)
                } else {
                    false
                }
            };
            if edge_hot || wind_wake {
                wake.insert(ncoord);
            }
        }
    }
    wake
}

/// At tick start, apply cross-chunk edge deposits from neighbour snapshots onto a woken chunk.
fn prime_woken_chunk_from_neighbors(
    dst_coord: IVec2,
    dst_sx: usize,
    dst_sy: usize,
    ovl: &mut ChunkFireOverlay,
    wx: &ChunkWeather,
    snaps: &HashMap<IVec2, ChunkFireTickSnap>,
    active: &HashSet<IVec2>,
    deposit_k: f32,
    deposit_fuel_k: f32,
) {
    let n = dst_sx * dst_sy;
    if deposit_k <= 0.0 || ovl.heat.len() != n || rain_mm_h_blocks_neighbor_spread(wx.rain_mm_h) {
        return;
    }
    let base_gain = scaled_neighbor_heat_gain(wx.fog_density);
    for &acoord in active {
        if acoord == dst_coord {
            continue;
        }
        let dx = dst_coord.x - acoord.x;
        let dy = dst_coord.y - acoord.y;
        if !(NEIGHBOR8.contains(&(dx, dy))) {
            continue;
        }
        let Some(src) = snaps.get(&acoord) else {
            continue;
        };
        if src.sx != dst_sx || src.sy != dst_sy {
            continue;
        }
        let wind = src.wind_mps;
        let gust = wx.gust_mps;
        if dx != 0 && dy != 0 {
            continue;
        }
        let along = if dx != 0 { dst_sy } else { dst_sx };
        for k in 0..along {
            let (si, di) = match (dx, dy) {
                (1, 0) => (k * dst_sx + (dst_sx - 1), k * dst_sx),
                (-1, 0) => (k * dst_sx, k * dst_sx + (dst_sx - 1)),
                (0, 1) => ((dst_sy - 1) * dst_sx + k, k),
                (0, -1) => (k, (dst_sy - 1) * dst_sx + k),
                _ => (0, 0),
            };
            let gain =
                directional_heat_gain(base_gain, dx, dy, wind, gust, 0.0) * deposit_k * WAKE_NEIGHBOR_DEPOSIT_MUL;
            let src_cell = FireCell {
                heat: src.heat[si],
                fuel: src.fuel[si],
                burned_hold: src.burn_hold[si],
            };
            let mut dst_cell = FireCell {
                heat: ovl.heat[di],
                fuel: ovl.fuel[di],
                burned_hold: ovl.burn_hold[di],
            };
            if take_neighbor_deposit(false, &src_cell, &mut dst_cell, gain) {
                ovl.heat[di] = dst_cell.heat.min(1.0);
                ovl.fuel[di] = (ovl.fuel[di] - NEIGHBOR_FUEL_DRAW * deposit_fuel_k).max(0.0);
            }
        }
    }
}

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
/// Cross-chunk priming on PHYS-06 wake. One tick-start deposit from the neighbour snapshot,
/// slightly above live edge spread so a woken island catches the heat it missed on the short path.
const WAKE_NEIGHBOR_DEPOSIT_MUL: f32 = 5.0;

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

/// Working buffers of [`chunk_fire_overlay_tick`], kept between frames so the tick reuses its
/// allocations instead of cloning every chunk's heat / fuel / hold each frame.
#[derive(Default)]
pub struct FireTickScratch {
    spread: Vec<f32>,
    /// Heat at the start of the tick; only filled for chunks that hold a saturated cell.
    heat_open: Vec<f32>,
    heat_src: Vec<f32>,
    fuel_src: Vec<f32>,
    hold_src: Vec<u16>,
    taken: Vec<bool>,
    /// Standing-water mask of the chunk being ticked.
    water: Vec<bool>,
    tick_snapshots: HashMap<IVec2, ChunkFireTickSnap>,
    sleep_wake: HashSet<IVec2>,
}

#[allow(clippy::too_many_arguments)]
pub fn chunk_fire_overlay_tick(
    mut bufs: Local<FireTickScratch>,
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    water_gate: Res<SurfaceWaterFireGate>,
    stand_cfg: Option<Res<StandBurnConfig>>,
    mut hooks: ResMut<ChunkEnvironmentPersistHooks>,
    mut witness: ResMut<FireEcologyStats>,
    mut q: Query<(
        Entity,
        &Chunk,
        &ChunkCellMatrix,
        &ChunkWeather,
        Option<&ChunkSimLod>,
        Option<&ChunkEcology>,
        Option<&VegetationField>,
        Option<&ChunkFuelProfile>,
        Option<&FireFuelField>,
        Option<&ChunkCellStand>,
        Option<&ChunkCellFuelMoisture>,
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
    let _perf = crate::render::PerfScope::new("upd_fire_overlay_tick");

    let ember_clock = (time.elapsed_secs() * 60.0) as u64;
    let default_cfg = StandBurnConfig::default();
    let stand_cfg = stand_cfg.as_deref().unwrap_or(&default_cfg);

    bufs.tick_snapshots.clear();
    bufs.sleep_wake.clear();
    // Pass 1: which chunks have a live spread source (no vector clones yet).
    let mut active_coords = HashSet::new();
    for (_entity, chunk, matrix, _wx, _, _, _, _, _, _, _, ovl, ..) in q.iter() {
        let sx = matrix.size.x as usize;
        let sy = matrix.size.y as usize;
        let n = sx * sy;
        if n == 0 || ovl.heat.len() != n || ovl.fuel.len() != n || ovl.burn_hold.len() != n {
            continue;
        }
        if chunk_has_spread_source(&ovl.heat, &ovl.fuel, &ovl.burn_hold) {
            active_coords.insert(chunk.coord);
        }
    }
    // Pass 2: snapshot only active chunks + their 8-neighbour ring (CQ-60).
    let mut snap_coords = active_coords.clone();
    for &acoord in &active_coords {
        for (dx, dy) in NEIGHBOR8 {
            snap_coords.insert(acoord + IVec2::new(dx, dy));
        }
    }
    for (_entity, chunk, matrix, wx, _, _, _, _, _, _, _, ovl, ..) in q.iter() {
        if !snap_coords.contains(&chunk.coord) {
            continue;
        }
        let sx = matrix.size.x as usize;
        let sy = matrix.size.y as usize;
        let n = sx * sy;
        if n == 0 || ovl.heat.len() != n || ovl.fuel.len() != n || ovl.burn_hold.len() != n {
            continue;
        }
        bufs.tick_snapshots.insert(
            chunk.coord,
            ChunkFireTickSnap {
                sx,
                sy,
                heat: ovl.heat.clone(),
                fuel: ovl.fuel.clone(),
                burn_hold: ovl.burn_hold.clone(),
                wind_mps: wx.wind_mps,
            },
        );
    }
    bufs.sleep_wake = sleeping_chunks_to_wake(&bufs.tick_snapshots, &active_coords);
    // Move out of bufs so the mut query below can borrow without a full HashMap clone.
    let sleep_wake = std::mem::take(&mut bufs.sleep_wake);
    let tick_snapshots = std::mem::take(&mut bufs.tick_snapshots);

    let FireTickScratch {
        spread: scratch,
        heat_open,
        heat_src,
        fuel_src,
        hold_src,
        taken,
        water,
        ..
    } = &mut *bufs;

    for (
        entity,
        chunk,
        matrix,
        wx,
        lod,
        eco_opt,
        veg_opt,
        prof_opt,
        fuel_opt,
        stand_opt,
        fuel_m_opt,
        mut ovl,
        mut surf,
        mut dirty,
    ) in &mut q
    {
        // One change-detection write per chunk instead of one per cell access.
        let ovl: &mut ChunkFireOverlay = &mut ovl;
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
        // A deposit treats a cell that started the tick saturated as saturated; copy only when one is.
        let saturated = ovl.heat.iter().any(|h| *h >= 1.0);
        if saturated {
            heat_open.clear();
            heat_open.extend_from_slice(&ovl.heat);
        }
        let mut fuel_gated_sparks = 0u64;
        let mut ungated_sparks = 0u64;
        let mut neighbor_spread = 0u64;
        let mut fuel_depleted = 0u64;

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
        water.clear();
        water.extend((0..n).map(|i| water_gate.cell_has_standing_water(matrix, i)));

        // Most chunks are not burning: no heat, smoke, toxic or burned window anywhere. While a
        // chunk stays like that (no cell has lit yet this tick), a cell that does not ignite ends
        // the tick as it started apart from regrowth, and no cell receives diffusion or deposits,
        // so those cells take the short path below and the chunk skips the spread passes.
        // PHYS-06: sleeping chunks wake when an active neighbour can spread across the shared edge
        // (or downwind within reach), decided from the previous-tick snapshot above.
        let mut quiet = overlay_sleeping(ovl);
        if quiet && sleep_wake.contains(&chunk.coord) {
            quiet = false;
            prime_woken_chunk_from_neighbors(
                chunk.coord,
                sx,
                sy,
                &mut *ovl,
                wx,
                &tick_snapshots,
                &active_coords,
                dt_e * DEPOSIT_RATE,
                dt_e * DEPOSIT_FUEL_RATE,
            );
        }

        for y in 0..sy {
            for x in 0..sx {
                let i = y * sx + x;
                if water[i] {
                    if quiet {
                        continue;
                    }
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
                    fuel_gated_sparks += 1;
                    0.0
                } else {
                    if raw_spark > 0.001 {
                        ungated_sparks += 1;
                    }
                    admitted * profile_mult * crown
                };

                let wet_line = (m * 0.12 + wx.rain_intensity * 0.18) * dt_e;
                let rain_damp = wx.rain_intensity * 0.28 * dt_e;

                let h = ovl.heat[i];
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
                // Quiet chunk, cold cell, no ignition: heat, smoke and toxic stay zero and burning
                // consumes nothing, so only regrowth applies.
                if quiet && ignite == 0.0 && wet_line >= 0.0 {
                    if let Some(s) = stand {
                        if ovl.fuel[i] < s.fuel_cap[i] {
                            ovl.fuel[i] = regrow_fuel(ovl.fuel[i], s.fuel_cap[i], stand_cfg.litter(s.kind[i]), dt_e);
                        }
                    }
                    continue;
                }
                quiet = false;
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
                            ovl.fuel[i] = regrow_fuel(ovl.fuel[i], s.fuel_cap[i], stand_cfg.litter(kind), dt_e);
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
                    fuel_depleted += 1;
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

                // Residual heat bleed must stay wind-aware. Isotropic laplacian (old 0.45 *
                // wind_spread) re-circularized fronts and fought the 8-neigh deposit pass.
                let wind = wx.wind_mps;
                let gust = wx.gust_mps;
                let elev = &matrix.elevation;
                let cell_m = TILE_METRES_GAMEPLAY;
                let mut sum = 0f32;
                let mut wsum = 0f32;
                for &(dx, dy) in &NEIGHBOR8 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx as usize >= sx || ny as usize >= sy {
                        continue;
                    }
                    let ni = ny as usize * sx + nx as usize;
                    if water[ni] {
                        continue;
                    }
                    let rise = if elev.len() == n {
                        slope_rise(elev[i], elev[ni], cell_m * neighbor_step_dist(dx, dy))
                    } else {
                        0.0
                    };
                    // Weight by the same anisotropic kernel as deposits (incoming from neighbour).
                    let w = directional_heat_gain(1.0, -dx, -dy, wind, gust, rise).max(0.05);
                    sum += ovl.heat[ni] * w;
                    wsum += w;
                }
                if wsum <= 0.0 {
                    continue;
                }
                let neigh_mean = sum / wsum;
                let lap = neigh_mean - nh;
                let b_here = wet_diffusion_barrier(m);
                // Calm air only: a tiny bleed. Under wind the deposit pass owns advance.
                let calm_bleed = if wind.length() < 1.25 { 0.08 } else { 0.02 };
                scratch[i] = if ovl.fuel[i] < MIN_CELL_FUEL_FOR_SPREAD || fuel_gate <= 0.0 {
                    0.0
                } else {
                    let spread = lap * calm_bleed * b_here * rain_suppress * dt_e;
                    if spread.abs() > 1e-8 {
                        neighbor_spread += 1;
                    }
                    spread
                };
            }
        }

        // Whether any cell can spread this tick (the same gate `take_neighbor_deposit` applies to
        // its source). A chunk without one makes no cardinal deposit, so it skips the pass.
        let mut any_source = false;
        // A chunk still quiet after the cell pass has no heat to diffuse and no burned window.
        if !quiet {
            for i in 0..n {
                if water[i] {
                    ovl.heat[i] = 0.0;
                    ovl.burn_hold[i] = 0;
                    continue;
                }
                ovl.heat[i] = (ovl.heat[i] + scratch[i]).clamp(0.0, 1.0);
            }

            for i in 0..n {
                if water[i] {
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
                any_source |= can_spread(&cell);
            }
        }

        let rain_closed = rain_mm_h_blocks_neighbor_spread(wx.rain_mm_h);
        if !rain_closed && any_source {
            // Deposits read the sources as they stood after the hold step, while writing in place.
            heat_src.clear();
            heat_src.extend_from_slice(&ovl.heat);
            fuel_src.clear();
            fuel_src.extend_from_slice(&ovl.fuel);
            hold_src.clear();
            hold_src.extend_from_slice(&ovl.burn_hold);
            taken.clear();
            taken.resize(n, false);
            let elev = &matrix.elevation;
            let wind = wx.wind_mps;
            let gust = wx.gust_mps;
            let base_gain = scaled_neighbor_heat_gain(wx.fog_density);
            let deposit_k = dt_e * DEPOSIT_RATE;
            let deposit_fuel_k = dt_e * DEPOSIT_FUEL_RATE;
            let cell_m = TILE_METRES_GAMEPLAY;
            for y in 0..sy {
                for x in 0..sx {
                    let i = y * sx + x;
                    if water[i] {
                        continue;
                    }
                    let src = FireCell {
                        heat: heat_src[i],
                        fuel: fuel_src[i],
                        burned_hold: hold_src[i],
                    };
                    if !can_spread(&src) {
                        continue;
                    }
                    // Spatial front noise + per-direction roll → fingers / unburnt islands (W2).
                    let chunk_bits = entity.to_bits();
                    let bucket = ember_clock / 5;
                    for (dir_i, &(dx, dy)) in NEIGHBOR8.iter().enumerate() {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx as usize >= sx || ny as usize >= sy {
                            continue;
                        }
                        let j = ny as usize * sx + nx as usize;
                        let dest_m = stand_opt
                            .map(|s| {
                                let fine = fuel_m_opt
                                    .and_then(|m| m.fine_01.get(j).copied())
                                    .filter(|_| s.flood_dead.get(j).copied().unwrap_or(false));
                                crate::systems::ecology::effective_fuel_moisture01(
                                    matrix.moisture[j],
                                    s.flood_dead.get(j).copied().unwrap_or(false),
                                    s.submerged_since.get(j).copied().unwrap_or(-1.0) >= 0.0,
                                    fine,
                                )
                            })
                            .unwrap_or(matrix.moisture[j]);
                        if water[j] || !spread_allowed(wx.snow_depth, dest_m) {
                            continue;
                        }
                        let rise = if elev.len() == n {
                            slope_rise(elev[i], elev[j], cell_m * neighbor_step_dist(dx, dy))
                        } else {
                            0.0
                        };
                        let align = {
                            let sp = wind.length();
                            if sp > 1e-4 {
                                Vec2::new(dx as f32, dy as f32)
                                    .normalize()
                                    .dot(wind / sp)
                            } else {
                                0.0
                            }
                        };
                        let (gain_mul, skip_p) = fire_front_deposit_jitter(
                            chunk_bits,
                            sx as u32,
                            j as u32,
                            bucket,
                            align,
                        );
                        let roll = fire_cell_hash01(
                            chunk_bits,
                            (j as u32).wrapping_mul(8).wrapping_add(dir_i as u32),
                            bucket,
                        );
                        // Always allow strong head fire; noise skips flanks/back and weak pockets.
                        if roll < skip_p && align < 0.92 {
                            continue;
                        }
                        let gain = directional_heat_gain(base_gain, dx, dy, wind, gust, rise)
                            * deposit_k
                            * gain_mul;
                        if gain <= 1e-8 {
                            continue;
                        }
                        let fuel_before = ovl.fuel[j];
                        let mut dst = FireCell {
                            heat: if saturated && heat_open[j] >= 1.0 {
                                heat_open[j]
                            } else {
                                ovl.heat[j]
                            },
                            fuel: ovl.fuel[j],
                            burned_hold: ovl.burn_hold[j],
                        };
                        if take_neighbor_deposit(taken[j], &src, &mut dst, gain) {
                            taken[j] = true;
                            ovl.heat[j] = dst.heat;
                            ovl.fuel[j] = (fuel_before - NEIGHBOR_FUEL_DRAW * deposit_fuel_k).max(0.0);
                            neighbor_spread += 1;
                        }
                    }
                }
            }
        }

        witness.fuel_gated_spark_cells = witness.fuel_gated_spark_cells.saturating_add(fuel_gated_sparks);
        witness.ungated_spark_cells = witness.ungated_spark_cells.saturating_add(ungated_sparks);
        witness.neighbor_spread_cells = witness.neighbor_spread_cells.saturating_add(neighbor_spread);
        witness.fuel_depleted_cells = witness.fuel_depleted_cells.saturating_add(fuel_depleted);

        let mut sum_h = 0f32;
        let mut burnable = 0u32;
        if !quiet {
            for i in 0..n {
                if water[i] {
                    continue;
                }
                sum_h += ovl.heat[i];
                burnable += 1;
            }
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
        } else if mean_h < 0.008 && dirty.fire_field {
            dirty.fire_field = false;
        }
    }
}

/// Litter fall / regrowth of one cool cell toward what its stand can carry.
/// Cap is already phase-gated by [`ChunkCellStand`] (0 while snags, ramping in regrowth) — never
/// refill toward a live canopy while the stand is still scarred.
#[inline]
fn regrow_fuel(fuel: f32, cap: f32, litter_secs: f32, dt: f32) -> f32 {
    if cap <= 1e-6 || litter_secs <= 0.0 {
        return fuel.min(cap.max(0.0));
    }
    (fuel + cap / litter_secs.max(1.0) * dt).min(cap)
}

/// A cell that may deposit on its neighbours: lit, fuelled, and out of its burned window
/// (the source gate of [`take_neighbor_deposit`]).
#[inline]
fn can_spread(cell: &FireCell) -> bool {
    cell.burned_hold == 0 && cell.fuel >= MIN_CELL_FUEL_FOR_SPREAD && cell.heat >= SPREAD_HEAT_MIN
}

/// One edge deposit produced by the collect pass — applied only via [`apply_edge_fire_deposits`] (ECS-08).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeFireDeposit {
    pub entity: Entity,
    pub cell: usize,
    pub heat_gain: f32,
}

/// Main-thread sole apply of collected edge deposits (ECS-08 / no dual writer).
///
/// Job bodies must push [`EdgeFireDeposit`] values; they must not mutate overlays themselves.
pub fn apply_edge_fire_deposits(
    deposits: &[EdgeFireDeposit],
    deposit_fuel_k: f32,
    q: &mut Query<(
        Entity,
        &Chunk,
        &ChunkCellMatrix,
        &ChunkWeather,
        &mut ChunkFireOverlay,
    )>,
) {
    for dep in deposits {
        if let Ok((_, _, _, _, mut ovl)) = q.get_mut(dep.entity) {
            apply_one_edge_deposit(&mut ovl, dep.cell, dep.heat_gain, deposit_fuel_k);
        }
    }
}

/// Pure per-cell apply used by [`apply_edge_fire_deposits`] (testable without a full Query).
#[inline]
pub fn apply_one_edge_deposit(
    ovl: &mut ChunkFireOverlay,
    cell: usize,
    heat_gain: f32,
    deposit_fuel_k: f32,
) {
    if cell >= ovl.heat.len() || cell >= ovl.fuel.len() {
        return;
    }
    ovl.heat[cell] = (ovl.heat[cell] + heat_gain).min(1.0);
    ovl.fuel[cell] = (ovl.fuel[cell] - NEIGHBOR_FUEL_DRAW * deposit_fuel_k).max(0.0);
}

/// Fire crosses chunk borders: a hot cell on a chunk edge deposits heat on the facing cell of the neighbouring
/// chunk under the same rules as the in-chunk 8-neighbour deposit ([`take_neighbor_deposit`] +
/// [`directional_heat_gain`]). Without this each chunk burns as an island.
///
/// Collect-then-apply (ECS-08): reads every chunk's edge cells as they stand after
/// [`chunk_fire_overlay_tick`], builds [`EdgeFireDeposit`] jobs, then
/// [`apply_edge_fire_deposits`] on the main thread so the exchange is symmetric and order-independent.
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
    let mut deposits: Vec<EdgeFireDeposit> = Vec::new();
    let mut taken: HashSet<(Entity, usize)> = HashSet::new();
    let cell_m = TILE_METRES_GAMEPLAY;
    for (_, chunk, matrix, wx, ovl) in q.iter() {
        let (sx, sy) = (matrix.size.x as usize, matrix.size.y as usize);
        let n = sx * sy;
        if n == 0 || ovl.heat.len() != n || ovl.fuel.len() != n || ovl.burn_hold.len() != n {
            continue;
        }
        if ovl.heat.iter().all(|h| *h < SPREAD_HEAT_MIN) || rain_mm_h_blocks_neighbor_spread(wx.rain_mm_h) {
            continue;
        }
        let wind = wx.wind_mps;
        let gust = wx.gust_mps;
        // Cardinal seams (full edge) plus diagonal corner hops so 45° wind does not stall at chunk corners.
        for (dx, dy) in [
            (1i32, 0i32),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ] {
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
            let base = scaled_neighbor_heat_gain(nwx.fog_density);
            let diagonal = dx != 0 && dy != 0;
            if diagonal {
                // One corner cell → opposite corner of the diagonal neighbour chunk.
                let (sx_i, sy_i) = (sx as i32, sy as i32);
                let (si_x, si_y) = match (dx, dy) {
                    (1, 1) => (sx_i - 1, sy_i - 1),
                    (1, -1) => (sx_i - 1, 0),
                    (-1, 1) => (0, sy_i - 1),
                    _ => (0, 0),
                };
                let (di_x, di_y) = (si_x - dx * (sx_i - 1), si_y - dy * (sy_i - 1));
                let si = (si_y * sx_i + si_x) as usize;
                let di = (di_y * sx_i + di_x) as usize;
                if water_gate.cell_has_standing_water(matrix, si)
                    || water_gate.cell_has_standing_water(nm, di)
                    || !spread_allowed(nwx.snow_depth, nm.moisture[di])
                {
                    continue;
                }
                let rise = if matrix.elevation.len() == n && nm.elevation.len() == n {
                    slope_rise(
                        matrix.elevation[si],
                        nm.elevation[di],
                        cell_m * neighbor_step_dist(dx, dy),
                    )
                } else {
                    0.0
                };
                let gain = directional_heat_gain(base, dx, dy, wind, gust, rise) * deposit_k;
                let src = FireCell {
                    heat: ovl.heat[si],
                    fuel: ovl.fuel[si],
                    burned_hold: ovl.burn_hold[si],
                };
                let mut dst = FireCell {
                    heat: novl.heat[di],
                    fuel: novl.fuel[di],
                    burned_hold: novl.burn_hold[di],
                };
                if taken.contains(&(ne, di)) || !take_neighbor_deposit(false, &src, &mut dst, gain) {
                    continue;
                }
                taken.insert((ne, di));
                deposits.push(EdgeFireDeposit {
                    entity: ne,
                    cell: di,
                    heat_gain: gain,
                });
                continue;
            }
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
                let rise = if matrix.elevation.len() == n && nm.elevation.len() == n {
                    slope_rise(matrix.elevation[si], nm.elevation[di], cell_m)
                } else {
                    0.0
                };
                let gain = directional_heat_gain(base, dx, dy, wind, gust, rise) * deposit_k;
                let src = FireCell {
                    heat: ovl.heat[si],
                    fuel: ovl.fuel[si],
                    burned_hold: ovl.burn_hold[si],
                };
                let mut dst = FireCell {
                    heat: novl.heat[di],
                    fuel: novl.fuel[di],
                    burned_hold: novl.burn_hold[di],
                };
                if taken.contains(&(ne, di)) || !take_neighbor_deposit(false, &src, &mut dst, gain) {
                    continue;
                }
                taken.insert((ne, di));
                deposits.push(EdgeFireDeposit {
                    entity: ne,
                    cell: di,
                    heat_gain: gain,
                });
            }
        }
    }
    apply_edge_fire_deposits(&deposits, deposit_fuel_k, &mut q);
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
        ChunkFireOverlay, ChunkSurfaceFire, FireEcologyStats, SurfaceWaterFireGate,
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
            .init_resource::<FireEcologyStats>()
            .add_systems(Update, chunk_fire_overlay_tick);
        let mut wx = ChunkWeather::default();
        wx.wind_mps = Vec2::new(-8.0, 0.0);
        wx.wind_speed = 0.65;
        app.world_mut().spawn((
            Chunk::default(),
            matrix,
            wx,
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
            assert!(
                out.heat[diagonal] < 0.01,
                "diagonal {diagonal} leaked {}",
                out.heat[diagonal]
            );
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
    fn sleeping_neighbor_west_edge_ignites_within_two_ticks() {
        use super::super::cell_spread::SPREAD_HEAT_MIN;
        use bevy::prelude::{IVec2, UVec2};

        let size = UVec2::new(8, 8);
        let n = 64usize;
        let mk = || {
            let mut matrix = ChunkCellMatrix::new(size);
            for e in &mut matrix.elevation {
                *e = 1.0;
            }
            let overlay = ChunkFireOverlay {
                heat: vec![0.0; n],
                fuel: vec![0.4; n],
                smoke: vec![0.0; n],
                toxic: vec![0.0; n],
                burn_hold: vec![0; n],
            };
            (matrix, overlay)
        };
        let (m0, mut o0) = mk();
        let (m1, o1) = mk();
        let row = 4usize;
        let east_i = row * 8 + 7;
        let west_i = row * 8;
        o0.heat[east_i] = 0.95;
        o0.fuel[east_i] = 0.95;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(16)))
            .insert_resource(SimControlState {
                paused: false,
                steps_remaining: 0,
                speed: 1.0,
            })
            .init_resource::<SurfaceWaterFireGate>()
            .init_resource::<ChunkEnvironmentPersistHooks>()
            .init_resource::<FireEcologyStats>()
            .add_systems(Update, (chunk_fire_overlay_tick, chunk_fire_edge_spread).chain());
        let mut wx0 = ChunkWeather::default();
        wx0.wind_mps = Vec2::new(12.0, 0.0);
        wx0.wind_speed = 0.8;
        wx0.gust_mps = 14.0;
        let mut wx1 = ChunkWeather::default();
        wx1.wind_mps = Vec2::new(12.0, 0.0);
        wx1.wind_speed = 0.8;
        app.world_mut().spawn((
            Chunk { coord: IVec2::ZERO },
            m0,
            wx0,
            o0,
            ChunkSurfaceFire::default(),
            ChunkEnvironmentDirty::default(),
        ));
        app.world_mut().spawn((
            Chunk { coord: IVec2::new(1, 0) },
            m1,
            wx1,
            o1,
            ChunkSurfaceFire::default(),
            ChunkEnvironmentDirty::default(),
        ));
        for _ in 0..2 {
            app.update();
            app.update();
        }
        let mut q = app.world_mut().query::<(&Chunk, &ChunkFireOverlay)>();
        let west_heat = q
            .iter(app.world())
            .find(|(c, _)| c.coord == IVec2::new(1, 0))
            .map(|(_, o)| o.heat[west_i])
            .expect("neighbor overlay");
        assert!(
            west_heat >= SPREAD_HEAT_MIN,
            "west edge heat {west_heat} should ignite within 2 ticks"
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

    /// ECS-08: edge job results mutate overlay only through the main-thread apply helper.
    #[test]
    fn edge_deposits_apply_on_main_thread_no_dual_writer() {
        let n = 9usize;
        let mut ovl = ChunkFireOverlay {
            heat: vec![0.0; n],
            fuel: vec![0.5; n],
            smoke: vec![0.0; n],
            toxic: vec![0.0; n],
            burn_hold: vec![0; n],
        };
        let cell = 4usize;
        let heat_before = ovl.heat[cell];
        let fuel_before = ovl.fuel[cell];
        let gain = 0.2f32;
        let deposit_fuel_k = 0.016 * DEPOSIT_FUEL_RATE;
        // Job product only — no mutation yet.
        let jobs = [EdgeFireDeposit {
            entity: Entity::from_bits(1),
            cell,
            heat_gain: gain,
        }];
        assert!((ovl.heat[cell] - heat_before).abs() < 1e-9);
        // Sole apply path.
        apply_one_edge_deposit(&mut ovl, jobs[0].cell, jobs[0].heat_gain, deposit_fuel_k);
        assert!((ovl.heat[cell] - (heat_before + gain).min(1.0)).abs() < 1e-5);
        let drawn = fuel_before - ovl.fuel[cell];
        assert!((drawn - NEIGHBOR_FUEL_DRAW * deposit_fuel_k).abs() < 1e-5);
        // Second apply would be a dual writer if a worker also mutated — contract: one apply call.
        assert_eq!(jobs.len(), 1);
    }
}
