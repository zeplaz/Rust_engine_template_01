//! Minimap GPU compositor diagnostics — justified dispatch + overflow guard telemetry.

use std::sync::atomic::{AtomicU64, Ordering};

use bevy::prelude::*;
use serde::Serialize;

/// Render-world execute count (one increment per unique `commit_stamp` dispatched).
pub static MINIMAP_GPU_EXECUTE_COUNT: AtomicU64 = AtomicU64::new(0);
/// Render-world dedup skips (same `commit_stamp` seen again before main queued a new one).
pub static MINIMAP_GPU_DEDUP_SKIP_COUNT: AtomicU64 = AtomicU64::new(0);
/// Set when compute pipeline fails to compile — main thread falls back to CPU raster.
pub static MINIMAP_GPU_SHADER_FAILED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[must_use]
pub fn minimap_gpu_debug_logging_enabled() -> bool {
    match std::env::var("MINIMAP_GPU_DEBUG").ok().as_deref() {
        Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("NO") => false,
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES") => true,
        None => false,
        _ => true,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MinimapGpuSkipReason {
    #[default]
    None,
    EnvDisabled,
    ShellHidden,
    NoRenderTarget,
    NoTerrain,
    UploadFailed,
    NoChange,
    RateCapped,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MinimapGpuDispatchReason {
    #[default]
    None,
    Initial,
    OverlayChanged,
    LogisticsChanged,
    TerrainChanged,
    RtResize,
    ToggleChanged,
    StaleRefresh,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct MinimapGpuCompositorDiagnostics {
    pub commits_queued: u64,
    pub skips_no_change: u64,
    pub skips_hidden: u64,
    pub skips_no_terrain: u64,
    pub skips_no_rt: u64,
    pub skips_upload_failed: u64,
    pub skips_rate_capped: u64,
    pub last_skip: MinimapGpuSkipReason,
    pub last_dispatch: MinimapGpuDispatchReason,
    pub last_fingerprint: u64,
    pub last_commit_stamp: u64,
    pub last_commit_at_secs: f64,
    pub window_commits: u32,
    pub window_start_secs: f64,
    pub target_hz: f32,
    pub overflow_ratio: f32,
    pub justified: bool,
}

impl MinimapGpuCompositorDiagnostics {
    pub fn record_skip(&mut self, reason: MinimapGpuSkipReason) {
        self.last_skip = reason;
        match reason {
            MinimapGpuSkipReason::ShellHidden => self.skips_hidden += 1,
            MinimapGpuSkipReason::NoTerrain => self.skips_no_terrain += 1,
            MinimapGpuSkipReason::NoRenderTarget => self.skips_no_rt += 1,
            MinimapGpuSkipReason::UploadFailed => self.skips_upload_failed += 1,
            MinimapGpuSkipReason::NoChange => self.skips_no_change += 1,
            MinimapGpuSkipReason::RateCapped => self.skips_rate_capped += 1,
            _ => {}
        }
    }

    pub fn record_commit(
        &mut self,
        reason: MinimapGpuDispatchReason,
        fingerprint: u64,
        stamp: u64,
        now_secs: f64,
    ) {
        self.commits_queued += 1;
        self.window_commits = self.window_commits.saturating_add(1);
        self.last_dispatch = reason;
        self.last_fingerprint = fingerprint;
        self.last_commit_stamp = stamp;
        self.last_commit_at_secs = now_secs;
        self.last_skip = MinimapGpuSkipReason::None;
    }

    pub fn refresh_budget_verdict(&mut self, now_secs: f64, target_hz: f32) {
        self.target_hz = target_hz;
        if self.window_start_secs <= 0.0 {
            self.window_start_secs = now_secs;
        }
        let elapsed = (now_secs - self.window_start_secs).max(1e-3);
        let effective_hz = self.window_commits as f32 / elapsed as f32;
        let executes = MINIMAP_GPU_EXECUTE_COUNT.load(Ordering::Relaxed);
        let expected_max_hz = target_hz * 1.25;
        let execute_overflow = if self.commits_queued > 0 {
            executes as f32 / self.commits_queued as f32
        } else {
            0.0
        };
        self.overflow_ratio = execute_overflow;
        self.justified = effective_hz <= expected_max_hz && execute_overflow <= 1.5;
        if elapsed >= 2.0 {
            self.window_commits = 0;
            self.window_start_secs = now_secs;
        }
    }
}

#[must_use]
pub fn composite_fingerprint(
    terrain: &Handle<Image>,
    overlay_revision: u64,
    logistics_rows: u32,
    construction_rows: u32,
    ecology_rows: u32,
    registry_revision: u64,
    fallback_revision: u64,
    fire_heat: bool,
    logistics_heat: bool,
    construction_heat: bool,
    ecology_heat: bool,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    terrain.id().hash(&mut hasher);
    overlay_revision.hash(&mut hasher);
    logistics_rows.hash(&mut hasher);
    construction_rows.hash(&mut hasher);
    ecology_rows.hash(&mut hasher);
    registry_revision.hash(&mut hasher);
    fallback_revision.hash(&mut hasher);
    fire_heat.hash(&mut hasher);
    logistics_heat.hash(&mut hasher);
    construction_heat.hash(&mut hasher);
    ecology_heat.hash(&mut hasher);
    hasher.finish()
}

#[must_use]
pub fn diagnostics_json_snapshot(diag: &MinimapGpuCompositorDiagnostics) -> serde_json::Value {
    let executes = MINIMAP_GPU_EXECUTE_COUNT.load(Ordering::Relaxed);
    let dedup_skips = MINIMAP_GPU_DEDUP_SKIP_COUNT.load(Ordering::Relaxed);
    serde_json::json!({
        "commits_queued": diag.commits_queued,
        "gpu_executes": executes,
        "gpu_dedup_skips": dedup_skips,
        "skips_no_change": diag.skips_no_change,
        "skips_hidden": diag.skips_hidden,
        "skips_no_terrain": diag.skips_no_terrain,
        "skips_no_rt": diag.skips_no_rt,
        "skips_upload_failed": diag.skips_upload_failed,
        "skips_rate_capped": diag.skips_rate_capped,
        "last_skip": diag.last_skip,
        "last_dispatch": diag.last_dispatch,
        "last_fingerprint": diag.last_fingerprint,
        "last_commit_stamp": diag.last_commit_stamp,
        "target_hz": diag.target_hz,
        "overflow_ratio": diag.overflow_ratio,
        "justified": diag.justified,
        "debug_logging": minimap_gpu_debug_logging_enabled(),
    })
}
