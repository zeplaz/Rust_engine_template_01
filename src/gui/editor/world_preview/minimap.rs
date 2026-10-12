//! World Preview overview inset (D-07 corner on map; sidebar thumb removed).

use bevy::math::UVec2;

use crate::gui::map_presentation_fit::{compute_map_fit_strict, MapFitMode, MAP_PANEL_INSET_PX};
use crate::gui::style::UiPalette;

/// D-07 A: bottom-right corner inset margin from map viewport edge (px).
const D07_CORNER_MARGIN_PX: f32 = 10.0;

/// Paint overview minimap as a corner inset on the map viewport (D-07). Returns inset side (px).
#[allow(dead_code)] // Corner inset is a Bevy ImageNode on the WorldGen preview panel.
pub fn paint_world_preview_minimap_corner_inset() {}
