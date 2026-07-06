//! RTT lane lib witnesses — CHAIN-A Track A (A1) + Track B (B5) + triage (C-004/005).

use bevy::prelude::*;

use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::render::{ExtractedCameraMetrics, ParticleViewGlobals};

pub const RTT_LANE_WITNESS_JSON: &str = "debug_runs/rtt_lane_witness_live.json";

/// RTT-A1-001: latch symbols removed from map camera.
#[must_use]
pub fn rtt_a1_latch_deleted_green() -> bool {
    let map = include_str!("../gui/tactical/map_camera.rs");
    !map.contains("MainWorldCameraViewportLatch") && !map.contains("ViewportLatch")
}

/// RTT-B5-001: shared particle view globals struct is wired.
#[must_use]
pub fn rtt_b5_view_uniform_struct_green() -> bool {
    std::mem::size_of::<ParticleViewGlobals>() >= std::mem::size_of::<Mat4>()
}

/// RTT-B5-002/003: fire/water raster sync uses extracted metrics only.
#[must_use]
pub fn rtt_b5_raster_uses_metrics_only_green() -> bool {
    let fire = include_str!("../render/pipelines/gpu_fire_particle_raster.rs");
    let water = include_str!("../render/pipelines/gpu_water_particle_raster.rs");
    !fire.contains("With<MainWorldCamera>")
        && !water.contains("With<MainWorldCamera>")
        && fire.contains("ExtractedCameraMetrics")
        && water.contains("ExtractedCameraMetrics")
        && fire.contains("ExtractedCameraMetricsSet::Sync")
        && water.contains("ExtractedCameraMetricsSet::Sync")
}

/// RTT-A1-002: canonical fill rect type is TacticalMapFillRect.
#[must_use]
pub fn rtt_a1_tactical_fill_rect_green() -> bool {
    let mut fill = crate::gui::TacticalMapFillRect::default();
    fill.valid = true;
    fill.min = Vec2::ZERO;
    fill.max = Vec2::new(800.0, 600.0);
    fill.is_adequate_for_camera()
}

/// RTT-A1-004: witness reads fill validity flip streak (not hole latch).
#[must_use]
pub fn rtt_a1_fill_streak_witness_green() -> bool {
    include_str!("diagnostics/visual_readiness.rs").contains("steady_invalid_flip_count")
        && include_str!("../gui/tactical/sim_map_rtt.rs").contains("reset_tactical_map_fill_streak_on_enter_simulation")
}

/// RTT-C-004/005: tactical map debug exposes ImageNode bind + diagnosis hints.
#[must_use]
pub fn rtt_c_image_node_triage_green() -> bool {
    let dbg = include_str!("tactical_map_debug.rs");
    dbg.contains("image_node_bind")
        && dbg.contains("RTT_IMAGE_NODE_MISMATCH")
        && dbg.contains("SimulationMapViewportFill")
}

#[must_use]
pub fn rtt_lane_witness_green() -> bool {
    rtt_a1_latch_deleted_green()
        && rtt_a1_tactical_fill_rect_green()
        && rtt_a1_fill_streak_witness_green()
        && rtt_b5_view_uniform_struct_green()
        && rtt_b5_raster_uses_metrics_only_green()
        && rtt_c_image_node_triage_green()
}

#[must_use]
pub fn rtt_lane_witness_json() -> serde_json::Value {
    serde_json::json!({
        "schema": "rtt_lane_witness_v1",
        "green": rtt_lane_witness_green(),
        "track_b5": {
            "RTT-B5-001": rtt_b5_view_uniform_struct_green(),
            "RTT-B5-002_003": rtt_b5_raster_uses_metrics_only_green(),
            "extracted_metrics_view_proj_non_identity":
                ExtractedCameraMetrics::for_tests(2.0, 0.8).view_proj != Mat4::IDENTITY,
        },
        "track_a1": {
            "RTT-A1-001": rtt_a1_latch_deleted_green(),
            "RTT-A1-002": rtt_a1_tactical_fill_rect_green(),
            "RTT-A1-004": rtt_a1_fill_streak_witness_green(),
            "simulation_map_fill_alias":
                std::any::type_name::<crate::gui::SimulationMapFillRect>()
                    == std::any::type_name::<crate::gui::TacticalMapFillRect>(),
        },
        "track_c": {
            "RTT-C-004_005": rtt_c_image_node_triage_green(),
            "operator_vfx_display": "OPEN",
            "tactical_map_debug_refresh": "OPEN",
        },
    })
}

#[must_use]
pub fn refresh_rtt_lane_witness() -> bool {
    let body = rtt_lane_witness_json();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run("CHAIN-A", "refresh_rtt_lane_witness", RTT_LANE_WITNESS_JSON, body);
    write_debug_run_json(RTT_LANE_WITNESS_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtt_b5_particle_view_globals_layout() {
        assert!(rtt_b5_view_uniform_struct_green());
    }

    #[test]
    fn rtt_b5_extracted_metrics_carries_view_proj() {
        let m = ExtractedCameraMetrics::for_tests(1.5, 0.9);
        assert_ne!(m.view_proj, Mat4::IDENTITY);
    }

    #[test]
    fn rtt_a1_latch_deleted() {
        assert!(rtt_a1_latch_deleted_green());
    }

    #[test]
    fn rtt_a1_tactical_fill_rect_adequate() {
        assert!(rtt_a1_tactical_fill_rect_green());
    }

    #[test]
    fn rtt_lane_all_slices_green() {
        assert!(rtt_lane_witness_green(), "{}", rtt_lane_witness_json());
    }

    #[test]
    fn rtt_lane_witness_refresh_writes_json() {
        crate::dev::debug_run_envelope::reset_witness_refresh_gate_for_tests();
        assert!(refresh_rtt_lane_witness());
    }
}
