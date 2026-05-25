//! Small overview image (same atlas); optional richer minimap later.

use bevy::math::UVec2;
use bevy_egui::egui;

use crate::gui::map_presentation_fit::{compute_map_fit_strict, MapFitMode, MAP_PANEL_INSET_PX};
use crate::gui::style::{section_heading, CmdHeadingStyle, UiPalette};

pub fn world_preview_minimap(
    ui: &mut egui::Ui,
    texture_id: egui::TextureId,
    tex_w: u32,
    tex_h: u32,
    palette: &UiPalette,
) {
    section_heading(ui, palette, CmdHeadingStyle::None, "Overview");
    let max_side = 140.0f32;
    let (panel_rect, _) = ui.allocate_exact_size(
        egui::vec2(max_side, max_side),
        egui::Sense::hover(),
    );
    let fit = compute_map_fit_strict(
        panel_rect.shrink(MAP_PANEL_INSET_PX),
        UVec2::new(tex_w.max(1), tex_h.max(1)),
        MapFitMode::Contain,
    );
    ui.painter().image(texture_id, fit.image_rect, fit.uv_rect, egui::Color32::WHITE);
}
