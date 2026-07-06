//! On-screen viewport debug window + stroke overlays (RTT fill rect + ortho trace).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::gui::hud::{
    stroke_viewport_debug_rect, viewport_debug_overlay_enabled, CameraProjectionInfo,
    UiViewportRect,
};
use crate::gui::{
    MainWorldCameraOrthoTrace, SimulationMapTexture, SimulationMapViewport,
    SimulationMapViewportDebug, SimulationMapViewportTrace,
};
use crate::gui::simulation_map_texture_extent;
use crate::render::ResolvedViewports;

#[inline]
pub fn debug_viewport_overlay_enabled() -> bool {
    viewport_debug_overlay_enabled()
}

/// F4-adjacent: always on when `VIEWPORT_DEBUG_OVERLAY=1` / `VISUAL_DIAG=1`.
pub fn draw_debug_viewport_overlay(
    mut contexts: EguiContexts,
    fill: Res<SimulationMapViewport>,
    trace: Res<SimulationMapViewportTrace>,
    sim_dbg: Res<SimulationMapViewportDebug>,
    ortho: Res<MainWorldCameraOrthoTrace>,
    tex: Res<SimulationMapTexture>,
    images: Res<Assets<Image>>,
    resolved: Res<ResolvedViewports>,
    win: Query<&Window, With<PrimaryWindow>>,
) {
    if !debug_viewport_overlay_enabled() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let win_logical = win
        .single()
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::ONE);
    let scale = win.single().map(|w| w.scale_factor()).unwrap_or(1.0);
    let rtt_extent = simulation_map_texture_extent(tex.as_ref(), images.as_ref());

    let ui_fill = UiViewportRect::from_sim(&fill);

    let cam_proj = CameraProjectionInfo {
        world_width: ortho.fixed_width,
        world_height: ortho.fixed_height,
        view_pixels: ortho.view_pixels,
    };

    egui::Window::new("VIEWPORT DEBUG (RTT)")
        .default_pos([10.0, 10.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.heading("Window");
            ui.label(format!("logical: {:.0} × {:.0}", win_logical.x, win_logical.y));
            ui.label(format!("scale: {scale:.3}"));

            ui.separator();
            ui.heading("Map fill (UI logical)");
            ui.label(format!("valid: {}", fill.valid));
            ui.label(format!("fill: {}", ui_fill));
            ui.label(format!("adequate: {}", fill.is_adequate_for_camera()));
            ui.label(format!("rtt extent: {:.0} × {:.0}", rtt_extent.x, rtt_extent.y));
            ui.label(format!("last_commit: {}", sim_dbg.last_commit));
            ui.label(format!(
                "trace: measured={:?} committed={:?} settled={}",
                trace.measured_size, trace.committed_size, trace.layout_settled
            ));

            ui.separator();
            ui.heading("Camera ortho (RTT target)");
            ui.label(format!("{cam_proj}"));
            ui.label("camera viewport: RTT image (no window scissor)");

            ui.separator();
            ui.heading("Resolved (render spine)");
            ui.label(format!(
                "sim valid={} logical_wh=({:.0}, {:.0}) rev={}",
                resolved.simulation_map.valid,
                resolved.simulation_map.logical_size.x,
                resolved.simulation_map.logical_size.y,
                resolved.revision
            ));

            ui.separator();
            ui.colored_label(
                egui::Color32::LIGHT_GRAY,
                "Overlay: green=fill rect (map ImageNode screen AABB)",
            );
        });

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("debug_viewport_overlay_strokes"),
    ));

    if ui_fill.valid {
        stroke_viewport_debug_rect(
            &painter,
            ui_fill.logical_min,
            ui_fill.logical_max,
            egui::Color32::from_rgb(40, 220, 80),
            "fill",
        );
    }
}

pub struct DebugViewportOverlayPlugin;

impl Plugin for DebugViewportOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            draw_debug_viewport_overlay
                .after(crate::gui::hud::hud_root_tick::hud_product_shell_egui_root),
        );
    }
}
