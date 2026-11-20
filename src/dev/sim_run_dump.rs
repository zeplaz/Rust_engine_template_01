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
use bevy::prelude::*;
use serde_json::{json, Value};

use crate::engine::EngineLaunchArgs;
use crate::render::probes::frame_hitch::STALL_MS;
use crate::render::{FrameHitchLog, HitchRecord};
use crate::systems::sim_control::SimTick;

pub const SIM_RUN_DUMP_JSON: &str = "debug_runs/sim_run_dump.json";
pub const SIM_RUN_DUMP_SCHEMA: &str = "sim_run_dump_v2";

/// Per-domain dump sections (CQ-68). Reporters write keys; the JSON writer reads only this.
#[derive(Resource, Debug, Default)]
pub struct SimRunDump {
    pub sections: BTreeMap<&'static str, Value>,
}

impl SimRunDump {
    pub fn put_value(&mut self, key: &'static str, value: Value) {
        self.sections.insert(key, value);
    }

    pub fn take(&mut self, key: &str) -> Option<Value> {
        self.sections.remove(key)
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.sections.get(key)
    }

    pub fn clear(&mut self) {
        self.sections.clear();
    }
}

/// Armed when a dump sample is due this frame (CQ-68).
#[derive(Resource, Debug, Default)]
pub struct SimDumpArm {
    pub due: bool,
    pub exiting: bool,
    pub tick: u64,
}

/// Debug-only set for per-domain dump reporters (CQ-68).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimDumpSet;

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
pub(crate) const TERRITORY_CONTROL_MIN: f32 = 0.5;

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
pub(crate) struct AiTally {
    pub(crate) last_seen_tick: Option<u64>,
    pub(crate) decisions: u64,
    pub(crate) orders_logistics: u64,
    pub(crate) orders_deployment: u64,
    pub(crate) orders_build: u64,
    pub(crate) last_goal: String,
}

#[derive(Resource, Debug)]
pub(crate) struct SimRunDumpState {
    pub(crate) last_tick: u64,
    pub(crate) series: BTreeMap<&'static str, Series>,
    pub(crate) territory_first: BTreeMap<u8, u32>,
    pub(crate) prev_ever_burning: u64,
    pub(crate) ignited_cells_cum: u64,
    pub(crate) ai: BTreeMap<u8, AiTally>,
    pub(crate) frame_ms: Vec<f32>,
    pub(crate) frames_seen: u64,
    pub(crate) first_frame_at: Option<std::time::Instant>,
    pub(crate) hitches: u64,
    pub(crate) stalls: u64,
    pub(crate) worst_hitches: Vec<HitchRecord>,
    pub(crate) history: Vec<Value>,
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
    pub(crate) fn track(&mut self, key: &'static str, v: f64) {
        self.series.entry(key).or_default().push(v);
    }

    pub(crate) fn series_json(&self, key: &str) -> Value {
        self.series.get(key).map_or_else(|| Series::default().json(), Series::json)
    }

    /// Frame-time stats after warm-up, hitch counts (threshold from [`FrameHitchLog`]), the worst
    /// hitch frames with their top `PerfScope`s, and a downsampled series for the dashboard.
    pub(crate) fn perf_json(&self, hitch: Option<&FrameHitchLog>) -> Value {
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


pub(crate) fn push_history(state: &mut SimRunDumpState, entry: Value) {
    state.history.push(entry);
    if state.history.len() > HISTORY_CAP {
        let mut i = 0usize;
        state.history.retain(|_| {
            i += 1;
            i % 2 == 1
        });
    }
}

pub(crate) fn write_dump_files_pub(body: &Value) -> bool {
    write_dump_files(body)
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

fn arm_sim_dump(
    tick: Res<SimTick>,
    launch: Option<Res<EngineLaunchArgs>>,
    exit: MessageReader<AppExit>,
    mut arm: ResMut<SimDumpArm>,
    mut dump: ResMut<SimRunDump>,
    state: Res<SimRunDumpState>,
) {
    arm.due = false;
    arm.exiting = false;
    arm.tick = tick.0;
    if !sim_dump_enabled(launch.as_deref()) {
        return;
    }
    let exiting = !exit.is_empty();
    let due = exiting
        || (tick.0 >= 8
            && (state.last_tick == u64::MAX
                || tick.0.saturating_sub(state.last_tick) >= DUMP_STRIDE_TICKS));
    if !due {
        return;
    }
    arm.due = true;
    arm.exiting = exiting;
    dump.clear();
}

pub struct SimRunDumpPlugin;

impl Plugin for SimRunDumpPlugin {
    fn build(&self, app: &mut App) {
        if !cfg!(debug_assertions) && !cfg!(feature = "sim_dump") {
            return;
        }
        use super::sim_run_dump_reporters as r;
        app.add_plugins(super::sim_run_dump_weather::WeatherDumpPlugin);
        app.add_plugins(super::sim_run_dump_fidelity::FidelityDumpPlugin);
        app.add_plugins(super::sim_run_dump_massing::PresentationDumpPlugin);
        app.init_resource::<SimRunDumpState>()
            .init_resource::<SimRunDump>()
            .init_resource::<SimDumpArm>()
            .add_systems(
                Last,
                (
                    sample_frame_time
                        .after(crate::render::probes::frame_hitch::detect_frame_hitches),
                    arm_sim_dump.after(sample_frame_time),
                    (
                        r::dump_series_sections,
                        r::dump_construction_section,
                        r::dump_vegetation_section,
                        r::dump_canopy_section,
                        r::dump_voxel_camera_section,
                        r::dump_economy_section,
                        r::dump_logistics_section,
                        r::dump_ai_section,
                        r::dump_combat_section,
                        r::dump_fire_section,
                        r::dump_fire_chain_section,
                        super::sim_run_dump_dam_flood::dump_dam_flood_section,
                        super::sim_run_dump_dam_flood::dump_dam_site_section,
                        super::sim_run_dump_quadrants::dump_quadrants_section,
                    )
                        .in_set(SimDumpSet)
                        .after(arm_sim_dump),
                    r::dump_world_envelope_section
                        .in_set(SimDumpSet)
                        .after(r::dump_construction_section)
                        .after(r::dump_economy_section)
                        .after(r::dump_logistics_section)
                        .after(r::dump_combat_section)
                        .after(r::dump_fire_section)
                        .after(r::dump_series_sections),
                    r::flush_sim_run_dump.after(SimDumpSet),
                ),
            );
    }
}
