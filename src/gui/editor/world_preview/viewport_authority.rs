//! World-preview presentation fields derived from [`crate::render::ResolvedViewports`].

use bevy::diagnostic::FrameCount;
use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy_egui::egui;

use super::layers::PreviewLayers;
use super::preview_render_contract::{
    PreviewAuthoritativeSurface, PreviewCameraState, PreviewPathAuthority, PreviewRenderMode,
};
use super::render_target_barrier::{WorldPreviewGpuResizeQueue, WorldPreviewViewportEvent};
use crate::gui::map_view::MapViewInstances;
use crate::gui::map_view_projection::{map_display_rect, map_texture_uv_rect};
use crate::render::{trace_camera_sync, trace_viewport, DebugRenderTraceConfig, ResolvedViewports};

/// Committed layout + render routing for the world preview window (one frame).
#[derive(Resource, Debug, Clone)]
pub struct WorldPreviewViewportAuthority {
    pub egui_clip_rect: Option<egui::Rect>,
    pub display_rect: Option<egui::Rect>,
    pub sample_uv_rect: egui::Rect,
    pub logical_viewport: Vec2,
    pub physical_render_extent: UVec2,
    pub world_map_extent: UVec2,
    pub gpu_authoritative: bool,
    pub committed: bool,
    pub revision: u64,
}

impl Default for WorldPreviewViewportAuthority {
    fn default() -> Self {
        Self {
            egui_clip_rect: None,
            display_rect: None,
            sample_uv_rect: map_texture_uv_rect(),
            logical_viewport: Vec2::ONE,
            physical_render_extent: UVec2::ONE,
            world_map_extent: UVec2::ONE,
            gpu_authoritative: false,
            committed: false,
            revision: 0,
        }
    }
}

/// Mirror the resolved preview viewport into presentation state for GPU consumers.
pub fn sync_world_preview_viewport_authority(
    resolved: Res<ResolvedViewports>,
    path: Res<PreviewPathAuthority>,
    preview_cam: Res<PreviewCameraState>,
    views: Res<MapViewInstances>,
    mut authority: ResMut<WorldPreviewViewportAuthority>,
) {
    if !resolved.world_preview.valid {
        authority.committed = false;
        return;
    }

    let gpu = path.authoritative_surface == PreviewAuthoritativeSurface::GpuRenderTarget
        && preview_cam.mode == PreviewRenderMode::GpuRenderTarget;
    let logical_v = resolved.world_preview.logical_size;
    let egui_clip = egui::Rect::from_min_size(
        egui::pos2(0.0, 0.0),
        egui::vec2(logical_v.x, logical_v.y),
    );

    authority.gpu_authoritative = gpu;
    authority.logical_viewport = logical_v;
    authority.physical_render_extent = resolved.world_preview.physical_extent;
    authority.world_map_extent = resolved.world_preview.world_extent;
    authority.egui_clip_rect = Some(egui_clip);

    if gpu {
        authority.display_rect = Some(egui_clip);
        authority.sample_uv_rect = map_texture_uv_rect();
    } else {
        let view = &views.world_preview;
        let z = view.zoom.clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
        authority.display_rect = Some(map_display_rect(
            egui_clip.center(),
            view.camera_center,
            z,
            authority.world_map_extent.x as f32,
            authority.world_map_extent.y as f32,
        ));
        authority.sample_uv_rect = map_texture_uv_rect();
    }

    authority.committed = true;
    authority.revision = resolved.revision;
}

/// Record GPU resize intent from the resolved preview viewport (no GPU allocation here).
pub fn queue_world_preview_gpu_resize_request(
    frame: Res<FrameCount>,
    resolved: Res<ResolvedViewports>,
    path: Res<PreviewPathAuthority>,
    preview_cam: Res<PreviewCameraState>,
    mut queue: ResMut<WorldPreviewGpuResizeQueue>,
    mut events: MessageWriter<WorldPreviewViewportEvent>,
    mut last_extent: Local<UVec2>,
) {
    if !resolved.world_preview.valid
        || path.authoritative_surface != PreviewAuthoritativeSurface::GpuRenderTarget
        || preview_cam.mode != PreviewRenderMode::GpuRenderTarget
    {
        return;
    }
    let extent = resolved.world_preview.physical_extent;
    if extent == UVec2::ZERO || extent == *last_extent {
        return;
    }
    queue.requested_extent = Some(extent);
    queue.frame_requested = frame.0;
    *last_extent = extent;
    events.write(WorldPreviewViewportEvent::ResizeRequested {
        size: extent,
        frame_requested: frame.0,
    });
}

pub fn debug_trace_world_preview_viewport_authority(
    cfg: Res<DebugRenderTraceConfig>,
    authority: Res<WorldPreviewViewportAuthority>,
    views: Res<MapViewInstances>,
) {
    if !authority.committed {
        return;
    }
    if cfg.viewport_trace {
        let display = authority
            .display_rect
            .map(|rect| format!("{rect:?}"))
            .unwrap_or_else(|| "none".into());
        let clip = authority
            .egui_clip_rect
            .map(|rect| format!("{rect:?}"))
            .unwrap_or_else(|| "none".into());
        trace_viewport(
            &cfg,
            &format!(
                "authority rev={} gpu={} clip={clip} display={display} logical=({:.1},{:.1}) physical={}x{} world={}x{}",
                authority.revision,
                authority.gpu_authoritative,
                authority.logical_viewport.x,
                authority.logical_viewport.y,
                authority.physical_render_extent.x,
                authority.physical_render_extent.y,
                authority.world_map_extent.x,
                authority.world_map_extent.y,
            ),
        );
    }
    if cfg.camera_sync_trace {
        trace_camera_sync(
            &cfg,
            &format!(
                "editor_viewport viewport_size=({:.1},{:.1}) camera_center=({:.1},{:.1}) zoom={:.3}",
                views.world_preview.viewport_size.x,
                views.world_preview.viewport_size.y,
                views.world_preview.camera_center.x,
                views.world_preview.camera_center.y,
                views.world_preview.zoom,
            ),
        );
    }
}
