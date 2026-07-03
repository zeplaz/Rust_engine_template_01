//! Zone paint tile highlights on the simulation map viewport.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::engine::states::BaseState;
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::super::build_tool_authority::{ActiveBuildTool, BuildTool, ZoneTool};
use super::super::map_egui_projection::{map_zoom_screen_scale, world_to_sim_map_egui};
use super::placement::ActiveZonePaint;

pub fn zone_fill(zone: ZoneTool) -> egui::Color32 {
    match zone {
        ZoneTool::ResidentialLow => egui::Color32::from_rgba_unmultiplied(80, 200, 120, 90),
        ZoneTool::ResidentialMedium => egui::Color32::from_rgba_unmultiplied(100, 220, 140, 100),
        ZoneTool::ResidentialHigh => egui::Color32::from_rgba_unmultiplied(60, 180, 200, 100),
        ZoneTool::Apartments => egui::Color32::from_rgba_unmultiplied(120, 140, 220, 110),
        ZoneTool::MixedUse => egui::Color32::from_rgba_unmultiplied(200, 160, 80, 100),
    }
}

pub fn draw_zone_paint_ghost_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    tool: Res<ActiveBuildTool>,
    paint: Res<ActiveZonePaint>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation | BaseState::Editor) {
        return Ok(());
    }
    let BuildTool::Zone(zone) = tool.tool else {
        return Ok(());
    };
    if paint.painted.is_empty() || !map_vp.is_adequate_for_camera() {
        return Ok(());
    }

    let zoom = map_zoom_screen_scale(authority.as_deref(), desired.as_ref());
    let ctx = contexts.ctx_mut()?;
    let layer = egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("zone_paint_ghost_layer"),
    );
    let painter = ctx.layer_painter(layer);
    let fill = zone_fill(zone);
    let stroke = egui::Stroke::new(1.0, fill.gamma_multiply(1.8));

    for tile in &paint.painted {
        let center = Vec3::new(tile.x as f32 + 0.5, 0.05, tile.z as f32 + 0.5);
        let Some(screen) =
            world_to_sim_map_egui(
                center,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        else {
            continue;
        };
        let (fw, fh) = crate::gui::sim_map_visible_world_span(
            map_vp.as_ref(),
            zoom,
            params.width as f32,
            params.height as f32,
        );
        let vp = map_vp.logical_size();
        let rect_w = vp.x / fw.max(1.0);
        let rect_h = vp.y / fh.max(1.0);
        let rect = egui::Rect::from_center_size(screen, egui::vec2(rect_w.max(4.0), rect_h.max(4.0)));
        painter.rect_filled(rect, 2.0, fill);
        painter.rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Inside);
    }

    Ok(())
}
