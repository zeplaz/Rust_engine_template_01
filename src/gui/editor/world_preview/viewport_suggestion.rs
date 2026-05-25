//! UI layout requests for world preview — not authoritative until resolved.

use bevy::math::{UVec2, Vec2};
use bevy_egui::egui;

use crate::gui::{
    submit_viewport_request, ViewportAuthority, ViewportRequest, VIEWPORT_PRIORITY_PREVIEW,
};
use crate::gui::hud::{
    ProductShellDiagnostics, ViewportRectSanity, ViewportRectSource, VIEWPORT_RECT_SAFE_MIN,
};

/// Record layout intent from the central preview panel.
pub fn write_world_preview_viewport_request(
    authority: &mut ViewportAuthority,
    egui_clip: egui::Rect,
    world_w: u32,
    world_h: u32,
    sanity: &mut ViewportRectSanity,
    diag: Option<&mut ProductShellDiagnostics>,
    emit_layout_capture: bool,
) {
    if !emit_layout_capture {
        return;
    }
    let fallback = Vec2::new(
        world_w.max(1) as f32,
        world_h.max(1) as f32,
    )
    .max(Vec2::splat(VIEWPORT_RECT_SAFE_MIN));
    let logical_rect = sanity.inspect_egui_rect(
        egui_clip,
        ViewportRectSource::WorldPreviewCentralPanel,
        fallback,
        diag,
    );
    submit_viewport_request(
        authority,
        ViewportRequest {
            logical_rect,
            priority: VIEWPORT_PRIORITY_PREVIEW,
            world_extent: UVec2::new(world_w.max(1), world_h.max(1)),
        },
    );
}

pub fn clear_world_preview_viewport_requests(authority: &mut ViewportAuthority) {
    authority.pending.retain(|request| request.priority != VIEWPORT_PRIORITY_PREVIEW);
}
