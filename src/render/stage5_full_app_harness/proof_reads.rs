//! RGR-H3-001 split — `Stage5FullAppLiveProofReads` SystemParam + small presentation-label
//! helpers used when assembling the FULL_APP live proof payload.
//! Carved verbatim from `stage5_full_app_harness.rs` (pre-split monolith).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::gui::editor::world_preview::{
    PreviewCameraState, PreviewPathAuthority, WorldPreviewRenderTargetRegistry,
    WorldPreviewRenderViewportContract, WorldPreviewUiState, WorldPreviewViewportAuthority,
};
use crate::gui::{
    MapTextureSource, MapPresentationDiagnostics, MapViewInstanceId, MapViewInstances,
    MapViewPresentationStates, MapViewTextureCache, MapFitValidationLog, ResolvedMapViewFrames,
    MinimapPresentationSource, MinimapShellState, SimulationMapViewport,
    ViewRepresentationSnapshot,
};
use crate::render::extraction::RenderProjectionGraph;
use crate::render::gpu_indirect_draw::GpuIndirectDrawSpine;
use crate::render::gpu_particle_draw::WorldFireParticleDrawDispatch;
use crate::render::gpu_particles::WorldFireParticleFrame;
use crate::render::gpu_water_particles::WorldWaterParticleFrame;
use crate::render::phase_f_lod_proof::PhaseFLodProofReport;
use crate::render::viewport_pipeline::{ResolvedViewports, ViewportPresentationMismatch};
use crate::render::overlay_field_buffers::SharedOverlayFieldBuffers;
use crate::render::{
    build_minimap_compositor_proof_payload_with_tray, minimap_gpu_compositor_env_enabled,
    MinimapGpuCompositorDiagnostics, MinimapCompositorState, MinimapRenderTargetRegistry,
};
use crate::render::GpuRepresentationMetrics;
use crate::render::WaterSurfaceVisualCatalog;
use crate::gui::hud::HudOverlayTrayState;
use crate::dev::{Stage5FinishTodoBoard, Stage5FinishUx06Streak, Stage5LiveTodoBoard, TodoStatus, STAGE5_FINISH_TODOS, STAGE5_TODOS};
use crate::systems::sim_control::{SimTick, SimTimeMicros};

#[derive(SystemParam)]
pub(crate) struct Stage5FullAppLiveProofReads<'w> {
    pub(super) sim_tick: Res<'w, SimTick>,
    pub(super) sim_time: Res<'w, SimTimeMicros>,
    pub(super) eval_inv: Res<'w, crate::render::Stage5ReadinessEvalInvocation>,
    pub(super) visual_fence: Res<'w, crate::render::CommittedVisualSnapshotFence>,
    pub(super) resolved: Res<'w, ResolvedViewports>,
    pub(super) viewport_mismatch: Res<'w, ViewportPresentationMismatch>,
    pub(super) preview_authority: Res<'w, WorldPreviewViewportAuthority>,
    pub(super) preview_path: Res<'w, PreviewPathAuthority>,
    pub(super) render_contract: Res<'w, WorldPreviewRenderViewportContract>,
    pub(super) render_registry: Res<'w, WorldPreviewRenderTargetRegistry>,
    pub(super) view_snapshot: Res<'w, ViewRepresentationSnapshot>,
    pub(super) preview_ui: Res<'w, WorldPreviewUiState>,
    pub(super) preview_cam: Res<'w, PreviewCameraState>,
    pub(super) minimap: Res<'w, MinimapShellState>,
    pub(super) minimap_registry: Option<Res<'w, MinimapRenderTargetRegistry>>,
    pub(super) minimap_compositor: Option<Res<'w, MinimapCompositorState>>,
    pub(super) minimap_gpu_diagnostics: Option<Res<'w, MinimapGpuCompositorDiagnostics>>,
    pub(super) terrain_authority: Option<Res<'w, crate::render::TerrainRenderAuthority>>,
    pub(super) sim_map: Res<'w, SimulationMapViewport>,
    pub(super) policy: Option<Res<'w, crate::gui::RepresentationResult>>,
    pub(super) projection: Option<Res<'w, RenderProjectionGraph>>,
    pub(super) metrics: Option<Res<'w, GpuRepresentationMetrics>>,
    pub(super) phase_f: Option<Res<'w, PhaseFLodProofReport>>,
    pub(super) indirect: Option<Res<'w, GpuIndirectDrawSpine>>,
    pub(super) draw: Option<Res<'w, WorldFireParticleDrawDispatch>>,
    pub(super) particles: Option<Res<'w, WorldFireParticleFrame>>,
    pub(super) water_catalog: Option<Res<'w, WaterSurfaceVisualCatalog>>,
    pub(super) water_particles: Option<Res<'w, WorldWaterParticleFrame>>,
    pub(super) overlay: Option<Res<'w, SharedOverlayFieldBuffers>>,
    pub(super) map_presentation: Res<'w, MapViewPresentationStates>,
    pub(super) map_views: Res<'w, MapViewInstances>,
    pub(super) map_frames: Res<'w, ResolvedMapViewFrames>,
    pub(super) map_texture_cache: Res<'w, MapViewTextureCache>,
    pub(super) map_presentation_diag: Res<'w, MapPresentationDiagnostics>,
    pub(super) map_fit_log: Res<'w, MapFitValidationLog>,
    pub(super) todo_board: Option<Res<'w, Stage5LiveTodoBoard>>,
    pub(super) finish_todo_board: Option<Res<'w, Stage5FinishTodoBoard>>,
    pub(super) finish_ux06_streak: Option<Res<'w, Stage5FinishUx06Streak>>,
    pub(super) view_isolation: Res<'w, crate::gui::ViewIsolationDiagnostics>,
    pub(super) view_projection_authority: Option<Res<'w, crate::render::view_runtime::ViewProjectionAuthority>>,
    pub(super) view_runtime_witness: Option<Res<'w, crate::render::view_runtime::ViewRuntimeWitness>>,
    pub(super) fire_witness: Option<Res<'w, crate::render::Stage5FireViewChunkWitness>>,
    pub(super) fire_playback: Option<Res<'w, crate::render::FirePlaybackStabilityWitness>>,
    pub(super) view_manager: Option<Res<'w, crate::gui::ViewManager>>,
    pub(super) overlay_tray: Option<Res<'w, HudOverlayTrayState>>,
    pub(super) visual_witness: Option<Res<'w, crate::render::VisualReadinessWitness>>,
    pub(super) perf_attribution: Option<Res<'w, crate::render::PerfAttributionWitness>>,
    pub(super) tactical_vector: Option<Res<'w, crate::render::TacticalVectorOverlayState>>,
    pub(super) va2_board: Option<Res<'w, crate::dev::VisualAidV2LiveTodoBoard>>,
    pub(super) va2_witness: Option<Res<'w, crate::dev::VisualAidV2Witness>>,
    pub(super) va2_hud: Option<Res<'w, crate::gui::hud::HudPanelStateWitness>>,
}

pub(super) fn stage5_live_todo_board_snapshot(board: &Stage5LiveTodoBoard) -> serde_json::Value {
    let rows: Vec<serde_json::Value> = STAGE5_TODOS
        .iter()
        .zip(board.status.iter())
        .map(|(row, st)| {
            let status = match st {
                TodoStatus::Open => "Open",
                TodoStatus::InProgress => "InProgress",
                TodoStatus::Done => "Done",
            };
            serde_json::json!({
                "id": row.id,
                "status": status,
            })
        })
        .collect();
    let done = board
        .status
        .iter()
        .filter(|s| **s == TodoStatus::Done)
        .count();
    serde_json::json!({
        "registry_len": STAGE5_TODOS.len(),
        "done_count": done,
        "all_done": done == STAGE5_TODOS.len(),
        "rows": rows,
    })
}

pub(super) fn stage5_finish_todo_board_snapshot(board: &Stage5FinishTodoBoard) -> serde_json::Value {
    let rows: Vec<serde_json::Value> = STAGE5_FINISH_TODOS
        .iter()
        .zip(board.status.iter())
        .map(|(row, st)| {
            let status = match st {
                TodoStatus::Open => "Open",
                TodoStatus::InProgress => "InProgress",
                TodoStatus::Done => "Done",
            };
            serde_json::json!({
                "id": row.id,
                "status": status,
            })
        })
        .collect();
    let done = board
        .status
        .iter()
        .filter(|s| **s == TodoStatus::Done)
        .count();
    serde_json::json!({
        "registry_len": STAGE5_FINISH_TODOS.len(),
        "done_count": done,
        "all_done": done == STAGE5_FINISH_TODOS.len(),
        "rows": rows,
    })
}

pub(super) fn minimap_source_label(source: MinimapPresentationSource) -> &'static str {
    match source {
        MinimapPresentationSource::SharedCpuRaster => "CpuRaster",
        MinimapPresentationSource::SharedRenderTargetImage => "GpuRenderTarget",
    }
}

pub(super) fn minimap_gpu_composite_active(reads: &Stage5FullAppLiveProofReads) -> bool {
    reads.minimap_compositor.as_ref().is_some_and(|c| {
        reads
            .minimap_registry
            .as_ref()
            .is_some_and(|r| r.committed_image != Handle::default() && c.stamp > 0)
    })
}

pub(super) fn minimap_source_label_for_proof(reads: &Stage5FullAppLiveProofReads) -> &'static str {
    if minimap_gpu_compositor_env_enabled() && minimap_gpu_composite_active(reads) {
        "GpuRenderTarget"
    } else {
        minimap_source_label(reads.minimap.presentation_source)
    }
}

pub(super) fn write_minimap_compositor_live_proof_from_reads(reads: &Stage5FullAppLiveProofReads) {
    let Some(compositor) = reads.minimap_compositor.as_ref() else {
        return;
    };
    let Some(registry) = reads.minimap_registry.as_ref() else {
        return;
    };
    let overlay_revision = reads.overlay.as_ref().map(|o| o.revision).unwrap_or(0);
    let diagnostics = reads
        .minimap_gpu_diagnostics
        .as_ref()
        .map(|d| d.as_ref())
        .cloned()
        .unwrap_or_default();
    let body = build_minimap_compositor_proof_payload_with_tray(
        compositor,
        registry,
        &reads.minimap,
        overlay_revision,
        false,
        &diagnostics,
        reads.overlay_tray.as_deref(),
    );
    if !body
        .get("composite_ok")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        info!(
            target: "stage5_full_app_harness",
            stamp = compositor.stamp,
            rt_bound = registry.committed_image != Handle::default(),
            presentation = ?reads.minimap.presentation_source,
            "skipped minimap compositor live proof — composite_ok false"
        );
        return;
    }
    if crate::dev::runtime_witness::commit_minimap_compositor_live_proof(
        compositor,
        registry,
        &reads.minimap,
        overlay_revision,
        false,
        &diagnostics,
        reads.overlay_tray.as_deref(),
    ) {
        info!(
            target: "stage5_full_app_harness",
            path = crate::dev::runtime_witness::MINIMAP_COMPOSITOR_JSON,
            "wrote minimap compositor live proof (FULL_APP finalize)"
        );
    }
}

pub(super) fn map_texture_source_label(source: &MapTextureSource) -> &'static str {
    match source {
        MapTextureSource::GpuRenderTarget(_) => "GpuRenderTarget",
        MapTextureSource::SharedCpuRaster(_) => "SharedCpuRaster",
    }
}

pub(super) fn map_view_consumer_payload(
    id: MapViewInstanceId,
    presentation_aux: &MapViewPresentationStates,
    map_views: &MapViewInstances,
    frames: &ResolvedMapViewFrames,
    cache: &MapViewTextureCache,
    layout: &MapPresentationDiagnostics,
) -> serde_json::Value {
    let frame = frames.get(id);
    let binding = cache.binding(id);
    let (presentation_revision, fit_mode) = match id {
        MapViewInstanceId::WorldPreview => (
            map_views.world_preview.revision,
            map_views.world_preview.fit_mode,
        ),
        MapViewInstanceId::Minimap => (map_views.minimap.revision, map_views.minimap.fit_mode),
        MapViewInstanceId::SimulationMap
        | MapViewInstanceId::TacticalMap
        | MapViewInstanceId::FullscreenMap
        | MapViewInstanceId::CommanderMap
        | MapViewInstanceId::Stage7IntelMap => {
            let presentation_state = presentation_aux.get(id);
            (presentation_state.revision, presentation_state.fit_mode)
        }
    };
    let layout_slot = match id {
        MapViewInstanceId::Minimap => &layout.minimap,
        _ => &layout.world_preview,
    };
    serde_json::json!({
        "presentation_revision": presentation_revision,
        "fit_mode": fit_mode.label(),
        "texture_source": map_texture_source_label(&frame.texture_source),
        "viewport_extent": {
            "x": frame.viewport_extent.x,
            "y": frame.viewport_extent.y,
        },
        "projection_revision": frame.projection_revision,
        "overlay_revision": frame.overlay_revision,
        "texture_rebinds_frame": binding.rebinds_frame,
        "texture_rebinds_total": binding.rebinds_total,
        "stale_binding_count_frame": binding.stale_cache_frame,
        "allocated_rect": layout_slot.allocated_rect.map(rect_json),
        "image_rect": layout_slot.image_rect.map(rect_json),
        "uv_rect": rect_json(layout_slot.uv_rect),
        "padding": layout_slot.padding,
        "aspect_texture": layout_slot.aspect_texture,
        "aspect_panel": layout_slot.aspect_panel,
        "camera_zoom": layout_slot.camera_zoom,
        "fit_validation": layout_slot.validation.as_ref().map(|validation| serde_json::json!({
            "mismatch": validation.mismatch,
            "delta_pixels": validation.delta_pixels,
            "uv_delta": validation.uv_delta,
        })),
    })
}

pub(super) fn rect_json(rect: bevy_egui::egui::Rect) -> serde_json::Value {
    serde_json::json!({
        "min": { "x": rect.min.x, "y": rect.min.y },
        "max": { "x": rect.max.x, "y": rect.max.y },
        "width": rect.width(),
        "height": rect.height(),
    })
}
