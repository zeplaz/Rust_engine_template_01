//! On-screen viewport debug window + stroke overlays (logical UI vs physical camera).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::gui::hud::{
    stroke_viewport_debug_rect, viewport_debug_overlay_enabled, CameraProjectionInfo,
    RenderViewportRect, UiViewportRect,
};
use crate::gui::{
    AuthoritativeViewport, MainWorldCamera, MainWorldCameraOrthoTrace,
    MainWorldCameraViewportLatch, SimulationMapViewport, SimulationMapViewportDebug,
    SimulationMapViewportTrace,
};
use crate::render::ResolvedViewports;

#[inline]
pub fn debug_viewport_overlay_enabled() -> bool {
    viewport_debug_overlay_enabled()
}

/// F4-adjacent: always on when `VIEWPORT_DEBUG_OVERLAY=1` / `VISUAL_DIAG=1`.
pub fn draw_debug_viewport_overlay(
    mut contexts: EguiContexts,
    authority: Res<AuthoritativeViewport>,
    sim: Res<SimulationMapViewport>,
    trace: Res<SimulationMapViewportTrace>,
    sim_dbg: Res<SimulationMapViewportDebug>,
    ortho: Res<MainWorldCameraOrthoTrace>,
    latch: Res<MainWorldCameraViewportLatch>,
    resolved: Res<ResolvedViewports>,
    cam: Query<&Camera, With<MainWorldCamera>>,
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

    let ui_committed = UiViewportRect::from_sim(&sim);
    let ui_authoritative = UiViewportRect {
        logical_min: authority.min,
        logical_max: authority.max,
        valid: authority.valid,
    };
    let ui_measured = UiViewportRect {
        logical_min: sim_dbg.measured_min,
        logical_max: sim_dbg.measured_max,
        valid: sim_dbg.measured_valid,
    };
    let ui_solver = UiViewportRect {
        logical_min: sim_dbg.solver_min,
        logical_max: sim_dbg.solver_max,
        valid: sim_dbg.solver_valid,
    };

    let render_scissor = cam.single().ok().and_then(|c| {
        c.viewport.as_ref().map(|vp| RenderViewportRect {
            physical_min: vp.physical_position,
            physical_size: vp.physical_size,
            valid: true,
        })
    });

    let cam_proj = CameraProjectionInfo {
        world_width: ortho.fixed_width,
        world_height: ortho.fixed_height,
        view_pixels: ortho.view_pixels,
        using_hole: ortho.using_hole,
    };

    egui::Window::new("VIEWPORT DEBUG")
        .default_pos([10.0, 10.0])
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.heading("Window");
            ui.label(format!("logical: {:.0} × {:.0}", win_logical.x, win_logical.y));
            ui.label(format!("scale: {scale:.3}"));

            ui.separator();
            ui.heading("Simulation (logical UI)");
            ui.label(format!("valid: {}", sim.valid));
            ui.label(format!("authoritative: {}", ui_authoritative));
            ui.label(format!("committed: {}", ui_committed));
            ui.label(format!("measured:  {}", ui_measured));
            ui.label(format!("solver:    {}", ui_solver));
            ui.label(format!("auth gen: {}", authority.generation));
            ui.label(format!(
                "drift measured→committed: {:.1} × {:.1}",
                trace.measured_size.x - trace.committed_size.x,
                trace.measured_size.y - trace.committed_size.y
            ));
            ui.label(format!("last_commit: {}", sim_dbg.last_commit));
            ui.label(format!(
                "settle: streak={} settled={} frozen={}",
                trace.settle_streak, trace.layout_settled, sim_dbg.frozen
            ));

            ui.separator();
            ui.heading("Camera (do not compare to UI rects directly)");
            ui.label(format!("latch.using_hole: {}", latch.using_hole));
            ui.label(format!("render_hole (ortho): {}", ortho.using_hole));
            ui.label(format!("{cam_proj}"));
            if let Some(r) = render_scissor {
                ui.label(format!("scissor physical: {r}"));
                let logical = r.to_logical(scale);
                ui.label(format!("scissor logical: {}", logical));
            } else {
                ui.label("scissor: full window");
            }

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
                "Overlay: green=measured red=committed blue=camera (cyan=solver)",
            );
        });

    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("debug_viewport_overlay_strokes"),
    ));

    if ui_measured.valid {
        stroke_viewport_debug_rect(
            &painter,
            ui_measured.logical_min,
            ui_measured.logical_max,
            egui::Color32::from_rgb(40, 220, 80),
            "measured",
        );
    }
    if ui_solver.valid {
        stroke_viewport_debug_rect(
            &painter,
            ui_solver.logical_min,
            ui_solver.logical_max,
            egui::Color32::from_rgb(0, 200, 255),
            "solver",
        );
    }
    if ui_committed.valid || ui_authoritative.valid {
        stroke_viewport_debug_rect(
            &painter,
            ui_committed.logical_min,
            ui_committed.logical_max,
            egui::Color32::from_rgb(255, 50, 50),
            "committed",
        );
    }
    if let Some(r) = render_scissor {
        let logical = r.to_logical(scale);
        if logical.valid {
            stroke_viewport_debug_rect(
                &painter,
                logical.logical_min,
                logical.logical_max,
                egui::Color32::from_rgb(60, 120, 255),
                "camera",
            );
        }
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
