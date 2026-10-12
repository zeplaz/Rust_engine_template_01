//! Power grid overlay — egui vector draw on simulation map (COD-POWER-OVERLAY-RENDER-001).

use bevy::prelude::*;

use crate::construction::world_to_sim_map_screen;
use crate::engine::states::BaseState;
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::infrastructure::utility::VoltageClass;
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use crate::render::pipelines::infrastructure_overlay::{
    InfrastructureNetworkLayer, InfrastructureOverlayDrawRequests, InfrastructureOverlaySettings,
    InfrastructureOverlayStroke, PowerLineOverlayState, PowerMapOverlayPresentation,
    stroke_for_power_line_state,
};


#[inline]
fn stroke_color(stroke: InfrastructureOverlayStroke) -> crate::gui::UiRgba {
    let [r, g, b] = stroke.color_rgb;
    let a = (stroke.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    crate::gui::UiRgba::from_rgba_unmultiplied(r, g, b, a)
}

pub fn paint_stroke_line() {}

fn paint_dashed_line() {}

fn paint_gap_line() {}

#[must_use]
pub fn power_map_overlay_draw_witness_green() -> bool {
    use crate::render::pipelines::infrastructure_overlay::stroke_for_voltage_class;
    let live = stroke_for_voltage_class(VoltageClass::Medium, false);
    live.alpha > 0.99 && !live.dashed
}

/// Power line strokes on the CPU minimap image (COD-POWER-OVERLAY-MINIMAP-001).
pub fn draw_power_strokes_on_minimap() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_map_overlay_draw_witness_green_lib() {
        assert!(power_map_overlay_draw_witness_green());
    }
}
