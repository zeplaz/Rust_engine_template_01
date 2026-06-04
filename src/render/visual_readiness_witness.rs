//! Operational visual readiness witness — decouples frame budget / viewport health from spine `passes`.
//!
//! **VISUAL-STALL-SURFACE-001:** `readiness.passes` does not imply healthy frame time or swapchain size.

use bevy::prelude::*;

use crate::gui::hud::FrameBudgetDiagnostics;
use crate::gui::{
    bootstrap_authoritative_viewport_on_enter_simulation, reset_main_world_camera_viewport_latch_on_enter_simulation,
    MainWorldCameraViewportLatch, SimulationMapViewport,
};
use crate::render::{
    primary_window_logical_presentable, reset_perf_attribution_witness_on_enter_simulation,
    sync_perf_attribution_witness_system, FrameStallWatch, PerfAttributionWitness, ResolvedViewports,
};

/// Rolling witness for visual / perf health (written to `stage5_full_app_live.json`).
#[derive(Resource, Clone, Debug, Default)]
pub struct VisualReadinessWitness {
    pub sim_valid_streak: u32,
    pub primary_window_wh: Vec2,
    pub primary_window_presentable: bool,
    pub frame_budget_avg_ms: f32,
    pub frame_budget_max_ms: f32,
    pub frame_budget_last_ms: f32,
    /// PostUpdate VT/CI → readiness → pre-egui (uninstrumented gap attribution).
    pub post_vt_to_pre_egui_ms: f32,
    /// Ship policy cap from [`crate::render::TileRasterBudget`] (PERF-VIS-002 Slice 3).
    pub tile_raster_budget_chunks: usize,
    pub tile_raster_effective_chunks: usize,
    pub tile_raster_last_ms: f32,
    pub tile_raster_minimap_cpu_allowed: bool,
    pub frames_sampled: u64,
    /// Steady-state hole toggles after Simulation bootstrap (PERF-VIS-003).
    pub render_hole_steady_flip_count: u32,
    pub p95_frame_ms: f32,
    pub p95_raster_b_ms: f32,
    pub p95_view_fire_ms: f32,
    pub perf_window_samples: usize,
}

impl VisualReadinessWitness {
    #[must_use]
    pub fn soft_healthy(&self) -> bool {
        self.primary_window_presentable
            && self.sim_valid_streak >= 1
            && self.frame_budget_last_ms < 250.0
            && self.render_hole_steady_flip_count == 0
    }
}

#[must_use]
pub fn visual_readiness_witness_json(witness: &VisualReadinessWitness) -> serde_json::Value {
    serde_json::json!({
        "sim_valid_streak": witness.sim_valid_streak,
        "primary_window_wh": {
            "x": witness.primary_window_wh.x,
            "y": witness.primary_window_wh.y,
        },
        "primary_window_presentable": witness.primary_window_presentable,
        "frame_budget": {
            "avg_ms": witness.frame_budget_avg_ms,
            "max_ms": witness.frame_budget_max_ms,
            "last_frame_ms": witness.frame_budget_last_ms,
        },
        "post_vt_to_pre_egui_ms": witness.post_vt_to_pre_egui_ms,
        "tile_raster_budget": {
            "chunks_per_frame": witness.tile_raster_budget_chunks,
            "effective_chunks_per_frame": witness.tile_raster_effective_chunks,
            "last_raster_ms": witness.tile_raster_last_ms,
            "minimap_cpu_allowed": witness.tile_raster_minimap_cpu_allowed,
        },
        "viewport": {
            "render_hole_steady_flip_count": witness.render_hole_steady_flip_count,
        },
        "perf_attribution_60s": {
            "window_samples": witness.perf_window_samples,
            "p95_frame_ms": witness.p95_frame_ms,
            "p95_raster_b_ms": witness.p95_raster_b_ms,
            "p95_view_fire_ms": witness.p95_view_fire_ms,
        },
        "soft_healthy": witness.soft_healthy(),
        "frames_sampled": witness.frames_sampled,
    })
}

/// Lib / disk refresh fixture — non-zero perf attribution without a visual run.
#[must_use]
pub fn visual_readiness_witness_lib_fixture() -> VisualReadinessWitness {
    VisualReadinessWitness {
        sim_valid_streak: 12,
        primary_window_wh: Vec2::new(1280.0, 720.0),
        primary_window_presentable: true,
        frame_budget_avg_ms: 18.0,
        frame_budget_max_ms: 42.0,
        frame_budget_last_ms: 20.0,
        post_vt_to_pre_egui_ms: 4.5,
        tile_raster_budget_chunks: 4,
        tile_raster_effective_chunks: 4,
        tile_raster_last_ms: 6.0,
        tile_raster_minimap_cpu_allowed: false,
        frames_sampled: 120,
        render_hole_steady_flip_count: 0,
        p95_frame_ms: 22.0,
        p95_raster_b_ms: 8.0,
        p95_view_fire_ms: 4.0,
        perf_window_samples: 120,
    }
}

pub fn reset_visual_readiness_witness_on_enter_simulation(mut witness: ResMut<VisualReadinessWitness>) {
    *witness = VisualReadinessWitness::default();
}

pub fn sync_visual_readiness_witness_system(
    sim: Res<SimulationMapViewport>,
    resolved: Res<ResolvedViewports>,
    budget: Option<Res<FrameBudgetDiagnostics>>,
    stall: Option<Res<FrameStallWatch>>,
    raster_budget: Option<Res<crate::render::TileRasterBudget>>,
    raster_policy: Option<Res<crate::render::TileFallbackRasterPolicy>>,
    perf: Option<Res<crate::render::FramePerf>>,
    perf_attrib: Option<Res<PerfAttributionWitness>>,
    cam_latch: Option<Res<MainWorldCameraViewportLatch>>,
    mut witness: ResMut<VisualReadinessWitness>,
) {
    if sim.valid {
        witness.sim_valid_streak = witness.sim_valid_streak.saturating_add(1);
    } else {
        witness.sim_valid_streak = 0;
    }
    witness.primary_window_wh = resolved.primary_window.logical_size;
    witness.primary_window_presentable = resolved.primary_window.valid
        && primary_window_logical_presentable(
            witness.primary_window_wh.x,
            witness.primary_window_wh.y,
        );
    if let Some(b) = budget.as_deref() {
        witness.frame_budget_avg_ms = b.avg_frame_ms;
        witness.frame_budget_max_ms = b.max_frame_ms;
        witness.frame_budget_last_ms = b.frame_time_ms;
    }
    if let Some(s) = stall.as_deref() {
        let sp = &s.spans;
        witness.post_vt_to_pre_egui_ms = sp.post_vt_to_pre_egui_ms;
    }
    if let Some(b) = raster_budget.as_deref() {
        witness.tile_raster_budget_chunks = b.chunks_per_frame;
        witness.tile_raster_minimap_cpu_allowed = b.minimap_cpu_allowed;
    }
    if let Some(p) = raster_policy.as_deref() {
        witness.tile_raster_effective_chunks = p.chunks_per_frame;
    }
    if let Some(p) = perf.as_deref() {
        witness.tile_raster_last_ms = if p.tile_raster_ran {
            p.tile_raster_ms
        } else {
            0.0
        };
    }
    if let Some(p) = perf_attrib.as_deref() {
        witness.p95_frame_ms = p.p95_frame_ms();
        witness.p95_raster_b_ms = p.p95_raster_b_ms();
        witness.p95_view_fire_ms = p.p95_view_fire_ms();
        witness.perf_window_samples = p.window_samples();
    }
    if let Some(l) = cam_latch.as_deref() {
        witness.render_hole_steady_flip_count = l.steady_flip_count;
    }
    witness.frames_sampled = witness.frames_sampled.saturating_add(1);
}

pub struct VisualReadinessWitnessPlugin;

impl Plugin for VisualReadinessWitnessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisualReadinessWitness>()
            .init_resource::<PerfAttributionWitness>()
            .add_systems(
                OnEnter(crate::engine::states::BaseState::Simulation),
                (
                    reset_visual_readiness_witness_on_enter_simulation,
                    reset_perf_attribution_witness_on_enter_simulation,
                    reset_main_world_camera_viewport_latch_on_enter_simulation,
                    bootstrap_authoritative_viewport_on_enter_simulation,
                )
                    .chain(),
            )
            .add_systems(
                Last,
                (
                    sync_perf_attribution_witness_system
                        .after(crate::gui::hud::finalize_frame_budget_diagnostics),
                    sync_visual_readiness_witness_system
                        .after(sync_perf_attribution_witness_system)
                        .after(crate::render::stall_watch::stall_pre_egui),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{perf_attribution_witness_json, PerfAttributionWitness, PERF_ATTRIBUTION_WINDOW};

    #[test]
    fn witness_json_includes_frame_budget_and_streak() {
        let w = VisualReadinessWitness {
            sim_valid_streak: 12,
            primary_window_wh: Vec2::new(1280.0, 720.0),
            primary_window_presentable: true,
            frame_budget_avg_ms: 18.0,
            frame_budget_max_ms: 42.0,
            frame_budget_last_ms: 20.0,
            post_vt_to_pre_egui_ms: 63.5,
            tile_raster_budget_chunks: 4,
            tile_raster_effective_chunks: 2,
            tile_raster_last_ms: 8.5,
            tile_raster_minimap_cpu_allowed: false,
            frames_sampled: 100,
            render_hole_steady_flip_count: 0,
            p95_frame_ms: 22.0,
            p95_raster_b_ms: 8.0,
            p95_view_fire_ms: 4.0,
            perf_window_samples: 120,
        };
        let j = visual_readiness_witness_json(&w);
        assert_eq!(j["sim_valid_streak"], 12);
        assert_eq!(j["primary_window_wh"]["x"], 1280.0);
        assert!(j["frame_budget"]["avg_ms"].as_f64().unwrap() > 0.0);
        assert_eq!(j["tile_raster_budget"]["chunks_per_frame"], 4);
        assert_eq!(j["perf_attribution_60s"]["p95_frame_ms"], 22.0);
        assert!(j["soft_healthy"].as_bool().unwrap());
    }

    #[test]
    fn perf_attribution_json_matches_witness_block() {
        let mut p = PerfAttributionWitness::default();
        for _ in 0..50 {
            p.record_frame(16.0, 4.0, 2.0);
        }
        let j = perf_attribution_witness_json(&p);
        assert_eq!(j["window_target_frames"], PERF_ATTRIBUTION_WINDOW);
        assert!(j["p95_frame_ms"].as_f64().unwrap() > 0.0);
    }
}
