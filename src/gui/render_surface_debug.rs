//! Operator overlay — what is rendered, viewport rects, zoom, ghost tile (F3 / VfxSandbox).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::construction::{BuildGhostState, BuildStripState};
use crate::engine::launch_args::{EngineLaunchArgs, TestScene};
use crate::gui::{
    MapCameraDesired, MapPresentationDiagnostics, MinimapShellState,
    SimulationMapViewport, ViewManager,
};
use crate::render::{SharedOverlayFieldBuffers, TileWorldFallbackState};

/// Toggle via diagnostics or `--test renderdebug` / VfxSandbox bootstrap.
#[derive(Resource, Clone, Debug)]
pub struct RenderSurfaceDebugOverlay {
    pub enabled: bool,
}

impl Default for RenderSurfaceDebugOverlay {
    fn default() -> Self {
        Self { enabled: false }
    }
}

pub fn draw_render_surface_debug_overlay(
    mut contexts: EguiContexts,
    overlay: Res<RenderSurfaceDebugOverlay>,
    launch: Option<Res<EngineLaunchArgs>>,
    map_vp: Res<SimulationMapViewport>,
    desired: Res<MapCameraDesired>,
    manager: Res<ViewManager>,
    fallback: Res<TileWorldFallbackState>,
    minimap: Res<MinimapShellState>,
    shared: Option<Res<SharedOverlayFieldBuffers>>,
    ghost: Option<Res<BuildGhostState>>,
    strip: Option<Res<BuildStripState>>,
    diag: Option<Res<MapPresentationDiagnostics>>,
) -> Result {
    let force = launch.as_ref().is_some_and(|l| {
        matches!(
            l.test_scene,
            TestScene::VfxSandbox | TestScene::Visual | TestScene::RenderDebug
        )
    });
    if !overlay.enabled && !force {
        return Ok(());
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return Ok(());
    };

    egui::Window::new("Render surfaces (debug)")
        .default_pos(egui::pos2(12.0, 120.0))
        .default_width(320.0)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Live rects + zoom — refactor placement / minimap / fire here")
                    .small()
                    .weak(),
            );
            if map_vp.valid {
                let size = map_vp.logical_size();
                ui.label(format!(
                    "Sim map viewport: {:.0}×{:.0} px @ ({:.0},{:.0})",
                    size.x,
                    size.y,
                    map_vp.min.x,
                    map_vp.min.y
                ));
            } else {
                ui.label("Sim map viewport: invalid / not measured");
            }
            ui.label(format!(
                "MapCameraDesired: xy=({:.1},{:.1}) scale={:.3}",
                desired.translation.x, desired.translation.y, desired.scale.x
            ));
            if let Some(v) = manager.view(crate::gui::ViewId::WorldMain) {
                ui.label(format!(
                    "ViewManager WorldMain: xy=({:.1},{:.1}) zoom={:.3}",
                    v.camera.translation.x, v.camera.translation.y, v.camera.zoom
                ));
            }
            ui.label(format!(
                "Fallback raster: {}×{} tiles · minimap zoom={:.2} center=({:.0},{:.0})",
                fallback.last_w,
                fallback.last_h,
                minimap.zoom,
                minimap.world_center.x,
                minimap.world_center.y
            ));
            ui.label(format!(
                "Minimap viewport frame: {} · image_rect={}",
                minimap.show_tactical_viewport_frame,
                minimap
                    .last_image_rect
                    .map(|r| format!("{:.0}×{:.0}", r.width(), r.height()))
                    .unwrap_or_else(|| "—".into())
            ));
            if let Some(o) = shared.as_ref() {
                ui.label(format!(
                    "Fire overlay chunks: {} · heat max {:.2}",
                    o.chunk_fire_heat.len(),
                    o.chunk_fire_heat.values().copied().fold(0.0f32, f32::max)
                ));
            }
            ui.label(
                egui::RichText::new(
                    "Fire tint on map = CPU chunk heat overlay (squares). GPU field / particles are separate.",
                )
                .small()
                .weak(),
            );
            ui.label(
                egui::RichText::new(
                    "Red box = FIRE/SPARKS TEST region — zoom in here for GPU sparks (operational zoom α≥0.42).",
                )
                .small()
                .color(egui::Color32::from_rgb(255, 100, 90)),
            );
            if let (Some(g), Some(s)) = (ghost.as_ref(), strip.as_ref()) {
                ui.label(format!(
                    "Build strip: {:?} · ghost origin {:?} footprint {}×{} · scale {:.2}",
                    s.active,
                    g.origin,
                    g.footprint.width,
                    g.footprint.depth,
                    g.scale_factor
                ));
                ui.label(
                    egui::RichText::new(
                        "Full placement debug → window \"Construction placement (debug)\" (--test vfx auto)",
                    )
                    .small()
                    .weak(),
                );
            }
            if let Some(d) = diag.as_ref() {
                let row = &d.minimap;
                if let Some(image_rect) = row.image_rect {
                    ui.label(format!(
                        "Minimap fit: panel {:?} · image {:.0}×{:.0} · fit_scale {:.4}",
                        row.allocated_rect.map(|r| format!("{:.0}×{:.0}", r.width(), r.height())),
                        image_rect.width(),
                        image_rect.height(),
                        row.fit_scale
                    ));
                }
            }
        });
    Ok(())
}
