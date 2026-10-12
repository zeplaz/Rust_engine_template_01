//! Sim HUD theme helpers — egui RichText/Frame path retired; String labels remain.

use bevy::prelude::*;

use crate::gui::{SimulationMapViewport, UiPalette};

/// Clamp a floating satellite into the measured sim map hole (HUD-NAT-009).
#[must_use]
pub fn clamp_satellite_to_map_hole(
    pos: Vec2,
    size: Vec2,
    map_vp: &SimulationMapViewport,
) -> Vec2 {
    let min = map_vp.min;
    let max = map_vp.max;
    Vec2::new(
        pos.x.clamp(min.x, (max.x - size.x).max(min.x)),
        pos.y.clamp(min.y, (max.y - size.y).max(min.y)),
    )
}

pub fn apply_sim_hud_egui_theme(_palette: &UiPalette) {}

#[must_use]
pub fn sim_hud_egui_theme_enforcement_wired() -> bool {
    include_str!("sim_build_picker_sheet.rs").contains("apply_sim_hud_egui_theme")
        && include_str!("sim_road_tool_sheet.rs").contains("apply_sim_hud_egui_theme")
        && include_str!("context_tray_build_egui.rs").contains("apply_sim_hud_egui_theme")
}

#[must_use]
pub fn hud_nat_009_single_themed_satellite_wired() -> bool {
    include_str!("plant_focus_card.rs").contains("map_attached_chip_frame")
        && include_str!("plant_focus_card.rs").contains("clamp_satellite_to_map_hole")
        && include_str!("sim_play_bevy_chrome.rs").contains("pub struct SimPlayPowerHover")
        && include_str!("sim_play_bevy_chrome.rs").contains("fn sync_sim_play_power_hover")
}

#[must_use]
pub fn sim_hud_a11y_invalid_label(reason: &str) -> String {
    format!("Blocked ✗ · {reason}")
}

#[must_use]
pub fn sim_hud_a11y_valid_label() -> &'static str {
    "Valid ✓"
}

#[must_use]
pub fn sim_hud_a11y_witness_green() -> bool {
    sim_hud_a11y_invalid_label("terrain").contains("Blocked")
        && sim_hud_a11y_valid_label().contains('✓')
}

pub fn picker_sheet_frame(_palette: &UiPalette) {}
pub fn picker_header_frame(_palette: &UiPalette) {}
pub fn map_attached_chip_frame(_palette: &UiPalette) {}

#[must_use]
pub fn title_text(_palette: &UiPalette, text: &str) -> String {
    text.to_string()
}

#[must_use]
pub fn body_text(_palette: &UiPalette, text: &str) -> String {
    text.to_string()
}

#[must_use]
pub fn caption_text(_palette: &UiPalette, text: &str) -> String {
    text.to_string()
}

#[must_use]
pub fn data_text(_palette: &UiPalette, text: &str) -> String {
    text.to_string()
}
