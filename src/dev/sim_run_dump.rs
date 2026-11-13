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
use crate::economy::logistics::orders::{LogisticsOrdersSnapshotRes, OrderStatus};
use crate::economy::resource_flow::{
    FacilityFlowState, ResourceFlowNode, ResourceFlowSimStats, OUTPUT_BUFFER_CAP,
};
use crate::engine::states::BaseState;
use crate::engine::EngineLaunchArgs;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::WorldFireParticleFrame;
use crate::strategic::{
    ChunkStrategicOverlay, ConstructionSite, FrontlineState, SiteConstructionPhase,
};
use crate::systems::ecology::{
    ChunkCellStand, CELL_BURNING, CELL_BURNT, CELL_GRASS, CELL_OLD, CELL_REGROWTH, CELL_SHRUB,
    CELL_WATER, CELL_YOUNG,
};
use crate::systems::fire::{ChunkFireOverlay, FireEcologyStats};
use crate::systems::sim_control::SimTick;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{ChunkCellMatrix, WaterClass};

pub const SIM_RUN_DUMP_JSON: &str = "debug_runs/sim_run_dump.json";
pub const SIM_RUN_DUMP_SCHEMA: &str = "sim_run_dump_v2";

const DUMP_STRIDE_TICKS: u64 = 30;
/// Frames skipped before frame-time sampling starts (world gen / shader warm-up).
const PERF_WARMUP_FRAMES: u64 = 120;
const PERF_SAMPLE_CAP: usize = 50_000;
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

    fn perf_json(&self) -> Value {
        if self.frame_ms.is_empty() {
            return json!({ "samples": 0, "avg_ms": 0.0, "p95_ms": 0.0, "max_ms": 0.0 });
        }
        let mut s = self.frame_ms.clone();
        s.sort_by(f32::total_cmp);
        let avg = s.iter().sum::<f32>() / s.len() as f32;
        let p95 = s[((s.len() - 1) as f32 * 0.95) as usize];
        json!({ "samples": s.len(), "avg_ms": avg, "p95_ms": p95, "max_ms": s[s.len() - 1] })
    }
}

/// Dump writes **only** in debug builds when `RUST_ENGINE_SIM_DUMP=1`.
/// Release / production binaries never collect or write a dump.
#[must_use]
pub fn sim_dump_enabled(_launch: Option<&EngineLaunchArgs>) -> bool {
    cfg!(debug_assertions) && env_on("RUST_ENGINE_SIM_DUMP")
}

fn env_on(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}


fn count_water(matrices: &Query<&ChunkCellMatrix>) -> (u32, u32, u32, u32) {
    let mut cells = 0u32;
    let mut open = 0u32;
    let mut river = 0u32;
    let mut standing = 0u32;
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
                WaterClass::Dry | WaterClass::Wetland => {}
            }
        }
    }
    (cells, open, river, standing)
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
    let archive_dir = root.join("debug_runs").join("sim_runs");
    let _ = fs::create_dir_all(&archive_dir);
    let _ = fs::write(archive_dir.join(format!("{run_id}.json")), text);
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
    overlays: Query<'w, 's, &'static ChunkFireOverlay>,
    ecology: Option<Res<'w, FireEcologyStats>>,
    graph: Option<Res<'w, RenderProjectionGraph>>,
    particles: Option<Res<'w, WorldFireParticleFrame>>,
}

#[derive(SystemParam)]
struct GameParams<'w, 's> {
    sites: Query<'w, 's, &'static ConstructionSite>,
    hqs: Query<'w, 's, &'static FactionHq>,
    start: Option<Res<'w, FactionStartConfig>>,
    roster: Option<Res<'w, FactionAiRoster>>,
    plants: Query<'w, 's, (&'static ResourceFlowNode, &'static FacilityFlowState)>,
    flow_stats: Option<Res<'w, ResourceFlowSimStats>>,
    logistics: Option<Res<'w, LogisticsOrdersSnapshotRes>>,
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
    for (node, state) in &g.plants {
        if state.starved {
            starved += 1;
        } else if state.output_full {
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
            "starving_demands": starving,
            "vehicles_total": vehicles_total,
            "vehicles_busy": vehicles_busy,
            "rejected_requests": snap.rejected.len(),
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
    json!({
        "front_segments": g.frontline.as_ref().map_or(0, |f| f.segments.len()),
        "front_cells": g.frontline.as_ref().map_or(0, |f| f.segments.iter().map(|s| s.cells.len()).sum::<usize>()),
        "contested_cells": g.frontline.as_ref().map_or(0, |f| f.segments.iter().map(|s| s.contested_cells as usize).sum::<usize>()),
        "theaters": g.theaters.as_ref().map_or(0, |t| t.theaters.len()),
        "formations": g.formations.iter().count(),
        "engaged_formations": engaged,
        "engaged_formations_series": state.series_json("engaged_formations"),
        "deployment_orders_resolved_ok": ok,
        "deployment_orders_resolved_rejected": rejected,
        "territory_cells_by_slot": held,
        "territory_cells_first_by_slot": state.territory_first,
        "territory_change_cells": changed,
    })
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
    json!({
        "ignited_cells_cum": state.ignited_cells_cum,
        "burning_cells": state.series_json("burning_cells"),
        "burnt_cells": state.series_json("burnt_cells"),
        "regrowth_cells": state.series_json("regrowth_cells"),
        "heat_cells": state.series_json("heat_cells"),
        "veg_live_cells": state.series_json("veg_live_cells"),
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

fn collect_dump(w: &WorldParams, g: &GameParams, state: &mut SimRunDumpState) -> Value {
    let water = count_water(&w.matrices);
    let veg = count_veg(&w.stands);
    let regrowth = w
        .stands
        .iter()
        .map(|s| s.render.iter().filter(|c| **c == CELL_REGROWTH).count() as u32)
        .sum::<u32>();
    let mut fire_heat_cells = 0u32;
    for overlay in &w.overlays {
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
    let river_frac = water.2 as f32 / cells as f32;
    let scene = w
        .launch
        .as_ref()
        .map(|l| format!("{:?}", l.test_scene))
        .unwrap_or_else(|| "None".into());
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
            "sites_operational": sites_operational,
            "deliveries": deliveries,
            "produced": produced_total,
            "engaged": g.formations.iter().filter(|f| f.engaged).count(),
            "frame_ms": frame_ms_last,
        }),
    );
    json!({
        "schema": SIM_RUN_DUMP_SCHEMA,
        "proof_grade": "sim_dump",
        "tick": tick,
        "seed": w.params.as_ref().map_or(0, |p| p.seed),
        "debug": cfg!(debug_assertions),
        "launch": { "test_scene": scene },
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
        "perf": state.perf_json(),
        "history": state.history,
    })
}

/// Frame wall time from the real clock; counted after warm-up. Sampled every frame, not per dump.
fn sample_frame_time(time: Res<Time<Real>>, mut state: ResMut<SimRunDumpState>) {
    if !sim_dump_enabled(None) {
        return;
    }
    state.frames_seen += 1;
    if state.frames_seen <= PERF_WARMUP_FRAMES || state.frame_ms.len() >= PERF_SAMPLE_CAP {
        return;
    }
    state.frame_ms.push(time.delta_secs() * 1000.0);
}

fn flush_sim_run_dump(
    w: WorldParams,
    g: GameParams,
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
    let body = collect_dump(&w, &g, &mut state);
    if write_dump_files(&body) {
        state.last_tick = tick;
    }
}

pub struct SimRunDumpPlugin;

impl Plugin for SimRunDumpPlugin {
    fn build(&self, app: &mut App) {
        if !cfg!(debug_assertions) {
            return;
        }
        app.init_resource::<SimRunDumpState>()
            .add_systems(Last, (sample_frame_time, flush_sim_run_dump).chain());
    }
}
