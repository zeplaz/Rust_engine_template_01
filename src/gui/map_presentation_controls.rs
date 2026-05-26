//! Per-consumer map layer / overlay / follow controls (shared UI widget, local state).

use bevy_egui::egui;

use crate::gui::editor::world_gen_hints as hints;
use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::map_view::MapViewState;
use crate::gui::style::{section_heading, CmdHeadingStyle, UiPalette};
use crate::gui::MinimapFollowMode;

#[inline]
fn tt(response: egui::Response, text: &'static str) -> egui::Response {
    response.on_hover_text(text)
}

fn base_layer_label(b: PreviewLayers) -> &'static str {
    if b.is_empty() {
        "None"
    } else if b.contains(PreviewLayers::REGIONS) {
        "Regions"
    } else if b.contains(PreviewLayers::ECOLOGY) {
        "Ecology"
    } else if b.contains(PreviewLayers::BIOME) {
        "Biome"
    } else if b.contains(PreviewLayers::HEIGHT) {
        "Height"
    } else if b.contains(PreviewLayers::MOISTURE) {
        "Moisture"
    } else if b.contains(PreviewLayers::TEMPERATURE) {
        "Temperature"
    } else {
        "Base"
    }
}

/// Layer / overlay / follow controls for one map consumer.
pub fn map_overlay_controls_ui(
    ui: &mut egui::Ui,
    presentation: &mut MapViewState,
    palette: &UiPalette,
    id_prefix: &str,
) {
    ui.horizontal_wrapped(|ui| {
        section_heading(ui, palette, CmdHeadingStyle::None, "Base");
        let mut base = presentation.layers.base_bits();
        egui::ComboBox::from_id_salt(format!("{id_prefix}_base_layer"))
            .selected_text(base_layer_label(base))
            .show_ui(ui, |ui| {
                let _ = tt(ui.selectable_value(&mut base, PreviewLayers::empty(), "None"), hints::PREVIEW_NONE);
                let _ = tt(ui.selectable_value(&mut base, PreviewLayers::HEIGHT, "Height"), hints::PREVIEW_HEIGHT);
                let _ = tt(ui.selectable_value(&mut base, PreviewLayers::MOISTURE, "Moisture"), hints::PREVIEW_MOIST);
                let _ = tt(
                    ui.selectable_value(&mut base, PreviewLayers::TEMPERATURE, "Temperature"),
                    hints::PREVIEW_TEMP,
                );
                let _ = tt(ui.selectable_value(&mut base, PreviewLayers::BIOME, "Biome"), hints::PREVIEW_BIOME);
                let _ = tt(ui.selectable_value(&mut base, PreviewLayers::ECOLOGY, "Ecology"), hints::PREVIEW_ECOLOGY);
                let _ = tt(ui.selectable_value(&mut base, PreviewLayers::REGIONS, "Regions"), hints::PREVIEW_REGIONS);
            });
        presentation.layers.replace_base(base);
        presentation.bump_revision();
    });
    ui.horizontal_wrapped(|ui| {
        section_heading(ui, palette, CmdHeadingStyle::None, "Overlays");
        let mut tag = presentation.layers.contains(PreviewLayers::TAG_OVERLAY);
        if tt(ui.checkbox(&mut tag, "Tags"), hints::PREVIEW_TAG).changed() {
            presentation.layers ^= PreviewLayers::TAG_OVERLAY;
            presentation.bump_revision();
        }
        let mut slope = presentation.layers.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY);
        if tt(ui.checkbox(&mut slope, "Slope"), hints::PREVIEW_SLOPE).changed() {
            presentation.layers ^= PreviewLayers::DERIVED_SLOPE_OVERLAY;
            presentation.bump_revision();
        }
        let mut mob = presentation.layers.contains(PreviewLayers::MOBILITY_OVERLAY);
        if tt(ui.checkbox(&mut mob, "Mobility"), hints::PREVIEW_MOBILITY).changed() {
            presentation.layers ^= PreviewLayers::MOBILITY_OVERLAY;
            presentation.bump_revision();
        }
        if ui
            .checkbox(&mut presentation.overlays.fire_heat, "Fire heat")
            .on_hover_text("Chunk fire heat tint from shared overlay buffers")
            .changed()
        {
            presentation.bump_revision();
        }
        if ui
            .checkbox(
                &mut presentation.overlays.logistics_heat,
                "Logistics heat",
            )
            .on_hover_text("Corridor traffic heat from LogisticsVisualSnapshot")
            .changed()
        {
            presentation.bump_revision();
        }
        if ui
            .checkbox(
                &mut presentation.overlays.construction_heat,
                "Construction heat",
            )
            .on_hover_text("Corridor / site construction phases on minimap")
            .changed()
        {
            presentation.bump_revision();
        }
        if ui
            .checkbox(
                &mut presentation.overlays.ecology_heat,
                "Ecology heat",
            )
            .on_hover_text("Ecology macro band on minimap")
            .changed()
        {
            presentation.bump_revision();
        }
    });
    ui.horizontal(|ui| {
        ui.label("Follow");
        egui::ComboBox::from_id_salt(format!("{id_prefix}_follow_mode"))
            .selected_text(match presentation.follow_mode {
                MinimapFollowMode::Free => "Free",
                MinimapFollowMode::FollowCamera => "Follow camera",
                MinimapFollowMode::FollowBookmark => "Follow bookmark",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut presentation.follow_mode, MinimapFollowMode::Free, "Free");
                ui.selectable_value(
                    &mut presentation.follow_mode,
                    MinimapFollowMode::FollowCamera,
                    "Follow camera",
                );
                ui.selectable_value(
                    &mut presentation.follow_mode,
                    MinimapFollowMode::FollowBookmark,
                    "Follow bookmark",
                );
            });
    });
}
