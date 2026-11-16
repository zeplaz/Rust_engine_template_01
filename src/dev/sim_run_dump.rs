//! One sim-run dump. Counts only — no `green` field.
//!
//! Score with `python tools/sim_review/score_dump.py`. Do not treat `*_live.json`
//! refresh writers as a run. Every number is read from the authoritative resource or
//! component that owns it; the only state kept here is run-over-run accumulation
//! (first/last/min/max series, AI decision tallies, frame times).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::app::AppExit;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde_json::{json, Map, Value};

use crate::ai::faction::{
    AiOrder, FactionAiDecisionLog, FactionAiRoster, FactionHq, FactionStartConfig,
};
use crate::combat::{DeploymentOutcomes, Formation, TheaterBook};
use crate::economy::logistics::orders::{
    LogisticsOrderSimRes, LogisticsOrdersSnapshotRes, OrderStatus,
};
use crate::economy::resource_flow::{
    FacilityFlowState, ResourceFlowNode, ResourceFlowSimStats, OUTPUT_BUFFER_CAP,
};
use crate::engine::states::BaseState;
use crate::engine::EngineLaunchArgs;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::probes::frame_hitch::STALL_MS;
use crate::render::{FrameHitchLog, HitchRecord};
use crate::render::WorldFireParticleFrame;
use crate::strategic::{
    ChunkStrategicOverlay, ConstructionSite, FrontlineState, SiteConstructionPhase,
};
use crate::systems::ecology::{
    ChunkCellStand, CELL_BURNING, CELL_BURNT, CELL_GRASS, CELL_OLD, CELL_REGROWTH, CELL_SHRUB,
    CELL_WATER, CELL_YOUNG,
};
use crate::systems::fire::{ChunkFireOverlay, FireEcologyStats, FireEntityExposureStats};
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::Chunk;
use crate::systems::sim_control::SimTick;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{ChunkCellMatrix, WaterClass};

pub const SIM_RUN_DUMP_JSON: &str = "debug_runs/sim_run_dump.json";
pub const SIM_RUN_DUMP_SCHEMA: &str = "sim_run_dump_v2";

const DUMP_STRIDE_TICKS: u64 = 30;
/// Frames skipped before frame-time sampling starts (world gen / shader warm-up) ...
const PERF_WARMUP_FRAMES: u64 = 120;
/// ... and at least this much wall time, so a fast release build is not sampled while still loading.
const PERF_WARMUP_SECS: f32 = 10.0;
const PERF_SAMPLE_CAP: usize = 50_000;
/// Worst hitch frames kept for the dump (by frame time).
const PERF_WORST_HITCHES: usize = 12;
/// Points in the downsampled frame-time series the dashboard draws.
const PERF_SERIES_POINTS: usize = 240;
const HISTORY_CAP: usize = 400;
/// A cell counts as held by the slot with the highest control when it exceeds this.
const TERRITORY_CONTROL_MIN: f32 = 0.5;

/// first / last / min / max of one counter across the run.
#[derive(Clone, Copy, Debug, Default)]
struct Series {
    first: Option<f64>,
    last: f64,
    min: f64,
    max: f64,
}

impl Series {
    fn push(&mut self, v: f64) {
        if self.first.is_none() {
            self.first = Some(v);
            self.min = v;
            self.max = v;
        }
        self.last = v;
        self.min = self.min.min(v);
        self.max = self.max.max(v);
    }

    fn json(&self) -> Value {
        json!({ "first": self.first.unwrap_or(0.0), "last": self.last, "min": self.min, "max": self.max })
    }
}

/// What one AI slot has done so far, tallied from newly logged decision records.
#[derive(Clone, Debug, Default)]
struct AiTally {
    last_seen_tick: Option<u64>,
    decisions: u64,
    orders_logistics: u64,
    orders_deployment: u64,
    orders_build: u64,
    last_goal: String,
}

#[derive(Resource, Debug)]
struct SimRunDumpState {
    last_tick: u64,
    series: BTreeMap<&'static str, Series>,
    territory_first: BTreeMap<u8, u32>,
    prev_ever_burning: u64,
    ignited_cells_cum: u64,
    ai: BTreeMap<u8, AiTally>,
    frame_ms: Vec<f32>,
    frames_seen: u64,
    first_frame_at: Option<std::time::Instant>,
    hitches: u64,
    stalls: u64,
    worst_hitches: Vec<HitchRecord>,
    history: Vec<Value>,
}

impl Default for SimRunDumpState {
    fn default() -> Self {
        Self {
            last_tick: u64::MAX,
            series: BTreeMap::new(),
            territory_first: BTreeMap::new(),
            prev_ever_burning: 0,
            ignited_cells_cum: 0,
            ai: BTreeMap::new(),
            frame_ms: Vec::new(),
            frames_seen: 0,
            first_frame_at: None,
            hitches: 0,
            stalls: 0,
            worst_hitches: Vec::new(),
            history: Vec::new(),
        }
    }
}

impl SimRunDumpState {
    fn track(&mut self, key: &'static str, v: f64) {
        self.series.entry(key).or_default().push(v);
    }

    fn series_json(&self, key: &str) -> Value {
        self.series.get(key).map_or_else(|| Series::default().json(), Series::json)
    }

    /// Frame-time stats after warm-up, hitch counts (threshold from [`FrameHitchLog`]), the worst
    /// hitch frames with their top `PerfScope`s, and a downsampled series for the dashboard.
    fn perf_json(&self, hitch: Option<&FrameHitchLog>) -> Value {
        let build = if cfg!(debug_assertions) { "debug" } else { "release" };
        let threshold = hitch.map_or(0.0, |h| h.threshold_ms);
        if self.frame_ms.is_empty() {
            return json!({ "build": build, "samples": 0, "avg_ms": 0.0, "p50_ms": 0.0, "p95_ms": 0.0,
                "p99_ms": 0.0, "max_ms": 0.0, "hitch_threshold_ms": threshold, "hitches": 0, "stalls": 0,
                "hitches_per_min": 0.0, "worst_hitches": [], "series_avg_ms": [], "series_max_ms": [] });
        }
        let mut s = self.frame_ms.clone();
        s.sort_by(f32::total_cmp);
        let total_ms = s.iter().sum::<f32>();
        let avg = total_ms / s.len() as f32;
        let q = |f: f32| s[((s.len() - 1) as f32 * f) as usize];
        let bin = self.frame_ms.len().div_ceil(PERF_SERIES_POINTS).max(1);
        let (series_avg, series_max): (Vec<f32>, Vec<f32>) = self
            .frame_ms
            .chunks(bin)
            .map(|c| (c.iter().sum::<f32>() / c.len() as f32, c.iter().copied().fold(0.0, f32::max)))
            .unzip();
        let worst: Vec<Value> = self
            .worst_hitches
            .iter()
            .map(|h| {
                json!({
                    "frame": h.frame,
                    "dt_ms": h.dt_ms,
                    "main_ms": h.main_ms,
                    "outside_main_ms": h.outside_main_ms,
                    "tick": h.sim_tick,
                    "scopes": h.scopes.iter().filter(|(l, _)| !l.is_empty())
                        .map(|(l, ms)| json!({ "label": l, "ms": ms })).collect::<Vec<_>>(),
                })
            })
            .collect();
        json!({
            "build": build,
            "samples": s.len(),
            "avg_ms": avg,
            "p50_ms": q(0.50),
            "p95_ms": q(0.95),
            "p99_ms": q(0.99),
            "max_ms": s[s.len() - 1],
            "hitch_threshold_ms": threshold,
            "hitches": self.hitches,
            "stalls": self.stalls,
            "hitches_per_min": self.hitches as f32 / (total_ms / 60_000.0).max(1e-3),
            "worst_hitches": worst,
            "series_avg_ms": series_avg,
            "series_max_ms": series_max,
        })
    }
}

/// Dump writes **only** in debug builds (or release built with the non-default `sim_dump` feature,
/// for `playcheck.py --release`) when `RUST_ENGINE_SIM_DUMP=1`. Shipped release binaries never
/// collect or write a dump. The env var is read once.
#[must_use]
pub fn sim_dump_enabled(_launch: Option<&EngineLaunchArgs>) -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    (cfg!(debug_assertions) || cfg!(feature = "sim_dump"))
        && *ON.get_or_init(|| env_on("RUST_ENGINE_SIM_DUMP"))
}

fn env_on(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}


fn count_water(matrices: &Query<&ChunkCellMatrix>) -> (u32, u32, u32, u32, u32) {
    let mut cells = 0u32;
    let mut open = 0u32;
    let mut river = 0u32;
    let mut standing = 0u32;
    let mut wetland = 0u32;
    for m in matrices {
        for w in &m.water {
            cells = cells.saturating_add(1);
            match *w {
                WaterClass::River => {
                    open = open.saturating_add(1);
                    river = river.saturating_add(1);
                }
                WaterClass::Lake | WaterClass::Ocean => {
                    open = open.saturating_add(1);
                    standing = standing.saturating_add(1);
                }
                WaterClass::Wetland => wetland = wetland.saturating_add(1),
                WaterClass::Dry => {}
            }
        }
    }
    (cells, open, river, standing, wetland)
}

fn count_veg(stands: &Query<&ChunkCellStand>) -> (u32, u32, u32, u32) {
    let mut live = 0u32;
    let mut burning = 0u32;
    let mut burnt = 0u32;
    let mut water = 0u32;
    for stand in stands {
        for c in stand.render.iter() {
            match *c {
                CELL_GRASS | CELL_SHRUB | CELL_YOUNG | CELL_OLD | CELL_REGROWTH => {
                    live = live.saturating_add(1);
                }
                CELL_BURNING => burning = burning.saturating_add(1),
                CELL_BURNT => burnt = burnt.saturating_add(1),
                CELL_WATER => water = water.saturating_add(1),
                _ => {}
            }
        }
    }
    (live, burning, burnt, water)
}



fn write_dump_files(body: &Value) -> bool {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let seed = body.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);
    let tick = body.get("tick").and_then(|v| v.as_u64()).unwrap_or(0);
    let run_id = format!("{epoch}-s{seed}-t{tick}");
    let mut stamped = body.clone();
    if let Some(obj) = stamped.as_object_mut() {
        obj.insert("run_id".into(), json!(run_id));
        obj.insert("written_at_epoch_secs".into(), json!(epoch));
    }
    let Ok(text) = serde_json::to_string_pretty(&stamped) else {
        return false;
    };
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let latest = root.join(SIM_RUN_DUMP_JSON);
    if let Some(dir) = latest.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if fs::write(&latest, &text).is_err() {
        return false;
    }
    // One archive file per run (overwritten each dump), not one per 30-tick write.
    static RUN_START: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    let start = *RUN_START.get_or_init(|| epoch);
    let archive_dir = root.join("debug_runs").join("sim_runs");
    let _ = fs::create_dir_all(&archive_dir);
    let _ = fs::write(archive_dir.join(format!("{start}-s{seed}.json")), text);
    true
}


#[derive(SystemParam)]
struct WorldParams<'w, 's> {
    tick: Res<'w, SimTick>,
    params: Option<Res<'w, WorldGenParams>>,
    launch: Option<Res<'w, EngineLaunchArgs>>,
    base: Option<Res<'w, State<BaseState>>>,
    matrices: Query<'w, 's, &'static ChunkCellMatrix>,
    stands: Query<'w, 's, &'static ChunkCellStand>,
    overlays: Query<'w, 's, (&'static Chunk, &'static ChunkFireOverlay, Option<&'static ChunkWeather>)>,
    ecology: Option<Res<'w, FireEcologyStats>>,
    fire_entities: Option<Res<'w, FireEntityExposureStats>>,
    graph: Option<Res<'w, RenderProjectionGraph>>,
    particles: Option<Res<'w, WorldFireParticleFrame>>,
    hitch: Option<Res<'w, FrameHitchLog>>,
    weather: Option<Res<'w, super::sim_run_dump_weather::WeatherDumpSeries>>,
    fidelity: Option<Res<'w, super::sim_run_dump_fidelity::FidelityDumpSeries>>,
    atmosphere: Option<Res<'w, crate::systems::atmosphere::AtmosphereDiagnostics>>,
    atmosphere_frame: Option<Res<'w, crate::systems::atmosphere::AtmosphereFrame>>,
    presentation: Option<Res<'w, super::sim_run_dump_massing::PresentationDumpSeries>>,
    voxel_occupation: Option<Res<'w, crate::strategic::VoxelOccupationBook>>,
    voxel_occupation_cfg: Option<Res<'w, crate::strategic::VoxelOccupationConfig>>,
}

/// Fire visual chain, stage by stage: sim heat -> snapshot -> active/visible chunks -> per-view
/// frame -> projection graph. Lets a dump say where burning cells stop producing instances.
#[derive(SystemParam)]
struct FireChainParams<'w, 's> {
    index: Option<Res<'w, crate::render::ChunkFireEntityIndex>>,
    residency: Option<Res<'w, crate::io::streaming::ChunkResidencyTable>>,
    fire_chunks: Query<
        'w,
        's,
        Entity,
        (
            With<crate::terrain::generation::Chunk>,
            With<ChunkCellMatrix>,
            With<crate::systems::fire::ChunkSurfaceFire>,
        ),
    >,
    snapshot: Option<Res<'w, crate::render::FireSimulationSnapshot>>,
    extract: Option<Res<'w, crate::render::FireExtractDiagnostics>>,
    active: Option<Res<'w, crate::render::ActiveFireChunkSet>>,
    visible: Option<Res<'w, crate::render::VisibleFireChunkSet>>,
    by_view: Option<Res<'w, crate::render::FireVisualFramesByView>>,
    fence: Option<Res<'w, crate::render::CommittedVisualSnapshotFence>>,
    policy: Option<Res<'w, crate::gui::RepresentationResult>>,
}

fn fire_chain_json(w: &WorldParams, f: &FireChainParams) -> Value {
    use crate::gui::ViewId;
    use crate::render::FIRE_VISUAL_ACTIVE_HEAT_EPS;
    // Cells the extractor should turn into instances: heat above the visual floor with fuel or burn-hold.
    let mut emitting_cells = 0u32;
    for (_chunk, o, _wx) in &w.overlays {
        for (i, h) in o.heat.iter().enumerate() {
            let fuel = o.fuel.get(i).copied().unwrap_or(0.0);
            let hold = o.burn_hold.get(i).copied().unwrap_or(0);
            if *h >= FIRE_VISUAL_ACTIVE_HEAT_EPS && (fuel > 0.0 || hold > 0) {
                emitting_cells += 1;
            }
        }
    }
    let view_rows = |id: ViewId| f.by_view.as_ref().and_then(|b| b.by_id.get(&id)).map_or(0, |fr| fr.instances.len());
    let visible = |id: ViewId| f.visible.as_ref().and_then(|v| v.per_view.get(&id)).map_or(-1, |s| s.len() as i64);
    let r = f.extract.as_ref().map(|d| d.last.clone()).unwrap_or_default();
    // Index entries that resolve to an entity the extract query can read.
    let index_resolving = f.index.as_ref().map_or(0, |ix| {
        ix.by_coord.values().filter(|e| f.fire_chunks.contains(**e)).count()
    });
    let residency_bounds = f.residency.as_ref().and_then(|t| {
        let mut it = t.entries.keys();
        let first = *it.next()?;
        Some(t.entries.keys().fold((first, first), |(lo, hi), c| (lo.min(*c), hi.max(*c))))
    });
    json!({
        "index_len": f.index.as_ref().map_or(0, |ix| ix.by_coord.len()),
        "index_resolving": index_resolving,
        "fire_chunk_entities": f.fire_chunks.iter().count(),
        "residency_bounds": residency_bounds.map(|(lo, hi)| [lo.x, lo.y, hi.x, hi.y]),
        "emitting_cells": emitting_cells,
        "snapshot_instances": f.snapshot.as_ref().map_or(0, |s| s.instances.len()),
        "snapshot_chunk_heat": f.snapshot.as_ref().map_or(0, |s| s.chunk_heat.len()),
        "snapshot_tick": f.snapshot.as_ref().map_or(0, |s| s.stamp.tick),
        "fence_tick": f.fence.as_ref().map_or(0, |s| s.fire.tick),
        "graph_snapshot_tick": w.graph.as_ref().map_or(0, |g| g.fire.snapshot_stamp),
        "active_chunks": f.active.as_ref().map_or(0, |a| a.chunks.len()),
        "visible_world_main": visible(ViewId::WorldMain),
        "visible_sim_map": visible(ViewId::SimulationMap),
        "view_rows_world_main": view_rows(ViewId::WorldMain),
        "view_rows_sim_map": view_rows(ViewId::SimulationMap),
        "graph_rows": w.graph.as_ref().map_or(0, |g| g.fire.instance_buffer.len()),
        "policy_fire_instances": f.policy.as_ref().is_some_and(|p| p.extract_plan.fire_instances),
        "policy_fire_cap": f.policy.as_ref().map_or(0, |p| p.extract_plan.fire_instance_cap.min(1 << 30)),
        "extract": {
            "ran_full_scan": r.ran_full_scan,
            "full_reconcile": r.full_reconcile,
            "cadence_skipped": r.cadence_skipped,
            "fingerprint_skipped": r.fingerprint_skipped,
            "scan_set_len": r.scan_set_len,
            "residency_len": r.residency_len,
            "chunks_iterated": r.chunks_iterated,
            "chunks_profiled": r.chunks_profiled,
            "instances_written": r.instances_written,
            "extract_ms": r.extract_ms,
        },
    })
}

#[derive(SystemParam)]
struct GameParams<'w, 's> {
    sites: Query<'w, 's, &'static ConstructionSite>,
    hqs: Query<'w, 's, &'static FactionHq>,
    start: Option<Res<'w, FactionStartConfig>>,
    roster: Option<Res<'w, FactionAiRoster>>,
    plants: Query<
        'w,
        's,
        (
            &'static ResourceFlowNode,
            &'static FacilityFlowState,
            Option<&'static crate::economy::facility_inputs::FacilityInputs>,
        ),
    >,
    flow_stats: Option<Res<'w, ResourceFlowSimStats>>,
    logistics: Option<Res<'w, LogisticsOrdersSnapshotRes>>,
    logistics_sim: Option<Res<'w, LogisticsOrderSimRes>>,
    fire_entities: Option<Res<'w, FireEntityExposureStats>>,
    ai_log: Option<Res<'w, FactionAiDecisionLog>>,
    frontline: Option<Res<'w, FrontlineState>>,
    theaters: Option<Res<'w, TheaterBook>>,
    formations: Query<'w, 's, &'static Formation>,
    outcomes: Option<Res<'w, DeploymentOutcomes>>,
    strategic: Query<'w, 's, &'static ChunkStrategicOverlay>,
}

fn construction_json(g: &GameParams) -> (Value, u32) {
    let mut by_phase: BTreeMap<String, u32> = BTreeMap::new();
    let mut operational = 0u32;
    let mut total = 0u32;
    for site in &g.sites {
        total += 1;
        *by_phase.entry(format!("{:?}", site.phase)).or_default() += 1;
        if site.phase == SiteConstructionPhase::Operational {
            operational += 1;
        }
    }
    let hq_slots: Vec<u8> = g.hqs.iter().map(|h| h.slot).collect();
    let configured = g.start.as_ref().map_or(0, |c| c.factions.len() as u32);
    (
        json!({
            "sites_total": total,
            "sites_operational": operational,
            "sites_by_phase": by_phase,
            "hq_present": hq_slots.len(),
            "hq_configured": configured,
            "hq_slots": hq_slots,
        }),
        operational,
    )
}

fn economy_json(g: &GameParams) -> (Value, f64) {
    let (mut operational, mut idle, mut starved) = (0u32, 0u32, 0u32);
    let (mut buf_units, mut buf_cap) = (0.0f64, 0.0f64);
    let mut rows: Vec<Value> = Vec::new();
    for (node, state, inputs) in &g.plants {
        // Workforce / power cap output: a plant with neither makes nothing, so it is not "operational".
        let supply = inputs.map_or(1.0, |i| i.factor());
        rows.push(json!({
            "catalog_id": node.catalog_id,
            "produces": node.production.iter().map(|r| r.tag.clone()).collect::<Vec<_>>(),
            "starved": state.starved,
            "output_full": state.output_full,
            "output_scale": state.output_scale,
            "workforce": inputs.map(|i| i.workforce),
            "power": inputs.map(|i| i.power),
            "power_state": inputs.map(|i| format!("{:?}", i.power_state)),
        }));
        if state.starved {
            starved += 1;
        } else if state.output_full || supply <= 0.0 {
            idle += 1;
        } else {
            operational += 1;
        }
        for rate in &node.production {
            let have = match rate.resource {
                Some(rt) => node.inventory.get(&rt).copied().unwrap_or(0.0),
                None => node.buffer_by_tag.get(&rate.tag).copied().unwrap_or(0.0),
            };
            buf_units += f64::from(have);
            buf_cap += f64::from(OUTPUT_BUFFER_CAP);
        }
    }
    let plants_total = operational + idle + starved;
    let produced: BTreeMap<String, f64> = g
        .flow_stats
        .as_ref()
        .map(|s| s.produced_by_tag.iter().map(|(k, v)| (k.clone(), f64::from(*v))).collect())
        .unwrap_or_default();
    let produced_total: f64 = produced.values().sum();
    let depot_stock_units: u64 = g.logistics.as_ref().map_or(0, |l| {
        l.0.sites
            .iter()
            .flat_map(|s| s.stock.iter())
            .map(|r| u64::from(r.on_hand))
            .sum()
    });
    (
        json!({
            "plants_total": plants_total,
            "plants_operational": operational,
            "plants_idle": idle,
            "plants_starved": starved,
            "produced_total": produced_total,
            "produced_by_tag": produced,
            "storage_units": buf_units,
            "storage_capacity": buf_cap,
            "storage_fill": if buf_cap > 0.0 { buf_units / buf_cap } else { 0.0 },
            "depot_stock_units": depot_stock_units,
            "plants": rows,
        }),
        produced_total,
    )
}

fn logistics_json(g: &GameParams) -> (Value, u64) {
    let Some(snap) = g.logistics.as_ref().map(|l| &l.0) else {
        return (json!({ "available": false }), 0);
    };
    let standing = snap.standing_lines().count();
    let one_off = snap.one_off_runs().count();
    let one_off_completed = snap
        .one_off_runs()
        .filter(|o| matches!(o.status, OrderStatus::Completed))
        .count();
    let trips_done: u64 = snap.orders.iter().map(|o| u64::from(o.trips_done)).sum();
    let delivered_units: u64 = snap.orders.iter().map(|o| u64::from(o.delivered)).sum();
    let draw_total = snap.demands.iter().filter(|d| d.draw_rate_per_min > 0).count();
    let unserved = snap
        .demands
        .iter()
        .filter(|d| d.draw_rate_per_min > 0 && d.served_by == 0)
        .count();
    let starving = snap
        .demands
        .iter()
        .filter(|d| d.starved_ticks > 0 || d.on_hand == 0 && d.draw_rate_per_min > 0)
        .count();
    let vehicles_busy = snap.fleets.iter().map(|f| u64::from(f.busy)).sum::<u64>();
    let vehicles_total = snap.fleets.iter().map(|f| u64::from(f.total)).sum::<u64>();
    let (fire_delayed, vehicles_lost, cargo_lost) = g.logistics_sim.as_ref().map_or((0, 0, 0), |s| {
        (
            s.sim.fire_delayed_trips,
            s.sim.vehicles_lost_to_fire,
            s.sim.cargo_lost_to_fire,
        )
    });
    (
        json!({
            "available": true,
            "sites": snap.sites.len(),
            "orders_total": snap.orders.len(),
            "standing_lines": standing,
            "one_off_orders": one_off,
            "one_off_completed": one_off_completed,
            "deliveries_completed": trips_done,
            "delivered_units": delivered_units,
            "draw_demands": draw_total,
            "unserved_draw_demands": unserved,
            "draw_demand_rows": snap
                .demands
                .iter()
                .filter(|d| d.draw_rate_per_min > 0)
                .map(|d| format!(
                    "owner {} {} site {} {} draw {}/min on_hand {} served_by {}",
                    d.owner.0, d.consumer.label(), d.site.0, d.cargo, d.draw_rate_per_min, d.on_hand, d.served_by
                ))
                .collect::<Vec<_>>(),
            "starving_demands": starving,
            "vehicles_total": vehicles_total,
            "vehicles_busy": vehicles_busy,
            "rejected_requests": snap.rejected.len(),
            "fire_delayed_trips": fire_delayed,
            "vehicles_lost_to_fire": vehicles_lost,
            "cargo_lost_to_fire": cargo_lost,
        }),
        trips_done,
    )
}

fn ai_json(g: &GameParams, state: &mut SimRunDumpState) -> Value {
    if let Some(log) = g.ai_log.as_ref() {
        for rec in &log.records {
            let t = state.ai.entry(rec.slot).or_default();
            if t.last_seen_tick.is_some_and(|seen| rec.sim_tick <= seen) {
                continue;
            }
            t.last_seen_tick = Some(rec.sim_tick);
            t.decisions += 1;
            t.last_goal = rec.goal.label().to_string();
            for (order, _) in &rec.orders {
                match order {
                    AiOrder::StandingLine { .. } | AiOrder::SupplyRun { .. } | AiOrder::SupplyLine { .. } => {
                        t.orders_logistics += 1;
                    }
                    AiOrder::Deploy { .. } | AiOrder::Reinforce { .. } | AiOrder::Attack { .. } => {
                        t.orders_deployment += 1;
                    }
                    AiOrder::Build { .. } | AiOrder::Develop { .. } | AiOrder::FoundDepot { .. } => {
                        t.orders_build += 1;
                    }
                }
            }
        }
    }
    let roster: Vec<u8> = g.roster.as_ref().map(|r| r.slots.iter().map(|(s, _)| *s).collect()).unwrap_or_default();
    let mut factions = Map::new();
    let mut slots: Vec<u8> = roster.clone();
    for s in state.ai.keys() {
        if !slots.contains(s) {
            slots.push(*s);
        }
    }
    slots.sort_unstable();
    for slot in &slots {
        let t = state.ai.get(slot).cloned().unwrap_or_default();
        factions.insert(
            slot.to_string(),
            json!({
                "decisions": t.decisions,
                "orders_logistics": t.orders_logistics,
                "orders_deployment": t.orders_deployment,
                "orders_build": t.orders_build,
                "orders_total": t.orders_logistics + t.orders_deployment + t.orders_build,
                "last_goal": t.last_goal,
            }),
        );
    }
    json!({
        "roster_slots": roster,
        "total_decisions": g.ai_log.as_ref().map_or(0, |l| l.total_decisions),
        "goal_switches": g.ai_log.as_ref().map_or(0, |l| l.total_goal_switches),
        "factions": Value::Object(factions),
    })
}

fn combat_json(g: &GameParams, state: &mut SimRunDumpState) -> Value {
    let engaged = g.formations.iter().filter(|f| f.engaged).count();
    state.track("engaged_formations", engaged as f64);
    let mut held: BTreeMap<u8, u32> = BTreeMap::new();
    for ov in &g.strategic {
        for control in &ov.faction_control {
            let (slot, best) = control
                .iter()
                .enumerate()
                .fold((0usize, 0.0f32), |a, (i, v)| if *v > a.1 { (i, *v) } else { a });
            if best > TERRITORY_CONTROL_MIN {
                *held.entry(slot as u8).or_default() += 1;
            }
        }
    }
    if state.territory_first.is_empty() && !held.is_empty() {
        state.territory_first = held.clone();
    }
    let mut changed = 0i64;
    let mut slots: Vec<u8> = held.keys().chain(state.territory_first.keys()).copied().collect();
    slots.sort_unstable();
    slots.dedup();
    for s in &slots {
        let now = i64::from(held.get(s).copied().unwrap_or(0));
        let first = i64::from(state.territory_first.get(s).copied().unwrap_or(0));
        changed += (now - first).abs();
    }
    let (ok, rejected) = g.outcomes.as_ref().map_or((0, 0), |o| {
        let ok = o.recent.iter().filter(|r| r.result.is_ok()).count();
        (ok, o.recent.len() - ok)
    });
    let fire_exposed = g.fire_entities.as_ref().map_or(0u32, |e| e.formations_exposed);
    let fire_heat_mean = g.fire_entities.as_ref().map_or(0.0f32, |e| {
        if e.formations_exposed == 0 {
            0.0
        } else {
            e.formation_heat_sum / e.formations_exposed as f32
        }
    });
    state.track("fire_exposed_formations", f64::from(fire_exposed));
    json!({
        "front_segments": g.frontline.as_ref().map_or(0, |f| f.segments.len()),
        "front_cells": g.frontline.as_ref().map_or(0, |f| f.segments.iter().map(|s| s.cells.len()).sum::<usize>()),
        "contested_cells": g.frontline.as_ref().map_or(0, |f| f.segments.iter().map(|s| s.contested_cells as usize).sum::<usize>()),
        "theaters": g.theaters.as_ref().map_or(0, |t| t.theaters.len()),
        "formations": g.formations.iter().count(),
        "engaged_formations": engaged,
        "engaged_formations_series": state.series_json("engaged_formations"),
        "fire_exposed_formations": fire_exposed,
        "fire_exposed_formations_series": state.series_json("fire_exposed_formations"),
        "fire_exposed_mean_heat": fire_heat_mean,
        "deployment_orders_resolved_ok": ok,
        "deployment_orders_resolved_rejected": rejected,
        "territory_cells_by_slot": held,
        "territory_cells_first_by_slot": state.territory_first,
        "territory_change_cells": changed,
    })
}

/// Axis-aligned heat footprint vs mean wind: downwind span / upwind span (W2 anisotropy).
fn fire_wind_spread_ratio(w: &WorldParams) -> (f64, f64, f64) {
    let mut wind = Vec2::ZERO;
    let mut wn = 0u32;
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    let mut hot = 0u32;
    for (chunk, ovl, wx) in &w.overlays {
        if let Some(wx) = wx {
            wind += wx.wind_mps;
            wn += 1;
        }
        let sx = (ovl.heat.len() as f32).sqrt() as i32;
        if sx <= 0 {
            continue;
        }
        for (i, h) in ovl.heat.iter().enumerate() {
            if *h < 0.08 {
                continue;
            }
            hot = hot.saturating_add(1);
            let lx = (i as i32) % sx;
            let ly = (i as i32) / sx;
            let x = chunk.coord.x.wrapping_mul(sx).wrapping_add(lx);
            let y = chunk.coord.y.wrapping_mul(sx).wrapping_add(ly);
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }
    if hot == 0 || wn == 0 {
        return (0.0, 0.0, 0.0);
    }
    let wind = wind / wn as f32;
    let speed = wind.length();
    let (dx, dy) = ((max_x - min_x).max(0) as f64, (max_y - min_y).max(0) as f64);
    let along = if speed < 0.5 {
        1.0
    } else if wind.x.abs() >= wind.y.abs() {
        (dx + 1.0) / (dy + 1.0)
    } else {
        (dy + 1.0) / (dx + 1.0)
    };
    (along, speed as f64, f64::from(hot))
}

fn fire_json(w: &WorldParams, veg: (u32, u32, u32, u32), regrowth: u32, heat_cells: u32, state: &mut SimRunDumpState) -> Value {
    let ever = u64::from(veg.1) + u64::from(veg.2);
    state.ignited_cells_cum += ever.saturating_sub(state.prev_ever_burning);
    state.prev_ever_burning = ever;
    state.track("burning_cells", f64::from(veg.1));
    state.track("burnt_cells", f64::from(veg.2));
    state.track("regrowth_cells", f64::from(regrowth));
    state.track("heat_cells", f64::from(heat_cells));
    state.track("veg_live_cells", f64::from(veg.0));
    let burnt_frac = {
        let land = veg.0.saturating_add(veg.1).saturating_add(veg.2).max(1);
        f64::from(veg.2) / f64::from(land)
    };
    state.track("burnt_fraction", burnt_frac);
    let (lb_ratio, mean_wind, hot_n) = fire_wind_spread_ratio(w);
    state.track("fire_length_breadth", lb_ratio);
    state.track("fire_mean_wind_mps", mean_wind);
    let ember_emitted = w.ecology.as_ref().map_or(0u64, |e| e.ember_spots_emitted);
    let ember_applied = w.ecology.as_ref().map_or(0u64, |e| e.ember_spots_applied);
    state.track("ember_spots_emitted", ember_emitted as f64);
    state.track("ember_spots_applied", ember_applied as f64);
    let collapsed = w
        .fire_entities
        .as_ref()
        .map_or(0u64, |e| e.sites_collapsed_to_fire);
    let threatened = w.fire_entities.as_ref().map_or(0u32, |e| e.sites_threatened);
    let disrupted = w.fire_entities.as_ref().map_or(0u32, |e| e.sites_disrupted);
    let deteriorating = w
        .fire_entities
        .as_ref()
        .map_or(0u32, |e| e.sites_deteriorating);
    state.track("sites_collapsed_to_fire", collapsed as f64);
    state.track("sites_threatened", f64::from(threatened));
    state.track("sites_disrupted", f64::from(disrupted));
    state.track("sites_deteriorating", f64::from(deteriorating));
    json!({
        "ignited_cells_cum": state.ignited_cells_cum,
        "burning_cells": state.series_json("burning_cells"),
        "burnt_cells": state.series_json("burnt_cells"),
        "regrowth_cells": state.series_json("regrowth_cells"),
        "heat_cells": state.series_json("heat_cells"),
        "veg_live_cells": state.series_json("veg_live_cells"),
        "burnt_fraction": state.series_json("burnt_fraction"),
        "length_breadth": state.series_json("fire_length_breadth"),
        "mean_wind_mps": state.series_json("fire_mean_wind_mps"),
        "ember_spots_emitted": state.series_json("ember_spots_emitted"),
        "ember_spots_applied": state.series_json("ember_spots_applied"),
        "sites_collapsed_to_fire": state.series_json("sites_collapsed_to_fire"),
        "sites_threatened": state.series_json("sites_threatened"),
        "sites_disrupted": state.series_json("sites_disrupted"),
        "sites_deteriorating": state.series_json("sites_deteriorating"),
        "hot_cells_bbox": hot_n,
        "max_heat": w.ecology.as_ref().map_or(0.0, |e| e.max_heat),
    })
}

fn push_history(state: &mut SimRunDumpState, entry: Value) {
    state.history.push(entry);
    if state.history.len() > HISTORY_CAP {
        // Thin to every other sample so a long run keeps its whole shape.
        let mut i = 0usize;
        state.history.retain(|_| {
            i += 1;
            i % 2 == 1
        });
    }
}

fn collect_dump(
    w: &WorldParams,
    g: &GameParams,
    q: &super::sim_run_dump_quadrants::QuadrantDumpParams,
    f: &FireChainParams,
    state: &mut SimRunDumpState,
) -> Value {
    let water = count_water(&w.matrices);
    let veg = count_veg(&w.stands);
    let regrowth = w
        .stands
        .iter()
        .map(|s| s.render.iter().filter(|c| **c == CELL_REGROWTH).count() as u32)
        .sum::<u32>();
    let mut fire_heat_cells = 0u32;
    for (_chunk, overlay, _wx) in &w.overlays {
        for h in &overlay.heat {
            if *h > 0.04 {
                fire_heat_cells = fire_heat_cells.saturating_add(1);
            }
        }
    }
    let (construction, sites_operational) = construction_json(g);
    let (economy, produced_total) = economy_json(g);
    let (logistics, deliveries) = logistics_json(g);
    let ai = ai_json(g, state);
    let combat = combat_json(g, state);
    let fire = fire_json(w, veg, regrowth, fire_heat_cells, state);

    let tick = w.tick.0;
    let cells = water.0.max(1);
    let world_w = w.params.as_ref().map_or(0, |p| p.width);
    let world_h = w.params.as_ref().map_or(0, |p| p.height);
    let mpt = w.params.as_ref().map_or(100.0, |p| p.map_scale.meters_per_tile);
    let test_scene = w.launch.as_ref().map(|l| l.test_scene).unwrap_or_default();
    let quadrants = super::sim_run_dump_quadrants::quadrants_json(test_scene, world_w, world_h, mpt, q);
    let mut weather = super::sim_run_dump_weather::weather_json(w.weather.as_deref());
    if let Some(inst) = q.precip.as_ref().map(|p| p.instance_count) {
        if let Some(obj) = weather.as_object_mut() {
            obj.insert("precip_instances".into(), json!(inst));
        }
    }
    let river_frac = water.2 as f32 / cells as f32;
    let scene = w
        .launch
        .as_ref()
        .map(|l| format!("{:?}", l.test_scene))
        .unwrap_or_else(|| "None".into());
    let presentation_kind = w
        .presentation
        .as_ref()
        .map(|p| p.kind.clone())
        .unwrap_or_else(|| "Unknown".into());
    let spark_view = w.particles.as_ref().map_or("", |p| p.spark_witness.projection_view);
    let frame_ms_last = state.frame_ms.last().copied().unwrap_or(0.0);
    push_history(
        state,
        json!({
            "tick": tick,
            "burning": veg.1,
            "burnt": veg.2,
            "regrowth": regrowth,
            "heat_cells": fire_heat_cells,
            "fire_inst": w.graph.as_ref().map_or(0, |g| g.fire.instance_buffer.len()),
            "sites_operational": sites_operational,
            "deliveries": deliveries,
            "produced": produced_total,
            "engaged": g.formations.iter().filter(|f| f.engaged).count(),
            "frame_ms": frame_ms_last,
        }),
    );
    let mut body = json!({
        "schema": SIM_RUN_DUMP_SCHEMA,
        "proof_grade": "sim_dump",
        "tick": tick,
        "seed": w.params.as_ref().map_or(0, |p| p.seed),
        "debug": cfg!(debug_assertions),
        "build": if cfg!(debug_assertions) { "debug" } else { "release" },
        "launch": {
            "test_scene": scene,
            "presentation": presentation_kind,
        },
        "world": {
            "width": w.params.as_ref().map_or(0, |p| p.width),
            "height": w.params.as_ref().map_or(0, |p| p.height),
            "cells": cells,
        },
        "flags": {
            "world_generated": cells > 1,
            "simulation_entered": w.base.as_ref().is_some_and(|s| matches!(*s.get(), BaseState::Simulation)),
        },
        "counts": {
            "terrain_cells": water.0,
            "water_open_cells": water.1,
            "river_cells": water.2,
            "lake_or_ocean_cells": water.3,
            "wetland_cells": water.4,
            "veg_live_cells": veg.0,
            "veg_burning_cells": veg.1,
            "veg_burnt_cells": veg.2,
            "veg_water_cells": veg.3,
            "fire_heat_cells": fire_heat_cells,
            "fire_instance_rows": w.graph.as_ref().map_or(0, |g| g.fire.instance_buffer.len() as u32),
            "spark_rows": w.particles.as_ref().map_or(0, |p| p.spark_witness.rows as u32),
        },
        "frac": { "river": river_frac },
        "max": {
            "fire_heat": w.ecology.as_ref().map_or(0.0, |e| e.max_heat),
            "mean_heat": w.ecology.as_ref().map_or(0.0, |e| e.mean_heat),
            "mean_fuel": w.ecology.as_ref().map_or(0.0, |e| e.mean_fuel),
        },
        "spark_projection_view": spark_view,
        "construction": construction,
        "economy": economy,
        "logistics": logistics,
        "ai": ai,
        "combat": combat,
        "fire": fire,
        "fire_chain": fire_chain_json(w, f),
        "weather": weather,
        "fidelity": super::sim_run_dump_fidelity::fidelity_json(w.fidelity.as_deref()),
        "atmosphere": super::sim_run_dump_atmosphere::atmosphere_json(
            w.atmosphere.as_deref(),
            w.atmosphere_frame.as_deref(),
        ),
        "presentation": super::sim_run_dump_massing::presentation_json(w.presentation.as_deref()),
        "occupation": super::sim_run_dump_occupation::occupation_json(
            w.voxel_occupation.as_deref(),
            w.voxel_occupation_cfg.as_deref(),
        ),
        "perf": state.perf_json(w.hitch.as_deref()),
        "history": state.history,
    });
    if let Some(qblock) = quadrants {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("quadrants".into(), qblock);
        }
    }
    body
}

/// Frame wall time from the hitch detector (end of `Last` to end of `Last`, uncapped); counted after
/// warm-up. Sampled every frame, not per dump. Hitch frames after warm-up are counted and the
/// worst ones kept with their top `PerfScope`s.
fn sample_frame_time(hitch: Option<Res<FrameHitchLog>>, mut state: ResMut<SimRunDumpState>) {
    if !sim_dump_enabled(None) {
        return;
    }
    let Some(hitch) = hitch else {
        return;
    };
    state.frames_seen += 1;
    let first = *state.first_frame_at.get_or_insert_with(std::time::Instant::now);
    if state.frames_seen <= PERF_WARMUP_FRAMES || first.elapsed().as_secs_f32() < PERF_WARMUP_SECS {
        return;
    }
    if state.frame_ms.len() < PERF_SAMPLE_CAP {
        state.frame_ms.push(hitch.last_frame_ms);
    }
    let Some(rec) = hitch.this_frame else {
        return;
    };
    state.hitches += 1;
    if rec.dt_ms >= STALL_MS {
        state.stalls += 1;
    }
    let worst = &mut state.worst_hitches;
    if worst.len() < PERF_WORST_HITCHES {
        worst.push(rec);
    } else if let Some(min) = worst.iter_mut().min_by(|a, b| a.dt_ms.total_cmp(&b.dt_ms)) {
        if rec.dt_ms > min.dt_ms {
            *min = rec;
        }
    }
    worst.sort_by(|a, b| b.dt_ms.total_cmp(&a.dt_ms));
}

fn flush_sim_run_dump(
    w: WorldParams,
    g: GameParams,
    q: super::sim_run_dump_quadrants::QuadrantDumpParams,
    f: FireChainParams,
    mut state: ResMut<SimRunDumpState>,
    exit: MessageReader<AppExit>,
) {
    if !sim_dump_enabled(w.launch.as_deref()) {
        return;
    }
    let tick = w.tick.0;
    let exiting = !exit.is_empty();
    let due = exiting
        || (tick >= 8
            && (state.last_tick == u64::MAX || tick.saturating_sub(state.last_tick) >= DUMP_STRIDE_TICKS));
    if !due {
        return;
    }
    let body = collect_dump(&w, &g, &q, &f, &mut state);
    if write_dump_files(&body) {
        state.last_tick = tick;
    }
}

pub struct SimRunDumpPlugin;

impl Plugin for SimRunDumpPlugin {
    fn build(&self, app: &mut App) {
        if !cfg!(debug_assertions) && !cfg!(feature = "sim_dump") {
            return;
        }
        app.add_plugins(super::sim_run_dump_weather::WeatherDumpPlugin);
        app.add_plugins(super::sim_run_dump_fidelity::FidelityDumpPlugin);
        app.add_plugins(super::sim_run_dump_massing::PresentationDumpPlugin);
        app.init_resource::<SimRunDumpState>().add_systems(
            Last,
            (
                sample_frame_time
                    .after(crate::render::probes::frame_hitch::detect_frame_hitches),
                flush_sim_run_dump.after(sample_frame_time),
            ),
        );
    }
}
