//! Top strip: base layer choice + overlay toggles + zoom controls.

use bevy_egui::egui;

use crate::gui::editor::world_gen_hints as hints;
use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::editor::world_preview::viewport::EditorViewport;
use crate::gui::style::{muted_label, section_heading, primary_label, CmdHeadingStyle, UiPalette};

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

pub fn world_preview_toolbar(
    ui: &mut egui::Ui,
    layers: &mut PreviewLayers,
    viewport: &mut EditorViewport,
    tex_w: u32,
    tex_h: u32,
    palette: &UiPalette,
) {
    ui.horizontal_wrapped(|ui| {
        section_heading(ui, palette, CmdHeadingStyle::None, "Base");
        let mut base = layers.base_bits();
        egui::ComboBox::from_id_salt("world_preview_base_layer")
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
        layers.replace_base(base);
    });
    ui.horizontal_wrapped(|ui| {
        section_heading(ui, palette, CmdHeadingStyle::None, "Overlays");
        let mut tag = layers.contains(PreviewLayers::TAG_OVERLAY);
        if tt(ui.checkbox(&mut tag, "Tags"), hints::PREVIEW_TAG).changed() {
            *layers ^= PreviewLayers::TAG_OVERLAY;
        }
        let mut slope = layers.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY);
        if tt(ui.checkbox(&mut slope, "Slope"), hints::PREVIEW_SLOPE).changed() {
            *layers ^= PreviewLayers::DERIVED_SLOPE_OVERLAY;
        }
        let mut mob = layers.contains(PreviewLayers::MOBILITY_OVERLAY);
        if tt(ui.checkbox(&mut mob, "Mobility"), hints::PREVIEW_MOBILITY).changed() {
            *layers ^= PreviewLayers::MOBILITY_OVERLAY;
        }
    });
    ui.horizontal(|ui| {
        primary_label(ui, palette, "Zoom:");
        ui.add(egui::Slider::new(
            &mut viewport.zoom,
            PreviewLayers::ZOOM_MIN..=PreviewLayers::ZOOM_MAX,
        ));
        if ui.button("1∶1").clicked() {
            viewport.zoom = 1.0;
        }
        if ui.button("Fit").clicked() {
            viewport.reset_camera_for_map(tex_w as f32, tex_h as f32);
            viewport.zoom = 1.0;
        }
    });
    muted_label(
        ui,
        palette,
        "Ctrl / ⌘ + scroll: zoom toward cursor. Middle-drag: pan.",
    );
}
