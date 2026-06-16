//! Dev gate: force legacy egui minimap instead of Bevy GPU chrome (MINIMAP-WIDGET-IMPL-001).

use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub struct MinimapEguiDevGate {
    /// When true, Bevy GPU minimap pointer systems stand down and egui texture path wins.
    pub force_egui_minimap: bool,
}

#[must_use]
pub fn minimap_egui_dev_enabled(gate: Option<&MinimapEguiDevGate>) -> bool {
    gate.is_some_and(|g| g.force_egui_minimap)
}
