//! General surface-water schedule: apply ROI request → step → project → gauges → substrate.

use bevy::prelude::*;

use crate::substrate::registry::WorldSubstrateRegistry;
use crate::substrate::slab::ChunkKey;
use crate::systems::sim_control::{SimControlSystemSet, SimStepClock};
use crate::systems::weather::WeatherField;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::world_scale_contract::CHUNK_TILES_SIM;

use super::gauge::GaugeBook;
use super::surface_water::{SurfaceWaterField, SurfaceWaterMode};
use super::water_runtime::WaterClassRuntime;
use super::{BarrierControlIntent, SlowDomainTimeSkip};

/// Spec any caller (player build, scenario, DamFlood harness) can enqueue to wire the ROI.
#[derive(Clone, Debug)]
pub struct SurfaceWaterRoiSpec {
    pub origin_x: i32,
    pub origin_z: i32,
    pub width: u32,
    pub height: u32,
    pub inflow_m3s: f32,
    pub min_release_m3s: f32,
    /// Optional river seed corridor (world x + half-width tiles + depth m).
    pub river_center_x: Option<i32>,
    pub river_half_width: u32,
    pub river_seed_depth_m: f32,
    /// Optional barrier crest row (world z, crest m, half-span tiles) — e.g. after dam commit.
    pub barrier_world_z: Option<i32>,
    pub barrier_crest_m: f32,
    pub barrier_half_span: u32,
    /// When true and no live DEM, carve a V-valley under `river_center_x`.
    pub synthetic_valley_if_no_dem: bool,
}

/// Pending ROI configure request — sole way to allocate/re-seed `SurfaceWaterField` from gameplay.
#[derive(Resource, Debug, Default)]
pub struct SurfaceWaterRoiRequest {
    pub pending: Option<SurfaceWaterRoiSpec>,
}

#[derive(Resource, Debug, Default)]
pub struct SurfaceWaterSeedLatch {
    pub seeded: bool,
}

fn sample_bed_from_chunks(
    chunks: &Query<(&Chunk, &ChunkCellMatrix)>,
    origin_x: i32,
    origin_z: i32,
    width: u32,
    height: u32,
    elevation_range_m: f32,
) -> (Vec<f32>, u32) {
    let n = (width as usize) * (height as usize);
    let mut bed = vec![12.0_f32; n];
    let mut sampled = 0_u32;
    for (chunk, matrix) in chunks.iter() {
        let origin = chunk.coord * CHUNK_TILES_SIM as i32;
        for lz in 0..height {
            for lx in 0..width {
                let wx = origin_x + lx as i32;
                let wz = origin_z + lz as i32;
                let lx_c = wx - origin.x;
                let ly_c = wz - origin.y;
                if lx_c < 0
                    || ly_c < 0
                    || lx_c >= matrix.size.x as i32
                    || ly_c >= matrix.size.y as i32
                {
                    continue;
                }
                let i = (lz as usize) * (width as usize) + (lx as usize);
                // Normalized 0..1 → metres via the world's height range (CLOSE-W3).
                bed[i] = matrix.elevation[matrix.idx(lx_c as u32, ly_c as u32)] * elevation_range_m;
                sampled += 1;
            }
        }
    }
    (bed, sampled)
}

fn carve_synthetic_valley(bed: &mut [f32], width: u32, height: u32, center_x: i32, origin_x: i32) {
    for lz in 0..height {
        for lx in 0..width {
            let wx = origin_x + lx as i32;
            let i = (lz as usize) * (width as usize) + (lx as usize);
            let dist = (wx - center_x).unsigned_abs() as f32;
            bed[i] = 2.0 + dist * 0.35;
        }
    }
}

/// Consume [`SurfaceWaterRoiRequest`] and allocate the field (any scene / player path).
pub fn seed_surface_water_from_request_system(
    mut request: ResMut<SurfaceWaterRoiRequest>,
    mut latch: ResMut<SurfaceWaterSeedLatch>,
    mut field: ResMut<SurfaceWaterField>,
    mut runtime: ResMut<WaterClassRuntime>,
    chunks: Query<(&Chunk, &ChunkCellMatrix)>,
    static_depth: Option<Res<super::WorldStaticWaterDepth>>,
    params: Option<Res<crate::terrain::generation::world_generator_enhanced::WorldGenParams>>,
) {
    let Some(spec) = request.pending.take() else {
        return;
    };
    if field.wired && latch.seeded {
        // Re-seed allowed when a new request arrives (e.g. world reload).
    }

    let n = (spec.width as usize) * (spec.height as usize);
    if n == 0 {
        return;
    }

    let (mut bed, sampled) = sample_bed_from_chunks(
        &chunks,
        spec.origin_x,
        spec.origin_z,
        spec.width,
        spec.height,
        params.as_ref().map_or(
            crate::terrain::world_scale_contract::DEFAULT_ELEVATION_RANGE_M,
            |p| p.elevation_range_m,
        ),
    );
    // CLOSE-W3: carve the synthetic valley only when the DEM doesn't cover the ROI. Carving
    // over a real DEM gave the solver a different bed than the drawn ground → water walls.
    let full = (spec.width as u64) * (spec.height as u64);
    if spec.synthetic_valley_if_no_dem && u64::from(sampled) < full / 2 {
        if let Some(cx) = spec.river_center_x {
            carve_synthetic_valley(&mut bed, spec.width, spec.height, cx, spec.origin_x);
        }
    }
    let _ = sampled;

    field.allocate(
        spec.origin_x,
        spec.origin_z,
        spec.width,
        spec.height,
        bed,
        spec.inflow_m3s,
    );
    field.mode = SurfaceWaterMode::VirtualPipes;
    field.hydraulics.min_release_m3s = spec.min_release_m3s;
    field.outlet_flow_m3s = spec.inflow_m3s;

    // CLOSE-W3 one water: start the live ROI from the world's static water (same river the
    // world drew), so there is one fluid. Corridor seed only when there is no static water.
    let seeded_static = static_depth
        .as_deref()
        .is_some_and(|st| field.seed_from_static_depth(st) > 0);
    if !seeded_static {
        if let Some(cx) = spec.river_center_x {
            field.seed_river_corridor(cx, spec.river_half_width.max(1), spec.river_seed_depth_m);
        }
    }
    if let Some(bz) = spec.barrier_world_z {
        if spec.barrier_crest_m > 0.0 {
            field.install_barrier_row(
                bz,
                spec.barrier_crest_m,
                spec.barrier_half_span.max(1),
                spec.river_center_x,
            );
        }
    }
    field.recompute_mass();
    runtime.project_from_field(&field, None, 0.0);
    latch.seeded = true;

    info!(
        target: "water::solver",
        "SurfaceWaterField ROI {}x{} origin=({},{}) dem_samples={sampled} inflow={}",
        spec.width,
        spec.height,
        spec.origin_x,
        spec.origin_z,
        spec.inflow_m3s
    );
}

pub fn step_surface_water_system(
    mut field: ResMut<SurfaceWaterField>,
    mut runtime: ResMut<WaterClassRuntime>,
    control: Res<BarrierControlIntent>,
    skip: Res<SlowDomainTimeSkip>,
    clock: Res<SimStepClock>,
    weather: Option<Res<WeatherField>>,
) {
    if !field.wired {
        return;
    }
    // Hitch attribution (measurement only; `HITCH_MS=<ms>` reports the top scopes per slow frame).
    let _perf = crate::render::PerfScope::new("upd_water_solver");

    let has_barrier = field.hydraulics.barrier_crest_m.iter().any(|c| *c > 0.5);
    let use_pool = control.prefer_level_pool
        && has_barrier
        && control.combined_outlet_open01() <= 0.2
        && control.gate_open01 <= 0.2
        && control.effective_turbine_m3s() < 1.0
        && field.breach_flow_m3s <= 0.0
        // DW-3: a damaged column is a notch — flow through it is 2D, not a level pool.
        && !field.hydraulics.barrier_damaged;
    field.mode = if use_pool {
        SurfaceWaterMode::LevelPool
    } else {
        SurfaceWaterMode::VirtualPipes
    };

    let mut dt = 1.0_f32;
    if skip.enabled && use_pool {
        // Time skip rides the sim clock like `SlowDomainElapsedSecs` (ecology): each due sim step
        // advances the reservoir by `dt_slow_secs`, and a frame with no step advances nothing.
        // Stepping `dt_slow` once per FRAME while ecology advanced `dt_slow × due` left the
        // pool ~due× behind the drown clocks, so the reservoir never filled in scenario time
        // (OP-12).
        if clock.due == 0 {
            return;
        }
        dt = skip.dt_slow_secs.max(1) as f32 * clock.due as f32;
    }

    if let Some(wx) = weather.as_ref() {
        let mean_mm_h = if !wx.next.rain_mm_h.is_empty() {
            let s: f32 = wx.next.rain_mm_h.iter().sum();
            s / wx.next.rain_mm_h.len() as f32
        } else {
            0.0
        };
        let area = (field.width as f32) * (field.height as f32) * field.dx_m * field.dx_m;
        field.rain_runoff_m3s = (mean_mm_h / 1000.0 / 3600.0) * area;
    }

    field.step(
        dt,
        control.gate_open01,
        control.outlet_open01,
        control.mid_outlet_open01,
        control.effective_turbine_m3s(),
    );
    runtime.project_from_field(&field, None, dt);
}

pub fn publish_gauges_from_solver_system(
    field: Option<Res<SurfaceWaterField>>,
    mut gauges: Option<ResMut<GaugeBook>>,
) {
    let (Some(field), Some(mut gauges)) = (field, gauges) else {
        return;
    };
    if !field.wired || gauges.sites.is_empty() {
        return;
    }
    let sites: Vec<(i32, i32, u32)> = gauges
        .sites
        .iter()
        .map(|s| (s.tile.x, s.tile.y, s.cross_section_half_tiles))
        .collect();
    for (i, (x, z, half)) in sites.into_iter().enumerate() {
        let (q, stage, vel) = field.gauge_discharge(x, z, half);
        if let Some(r) = gauges.readings.get_mut(i) {
            r.discharge_m3s = q;
            r.stage_m = stage;
            r.velocity_mps = vel;
        }
    }
}

/// Project depth into substrate hydrology for dirty chunks only (no erosion re-stamp).
pub fn project_surface_water_to_substrate_system(
    field: Res<SurfaceWaterField>,
    mut registry: ResMut<WorldSubstrateRegistry>,
) {
    if !field.wired || field.dirty_chunks.is_empty() {
        return;
    }
    let chunk_tiles = CHUNK_TILES_SIM as i32;
    for coord in &field.dirty_chunks {
        let key = ChunkKey::new(coord.x, coord.y);
        let Some(chunk) = registry.chunks.get_mut(key) else {
            continue;
        };
        let origin = IVec2::new(coord.x * chunk_tiles, coord.y * chunk_tiles);
        let n = chunk.hydrology.water_depth.len();
        if n == 0 {
            continue;
        }
        let side = (n as f32).sqrt() as i32;
        if side <= 0 {
            continue;
        }
        for ly in 0..side {
            for lx in 0..side {
                let wx = origin.x + lx;
                let wz = origin.y + ly;
                let i = (ly * side + lx) as usize;
                if i >= n {
                    continue;
                }
                if let Some(d) = field.depth_at_world(wx, wz) {
                    chunk.hydrology.water_depth[i] = d;
                }
            }
        }
    }
}

pub struct SurfaceWaterSolverPlugin;

impl Plugin for SurfaceWaterSolverPlugin {
    fn build(&self, app: &mut App) {
        crate::systems::sim_control::ensure_sim_step_clock(app);
        app.init_resource::<SurfaceWaterField>()
            .init_resource::<WaterClassRuntime>()
            .init_resource::<SurfaceWaterSeedLatch>()
            .init_resource::<SurfaceWaterRoiRequest>()
            .register_type::<SurfaceWaterField>()
            .add_systems(
                Update,
                (
                    seed_surface_water_from_request_system,
                    step_surface_water_system
                        .after(seed_surface_water_from_request_system)
                        .after(SimControlSystemSet::AdvanceSimTick),
                    publish_gauges_from_solver_system.after(step_surface_water_system),
                    project_surface_water_to_substrate_system.after(step_surface_water_system),
                ),
            );
    }
}
