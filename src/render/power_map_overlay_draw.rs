//! Power grid overlay — egui vector draw on simulation map (COD-POWER-OVERLAY-RENDER-001).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::construction::world_to_sim_map_egui;
use crate::engine::states::BaseState;
use crate::gui::{MapCameraDesired, SimulationMapViewport};
use crate::infrastructure::utility::VoltageClass;
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::infrastructure_overlay::{
    InfrastructureNetworkLayer, InfrastructureOverlayDrawRequests, InfrastructureOverlaySettings,
    InfrastructureOverlayStroke, PowerLineOverlayState, PowerMapOverlayPresentation,
    stroke_for_power_line_state,
};

pub fn draw_power_map_overlay_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    settings: Res<InfrastructureOverlaySettings>,
    presentation: Res<PowerMapOverlayPresentation>,
    overlays: Res<InfrastructureOverlayDrawRequests>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) {
        return Ok(());
    }
    if !map_vp.is_adequate_for_camera() {
        return Ok(());
    }
    let map_visible = settings.enabled && settings.power;
    let island_forced = presentation.island_highlight_active;
    if !map_visible && !island_forced {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("power_map_overlay"),
    ));
    let auth = authority.as_deref();
    let desired = desired.as_ref();
    let map_vp = map_vp.as_ref();
    let params = params.as_ref();

    for edge in &overlays.edges {
        if edge.layer != InfrastructureNetworkLayer::Power {
            continue;
        }
        let Some(from) = world_to_sim_map_egui(edge.head, auth, desired, map_vp, params) else {
            continue;
        };
        let Some(to) = world_to_sim_map_egui(edge.tail, auth, desired, map_vp, params) else {
            continue;
        };
        paint_stroke_line(&painter, from, to, edge.stroke);
        if edge.line_state == Some(PowerLineOverlayState::IslandBoundary)
            || presentation.island_boundary_link_ids.contains(&edge.link_id)
        {
            let gold = stroke_for_power_line_state(VoltageClass::Medium, PowerLineOverlayState::IslandBoundary);
            paint_stroke_line(&painter, from, to, gold);
        }
        if matches!(
            edge.line_state,
            Some(PowerLineOverlayState::Damaged) | Some(PowerLineOverlayState::Destroyed)
        ) {
            let mid = from.lerp(to, 0.5);
            let glyph = if edge.line_state == Some(PowerLineOverlayState::Destroyed) {
                "×"
            } else {
                "◆"
            };
            painter.text(
                mid,
                egui::Align2::CENTER_CENTER,
                glyph,
                egui::FontId::proportional(12.0),
                stroke_color(edge.stroke),
            );
        }
    }

    for &(head, tail, voltage) in &presentation.preview_segments {
        let stroke = stroke_for_power_line_state(voltage, PowerLineOverlayState::Preview);
        if let (Some(from), Some(to)) = (
            world_to_sim_map_egui(Vec3::new(head.x, head.y, 0.0), auth, desired, map_vp, params),
            world_to_sim_map_egui(Vec3::new(tail.x, tail.y, 0.0), auth, desired, map_vp, params),
        ) {
            paint_stroke_line(&painter, from, to, stroke);
        }
    }

    Ok(())
}

#[inline]
fn stroke_color(stroke: InfrastructureOverlayStroke) -> egui::Color32 {
    let [r, g, b] = stroke.color_rgb;
    let a = (stroke.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

pub fn paint_stroke_line(
    painter: &egui::Painter,
    from: egui::Pos2,
    to: egui::Pos2,
    stroke: InfrastructureOverlayStroke,
) {
    let color = stroke_color(stroke);
    let egui_stroke = egui::Stroke::new(stroke.weight_px, color);
    if stroke.gap_mode {
        paint_gap_line(painter, from, to, egui_stroke, stroke.dash_on_px, stroke.dash_off_px);
        return;
    }
    if !stroke.dashed {
        painter.line_segment([from, to], egui_stroke);
        return;
    }
    paint_dashed_line(
        painter,
        from,
        to,
        egui_stroke,
        stroke.dash_on_px,
        stroke.dash_off_px,
    );
}

fn paint_dashed_line(
    painter: &egui::Painter,
    from: egui::Pos2,
    to: egui::Pos2,
    stroke: egui::Stroke,
    on_px: f32,
    off_px: f32,
) {
    let delta = to - from;
    let len = delta.length();
    if len < 1e-3 {
        return;
    }
    let dir = delta / len;
    let mut dist = 0.0_f32;
    while dist < len {
        let seg_end = (dist + on_px).min(len);
        let a = from + dir * dist;
        let b = from + dir * seg_end;
        painter.line_segment([a, b], stroke);
        dist += on_px + off_px;
    }
}

fn paint_gap_line(
    painter: &egui::Painter,
    from: egui::Pos2,
    to: egui::Pos2,
    stroke: egui::Stroke,
    on_px: f32,
    off_px: f32,
) {
    paint_dashed_line(painter, from, to, stroke, on_px.max(2.0), off_px.max(6.0));
}

#[must_use]
pub fn power_map_overlay_draw_witness_green() -> bool {
    use super::infrastructure_overlay::stroke_for_voltage_class;
    let live = stroke_for_voltage_class(VoltageClass::Medium, false);
    live.alpha > 0.99 && !live.dashed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_map_overlay_draw_witness_green_lib() {
        assert!(power_map_overlay_draw_witness_green());
    }
}
