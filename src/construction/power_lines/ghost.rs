//! Power line path ghost on simulation map (gold family).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::engine::states::BaseState;
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::render::{paint_stroke_line, stroke_for_voltage_class, InfrastructureOverlayStroke};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool};
use crate::construction::map_egui_projection::{map_zoom_screen_scale, world_to_sim_map_egui};

use super::placement::ActivePowerLinePlacement;

pub fn draw_power_line_path_ghost_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    tool: Res<ActiveBuildTool>,
    placement: Res<ActivePowerLinePlacement>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation | BaseState::Editor) {
        return Ok(());
    }
    if !matches!(tool.tool, BuildTool::PowerLine(_)) || !map_vp.is_adequate_for_camera() {
        return Ok(());
    }

    let zoom = map_zoom_screen_scale(authority.as_deref(), desired.as_ref());
    let ctx = contexts.ctx_mut()?;
    let layer = egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("power_line_path_ghost_layer"),
    );
    let painter = ctx.layer_painter(layer);
    let preview_stroke = stroke_for_voltage_class(placement.voltage, true);
    let stroke_w = (preview_stroke.weight_px * zoom.sqrt()).clamp(1.5, 24.0);
    let dashed_stroke = InfrastructureOverlayStroke {
        weight_px: stroke_w,
        ..preview_stroke
    };

    for seg in &placement.generated_segments {
        let Some(a) = world_to_sim_map_egui(
            seg.start,
            authority.as_deref(),
            desired.as_ref(),
            map_vp.as_ref(),
            params.as_ref(),
        ) else {
            continue;
        };
        let Some(b) = world_to_sim_map_egui(
            seg.end,
            authority.as_deref(),
            desired.as_ref(),
            map_vp.as_ref(),
            params.as_ref(),
        ) else {
            continue;
        };
        if seg.valid {
            paint_stroke_line(&painter, a, b, dashed_stroke);
        } else {
            painter.line_segment(
                [a, b],
                egui::Stroke::new(stroke_w, egui::Color32::from_rgb(0xe0, 0x60, 0x60)),
            );
        }
    }

    for p in &placement.control_points {
        if let Some(screen) = world_to_sim_map_egui(
            *p,
            authority.as_deref(),
            desired.as_ref(),
            map_vp.as_ref(),
            params.as_ref(),
        ) {
            let r = (5.0 * zoom.sqrt()).clamp(3.0, 14.0);
            painter.circle_filled(screen, r, egui::Color32::from_rgb(0xff, 0xd8, 0x78));
        }
    }

    Ok(())
}

#[must_use]
pub fn power_line_ghost_preview_dashed_witness_green() -> bool {
    let stroke = stroke_for_voltage_class(
        crate::infrastructure::VoltageClass::Medium,
        true,
    );
    stroke.dashed && stroke.alpha <= 0.61
}
