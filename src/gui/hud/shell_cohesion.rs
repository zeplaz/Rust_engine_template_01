//! Shared product-shell visual language (consumer-only).

use bevy_egui::egui;

use crate::gui::style::{muted_text, section_heading, CmdHeadingStyle, UiPalette};

pub fn shell_panel_heading(ui: &mut egui::Ui, palette: &UiPalette, title: &str) {
    section_heading(ui, palette, CmdHeadingStyle::Gt, title);
}

pub fn shell_panel_subtitle(ui: &mut egui::Ui, palette: &UiPalette, text: &str) {
    muted_text(ui, palette, text);
}

pub fn shell_legend_row(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    color: egui::Color32,
    label: &str,
) {
    ui.horizontal(|ui| {
        ui.colored_label(color, "■");
        muted_text(ui, palette, label);
    });
}

pub fn shell_feed_line(ui: &mut egui::Ui, palette: &UiPalette, line: &str) {
    muted_text(ui, palette, line);
}
