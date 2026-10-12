//! Shared ghost color language (valid / invalid / pending / committed).

use crate::gui::ui_types::{UiRgba, UiStroke};

#[must_use]
pub fn road_segment_color(valid: bool) -> UiRgba {
    if valid {
        UiRgba::from_rgba_unmultiplied(80, 220, 180, 140)
    } else {
        UiRgba::from_rgba_unmultiplied(240, 90, 90, 160)
    }
}

#[must_use]
pub fn road_committed_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(42, 44, 52, 230)
}

#[must_use]
pub fn road_control_point_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(255, 230, 140, 200)
}

#[must_use]
pub fn footprint_valid_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(48, 140, 72, 220)
}

#[must_use]
pub fn footprint_risky_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(200, 140, 40, 230)
}

#[must_use]
pub fn footprint_invalid_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(180, 48, 48, 240)
}

#[must_use]
pub fn footprint_place_settle_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(100, 144, 220, 200)
}

#[must_use]
pub fn place_feedback_duration_secs() -> f32 {
    0.28
}

#[must_use]
pub fn footprint_lock_ring_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(232, 196, 96, 230)
}

#[must_use]
pub fn footprint_lock_ring_stroke_px() -> f32 {
    2.0
}

#[must_use]
pub fn footprint_lock_ring_stroke(locked: bool) -> Option<UiStroke> {
    if !locked {
        return None;
    }
    Some(UiStroke::new(
        footprint_lock_ring_stroke_px(),
        footprint_lock_ring_color(),
    ))
}

#[must_use]
pub fn corridor_planned_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(232, 176, 64, 180)
}

#[must_use]
pub fn corridor_in_progress_color() -> UiRgba {
    UiRgba::from_rgba_unmultiplied(80, 160, 232, 200)
}

#[must_use]
pub fn corridor_phase_tokens_wired_green() -> bool {
    corridor_planned_color() == UiRgba::from_rgba_unmultiplied(232, 176, 64, 180)
        && corridor_in_progress_color() == UiRgba::from_rgba_unmultiplied(80, 160, 232, 200)
}
