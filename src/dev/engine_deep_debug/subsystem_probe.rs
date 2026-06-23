//! Subsystem isolation + visual memory queue snapshots for deep debug witnesses.

use bevy::prelude::*;
use serde_json::{json, Value};

use crate::engine::launch_args::EngineLaunchArgs;
use crate::engine::states::BaseState;
use crate::engine::UxFrameSpikeGuard;
use crate::gui::hud::frame_budget_diagnostics::{FrameBudgetBucket, FrameBudgetDiagnostics};
use crate::gui::{MapViewInstances, MinimapOverlayMask, VisualBudgetSettings, VisualCadence};
use crate::render::minimap_compositor::{
    diagnostics_json_snapshot, MinimapGpuCompositorDiagnostics, MinimapRenderTargetRegistry,
};
use crate::render::{TileRasterBudget, TileRasterSpikeFeedback};

fn overlay_mask_json(mask: &MinimapOverlayMask) -> Value {
    json!({
        "fire_heat": mask.fire_heat,
        "logistics_heat": mask.logistics_heat,
        "construction_heat": mask.construction_heat,
        "ecology_heat": mask.ecology_heat,
        "fow": mask.fow,
        "ew": mask.ew,
        "units": mask.units,
        "replay_scrub": mask.replay_scrub,
    })
}

#[must_use]
pub fn subsystem_isolation_snapshot(
    base_state: Option<&State<BaseState>>,
    launch: Option<&EngineLaunchArgs>,
    budgets: Option<&VisualBudgetSettings>,
    cadence: Option<&VisualCadence>,
    raster_budget: Option<&TileRasterBudget>,
    raster_feedback: Option<&TileRasterSpikeFeedback>,
    spike: Option<&UxFrameSpikeGuard>,
    map_views: &MapViewInstances,
    shell_queue: Option<Value>,
) -> Value {
    let spike_active = spike.is_some_and(|g| g.spike_active);
    let effective_chunks = raster_budget
        .map(|b| b.effective_chunks_per_frame(spike_active))
        .unwrap_or(0);

    json!({
        "base_state": base_state.map(|s| format!("{:?}", s.get())),
        "witness_lane_active": launch.is_some_and(|l| l.test_mode()),
        "visual_budgets": budgets.map(|b| json!({
            "preview_hz": b.preview_hz,
            "minimap_hz": b.minimap_hz,
            "atmosphere_hz": b.atmosphere_hz,
            "overlay_hz": b.overlay_hz,
        })),
        "visual_cadence": cadence.map(|c| json!({
            "preview_hz": c.preview_hz,
            "minimap_hz": c.minimap_hz,
            "atmosphere_hz": c.atmosphere_hz,
            "overlay_hz": c.overlay_hz,
        })),
        "tile_raster": raster_budget.map(|b| json!({
            "chunks_per_frame": b.chunks_per_frame,
            "effective_chunks_per_frame": effective_chunks,
            "minimap_cpu_allowed": b.minimap_cpu_allowed,
            "fire_overlay_mark_interval_frames": b.fire_overlay_mark_interval_frames,
            "zoom_band_quantum": b.zoom_band_quantum,
        })),
        "tile_raster_feedback": raster_feedback.map(|f| json!({
            "defer_zoom_dirty": f.defer_zoom_dirty,
        })),
        "spike_guard": spike.map(|g| json!({
            "spike_active": g.spike_active,
            "last_frame_ms": g.last_frame_ms,
            "max_ms": g.max_ms,
            "suppress_preview_this_frame": g.suppress_preview_this_frame,
            "suppress_optional_diagnostics": g.suppress_optional_diagnostics,
            "over_budget_streak": g.spike_over_budget_streak,
        })),
        "minimap_overlay_mask": overlay_mask_json(&map_views.minimap.overlays),
        "minimap_view_revision": map_views.minimap.revision,
        "shell_refresh_queue": shell_queue,
    })
}

#[must_use]
pub fn visual_memory_queue_snapshot(
    image_count: usize,
    frame_budget: Option<&FrameBudgetDiagnostics>,
    gpu_diag: &MinimapGpuCompositorDiagnostics,
    registry: &MinimapRenderTargetRegistry,
) -> Value {
    let w = registry.committed_size.x.max(1);
    let h = registry.committed_size.y.max(1);
    let px = u64::from(w) * u64::from(h);
    // fire + logistics + construction + ecology + fow + ew heat (RGBA8) + committed RT.
    let heat_layers = 6u64;
    let heat_est_bytes = px.saturating_mul(4).saturating_mul(heat_layers);
    let rt_est_bytes = px.saturating_mul(4);

    let buckets: Vec<Value> = frame_budget
        .map(|fb| {
            FrameBudgetBucket::ALL
                .iter()
                .map(|bucket| {
                    let stats = fb.buckets[bucket.index()];
                    json!({
                        "bucket": bucket.label(),
                        "last_ms": stats.last_ms,
                        "avg_ms": stats.avg_ms,
                        "max_ms": stats.max_ms,
                        "events_last_frame": stats.events_last_frame,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let mut body = json!({
        "asset_images": image_count,
        "minimap_rt_est_bytes": rt_est_bytes,
        "minimap_heat_est_bytes": heat_est_bytes,
        "minimap_rt_size": [w, h],
        "gpu_compositor_queue": diagnostics_json_snapshot(gpu_diag),
    });

    if let Some(fb) = frame_budget {
        body["frame"] = json!({
            "frame_index": fb.frame_index,
            "frame_time_ms": fb.frame_time_ms,
            "avg_frame_ms": fb.avg_frame_ms,
            "max_frame_ms": fb.max_frame_ms,
            "upload_bytes_frame": fb.upload_bytes_frame,
            "upload_bytes_per_sec": fb.upload_bytes_per_sec,
            "texture_rebuilds_frame": fb.texture_rebuilds_frame,
            "texture_rebinds_frame": fb.texture_rebinds_frame,
        });
        body["stage6_virtualization"] = json!({
            "active_residency_cells": fb.stage6.active_residency_cells,
            "upload_bytes_frame": fb.stage6.upload_bytes_frame,
            "atlas_pressure": fb.stage6.atlas_pressure,
            "dirty_region_count": fb.stage6.dirty_region_count,
            "overlay_update_count": fb.stage6.overlay_update_count,
        });
        body["bucket_queues"] = json!(buckets);
        if let Some(anomaly) = &fb.last_anomaly {
            body["last_anomaly"] = json!({
                "kind": format!("{:?}", anomaly.kind),
                "detail": anomaly.detail,
                "suppressed": anomaly.suppressed,
            });
        }
    }

    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsystem_probe_emits_overlay_and_cadence_fields() {
        let map_views = MapViewInstances::default();
        let cadence = VisualCadence::from(&VisualBudgetSettings::simulation_play());
        let body = subsystem_isolation_snapshot(
            None,
            None,
            Some(&VisualBudgetSettings::simulation_play()),
            Some(&cadence),
            None,
            None,
            None,
            &map_views,
            None,
        );
        assert_eq!(body["visual_cadence"]["minimap_hz"], json!(6.0));
        assert_eq!(body["minimap_overlay_mask"]["logistics_heat"], json!(true));
    }
}
