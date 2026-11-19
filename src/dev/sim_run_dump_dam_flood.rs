//! Dam-flood benchmark dump block (counts only — no verdicts).
//! CQ-68: domain reporter writes into [`crate::dev::sim_run_dump::SimRunDump`], not a
//! cross-domain `SystemParam` on the central flush.

use bevy::prelude::*;
use serde_json::{json, Map, Value};

use crate::dev::sim_run_dump::{SimDumpArm, SimRunDump};
use crate::engine::launch_args::TestScene;
use crate::scenario::dam_flood::DamFloodBenchmarkBook;
use crate::systems::water::SlowDomainTimeSkip;
use crate::systems::water::{
    FloodRouteBlockBook, GaugeBook, GaugeId, SurfaceWaterField, SurfaceWaterGpuFieldMeta,
};

/// Counts-only `dam_flood` object for `sim_run_dump.json`.
#[must_use]
pub fn dam_flood_json(
    scene: TestScene,
    book: Option<&DamFloodBenchmarkBook>,
    gauges: Option<&GaugeBook>,
    time_skip: Option<&SlowDomainTimeSkip>,
    field: Option<&SurfaceWaterField>,
    gpu_field: Option<&SurfaceWaterGpuFieldMeta>,
    flood_routes: Option<&FloodRouteBlockBook>,
) -> Value {
    if !scene.is_dam_flood() && !book.is_some_and(|b| b.seeded) {
        return Value::Null;
    }

    let mut gauges_map = Map::new();
    if let Some(g) = gauges {
        for id in [GaugeId::G1, GaugeId::G2, GaugeId::G3] {
            if let Some(r) = g.reading(id) {
                gauges_map.insert(
                    id.as_str().into(),
                    json!({
                        "discharge_m3s": r.discharge_m3s,
                        "stage_m": r.stage_m,
                        "velocity_mps": r.velocity_mps,
                    }),
                );
            }
        }
    }

    let stage = book.map(|b| b.stage.as_str()).unwrap_or("S0River");
    let entry = book.map(|b| b.stage_entry_tick.to_vec()).unwrap_or_default();
    let skip = time_skip;
    let s0_entered = book
        .map(|b| b.stage_entry_tick[0] > 0 || b.seeded)
        .unwrap_or(false);
    let s0_err = book
        .map(|b| f64::from(b.s0_gauge_balance_err))
        .unwrap_or(1.0);
    let s2_ratio = book
        .map(|b| f64::from(b.s2_downstream_ratio))
        .unwrap_or(1.0);
    let s4_hydro = book
        .map(|b| b.s4_hydro_mw_peak.max(b.hydro_mw))
        .unwrap_or(0.0);
    // Live field count preferred; book mirror is fallback (DF-9/DF-10).
    let inundated_live = field
        .filter(|f| f.wired)
        .map(|f| {
            f.depth_m
                .iter()
                .filter(|h| **h >= crate::systems::water::WET_DEPTH_M)
                .count() as u32
        })
        .or_else(|| book.map(|b| b.inundated_tiles))
        .unwrap_or(0);

    json!({
        "stage": stage,
        "stage_entry_tick": entry,
        "s0_entered": s0_entered,
        "fixture_id": book.map(|b| b.fixture_id.clone()).unwrap_or_default(),
        "seeded": book.map(|b| b.seeded).unwrap_or(false),
        "gauges": gauges_map,
        "water_mass_m3": field.map(|f| f.mass_m3).unwrap_or_else(|| book.map(|b| b.water_mass_m3).unwrap_or(0.0)),
        "mass_error_frac": field.map(|f| f.mass_error_frac).unwrap_or(1.0),
        "inundated_tiles": inundated_live,
        "dam_integrity_mean": book.map(|b| b.dam_integrity_mean).unwrap_or(0.0),
        "outlet_flow_m3s": field.map(|f| f.outlet_flow_m3s).unwrap_or_else(|| book.map(|b| b.outlet_flow_m3s).unwrap_or(0.0)),
        "spillway_flow_m3s": field.map(|f| f.spillway_flow_m3s).unwrap_or_else(|| book.map(|b| b.spillway_flow_m3s).unwrap_or(0.0)),
        "overtopping_flow_m3s": field.map(|f| f.overtopping_flow_m3s).unwrap_or(0.0),
        "breach_flow_m3s": book.map(|b| b.breach_flow_m3s).unwrap_or(0.0),
        "stand_live": book.map(|b| b.stand_live).unwrap_or(0),
        "stand_drowned": book.map(|b| b.stand_drowned).unwrap_or(0),
        "stand_dead": book.map(|b| b.stand_dead).unwrap_or(0),
        "fuel_moisture_mean": book.map(|b| b.fuel_moisture_mean).unwrap_or(0.0),
        "hydro_mw": book.map(|b| b.hydro_mw).unwrap_or(0.0),
        "thermal_mw": book.map(|b| b.thermal_mw).unwrap_or(0.0),
        "solver_ms": field.map(|f| f.solver_ms).unwrap_or_else(|| book.map(|b| b.solver_ms).unwrap_or(0.0)),
        "time_skip_enabled": skip.map(|s| s.enabled).unwrap_or(false),
        "time_skip_dt_slow_secs": skip.map(|s| s.dt_slow_secs).unwrap_or(0),
        "s0_gauge_balance_err": s0_err,
        "s2_downstream_ratio": s2_ratio,
        "s3_drowned_cells": book.map(|b| b.stand_drowned).unwrap_or(0),
        "s4_hydro_mw": s4_hydro,
        "s5_impact_err_m": book.map(|b| b.s5_impact_err_m).unwrap_or(999.0),
        "s7_wave_arrival_g3_s": book.map(|b| b.s7_wave_arrival_g3_s).unwrap_or(0.0),
        "s7_hydro_mw": book.map(|b| b.hydro_mw).unwrap_or(0.0),
        "s8_inundated_tiles": inundated_live,
        "s9_burn_rate_dead_over_live": book.map(|b| b.s9_burn_rate_dead_over_live).unwrap_or(0.0),
        "water_solver_wired": field.map(|f| f.wired).unwrap_or(false),
        "water_gpu_field_wired": gpu_field.map(|g| g.wired).unwrap_or(false),
        "water_gpu_field_wet_texels": gpu_field.map(|g| g.wet_texels).unwrap_or(0),
        "water_upload_ms": gpu_field.map(|g| g.upload_ms).unwrap_or(0.0),
        "flood_route_blocked_tiles": flood_routes.map(|b| b.blocked.len() as u32).unwrap_or(0),
    })
}

/// DamFlood scene reporter — reads dam book + water gauges/field only.
pub fn dump_dam_flood_section(
    arm: Res<SimDumpArm>,
    book: Option<Res<DamFloodBenchmarkBook>>,
    gauges: Option<Res<GaugeBook>>,
    time_skip: Option<Res<SlowDomainTimeSkip>>,
    field: Option<Res<SurfaceWaterField>>,
    gpu_field: Option<Res<SurfaceWaterGpuFieldMeta>>,
    flood_routes: Option<Res<FloodRouteBlockBook>>,
    launch: Option<Res<crate::engine::launch_args::EngineLaunchArgs>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let scene = launch
        .as_ref()
        .map(|l| l.test_scene)
        .unwrap_or(TestScene::None);
    let v = dam_flood_json(
        scene,
        book.as_deref(),
        gauges.as_deref(),
        time_skip.as_deref(),
        field.as_deref(),
        gpu_field.as_deref(),
        flood_routes.as_deref(),
    );
    if !v.is_null() {
        dump.put_value("dam_flood", v);
    }
}
