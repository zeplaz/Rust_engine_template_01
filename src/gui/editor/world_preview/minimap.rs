//! World Preview overview inset (D-07 corner on map; sidebar thumb removed).

use bevy::math::UVec2;
use bevy_egui::egui;

use crate::gui::map_presentation_fit::{compute_map_fit_strict, MapFitMode, MAP_PANEL_INSET_PX};
use crate::gui::style::UiPalette;

/// D-07 A: bottom-right corner inset margin from map viewport edge (px).
const D07_CORNER_MARGIN_PX: f32 = 10.0;

/// Paint overview minimap as a corner inset on the map viewport (D-07). Returns inset side (px).
pub fn paint_world_preview_minimap_corner_inset(
    ui: &mut egui::Ui,
    map_viewport_rect: egui::Rect,
    texture_id: egui::TextureId,
    tex_w: u32,
    tex_h: u32,
    palette: &UiPalette,
) -> f32 {
    let side = super::d07_inset_side_px();
    let inset_rect = egui::Rect::from_min_size(
        egui::pos2(
            map_viewport_rect.right() - side - D07_CORNER_MARGIN_PX,
            map_viewport_rect.bottom() - side - D07_CORNER_MARGIN_PX,
        ),
        egui::vec2(side, side),
    );
    let painter = ui.painter().with_clip_rect(map_viewport_rect);
    let stroke = egui::Stroke::new(1.5, palette.wire_magenta);
    painter.rect_filled(
        inset_rect,
        4.0,
        egui::Color32::from_black_alpha(168),
    );
    painter.rect_stroke(inset_rect, 4.0, stroke, egui::StrokeKind::Inside);
    let fit = compute_map_fit_strict(
        inset_rect.shrink(MAP_PANEL_INSET_PX),
        UVec2::new(tex_w.max(1), tex_h.max(1)),
        MapFitMode::Contain,
    );
    painter.image(texture_id, fit.image_rect, fit.uv_rect, egui::Color32::WHITE);
    side
}
