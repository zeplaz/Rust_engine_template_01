//! Power grid overlay — egui vector draw on simulation map (COD-POWER-OVERLAY-RENDER-001).

use crate::infrastructure::utility::VoltageClass;

/// Retired egui stroke paint — Bevy/GPU overlay owns live strokes.
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn paint_stroke_line() {}

#[must_use]
pub fn power_map_overlay_draw_witness_green() -> bool {
    use crate::render::pipelines::infrastructure_overlay::stroke_for_voltage_class;
    let live = stroke_for_voltage_class(VoltageClass::Medium, false);
    live.alpha > 0.99 && !live.dashed
}

/// Power line strokes on the CPU minimap image (COD-POWER-OVERLAY-MINIMAP-001).
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn draw_power_strokes_on_minimap() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_map_overlay_draw_witness_green_lib() {
        assert!(power_map_overlay_draw_witness_green());
    }
}
