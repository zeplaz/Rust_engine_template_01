//! Rail path ghost — purple/magenta lane distinct from roads.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::engine::states::BaseState;
use crate::gui::{MapCameraDesired, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::placement::ActiveRailPlacement;
use super::super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::super::map_egui_projection::{map_zoom_screen_scale, world_to_sim_map_egui};

#[must_use]
fn rail_segment_color(valid: bool, slope_ok: bool) -> egui::Color32 {
    if valid && slope_ok {
        egui::Color32::from_rgba_unmultiplied(180, 120, 255, 150)
    } else if !slope_ok {
        egui::Color32::from_rgba_unmultiplied(255, 140, 60, 180)
    } else {
        egui::Color32::from_rgba_unmultiplied(220, 80, 120, 170)
    }
}

pub fn draw_rail_path_ghost_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    tool: Res<ActiveBuildTool>,
    placement: Res<ActiveRailPlacement>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation | BaseState::Editor) {
        return Ok(());
    }
    if !matches!(tool.tool, BuildTool::Rail(_)) || !map_vp.is_adequate_for_camera() {
        return Ok(());
    }
    let zoom = map_zoom_screen_scale(authority.as_deref(), desired.as_ref());
    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("rail_path_ghost_layer"),
    ));
    for seg in &placement.generated_segments {
        let Some(a) =
            world_to_sim_map_egui(
                seg.start,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        else {
            continue;
        };
        let Some(b) =
            world_to_sim_map_egui(
                seg.end,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        else {
            continue;
        };
        let color = rail_segment_color(seg.valid, seg.slope_ok);
        let stroke_w = (seg.width * 0.5 * zoom).clamp(1.0, 48.0);
        painter.line_segment([a, b], egui::Stroke::new(stroke_w, color));
    }
    for p in &placement.control_points {
        if let Some(screen) =
            world_to_sim_map_egui(
                *p,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        {
            let r = (5.0 * zoom.sqrt()).clamp(3.0, 14.0);
            painter.circle_filled(screen, r, egui::Color32::from_rgba_unmultiplied(200, 160, 255, 220));
        }
    }
    Ok(())
}
