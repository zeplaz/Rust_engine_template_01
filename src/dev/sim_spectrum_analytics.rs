//! Full-spectrum simulation analytics — multi-layer frame + ECS + loop telemetry to disk.
//!
//! Terminal `PERF=` lines are a lossy one-liner; this witness rolls structured samples into
//! `debug_runs/sim_spectrum_analytics_live.json` (and optional JSONL per-frame traces).
//!
//! Enable: `SIM_ANALYTICS=1` (or `PERF_DISK=1`), or automatically for any `--test …` CLI run.
//! Optional: `SIM_ANALYTICS_QUIET=1` suppresses terminal `perf` spam while disk capture runs
//! (auto-on for `--test`). Optional: `SIM_ANALYTICS_FRAMES=1` appends one JSON object per frame
//! under `debug_runs/perf_frames/` (auto-on for `--test vfx|visual|…`).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde_json::{json, Value};

use crate::dev::debug_run_envelope::{debug_runs_dir, wrap_debug_run, write_debug_run_json};
use crate::dev::perf_scope_frame_log;
use crate::dev::test_run_instrumentation::{
    self, EcsResourceInventory, TestRunInstrumentation,
};
use crate::engine::{AppState, UxFrameSpikeGuard, WorldGenState};
use crate::gui::hud::frame_budget_diagnostics::{FrameBudgetBucket, FrameBudgetDiagnostics};
use crate::io::streaming::{ChunkResidencyRole, ChunkResidencyTable};
use crate::render::minimap_compositor::MinimapGpuCompositorDiagnostics;
use crate::render::{
    percentile_from_slice, FireChunkRuntime, FireExtractFrameReport, FramePerf, FrameStallWatch,
    FrameUpdateAttrib, FrameWallClock, PerfAttributionWitness, TileFallbackRasterPolicy,
};
use crate::systems::sim_control::{SimTick, SimTimeMicros};

pub const SIM_SPECTRUM_LIVE_JSON: &str = "debug_runs/sim_spectrum_analytics_live.json";
const ROLLING_WINDOW: usize = 3600;

#[must_use]
pub fn sim_spectrum_analytics_enabled() -> bool {
    test_run_instrumentation::instrumentation_active()
}

#[must_use]
pub fn sim_spectrum_analytics_quiet_terminal() -> bool {
    test_run_instrumentation::instrumentation_quiet_terminal()
}

#[must_use]
pub fn sim_spectrum_frame_jsonl_enabled() -> bool {
    test_run_instrumentation::instrumentation_frame_jsonl()
}

fn env_on(key: &str) -> bool {
    test_run_instrumentation::env_flag(key)
}

fn flush_interval_secs() -> f32 {
    test_run_instrumentation::instrumentation_flush_secs()
}

#[derive(Clone, Debug)]
struct MetricRing {
    buf: Vec<f32>,
    head: usize,
    len: usize,
}

impl MetricRing {
    fn with_capacity(cap: usize) -> Self {
        Self {
            buf: vec![0.0; cap],
            head: 0,
            len: 0,
        }
    }

    fn push(&mut self, value: f32) {
        if !value.is_finite() || value < 0.0 {
            return;
        }
        let cap = self.buf.len();
        if self.len < cap {
            self.buf[self.len] = value;
            self.len += 1;
        } else {
            self.buf[self.head] = value;
            self.head = (self.head + 1) % cap;
        }
    }

    fn samples(&self) -> Vec<f32> {
        if self.len == 0 {
            return Vec::new();
        }
        let cap = self.buf.len();
        if self.len < cap {
            return self.buf[..self.len].to_vec();
        }
        let mut out = Vec::with_capacity(cap);
        out.extend_from_slice(&self.buf[self.head..]);
        out.extend_from_slice(&self.buf[..self.head]);
        out
    }

    fn p50(&self) -> f32 {
        percentile_from_slice(self.samples(), 0.50)
    }

    fn p95(&self) -> f32 {
        percentile_from_slice(self.samples(), 0.95)
    }

    fn max(&self) -> f32 {
        self.samples()
            .into_iter()
            .fold(0.0_f32, |a, b| a.max(b))
    }
}

/// Rolling multi-layer witness — flushed to disk on an interval.
#[derive(Resource, Debug)]
pub struct SimSpectrumAnalytics {
    pub session_started_epoch_secs: u64,
    pub frames_sampled: u64,
    pub disk_flushes: u64,
    pub spike_frames: u64,
    pub frames_over_250ms: u64,
    last_flush: Instant,
    last_frame: Option<Value>,
    wall_ms: MetricRing,
    cpu_pre_egui_ms: MetricRing,
    cpu_egui_ms: MetricRing,
    gpu_gap_ms: MetricRing,
    fire_pipeline_ms: MetricRing,
    view_fire_ms: MetricRing,
    map_camera_ms: MetricRing,
    streaming_ms: MetricRing,
    raster_ms: MetricRing,
    readiness_ms: MetricRing,
    merge_ms: MetricRing,
    graph_ms: MetricRing,
    world_repr_ms: MetricRing,
    post_vt_to_egui_ms: MetricRing,
    jsonl_path: Option<PathBuf>,
}

impl Default for SimSpectrumAnalytics {
    fn default() -> Self {
        Self {
            session_started_epoch_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            frames_sampled: 0,
            disk_flushes: 0,
            spike_frames: 0,
            frames_over_250ms: 0,
            last_flush: Instant::now(),
            last_frame: None,
            wall_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            cpu_pre_egui_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            cpu_egui_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            gpu_gap_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            fire_pipeline_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            view_fire_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            map_camera_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            streaming_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            raster_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            readiness_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            merge_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            graph_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            world_repr_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            post_vt_to_egui_ms: MetricRing::with_capacity(ROLLING_WINDOW),
            jsonl_path: None,
        }
    }
}

impl SimSpectrumAnalytics {
    fn metric_summary(ring: &MetricRing) -> Value {
        json!({
            "p50_ms": ring.p50(),
            "p95_ms": ring.p95(),
            "max_ms": ring.max(),
            "samples": ring.samples().len(),
        })
    }

    fn rolling_summary(&self) -> Value {
        json!({
            "window_target_frames": ROLLING_WINDOW,
            "frames_sampled": self.frames_sampled,
            "frame_wall_ms": Self::metric_summary(&self.wall_ms),
            "cpu_pre_egui_ms": Self::metric_summary(&self.cpu_pre_egui_ms),
            "cpu_egui_ms": Self::metric_summary(&self.cpu_egui_ms),
            "gpu_gap_ms": Self::metric_summary(&self.gpu_gap_ms),
            "fire_pipeline_ms": Self::metric_summary(&self.fire_pipeline_ms),
            "view_fire_ms": Self::metric_summary(&self.view_fire_ms),
            "map_camera_chain_ms": Self::metric_summary(&self.map_camera_ms),
            "streaming_ms": Self::metric_summary(&self.streaming_ms),
            "tile_raster_ms": Self::metric_summary(&self.raster_ms),
            "readiness_ms": Self::metric_summary(&self.readiness_ms),
            "domain_merge_ms": Self::metric_summary(&self.merge_ms),
            "projection_graph_ms": Self::metric_summary(&self.graph_ms),
            "world_repr_ms": Self::metric_summary(&self.world_repr_ms),
            "post_vt_to_egui_ms": Self::metric_summary(&self.post_vt_to_egui_ms),
            "spike_frames": self.spike_frames,
            "frames_over_250ms": self.frames_over_250ms,
        })
    }
}

#[derive(SystemParam)]
struct SpectrumCapture<'w> {
    wall: Option<Res<'w, FrameWallClock>>,
    perf: Option<Res<'w, FramePerf>>,
    attrib: Option<Res<'w, FrameUpdateAttrib>>,
    stall: Option<Res<'w, FrameStallWatch>>,
    budget: Option<Res<'w, FrameBudgetDiagnostics>>,
    spike: Option<Res<'w, UxFrameSpikeGuard>>,
    perf_witness: Option<Res<'w, PerfAttributionWitness>>,
    tick: Option<Res<'w, SimTick>>,
    sim_time: Option<Res<'w, SimTimeMicros>>,
    app_state: Option<Res<'w, State<AppState>>>,
    wgen_state: Option<Res<'w, State<WorldGenState>>>,
    fire_runtime: Option<Res<'w, FireChunkRuntime>>,
    residency: Option<Res<'w, ChunkResidencyTable>>,
    raster_policy: Option<Res<'w, TileFallbackRasterPolicy>>,
    minimap_diag: Option<Res<'w, MinimapGpuCompositorDiagnostics>>,
    inventory: Option<Res<'w, EcsResourceInventory>>,
    fire_extract: Option<Res<'w, crate::render::FireExtractDiagnostics>>,
}

fn budget_bucket_json(budget: &FrameBudgetDiagnostics) -> Value {
    let mut out = serde_json::Map::new();
    for bucket in FrameBudgetBucket::ALL {
        let s = budget.buckets[bucket.index()];
        out.insert(
            bucket.label().to_string(),
            json!({
                "last_ms": s.last_ms,
                "avg_ms": s.avg_ms,
                "max_ms": s.max_ms,
                "events_last_frame": s.events_last_frame,
            }),
        );
    }
    Value::Object(out)
}

fn fire_runtime_json(runtime: &FireChunkRuntime) -> Value {
    let total = runtime.chunks.len();
    let active = runtime.chunks.values().filter(|c| c.active).count();
    let visual = runtime
        .chunks
        .values()
        .filter(|c| c.visual_active)
        .count();
    json!({
        "chunks_total": total,
        "chunks_active": active,
        "chunks_visual_active": visual,
    })
}

fn residency_json(table: &ChunkResidencyTable) -> Value {
    let core = table
        .entries
        .values()
        .filter(|e| e.role == ChunkResidencyRole::Core)
        .count();
    let ghost = table.entries.len().saturating_sub(core);
    json!({
        "cells_total": table.entries.len(),
        "core_cells": core,
        "ghost_band_cells": ghost,
    })
}

fn schedule_spans_json(sp: &crate::render::FrameScheduleSpans) -> Value {
    json!({
        "first_to_preupdate_ms": sp.first_to_preupdate_ms,
        "update_ms": sp.update_ms,
        "update_pre_map_camera_ms": sp.update_pre_map_camera_ms,
        "map_camera_chain_ms": sp.map_camera_chain_ms,
        "after_view_sync_ms": sp.after_view_sync_ms,
        "after_fire_build_ms": sp.after_fire_build_ms,
        "before_world_repr_ms": sp.before_world_repr_ms,
        "post_world_repr_ms": sp.post_world_repr_ms,
        "post_fire_project_ms": sp.post_fire_project_ms,
        "post_streaming_spine_ms": sp.post_streaming_spine_ms,
        "domain_merge_ms": sp.domain_merge_ms,
        "postupdate_main_ms": sp.postupdate_main_ms,
        "postupdate_vt_ci_ms": sp.postupdate_vt_ci_ms,
        "readiness_ms": sp.readiness_ms,
        "post_vt_to_pre_egui_ms": sp.post_vt_to_pre_egui_ms,
        "egui_ms": sp.egui_ms,
        "post_egui_to_last_ms": sp.post_egui_to_last_ms,
    })
}

fn fire_extract_json(report: &FireExtractFrameReport) -> Value {
    json!({
        "ran_full_scan": report.ran_full_scan,
        "cadence_skipped": report.cadence_skipped,
        "cadence_due": report.cadence_due,
        "spike_active": report.spike_active,
        "tick_changed": report.tick_changed,
        "interval_elapsed": report.interval_elapsed,
        "residency_scoped": report.residency_scoped,
        "min_interval_secs": report.min_interval_secs,
        "extract_ms": report.extract_ms,
        "chunks_iterated": report.chunks_iterated,
        "chunks_fast_path": report.chunks_fast_path,
        "chunks_profiled": report.chunks_profiled,
        "instances_written": report.instances_written,
        "chunk_heat_written": report.chunk_heat_written,
        "runtime_chunks": report.runtime_chunks,
    })
}

fn build_bottleneck_triage(params: &SpectrumCapture) -> Value {
    let mut suspects: Vec<(String, f32)> = Vec::new();
    if let Some(sp) = params.stall.as_deref() {
        let s = &sp.spans;
        for (label, ms) in [
            ("update_pre_map_camera", s.update_pre_map_camera_ms),
            ("map_camera_chain", s.map_camera_chain_ms),
            ("after_view_sync", s.after_view_sync_ms),
            ("fire_build_profiles", s.after_fire_build_ms),
            ("before_world_repr", s.before_world_repr_ms),
            ("post_world_repr", s.post_world_repr_ms),
            ("post_fire_project", s.post_fire_project_ms),
            ("post_streaming_spine", s.post_streaming_spine_ms),
            ("domain_merge", s.domain_merge_ms),
            ("readiness", s.readiness_ms),
            ("egui", s.egui_ms),
        ] {
            if ms >= 8.0 {
                suspects.push((label.to_string(), ms));
            }
        }
    }
    if let Some(a) = params.attrib.as_deref() {
        if a.fire_pipeline_ms >= 8.0 {
            suspects.push(("attrib_fire_pipeline".to_string(), a.fire_pipeline_ms));
        }
        if a.fire_build_view_ms >= 8.0 {
            suspects.push(("attrib_fire_build_view".to_string(), a.fire_build_view_ms));
        }
        if a.streaming_reconstruct_ms >= 8.0 {
            suspects.push((
                "attrib_streaming_reconstruct".to_string(),
                a.streaming_reconstruct_ms,
            ));
        }
    }
    suspects.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let primary = suspects
        .iter()
        .take(6)
        .map(|(label, ms)| json!({ "label": label, "ms": ms }))
        .collect::<Vec<_>>();

    let spike = params
        .spike
        .as_deref()
        .is_some_and(|g| g.spike_active);
    let fire_skip = params
        .fire_extract
        .as_deref()
        .is_some_and(|d| d.last.cadence_skipped);

    json!({
        "primary_suspects": primary,
        "ux_spike_active": spike,
        "fire_extract_skipped_this_frame": fire_skip,
        "interpretation": if suspects.first().map(|(l, _)| l.as_str()) == Some("fire_build_profiles") {
            "Update fire BuildProfiles set (ECS extract + overlay sync + per-view fire frames) dominates."
        } else if suspects.first().map(|(l, _)| l.as_str()) == Some("map_camera_chain") {
            "Map camera smooth/chain dominates — check upd_map_camera_* scopes."
        } else if suspects.first().map(|(l, _)| l.as_str()) == Some("post_world_repr") {
            "WorldRepresentation ComputeFrame dominates — check world repr / LOD spine."
        } else {
            "See schedule_spans + perf_scopes for frame-local detail."
        },
    })
}

fn build_frame_snapshot(params: &SpectrumCapture) -> Value {
    let wall_ms = params
        .budget
        .as_deref()
        .map(|b| b.frame_time_ms)
        .unwrap_or_else(|| {
            params
                .wall
                .as_deref()
                .map(|w| w.cpu_pre_egui_ms + w.cpu_egui_ms + w.cpu_post_egui_ms + w.gpu_gap_ms)
                .unwrap_or(0.0)
        });

    let attrib = params.attrib.as_deref().cloned().unwrap_or_default();
    let perf = params.perf.as_deref().cloned().unwrap_or_default();

    let cpu = params.wall.as_deref().map(|w| {
        json!({
            "pre_egui_ms": w.cpu_pre_egui_ms,
            "egui_ms": w.cpu_egui_ms,
            "post_egui_ms": w.cpu_post_egui_ms,
            "gpu_gap_ms": w.gpu_gap_ms,
        })
    });

    let mut frame = json!({
        "frame_index": perf.frame_index,
        "wall_ms": wall_ms,
        "cpu": cpu,
        "spine": {
            "world_repr_ms": perf.world_repr_ms,
            "projection_graph_ms": perf.projection_graph_ms,
            "domain_merge_ms": perf.domain_merge_ms,
            "atmosphere_gpu_extract_ms": perf.atmosphere_gpu_extract_ms,
            "readiness_ms": perf.readiness_ms,
            "tile_raster_ms": if perf.tile_raster_ran { perf.tile_raster_ms } else { 0.0 },
            "tile_raster_ran": perf.tile_raster_ran,
        },
        "update_attrib": {
            "preview_cpu_raster_ms": attrib.preview_cpu_raster_ms,
            "preview_gpu_present_ms": attrib.preview_gpu_present_ms,
            "fire_pipeline_ms": attrib.fire_pipeline_ms,
            "fire_build_view_ms": attrib.fire_build_view_ms,
            "fire_project_ms": attrib.fire_project_ms,
            "fire_particles_ms": attrib.fire_particles_ms,
            "streaming_reconstruct_ms": attrib.streaming_reconstruct_ms,
            "tile_storage_apply_ms": attrib.tile_storage_apply_ms,
            "viewport_sync_ms": attrib.viewport_sync_ms,
            "map_fit_sync_ms": attrib.map_fit_sync_ms,
            "map_fit_validate_ms": attrib.map_fit_validate_ms,
            "hud_egui_ms": attrib.hud_egui_ms,
            "world_gen_ui_ms": attrib.world_gen_ui_ms,
            "sum_ms": attrib.attrib_sum_ms(),
        },
        "sim_loop": {
            "tick": params.tick.as_deref().map(|t| t.0),
            "sim_time_micros": params.sim_time.as_deref().map(|t| t.0),
            "app_state": params.app_state.as_deref().map(|s| format!("{:?}", s.get())),
            "world_gen_state": params.wgen_state.as_deref().map(|s| format!("{:?}", s.get())),
        },
    });

    if let Some(sp) = params.stall.as_deref() {
        frame["schedule_spans"] = schedule_spans_json(&sp.spans);
        if !sp.segments.is_empty() {
            frame["stall_hits"] = json!(sp
                .segments
                .iter()
                .map(|(l, ms)| json!({ "label": l, "ms": ms }))
                .collect::<Vec<_>>());
        }
    }

    if let Some(b) = params.budget.as_deref() {
        frame["frame_budget"] = json!({
            "avg_frame_ms": b.avg_frame_ms,
            "max_frame_ms": b.max_frame_ms,
            "upload_bytes_frame": b.upload_bytes_frame,
            "texture_rebuilds_frame": b.texture_rebuilds_frame,
            "viewport_mutations_frame": b.viewport_mutations_frame,
            "layout_invalidations_frame": b.layout_invalidations_frame,
            "buckets": budget_bucket_json(b),
            "stage6": {
                "active_residency_cells": b.stage6.active_residency_cells,
                "upload_bytes_frame": b.stage6.upload_bytes_frame,
                "atlas_pressure": b.stage6.atlas_pressure,
                "dirty_region_count": b.stage6.dirty_region_count,
            },
            "last_anomaly": b.last_anomaly.as_ref().map(|a| json!({
                "kind": format!("{:?}", a.kind),
                "detail": a.detail,
            })),
        });
    }

    if let Some(spike) = params.spike.as_deref() {
        frame["ux_spike_guard"] = json!({
            "spike_active": spike.spike_active,
            "last_frame_ms": spike.last_frame_ms,
            "max_ms": spike.max_ms,
            "suppress_preview": spike.suppress_preview_this_frame,
            "suppress_optional_diagnostics": spike.suppress_optional_diagnostics,
        });
    }

    if let Some(runtime) = params.fire_runtime.as_deref() {
        frame["fire_runtime"] = fire_runtime_json(runtime);
    }

    if let Some(table) = params.residency.as_deref() {
        frame["chunk_residency"] = residency_json(table);
    }

    if let Some(p) = params.raster_policy.as_deref() {
        frame["tile_raster_policy"] = json!({
            "chunks_per_frame": p.chunks_per_frame,
            "cpu_minimap_pass": p.cpu_minimap_pass,
            "defer_zoom_dirty": p.defer_zoom_dirty,
            "minimap_cadence_hz": p.minimap_cadence_hz,
        });
    }

    if let Some(d) = params.minimap_diag.as_deref() {
        frame["minimap_gpu"] = json!({
            "commits_queued": d.commits_queued,
            "skips_no_change": d.skips_no_change,
            "skips_no_terrain": d.skips_no_terrain,
            "skips_upload_failed": d.skips_upload_failed,
            "skips_rate_capped": d.skips_rate_capped,
            "last_skip": format!("{:?}", d.last_skip),
            "last_dispatch": format!("{:?}", d.last_dispatch),
        });
    }

    if let Some(w) = params.perf_witness.as_deref() {
        frame["perf_attribution_witness"] = json!({
            "window_samples": w.window_samples(),
            "p95_frame_ms": w.p95_frame_ms(),
            "p95_raster_b_ms": w.p95_raster_b_ms(),
            "p95_view_fire_ms": w.p95_view_fire_ms(),
        });
    }

    if let Some(inv) = params.inventory.as_deref().and_then(|i| i.last_json.clone()) {
        frame["ecs_inventory"] = inv;
    }

    if let Some(fe) = params.fire_extract.as_deref() {
        frame["fire_extract"] = fire_extract_json(&fe.last);
    }

    frame["perf_scopes"] = perf_scope_frame_log::take_perf_scopes_json();
    frame["bottleneck_triage"] = build_bottleneck_triage(params);

    frame
}

fn append_jsonl(path: &PathBuf, value: &Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", value)?;
    Ok(())
}

fn ensure_jsonl_path(analytics: &mut SimSpectrumAnalytics) -> PathBuf {
    if let Some(path) = analytics.jsonl_path.clone() {
        return path;
    }
    let dir = debug_runs_dir().join("perf_frames");
    let name = format!(
        "frames_{}.jsonl",
        analytics.session_started_epoch_secs
    );
    let path = dir.join(name);
    analytics.jsonl_path = Some(path.clone());
    path
}

fn capture_sim_spectrum_frame(
    mut analytics: ResMut<SimSpectrumAnalytics>,
    params: SpectrumCapture,
) {
    if !sim_spectrum_analytics_enabled() {
        return;
    }

    let snapshot = build_frame_snapshot(&params);
    let wall_ms = snapshot
        .get("wall_ms")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;

    let attrib = params.attrib.as_deref();
    let perf = params.perf.as_deref();
    let wall = params.wall.as_deref();
    let stall = params.stall.as_deref();
    let spike = params.spike.as_deref();

    analytics.wall_ms.push(wall_ms);
    if let Some(w) = wall {
        analytics.cpu_pre_egui_ms.push(w.cpu_pre_egui_ms);
        analytics.cpu_egui_ms.push(w.cpu_egui_ms);
        analytics.gpu_gap_ms.push(w.gpu_gap_ms);
    }
    if let Some(a) = attrib {
        analytics.fire_pipeline_ms.push(a.fire_pipeline_ms);
        analytics.streaming_ms.push(a.streaming_reconstruct_ms);
    }
    if let Some(p) = perf {
        analytics.raster_ms.push(if p.tile_raster_ran {
            p.tile_raster_ms
        } else {
            0.0
        });
        analytics.readiness_ms.push(p.readiness_ms);
        analytics.merge_ms.push(p.domain_merge_ms);
        analytics.graph_ms.push(p.projection_graph_ms);
        analytics.world_repr_ms.push(p.world_repr_ms);
    }
    if let Some(s) = stall {
        analytics.view_fire_ms.push(s.spans.after_fire_build_ms);
        analytics.map_camera_ms.push(s.spans.map_camera_chain_ms);
        analytics.post_vt_to_egui_ms.push(s.spans.post_vt_to_pre_egui_ms);
    }
    if spike.is_some_and(|g| g.spike_active) {
        analytics.spike_frames = analytics.spike_frames.saturating_add(1);
    }
    if wall_ms >= 250.0 {
        analytics.frames_over_250ms = analytics.frames_over_250ms.saturating_add(1);
    }

    analytics.frames_sampled = analytics.frames_sampled.saturating_add(1);
    analytics.last_frame = Some(snapshot.clone());

    if sim_spectrum_frame_jsonl_enabled() {
        let path = ensure_jsonl_path(&mut analytics);
        let _ = append_jsonl(&path, &snapshot);
    }
}

pub fn flush_sim_spectrum_analytics(
    mut analytics: ResMut<SimSpectrumAnalytics>,
    time: Res<Time>,
    inst: Option<Res<TestRunInstrumentation>>,
) {
    if !sim_spectrum_analytics_enabled() {
        return;
    }
    let interval = flush_interval_secs();
    if analytics.last_flush.elapsed().as_secs_f32() < interval {
        return;
    }
    analytics.last_flush = Instant::now();

    let body = json!({
        "schema_version": 1,
        "profile": "SIM_SPECTRUM",
        "session_started_epoch_secs": analytics.session_started_epoch_secs,
        "session_elapsed_secs": time.elapsed_secs_f64(),
        "frames_sampled": analytics.frames_sampled,
        "disk_flushes": analytics.disk_flushes,
        "rolling": analytics.rolling_summary(),
        "last_frame": analytics.last_frame.clone().unwrap_or(Value::Null),
        "jsonl_enabled": sim_spectrum_frame_jsonl_enabled(),
        "jsonl_path": analytics
            .jsonl_path
            .as_ref()
            .and_then(|p| p.to_str().map(str::to_string)),
        "env": {
            "SIM_ANALYTICS": env_on("SIM_ANALYTICS"),
            "PERF_DISK": env_on("PERF_DISK"),
            "SIM_ANALYTICS_QUIET": env_on("SIM_ANALYTICS_QUIET"),
            "SIM_ANALYTICS_FRAMES": env_on("SIM_ANALYTICS_FRAMES"),
            "STALL": env_on("STALL"),
            "flush_interval_secs": interval,
        },
        "test_instrumentation": inst.as_deref().map(|i| json!({
            "from_test_cli": i.from_test_cli,
            "test_scene": format!("{:?}", i.test_scene),
            "maneuver": format!("{:?}", i.maneuver),
            "active": i.active,
            "quiet_terminal": i.quiet_terminal,
            "frame_jsonl": i.frame_jsonl,
            "stall_spans": i.stall_spans,
        })),
    });

    let wrapped = wrap_debug_run("SIM_SPECTRUM", "sim_spectrum_analytics", SIM_SPECTRUM_LIVE_JSON, body);
    if write_debug_run_json(SIM_SPECTRUM_LIVE_JSON, wrapped) {
        analytics.disk_flushes = analytics.disk_flushes.saturating_add(1);
    }
}

pub struct SimSpectrumAnalyticsPlugin;

impl Plugin for SimSpectrumAnalyticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimSpectrumAnalytics>().add_systems(
            Last,
            (
                capture_sim_spectrum_frame.after(crate::render::emit_frame_perf_summary),
                flush_sim_spectrum_analytics.after(capture_sim_spectrum_frame),
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_ring_p95_tracks_spike() {
        let mut ring = MetricRing::with_capacity(100);
        for _ in 0..90 {
            ring.push(10.0);
        }
        for _ in 0..10 {
            ring.push(200.0);
        }
        assert!(ring.p95() >= 200.0);
        assert!(ring.p50() <= 15.0);
    }

    #[test]
    fn rolling_summary_has_core_layers() {
        let mut a = SimSpectrumAnalytics::default();
        a.wall_ms.push(16.0);
        a.fire_pipeline_ms.push(4.0);
        let summary = a.rolling_summary();
        assert!(summary.get("frame_wall_ms").is_some());
        assert!(summary.get("fire_pipeline_ms").is_some());
        assert!(summary.get("view_fire_ms").is_some());
    }
}
