//! Per-consumer map layer / overlay / follow controls (shared UI widget, local state).


use crate::gui::editor::world_gen_hints as hints;
use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::map_view::MapViewState;
use crate::gui::style::{section_heading, CmdHeadingStyle, UiPalette};
use crate::gui::MinimapFollowMode;

#[inline]
fn tt() {}

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
pub fn map_overlay_controls_ui() {}
