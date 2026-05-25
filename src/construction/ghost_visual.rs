//! Shared ghost color language (valid / invalid / pending / committed).

use bevy_egui::egui;

#[must_use]
pub fn road_segment_color(valid: bool) -> egui::Color32 {
    if valid {
        egui::Color32::from_rgba_unmultiplied(80, 220, 180, 140)
    } else {
        egui::Color32::from_rgba_unmultiplied(240, 90, 90, 160)
    }
}

/// Committed / executed road tiles on the tactical map.
#[must_use]
pub fn road_committed_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(42, 44, 52, 230)
}

#[must_use]
pub fn road_control_point_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(255, 230, 140, 200)
}

#[must_use]
pub fn footprint_valid_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(48, 140, 72, 220)
}

#[must_use]
pub fn footprint_risky_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(200, 140, 40, 230)
}

#[must_use]
pub fn footprint_invalid_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(180, 48, 48, 240)
}
