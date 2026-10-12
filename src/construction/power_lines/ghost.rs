//! Power line path ghosts — Bevy overlay / visual_authority (retired egui painter).

use crate::render::stroke_for_voltage_class;

#[must_use]
pub fn power_line_ghost_preview_dashed_witness_green() -> bool {
    let stroke = stroke_for_voltage_class(crate::infrastructure::VoltageClass::Medium, true);
    stroke.dashed && stroke.alpha <= 0.61
}
