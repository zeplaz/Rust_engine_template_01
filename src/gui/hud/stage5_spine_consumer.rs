//! Stage 5 spine consumer panel — read-only metrics display.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::editor::world_preview::{PreviewPathAuthority, PreviewPresentationDebug};
use crate::gui::style::{error_text, muted_text, UiPalette};
use crate::gui::WorldRepresentationFrame;
use crate::render::AppStage5ReadinessReport;

pub fn draw_stage5_spine_consumer_panel(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    readiness: Option<&AppStage5ReadinessReport>,
    world: Option<&WorldRepresentationFrame>,
    preview_authority: Option<&PreviewPathAuthority>,
    preview_debug: Option<&PreviewPresentationDebug>,
) {
    if let Some(report) = readiness {
        muted_text(
            ui,
            palette,
            format!(
                "VT-4={} VT-5={} fire_extract={} gpu_field={} overlay_shared={} dup_extract={} producers={}",
                report.vt4_ok,
                report.vt5_ok,
                report.single_fire_extract,
                report.gpu_field_authoritative,
                report.overlay_from_shared_buffers_only,
                report.duplicate_visual_scan_count,
                report.registered_producers,
            ),
        );
        for violation in &report.violations {
            error_text(ui, palette, violation);
        }
    } else {
        muted_text(ui, palette, "Stage 5 readiness report unavailable (consumer).");
    }
    if let Some(frame) = world {
        muted_text(
            ui,
            palette,
            format!(
                "LOD band {:?} · zoom={:.3} · interest_r={}",
                frame.global_band(),
                frame.zoom,
                frame.interest_radius_chunks,
            ),
        );
    }
    if let Some(authority) = preview_authority {
        muted_text(
            ui,
            palette,
            format!(
                "GPU preview authority={:?} gpu_requested={} cpu_fallback={}",
                authority.authoritative_surface,
                authority.gpu_render_target_requested,
                authority.cpu_raster_fallback_active,
            ),
        );
    }
    if let Some(debug) = preview_debug {
        muted_text(
            ui,
            palette,
            format!(
                "VT surface={:?} front_asset_bits={}",
                debug.authoritative_surface, debug.last_front_asset_id_bits,
            ),
        );
    }
}
