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

    let reservoir = reservoir_json(field);

    json!({
        "reservoir": reservoir,
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

/// Reservoir shape counts (OP-10/11/12): pool stage vs crest, forest-floodplain cells wet in the
/// live ROI, how flat the upstream water surface is, and the depth hand-over at the ROI south edge.
fn reservoir_json(field: Option<&SurfaceWaterField>) -> Value {
    let Some(f) = field.filter(|f| f.wired && f.width > 0 && f.depth_m.len() == f.bed_m.len()) else {
        return Value::Null;
    };
    let fixture = crate::scenario::dam_flood::load_dam_valley_fixture();
    let fp = &fixture.forest_floodplain;
    let (mut fp_total, mut fp_wet, mut fp_wet_up) = (0_u32, 0_u32, 0_u32);
    let (mut up_wet, mut surf_min, mut surf_max) = (0_u32, f32::INFINITY, f32::NEG_INFINITY);
    for lz in 0..f.height {
        for lx in 0..f.width {
            let i = f.idx(lx, lz);
            let (wx, wz) = (f.origin_x + lx as i32, f.origin_z + lz as i32);
            let wet = f.depth_m[i] >= crate::systems::water::WET_DEPTH_M;
            if wx >= fp.min.0 && wx <= fp.max.0 && wz >= fp.min.1 && wz <= fp.max.1 {
                fp_total += 1;
                fp_wet += u32::from(wet);
                fp_wet_up += u32::from(wet && f.is_upstream_lz(lz));
            }
            // Deep upstream water only: shallow film cells follow the bed, not the lake.
            if f.is_upstream_lz(lz) && f.depth_m[i] >= 2.0 {
                up_wet += 1;
                let surface = f.surface_at(i);
                surf_min = surf_min.min(surface);
                surf_max = surf_max.max(surface);
            }
        }
    }
    let cx = (fixture.narrows_tile.0 - f.origin_x).clamp(0, f.width as i32 - 1) as u32;
    // River centre column every 16 rows, south (lz 0) to north.
    let centre_profile: Vec<f32> = (0..f.height)
        .step_by(16)
        .map(|lz| f.depth_m[f.idx(cx, lz)])
        .collect();
    json!({
        "mode": format!("{:?}", f.mode),
        "pool_stage_m": f.pool.stage_m,
        "pool_storage_m3": f.pool.storage_m3,
        "channel_crest_m": f.channel_crest_m(),
        "floodplain_cells_in_roi": fp_total,
        "floodplain_cells_wet": fp_wet,
        "floodplain_cells_wet_upstream": fp_wet_up,
        "upstream_deep_cells": up_wet,
        "upstream_surface_spread_m": if up_wet > 0 { surf_max - surf_min } else { 0.0 },
        "south_edge_depth_m": f.depth_m[f.idx(cx, 0)],
        "south_edge_ref_m": f.ref_depth_m.get(f.idx(cx, 0)).copied().unwrap_or(0.0),
        "centre_depth_profile_m_south_to_north_step16": centre_profile,
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

/// Reservoir time series (one compact row per dump sample): lets a run show the impound / drown /
/// recession sequence instead of only the end state. Own section + own `Local` (one domain).
pub fn dump_dam_flood_history_section(
    arm: Res<SimDumpArm>,
    book: Option<Res<DamFloodBenchmarkBook>>,
    field: Option<Res<SurfaceWaterField>>,
    sim_time: Res<crate::systems::sim_control::SimTimeMicros>,
    skip: Res<crate::systems::water::SlowDomainElapsedSecs>,
    dams: Query<&crate::construction::DamBody>,
    mut rows: Local<Vec<Value>>,
    mut dump: ResMut<SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let Some(book) = book.filter(|b| b.seeded) else {
        return;
    };
    let r = reservoir_json(field.as_deref());
    let pick = |k: &str| r.get(k).cloned().unwrap_or(Value::Null);
    rows.push(json!({
        "tick": arm.tick,
        "eco_s": sim_time.0 as f64 / 1_000_000.0 + skip.0,
        "stage": book.stage.as_str(),
        "mode": pick("mode"),
        "pool_stage_m": pick("pool_stage_m"),
        "fp_wet": pick("floodplain_cells_wet"),
        "up_deep": pick("upstream_deep_cells"),
        "surface_spread_m": pick("upstream_surface_spread_m"),
        "south_edge_m": pick("south_edge_depth_m"),
        "drowned": book.stand_drowned,
        "dead": book.stand_dead,
        "live": book.stand_live,
        "inundated": book.inundated_tiles,
        // Dam state per sample: pins where a run diverges (erosion through a notch vs build
        // timing vs crest never installed).
        "crest_m": pick("channel_crest_m"),
        "dam_installed": dams.iter().any(|d| d.barrier_installed),
        "dam_integrity": dams.iter().map(|d| d.integrity_01).fold(f32::NAN, f32::min),
        "dam_col_min": dams
            .iter()
            .flat_map(|d| d.column_integrity.iter().copied())
            .fold(f32::NAN, f32::min),
        "notch": field.as_deref().is_some_and(|f| f.hydraulics.barrier_damaged),
        "breach_q": field.as_deref().map_or(0.0, |f| f.breach_flow_m3s),
        "overtop_q": field.as_deref().map_or(0.0, |f| f.overtopping_flow_m3s),
    }));
    let peak_drowned = rows
        .iter()
        .filter_map(|v| v.get("drowned").and_then(Value::as_u64))
        .max()
        .unwrap_or(0);
    dump.put_value(
        "dam_flood_history",
        json!({ "peak_drowned": peak_drowned, "rows": rows.clone() }),
    );
}

/// The scene's dam site build state + the logistics lines feeding it (counts only, no verdicts).
/// Own section so `dump_dam_flood_section` stays narrow.
pub fn dump_dam_site_section(
    arm: Res<crate::dev::sim_run_dump::SimDumpArm>,
    sites: Query<(
        &crate::strategic::ConstructionSite,
        &crate::strategic::PlannedSite,
        Option<&crate::construction::SiteStageProgress>,
        Option<&crate::strategic::SiteResourceManifest>,
        Option<&crate::strategic::SiteOperationalStats>,
        Option<&crate::economy::logistics::orders::LogisticsOrderSiteId>,
    )>,
    snapshot: Option<Res<crate::economy::logistics::orders::LogisticsOrdersSnapshotRes>>,
    mut dump: ResMut<crate::dev::sim_run_dump::SimRunDump>,
) {
    if !arm.due {
        return;
    }
    let mut out = Vec::new();
    for (site, planned, stage, manifest, stats, sid) in &sites {
        if site.archetype != crate::strategic::SiteArchetype::Dam {
            continue;
        }
        let snap = snapshot.as_ref().map(|s| &s.0);
        let sid = sid.map(|s| s.0);
        let orders: Vec<_> = snap
            .zip(sid)
            .map(|(snap, sid)| {
                snap.orders
                    .iter()
                    .filter(|o| o.sink == sid)
                    .map(|o| json!({
                        "cargo": o.cargo, "status": o.status_label, "delivered": o.delivered,
                        "in_flight": o.in_flight, "remaining": o.remaining, "vehicles": o.vehicles,
                        "source": o.source_name,
                    }))
                    .collect()
            })
            .unwrap_or_default();
        let demands: Vec<_> = snap
            .zip(sid)
            .map(|(snap, sid)| {
                snap.demands
                    .iter()
                    .filter(|d| d.site == sid)
                    .map(|d| json!({"cargo": d.cargo, "on_hand": d.on_hand, "inbound": d.inbound, "target": d.buffer_target}))
                    .collect()
            })
            .unwrap_or_default();
        let rejected: Vec<_> = snap.map(|s| s.rejected.iter().rev().take(5).cloned().collect()).unwrap_or_default();
        let trucks: Vec<_> = snap
            .zip(sid)
            .map(|(snap, sid)| {
                snap.vehicles
                    .iter()
                    .filter(|v| v.to == sid || v.from == sid || v.at == sid)
                    .take(6)
                    .map(|v| json!({"phase": v.phase, "load": v.load, "from": v.from.0, "to": v.to.0, "at": v.at.0, "fraction": v.fraction}))
                    .collect()
            })
            .unwrap_or_default();
        let events: Vec<_> = snap.map(|s| s.events.iter().rev().take(6).map(|e| e.text.clone()).collect()).unwrap_or_default();
        out.push(json!({
            "catalog_id": planned.catalog_id,
            "phase": format!("{:?}", site.phase),
            "progress": stage.map(|s| s.progress),
            "delivered_ratio": manifest.map(|m| m.min_delivered_ratio()),
            "short": manifest.map(|m| m.short_materials().iter().map(|(k, v)| json!([k, v])).collect::<Vec<_>>()),
            "stats": stats.map(|s| json!({"power": s.power_ratio, "supply": s.supply_ratio, "workforce": s.workforce_ratio})),
            "logistics_registered": sid.is_some(),
            "orders": orders,
            "demands": demands,
            "rejected_recent": rejected,
            "site_id": sid.map(|s| s.0),
            "trucks": trucks,
            "events_recent": events,
        }));
    }
    dump.put_value("dam_site", json!(out));
}
