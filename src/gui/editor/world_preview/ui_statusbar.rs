//! Bottom strip: hover tile, map size, active layer summary.

use bevy_egui::egui;

use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::editor::world_preview::viewport::EditorViewport;
use crate::gui::style::{primary_label, UiPalette};

pub fn world_preview_status_bar(
    ui: &mut egui::Ui,
    layers: PreviewLayers,
    viewport: &EditorViewport,
    tex_w: u32,
    tex_h: u32,
    palette: &UiPalette,
) {
    let hover = if let Some(t) = viewport.hovered_tile {
        format!("{} , {}", t.x, t.y)
    } else {
        "—".to_string()
    };
    let base = layers.base_bits();
    let mut parts = Vec::new();
    if base.is_empty() {
        parts.push("base: none");
    } else {
        if base.contains(PreviewLayers::REGIONS) {
            parts.push("regions");
        }
        if base.contains(PreviewLayers::BIOME) {
            parts.push("biome");
        }
        if base.contains(PreviewLayers::HEIGHT) {
            parts.push("height");
        }
        if base.contains(PreviewLayers::MOISTURE) {
            parts.push("moisture");
        }
        if base.contains(PreviewLayers::TEMPERATURE) {
            parts.push("temperature");
        }
    }
    if layers.contains(PreviewLayers::TAG_OVERLAY) {
        parts.push("+tags");
    }
    if layers.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY) {
        parts.push("+slope");
    }
    if layers.contains(PreviewLayers::MOBILITY_OVERLAY) {
        parts.push("+mobility");
    }
    primary_label(
        ui,
        palette,
        format!(
            "Tile: {} | {}×{} world | z={:.2} | {}",
            hover,
            tex_w,
            tex_h,
            viewport.zoom,
            parts.join(" ")
        ),
    );
}
