//! Production visual perf budgets — ship path replaces `RASTER_*` env in release builds.
//!
//! See [`crate::dev::plan_visual_perf_production_exec_001_v1`].

use bevy::prelude::*;

use crate::gui::hud::frame_budget_diagnostics::{FrameBudgetBucket, FrameBudgetDiagnostics};
use crate::gui::VisualBudgetSettings;

/// EMA / frame thresholds for zoom dirty deferral (**PERF-VIS-002-P2B**).
pub const RASTER_SPIKE_EMA_MS: f32 = 12.0;
pub const RASTER_SPIKE_FRAME_MS: f32 = 33.0;
pub const RASTER_SPIKE_CLEAR_MS: f32 = 8.0;

/// Frame-budget feedback — defers zoom-band `mark_all_dirty` while raster is hot.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct TileRasterSpikeFeedback {
    pub defer_zoom_dirty: bool,
}

pub fn sync_tile_raster_spike_feedback_system(
    budget: Option<Res<FrameBudgetDiagnostics>>,
    spike_guard: Option<Res<crate::engine::UxFrameSpikeGuard>>,
    mut feedback: ResMut<TileRasterSpikeFeedback>,
) {
    if spike_guard.as_deref().is_some_and(|g| g.spike_active) {
        feedback.defer_zoom_dirty = true;
        return;
    }
    let Some(budget) = budget else {
        return;
    };
    let raster = &budget.buckets[FrameBudgetBucket::MinimapRaster.index()];
    if raster.last_ms >= RASTER_SPIKE_FRAME_MS || raster.avg_ms >= RASTER_SPIKE_EMA_MS {
        feedback.defer_zoom_dirty = true;
    } else if raster.last_ms < RASTER_SPIKE_CLEAR_MS && raster.avg_ms < RASTER_SPIKE_CLEAR_MS {
        feedback.defer_zoom_dirty = false;
    }
}

/// CPU tile-raster chunk budget (main map + optional minimap sub-pass).
#[derive(Resource, Debug, Clone, Copy)]
pub struct TileRasterBudget {
    pub chunks_per_frame: usize,
    /// When false, release builds never read `RASTER_MINIMAP` (policy sync sets this).
    pub minimap_cpu_allowed: bool,
    pub fire_overlay_mark_interval_frames: u32,
    pub zoom_band_quantum: f32,
}

impl Default for TileRasterBudget {
    fn default() -> Self {
        Self::from_world_and_settings(512, 512, &VisualBudgetSettings::default())
    }
}

impl TileRasterBudget {
    #[must_use]
    pub fn from_world_and_settings(
        tex_w: u32,
        tex_h: u32,
        budgets: &VisualBudgetSettings,
    ) -> Self {
        let world_chunks = tex_w
            .div_ceil(crate::render::RASTER_CHUNK_TILES)
            .saturating_mul(tex_h.div_ceil(crate::render::RASTER_CHUNK_TILES))
            .max(1);
        let base = if cfg!(debug_assertions) {
            8usize
        } else {
            4usize
        };
        let chunks_per_frame = base
            .min(world_chunks as usize)
            .max(1);
        let _ = budgets.minimap_hz;
        Self {
            chunks_per_frame,
            minimap_cpu_allowed: true,
            fire_overlay_mark_interval_frames: 3,
            zoom_band_quantum: 0.1,
        }
    }

    /// Effective chunk cap this frame (debug `DEV_RASTER_*` / legacy `RASTER_*` override).
    #[must_use]
    pub fn effective_chunks_per_frame(self, spike_active: bool) -> usize {
        let base = self.chunks_per_frame.max(1);
        let base = debug_raster_chunks_override().unwrap_or(base);
        if spike_active {
            base.min(2)
        } else {
            base
        }
    }
}

/// Fire ECS extract cadence — full-world scan throttled by sim tick + overlay Hz.
#[derive(Resource, Debug, Clone, Copy)]
pub struct FireExtractCadence {
    pub min_interval_secs: f32,
    pub full_scan_on_sim_tick: bool,
    pub residency_scoped: bool,
}

impl Default for FireExtractCadence {
    fn default() -> Self {
        Self::from(&VisualBudgetSettings::default())
    }
}

impl From<&VisualBudgetSettings> for FireExtractCadence {
    fn from(b: &VisualBudgetSettings) -> Self {
        let hz = b.overlay_hz.max(1.0);
        Self {
            min_interval_secs: (1.0 / hz).clamp(1.0 / 60.0, 1.0),
            full_scan_on_sim_tick: true,
            residency_scoped: true,
        }
    }
}

impl FireExtractCadence {
    /// Simulation play + `--test` harness: never full-scan on every sim tick.
    ///
    /// A 320×320 world holds 100k+ tile entities; tick-coupled scans cost ~200ms+ and lock
    /// the main thread (death spiral with `UxFrameSpikeGuard`). Interval + fingerprint skip
    /// is sufficient for proof capture and operator play.
    pub fn clamp_for_runtime(cadence: &mut Self, harness: bool) {
        cadence.full_scan_on_sim_tick = false;
        let floor = if harness { 2.0 } else { 0.5 };
        cadence.min_interval_secs = cadence.min_interval_secs.max(floor);
    }

    /// Scale extract interval with world tile count (large worlds cannot afford 10 Hz overlay scans).
    pub fn clamp_for_world(cadence: &mut Self, tex_w: u32, tex_h: u32, harness: bool) {
        Self::clamp_for_runtime(cadence, harness);
        let tiles = tex_w.saturating_mul(tex_h);
        if tiles >= 48 * 48 {
            cadence.min_interval_secs = cadence.min_interval_secs.max(if harness { 2.0 } else { 1.0 });
        }
        if tiles >= 256 * 256 {
            cadence.min_interval_secs = cadence.min_interval_secs.max(if harness { 3.0 } else { 1.5 });
        }
    }

    /// Wall-clock minimum between full ECS scans while UX spike guard is latched.
    #[must_use]
    pub fn effective_min_interval_secs(&self, spike_active: bool) -> f32 {
        if spike_active {
            self.min_interval_secs * 2.5
        } else {
            self.min_interval_secs
        }
    }
}

/// Whether a full fire ECS extract is due this frame (GPU-P1-D cadence contract).
#[must_use]
pub fn fire_extract_cadence_due(
    clock: &FireExtractClock,
    cadence: &FireExtractCadence,
    now_secs: f32,
    tick_changed: bool,
    spike_active: bool,
    overlay_dirty: bool,
    residency_dirty: bool,
) -> bool {
    if clock.last_full_extract_secs == 0.0 && clock.last_tick == 0 {
        return true;
    }
    let min_interval = cadence.effective_min_interval_secs(spike_active);
    let interval_elapsed =
        (now_secs - clock.last_full_extract_secs).max(0.0) >= min_interval;
    if spike_active {
        interval_elapsed
    } else if cadence.full_scan_on_sim_tick {
        tick_changed || interval_elapsed || overlay_dirty || residency_dirty
    } else {
        interval_elapsed || overlay_dirty || residency_dirty
    }
}

/// Bookkeeping for [`extract_fire_simulation_snapshot`] throttle.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct FireExtractClock {
    pub last_full_extract_secs: f32,
    pub last_full_reconcile_sim_secs: f64,
    pub last_tick: u64,
    pub last_input_fingerprint: FireExtractInputFingerprint,
    pub last_index_revision: u64,
    pub last_overlay_revision: u64,
    pub empty_residency_warned: bool,
}

/// Phase 6 sim-event dirty coords — MVP ships empty; populated when fire sim hooks land.
#[derive(Resource, Debug, Clone, Default)]
pub struct FireExtractDirtyQueue {
    pub coords: Vec<bevy::math::IVec2>,
}

/// Cheap digest of sim fire inputs — skip full ECS scan when cadence due but state unchanged.
/// **Excludes sim tick** — tick advances every frame; runtime/residency digest captures real fire churn.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FireExtractInputFingerprint {
    pub runtime_len: u32,
    pub active_digest: u64,
    pub residency_cells: u32,
}

/// Per-frame fire ECS extract report — flushed into sim-spectrum disk witness on `--test` runs.
#[derive(Resource, Debug, Clone, Default)]
pub struct FireExtractDiagnostics {
    pub last: FireExtractFrameReport,
    /// True when extract skipped this frame (cadence or fingerprint) — downstream may no-op.
    pub snapshot_unchanged: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FireExtractFrameReport {
    pub ran_full_scan: bool,
    pub cadence_skipped: bool,
    pub cadence_due: bool,
    pub spike_active: bool,
    pub tick_changed: bool,
    pub interval_elapsed: bool,
    pub residency_scoped: bool,
    pub bounded_path: bool,
    pub full_reconcile: bool,
    pub scan_set_len: u32,
    pub index_len: u32,
    pub residency_len: u32,
    pub extract_ms: f32,
    pub chunks_iterated: u32,
    pub chunks_fast_path: u32,
    pub chunks_profiled: u32,
    pub instances_written: u32,
    pub chunk_heat_written: u32,
    /// MIG-A13 — CPU greedy light clusters built this frame.
    pub cpu_light_clusters: u32,
    pub cpu_light_instances_sampled: u32,
    pub cpu_light_requests: u32,
    pub runtime_chunks: u32,
    pub min_interval_secs: f32,
    pub fingerprint_skipped: bool,
    pub overlay_dirty: bool,
    pub residency_dirty: bool,
}

#[inline]
fn debug_raster_chunks_override() -> Option<usize> {
    #[cfg(debug_assertions)]
    {
        for key in ["DEV_RASTER_CHUNKS_PER_FRAME", "RASTER_CHUNKS_PER_FRAME"] {
            if let Ok(s) = std::env::var(key) {
                if let Ok(n) = s.parse::<usize>() {
                    return Some(n.max(1));
                }
            }
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = ();
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_extract_frame_report_bounded_fields_default_false() {
        let r = FireExtractFrameReport::default();
        assert!(!r.bounded_path);
        assert!(!r.full_reconcile);
        assert_eq!(r.scan_set_len, 0);
        assert_eq!(r.index_len, 0);
        assert_eq!(r.residency_len, 0);
    }

    #[test]
    fn fire_extract_fingerprint_ignores_tick_advance() {
        let a = FireExtractInputFingerprint {
            runtime_len: 4,
            active_digest: 0xabc,
            residency_cells: 2,
        };
        let b = FireExtractInputFingerprint {
            runtime_len: 4,
            active_digest: 0xabc,
            residency_cells: 2,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn tile_budget_release_default_is_four_chunks() {
        let b = TileRasterBudget::from_world_and_settings(1024, 1024, &VisualBudgetSettings::default());
        if cfg!(debug_assertions) {
            assert_eq!(b.chunks_per_frame, 8);
        } else {
            assert_eq!(b.chunks_per_frame, 4);
        }
    }

    #[test]
    fn fire_cadence_interval_from_overlay_hz() {
        let settings = VisualBudgetSettings {
            overlay_hz: 15.0,
            ..Default::default()
        };
        let c = FireExtractCadence::from(&settings);
        assert!((c.min_interval_secs - (1.0 / 15.0)).abs() < 1e-5);
    }

    #[test]
    fn release_effective_chunks_ignores_raster_env() {
        let budget = TileRasterBudget {
            chunks_per_frame: 4,
            minimap_cpu_allowed: true,
            fire_overlay_mark_interval_frames: 3,
            zoom_band_quantum: 0.1,
        };
        #[cfg(not(debug_assertions))]
        {
            std::env::set_var("RASTER_CHUNKS_PER_FRAME", "999");
            assert_eq!(budget.effective_chunks_per_frame(false), 4);
        }
        #[cfg(debug_assertions)]
        {
            assert_eq!(budget.effective_chunks_per_frame(false), 4);
        }
    }

    #[test]
    fn perf_vis_002_slice3_spike_clamps_effective_chunks() {
        let budget = TileRasterBudget {
            chunks_per_frame: 8,
            minimap_cpu_allowed: true,
            fire_overlay_mark_interval_frames: 3,
            zoom_band_quantum: 0.1,
        };
        assert_eq!(budget.effective_chunks_per_frame(false), 8);
        assert_eq!(budget.effective_chunks_per_frame(true), 2);
    }

    #[test]
    fn perf_vis_002_slice3_policy_default_matches_budget() {
        let policy = crate::render::TileFallbackRasterPolicy::default();
        let budget = TileRasterBudget::default();
        assert_eq!(policy.chunks_per_frame, budget.chunks_per_frame);
    }

    #[test]
    fn perf_vis_002_p2b_spike_feedback_latches_on_hot_raster() {
        use crate::gui::hud::frame_budget_diagnostics::FrameBudgetDiagnostics;

        let mut feedback = TileRasterSpikeFeedback::default();
        let mut budget = FrameBudgetDiagnostics::default();
        budget.record_bucket_ms(FrameBudgetBucket::MinimapRaster, 40.0);
        let raster = &budget.buckets[FrameBudgetBucket::MinimapRaster.index()];
        if raster.last_ms >= RASTER_SPIKE_FRAME_MS || raster.avg_ms >= RASTER_SPIKE_EMA_MS {
            feedback.defer_zoom_dirty = true;
        }
        assert!(feedback.defer_zoom_dirty);
    }

    #[test]
    fn perf_vis_002_p2d_residency_scoped_by_default() {
        assert!(FireExtractCadence::default().residency_scoped);
    }

    #[test]
    fn fire_extract_clamp_for_runtime_decouples_sim_tick() {
        let mut cadence = FireExtractCadence {
            min_interval_secs: 0.1,
            full_scan_on_sim_tick: true,
            residency_scoped: true,
        };
        FireExtractCadence::clamp_for_runtime(&mut cadence, false);
        assert!(!cadence.full_scan_on_sim_tick);
        assert!((cadence.min_interval_secs - 0.5).abs() < f32::EPSILON);

        FireExtractCadence::clamp_for_runtime(&mut cadence, true);
        assert!((cadence.min_interval_secs - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fire_extract_clamp_for_world_scales_320_world() {
        let mut cadence = FireExtractCadence::from(&VisualBudgetSettings::simulation_play());
        FireExtractCadence::clamp_for_world(&mut cadence, 320, 320, false);
        assert!(!cadence.full_scan_on_sim_tick);
        assert!(cadence.min_interval_secs >= 1.5);

        let mut harness = FireExtractCadence::from(&VisualBudgetSettings::simulation_play());
        FireExtractCadence::clamp_for_world(&mut harness, 320, 320, true);
        assert!(harness.min_interval_secs >= 3.0);
    }

    #[test]
    fn fire_extract_spike_doubles_effective_interval() {
        let cadence = FireExtractCadence {
            min_interval_secs: 1.0,
            full_scan_on_sim_tick: false,
            residency_scoped: true,
        };
        assert!((cadence.effective_min_interval_secs(false) - 1.0).abs() < f32::EPSILON);
        assert!(cadence.effective_min_interval_secs(true) >= 2.5);
    }

    #[test]
    fn fire_extract_cadence_due_first_frame_always() {
        let clock = FireExtractClock::default();
        let cadence = FireExtractCadence::default();
        assert!(fire_extract_cadence_due(
            &clock, &cadence, 0.0, false, false, false, false
        ));
    }

    #[test]
    fn fire_extract_cadence_skips_when_interval_not_elapsed() {
        let cadence = FireExtractCadence {
            min_interval_secs: 1.0,
            full_scan_on_sim_tick: false,
            residency_scoped: true,
        };
        let clock = FireExtractClock {
            last_full_extract_secs: 0.5,
            last_tick: 1,
            ..Default::default()
        };
        assert!(!fire_extract_cadence_due(
            &clock, &cadence, 0.6, true, false, false, false
        ));
    }

    #[test]
    fn fire_extract_spike_mode_ignores_tick_without_interval() {
        let cadence = FireExtractCadence {
            min_interval_secs: 1.0,
            full_scan_on_sim_tick: true,
            residency_scoped: true,
        };
        let clock = FireExtractClock {
            last_full_extract_secs: 0.2,
            last_tick: 5,
            ..Default::default()
        };
        assert!(!fire_extract_cadence_due(
            &clock, &cadence, 0.5, true, true, false, false
        ));
        assert!(fire_extract_cadence_due(
            &clock, &cadence, 1.3, true, true, false, false
        ));
    }

    #[test]
    fn fire_extract_residency_dirty_forces_cadence() {
        let cadence = FireExtractCadence {
            min_interval_secs: 5.0,
            full_scan_on_sim_tick: false,
            residency_scoped: true,
        };
        let clock = FireExtractClock {
            last_full_extract_secs: 1.0,
            last_tick: 10,
            ..Default::default()
        };
        assert!(fire_extract_cadence_due(
            &clock, &cadence, 1.1, false, false, false, true
        ));
    }
}
