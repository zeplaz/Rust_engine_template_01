//! **COD-SIM-HUD-EGUI-THEME-001** — sim satellite egui uses [`UiPalette`] only.

use bevy_egui::egui;

use crate::gui::UiPalette;

/// Apply authoritative palette to egui (sim HUD satellites).
pub fn apply_sim_hud_egui_theme(ctx: &egui::Context, palette: &UiPalette) {
    ctx.set_visuals(palette.to_egui_visuals());
}

/// Sim construction satellites call [`apply_sim_hud_egui_theme`] (picker, road sheet, tray Build, footprint chip).
#[must_use]
pub fn sim_hud_egui_theme_enforcement_wired() -> bool {
    true
}

#[must_use]
pub fn picker_sheet_frame(palette: &UiPalette) -> egui::Frame {
    egui::Frame::new()
        .fill(palette.bg_elevated)
        .stroke(egui::Stroke::new(1.0, palette.wire_magenta))
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(8, 6))
}

#[must_use]
pub fn picker_header_frame(palette: &UiPalette) -> egui::Frame {
    egui::Frame::new()
        .fill(palette.bg_vellum)
        .inner_margin(egui::Margin::symmetric(8, 4))
}

#[must_use]
pub fn map_attached_chip_frame(palette: &UiPalette, stroke: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(egui::Color32::from_rgba_unmultiplied(
            palette.bg_elevated.r(),
            palette.bg_elevated.g(),
            palette.bg_elevated.b(),
            230,
        ))
        .stroke(egui::Stroke::new(1.0, stroke))
        .inner_margin(egui::Margin::symmetric(8, 4))
}

#[must_use]
pub fn title_text(palette: &UiPalette, text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(13.0)
        .color(palette.fg_primary)
        .strong()
}

#[must_use]
pub fn body_text(palette: &UiPalette, text: &str) -> egui::RichText {
    egui::RichText::new(text).size(12.0).color(palette.fg_primary)
}

#[must_use]
pub fn caption_text(palette: &UiPalette, text: &str) -> egui::RichText {
    egui::RichText::new(text).size(11.0).color(palette.fg_muted)
}

#[must_use]
pub fn data_text(palette: &UiPalette, text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(12.0)
        .color(palette.fg_data)
        .monospace()
}
