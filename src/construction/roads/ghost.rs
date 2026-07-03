//! Road path polyline ghost drawn on the simulation map viewport (egui painter).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::engine::states::BaseState;
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool};
use crate::construction::ghost_visual::{road_control_point_color, road_segment_color};
use crate::construction::map_egui_projection::{map_zoom_screen_scale, world_to_sim_map_egui};
use crate::construction::roads::ActiveRoadPlacement;

pub fn draw_road_path_ghost_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    tool: Res<ActiveBuildTool>,
    placement: Res<ActiveRoadPlacement>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation | BaseState::Editor) {
        return Ok(());
    }
    if !matches!(tool.tool, BuildTool::Road(_)) || !map_vp.is_adequate_for_camera() {
        return Ok(());
    }

    let zoom = map_zoom_screen_scale(authority.as_deref(), desired.as_ref());
    let ctx = contexts.ctx_mut()?;
    let layer = egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("road_path_ghost_layer"),
    );
    let painter = ctx.layer_painter(layer);

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
        let color = road_segment_color(seg.valid);
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
            let c = road_control_point_color();
            let r = (5.0 * zoom.sqrt()).clamp(3.0, 14.0);
            painter.circle_filled(screen, r, c);
            painter.circle_stroke(
                screen,
                r + 2.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(20, 30, 40, 120)),
            );
        }
    }

    Ok(())
}
