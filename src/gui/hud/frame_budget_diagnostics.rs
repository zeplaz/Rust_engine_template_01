//! Lightweight frame-time and GPU-cost attribution for Stage 6 preparation.

use std::time::Instant;

use bevy::log::warn;
use bevy::prelude::*;

use crate::io::streaming::ChunkResidencyTable;
use crate::render::{GpuRepresentationMetrics, Stage6VirtualizationFrame};

use super::layout_store::HudLayoutStore;
use super::pending_hud_layout_commit::PendingHudLayoutCommit;
use super::shell_diagnostics::ProductShellDiagnostics;
use super::shell_framework::ProductShellWidgetId;
use super::stage6_telemetry::Stage6HudTelemetry;
use super::viewport_rect_sanity::ViewportRectSanity;

pub const FRAME_HISTORY_LEN: usize = 120;
const FRAME_SPIKE_MS: f32 = 33.0;
const UPLOAD_STORM_BYTES: u64 = 256 * 1024;
const VIEWPORT_CHURN_EVENTS: u64 = 4;
const LAYOUT_INVALIDATION_EVENTS: u64 = 3;
const ANOMALY_COOLDOWN_SECS: f32 = 2.0;
/// **WC-D04 / S6-26** — minimum residency cell delta before churn is considered (perf doc aligned).
pub const RESIDENCY_CHURN_CELL_DELTA: usize = 48;
/// Consecutive frames over threshold before `ResidencyChurn` anomaly fires.
pub const RESIDENCY_CHURN_HYSTERESIS_FRAMES: u8 = 2;
/// Suppress churn anomalies while residency table is bootstrapping after first populate.
pub const RESIDENCY_CHURN_BOOTSTRAP_FRAMES: u64 = 45;
const BUCKET_EMA_ALPHA: f32 = 0.12;

/// **WC-D04-CODER-B** — hysteresis gate for residency churn warnings.
#[must_use]
pub fn residency_churn_should_report(
    delta: usize,
    streak: u8,
    bootstrap_remaining: u64,
) -> bool {
    bootstrap_remaining == 0
        && delta >= RESIDENCY_CHURN_CELL_DELTA
        && streak >= RESIDENCY_CHURN_HYSTERESIS_FRAMES
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameBudgetBucket {
    HudShell,
    MinimapRaster,
    OverlayComposition,
    ParticleUpload,
    ResidencyUpdates,
    GpuTextureRegistration,
    RenderExtraction,
}

impl FrameBudgetBucket {
    pub const ALL: [Self; 7] = [
        Self::HudShell,
        Self::MinimapRaster,
        Self::OverlayComposition,
        Self::ParticleUpload,
        Self::ResidencyUpdates,
        Self::GpuTextureRegistration,
        Self::RenderExtraction,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::HudShell => "HUD shell",
            // Main overworld CPU tile raster (`tile_world_fallback_rasterize`), not the minimap widget.
            Self::MinimapRaster => "Tile world raster",
            Self::OverlayComposition => "Overlay composition",
            Self::ParticleUpload => "Particle upload",
            Self::ResidencyUpdates => "Residency updates",
            Self::GpuTextureRegistration => "GPU texture registration",
            Self::RenderExtraction => "Render extraction",
        }
    }

    #[inline]
    pub const fn index(self) -> usize {
        match self {
            Self::HudShell => 0,
            Self::MinimapRaster => 1,
            Self::OverlayComposition => 2,
            Self::ParticleUpload => 3,
            Self::ResidencyUpdates => 4,
            Self::GpuTextureRegistration => 5,
            Self::RenderExtraction => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameBudgetBucketStats {
    pub last_ms: f32,
    pub avg_ms: f32,
    pub max_ms: f32,
    pub events_last_frame: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stage6VirtualizationBudget {
    pub active_residency_cells: u32,
    pub upload_bytes_frame: u64,
    pub atlas_pressure: f32,
    pub dirty_region_count: u32,
    pub overlay_update_count: u32,
    /// **WC-D04** — `ResidencyChurn` anomalies emitted this session (post-hysteresis).
    pub residency_churn_anomalies_session: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameBudgetAnomalyKind {
    FrameSpike,
    ViewportChurn,
    UploadStorm,
    LayoutInvalidation,
    ResidencyChurn,
}

#[derive(Clone, Debug)]
pub struct FrameBudgetAnomalyReport {
    pub kind: FrameBudgetAnomalyKind,
    pub detail: String,
    pub suppressed: u64,
}

#[derive(Resource, Clone, Debug)]
pub struct FrameBudgetDiagnostics {
    pub frame_index: u64,
    pub frame_time_ms: f32,
    pub avg_frame_ms: f32,
    pub max_frame_ms: f32,
    pub egui_frame_ms: f32,
    pub render_extraction_ms: f32,
    pub upload_bytes_frame: u64,
    pub upload_bytes_per_sec: f32,
    pub texture_rebuilds_frame: u32,
    pub texture_rebinds_frame: u32,
    pub layout_invalidations_frame: u32,
    pub viewport_mutations_frame: u32,
    pub drag_frame_mutation_attempts: u32,
    pub layout_spam_source: Option<ProductShellWidgetId>,
    pub drag_mutation_source: Option<ProductShellWidgetId>,
    pub buckets: [FrameBudgetBucketStats; 7],
    pub frame_history: [f32; FRAME_HISTORY_LEN],
    pub history_cursor: usize,
    pub stage6: Stage6VirtualizationBudget,
    pub last_anomaly: Option<FrameBudgetAnomalyReport>,
    pub anomaly_suppressed_total: u64,
    /// **WC-D04** — residency churn anomalies after hysteresis (not cooldown-suppressed).
    pub residency_churn_anomalies_session: u64,
    cooldowns: [f32; 5],
}

impl Default for FrameBudgetDiagnostics {
    fn default() -> Self {
        Self {
            frame_index: 0,
            frame_time_ms: 0.0,
            avg_frame_ms: 0.0,
            max_frame_ms: 0.0,
            egui_frame_ms: 0.0,
            render_extraction_ms: 0.0,
            upload_bytes_frame: 0,
            upload_bytes_per_sec: 0.0,
            texture_rebuilds_frame: 0,
            texture_rebinds_frame: 0,
            layout_invalidations_frame: 0,
            viewport_mutations_frame: 0,
            drag_frame_mutation_attempts: 0,
            layout_spam_source: None,
            drag_mutation_source: None,
            buckets: [FrameBudgetBucketStats::default(); 7],
            frame_history: [0.0; FRAME_HISTORY_LEN],
            history_cursor: 0,
            stage6: Stage6VirtualizationBudget::default(),
            last_anomaly: None,
            anomaly_suppressed_total: 0,
            residency_churn_anomalies_session: 0,
            cooldowns: [0.0; 5],
        }
    }
}

impl FrameBudgetDiagnostics {
    pub fn record_bucket_ms(&mut self, bucket: FrameBudgetBucket, ms: f32) {
        let stats = &mut self.buckets[bucket.index()];
        stats.last_ms = ms.max(0.0);
        stats.avg_ms = if stats.avg_ms <= 0.0 {
            stats.last_ms
        } else {
            stats.avg_ms * (1.0 - BUCKET_EMA_ALPHA) + stats.last_ms * BUCKET_EMA_ALPHA
        };
        stats.max_ms = stats.max_ms.max(stats.last_ms);
        if bucket == FrameBudgetBucket::HudShell {
            self.egui_frame_ms = stats.last_ms;
        }
        if bucket == FrameBudgetBucket::RenderExtraction {
            self.render_extraction_ms = stats.last_ms;
        }
    }

    pub fn bump_bucket_event(&mut self, bucket: FrameBudgetBucket) {
        self.buckets[bucket.index()].events_last_frame =
            self.buckets[bucket.index()].events_last_frame.saturating_add(1);
    }

    pub fn top_buckets_by_last_ms(&self, limit: usize) -> Vec<(FrameBudgetBucket, f32)> {
        let mut ranked: Vec<_> = FrameBudgetBucket::ALL
            .iter()
            .map(|bucket| (*bucket, self.buckets[bucket.index()].last_ms))
            .filter(|(_, ms)| *ms > 0.0)
            .collect();
        ranked.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(limit);
        ranked
    }

    pub fn rebuild_spike_widgets(&self, shell_diag: &ProductShellDiagnostics) -> Vec<(ProductShellWidgetId, u64)> {
        ProductShellWidgetId::ALL
            .iter()
            .filter_map(|id| {
                let count = shell_diag.texture_rebuild_count(*id);
                if count > 0 {
                    Some((*id, count))
                } else {
                    None
                }
            })
            .collect()
    }

    fn reset_bucket_events(&mut self) {
        for stats in &mut self.buckets {
            stats.events_last_frame = 0;
        }
    }

    fn push_frame_history(&mut self, frame_ms: f32) {
        self.frame_history[self.history_cursor] = frame_ms;
        self.history_cursor = (self.history_cursor + 1) % FRAME_HISTORY_LEN;
    }

    fn anomaly_index(kind: FrameBudgetAnomalyKind) -> usize {
        match kind {
            FrameBudgetAnomalyKind::FrameSpike => 0,
            FrameBudgetAnomalyKind::ViewportChurn => 1,
            FrameBudgetAnomalyKind::UploadStorm => 2,
            FrameBudgetAnomalyKind::LayoutInvalidation => 3,
            FrameBudgetAnomalyKind::ResidencyChurn => 4,
        }
    }

    fn note_anomaly(&mut self, kind: FrameBudgetAnomalyKind, detail: String, now_secs: f32) {
        let idx = Self::anomaly_index(kind);
        if self.cooldowns[idx] > now_secs {
            self.anomaly_suppressed_total = self.anomaly_suppressed_total.wrapping_add(1);
            if let Some(report) = self.last_anomaly.as_mut() {
                if report.kind == kind {
                    report.suppressed = report.suppressed.wrapping_add(1);
                }
            }
            return;
        }
        self.cooldowns[idx] = now_secs + ANOMALY_COOLDOWN_SECS;
        if kind == FrameBudgetAnomalyKind::ResidencyChurn {
            self.residency_churn_anomalies_session =
                self.residency_churn_anomalies_session.saturating_add(1);
            self.stage6.residency_churn_anomalies_session = self
                .residency_churn_anomalies_session
                .min(u32::MAX as u64) as u32;
        }
        let report = FrameBudgetAnomalyReport {
            kind,
            detail: detail.clone(),
            suppressed: 0,
        };
        self.last_anomaly = Some(report);
        warn!(
            target: "proc_A_dine01::gui::hud::frame_budget",
            "frame budget anomaly {:?}: {}",
            kind,
            detail
        );
    }
}

pub struct FrameBudgetTimer {
    started: Instant,
}

#[derive(Default)]
pub struct FinalizeFrameBudgetLocals {
    last_gpu_upload: u64,
    last_texture_rebuilds: u64,
    last_viewport_issues: u64,
    last_layout_captures: u64,
    last_residency_cells: usize,
    residency_churn_streak: u8,
    residency_bootstrap_remaining: u64,
}

impl FrameBudgetTimer {
    #[must_use]
    pub fn start() -> Self {
        Self {
            started: Instant::now(),
        }
    }

    #[must_use]
    pub fn elapsed_ms_now(&self) -> f32 {
        self.started.elapsed().as_secs_f32() * 1000.0
    }

    #[must_use]
    pub fn elapsed_ms(self) -> f32 {
        self.started.elapsed().as_secs_f32() * 1000.0
    }
}

pub fn finalize_frame_budget_diagnostics(
    time: Res<Time>,
    mut budget: ResMut<FrameBudgetDiagnostics>,
    shell_diag: Res<ProductShellDiagnostics>,
    layout: Res<HudLayoutStore>,
    viewport_rect: Res<ViewportRectSanity>,
    pending_layout: Res<PendingHudLayoutCommit>,
    texture_cache: Res<crate::gui::MapViewTextureCache>,
    stage6: Option<Res<Stage6HudTelemetry>>,
    stage6_frame: Option<Res<Stage6VirtualizationFrame>>,
    residency: Option<Res<ChunkResidencyTable>>,
    overlay_revision: Option<Res<crate::render::SharedOverlayFieldBuffers>>,
    raster_revision: Option<Res<crate::render::TileWorldFallbackRasterDirty>>,
    gpu: Option<Res<GpuRepresentationMetrics>>,
    mut scratch: Local<FinalizeFrameBudgetLocals>,
) {
    let frame_ms = shell_diag.last_frame_delta_secs.max(0.0) * 1000.0;
    budget.frame_index = budget.frame_index.wrapping_add(1);
    budget.frame_time_ms = frame_ms;
    budget.avg_frame_ms = if budget.avg_frame_ms <= 0.0 {
        frame_ms
    } else {
        budget.avg_frame_ms * (1.0 - BUCKET_EMA_ALPHA) + frame_ms * BUCKET_EMA_ALPHA
    };
    budget.max_frame_ms = budget.max_frame_ms.max(frame_ms);
    budget.push_frame_history(frame_ms);

    let upload_bytes = gpu.as_deref().map(|m| m.upload_bytes).unwrap_or(0);
    let upload_delta = upload_bytes.saturating_sub(scratch.last_gpu_upload);
    scratch.last_gpu_upload = upload_bytes;
    budget.upload_bytes_frame = upload_delta;
    budget.upload_bytes_per_sec = upload_delta as f32 / time.delta_secs().max(1e-4);
    budget
        .buckets[FrameBudgetBucket::ParticleUpload.index()]
        .events_last_frame = (upload_delta / 4096).min(u32::MAX as u64) as u32;

    let rebuild_total: u64 = shell_diag.texture_rebuilds.values().sum();
    let rebuild_delta = rebuild_total.saturating_sub(scratch.last_texture_rebuilds);
    scratch.last_texture_rebuilds = rebuild_total;
    budget.texture_rebuilds_frame = rebuild_delta.min(u32::MAX as u64) as u32;
    budget.texture_rebinds_frame = texture_cache
        .binding(crate::gui::MapViewInstanceId::WorldPreview)
        .rebinds_frame
        .saturating_add(
            texture_cache
                .binding(crate::gui::MapViewInstanceId::Minimap)
                .rebinds_frame,
        );
    budget
        .buckets[FrameBudgetBucket::GpuTextureRegistration.index()]
        .events_last_frame = budget.texture_rebuilds_frame;

    if let Some(metrics) = gpu.as_deref() {
        let proxy_ms = metrics.dispatch_count as f32 * 0.05 + metrics.draw_instances as f32 * 0.002;
        budget.record_bucket_ms(FrameBudgetBucket::RenderExtraction, proxy_ms);
    }

    let now = time.elapsed_secs();
    if let Some(frame) = stage6_frame.as_deref() {
        let cell_count = frame.residency_chunk_count;
        if scratch.last_residency_cells == 0 && cell_count > 0 {
            scratch.residency_bootstrap_remaining = RESIDENCY_CHURN_BOOTSTRAP_FRAMES;
        }
        if scratch.residency_bootstrap_remaining > 0 {
            scratch.residency_bootstrap_remaining -= 1;
        }
        if scratch.last_residency_cells > 0 {
            let delta = cell_count.abs_diff(scratch.last_residency_cells);
            if delta >= RESIDENCY_CHURN_CELL_DELTA {
                scratch.residency_churn_streak = scratch.residency_churn_streak.saturating_add(1);
            } else {
                scratch.residency_churn_streak = 0;
            }
            if residency_churn_should_report(
                delta,
                scratch.residency_churn_streak,
                scratch.residency_bootstrap_remaining,
            ) {
                budget.note_anomaly(
                    FrameBudgetAnomalyKind::ResidencyChurn,
                    format!(
                        "residency cells changed by {delta} ({} → {})",
                        scratch.last_residency_cells, cell_count
                    ),
                    now,
                );
                scratch.residency_churn_streak = 0;
            }
        }
        scratch.last_residency_cells = cell_count;
        budget.stage6.active_residency_cells = cell_count.min(u32::MAX as usize) as u32;
        budget.stage6.upload_bytes_frame = upload_delta.max(frame.gpu_upload_bytes_frame);
        let upload_pressure = if frame.gpu_upload_bytes_frame > 0 {
            (upload_delta as f32 / frame.gpu_upload_bytes_frame as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        budget.stage6.atlas_pressure = upload_pressure;
    }
    if let Some(residency) = residency.as_deref() {
        budget.stage6.active_residency_cells = residency.entries.len().min(u32::MAX as usize) as u32;
    }
    if let Some(stage6) = stage6.as_deref() {
        budget.stage6.overlay_update_count = stage6.frame_revision.min(u32::MAX as u64) as u32;
    }
    budget.stage6.dirty_region_count = raster_revision
        .as_deref()
        .map(|dirty| dirty.revision().min(u32::MAX as u64) as u32)
        .unwrap_or(0)
        .saturating_add(
            overlay_revision
                .as_deref()
                .map(|overlay| overlay.revision.min(u32::MAX as u64) as u32)
                .unwrap_or(0),
        );

    let avg_frame_ms = budget.avg_frame_ms;
    if frame_ms >= FRAME_SPIKE_MS {
        budget.note_anomaly(
            FrameBudgetAnomalyKind::FrameSpike,
            format!("frame {:.1} ms (avg {:.1} ms)", frame_ms, avg_frame_ms),
            now,
        );
    }
    let viewport_delta = viewport_rect.issues_total.saturating_sub(scratch.last_viewport_issues);
    scratch.last_viewport_issues = viewport_rect.issues_total;
    budget.viewport_mutations_frame = viewport_delta.min(u32::MAX as u64) as u32;
    if viewport_delta >= VIEWPORT_CHURN_EVENTS {
        budget.note_anomaly(
            FrameBudgetAnomalyKind::ViewportChurn,
            format!("{viewport_delta} viewport rect issues this frame"),
            now,
        );
    }
    if upload_delta >= UPLOAD_STORM_BYTES {
        budget.note_anomaly(
            FrameBudgetAnomalyKind::UploadStorm,
            format!("{upload_delta} upload bytes this frame"),
            now,
        );
    }
    let layout_delta = layout.layout_captures_applied().saturating_sub(scratch.last_layout_captures);
    scratch.last_layout_captures = layout.layout_captures_applied();
    budget.layout_invalidations_frame = layout_delta.min(u32::MAX as u64) as u32;
    budget.layout_spam_source = layout.top_frame_capture_offender();
    if layout_delta >= LAYOUT_INVALIDATION_EVENTS {
        let offender = budget
            .layout_spam_source
            .map(|id| id.label())
            .unwrap_or("unknown");
        budget.note_anomaly(
            FrameBudgetAnomalyKind::LayoutInvalidation,
            format!("{layout_delta} layout captures applied this frame (top offender {offender})"),
            now,
        );
    }

    budget.drag_frame_mutation_attempts = pending_layout.drag_mutation_attempts_frame;
    budget.drag_mutation_source = pending_layout.top_drag_mutation_offender();
    if pending_layout.drag_active && pending_layout.drag_mutation_attempts_frame > 0 {
        warn!(
            target: "proc_A_dine01::gui::hud::frame_budget",
            "drag-frame layout mutation attempts={} top offender {}",
            pending_layout.drag_mutation_attempts_frame,
            budget
                .drag_mutation_source
                .map(|id| id.label())
                .unwrap_or("unknown"),
        );
    }

    budget.reset_bucket_events();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_ema_tracks_last_sample() {
        let mut budget = FrameBudgetDiagnostics::default();
        budget.record_bucket_ms(FrameBudgetBucket::HudShell, 10.0);
        budget.record_bucket_ms(FrameBudgetBucket::HudShell, 20.0);
        let stats = budget.buckets[FrameBudgetBucket::HudShell.index()];
        assert!(stats.avg_ms > 10.0);
        assert_eq!(stats.last_ms, 20.0);
    }

    #[test]
    fn wc_d04_residency_churn_hysteresis_blocks_single_frame_spike() {
        assert!(!residency_churn_should_report(64, 1, 0));
    }

    #[test]
    fn wc_d04_residency_churn_hysteresis_fires_after_two_frames() {
        assert!(residency_churn_should_report(64, 2, 0));
    }

    #[test]
    fn wc_d04_residency_churn_suppressed_during_bootstrap() {
        assert!(!residency_churn_should_report(256, 3, 12));
    }

    #[test]
    fn top_buckets_rank_by_last_ms() {
        let mut budget = FrameBudgetDiagnostics::default();
        budget.record_bucket_ms(FrameBudgetBucket::HudShell, 4.0);
        budget.record_bucket_ms(FrameBudgetBucket::MinimapRaster, 12.0);
        let top = budget.top_buckets_by_last_ms(2);
        assert_eq!(top[0].0, FrameBudgetBucket::MinimapRaster);
    }
}
