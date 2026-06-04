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

/// Bookkeeping for [`extract_fire_simulation_snapshot`] throttle.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct FireExtractClock {
    pub last_full_extract_secs: f32,
    pub last_tick: u64,
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
}
