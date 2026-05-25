//! Per-consumer map presentation state (layers, overlays, follow, bookmarks).

mod display_transform;
pub mod map_interaction;
mod stability;

use bevy::prelude::*;

pub use display_transform::{paint_map_display_debug_outlines, MapDisplayTransform};
pub use map_interaction::{
    advance_map_view_presentation_interactions, sync_map_view_interaction_targets,
    MapViewPresentationInteractions, SmoothedMapInteraction,
};
pub use stability::{
    clear_active_map_view_input_before_map_widgets, commit_map_view_interaction_system,
    commit_map_view_viewport_suggestions, paint_map_view_placeholder, sync_map_view_ready_states,
    sync_shell_layout_drag_gate, update_minimap_view, update_world_preview_view, ActiveMapViewInput,
    MapShellPointerGate,
    MapViewInteractionByView, MapViewReadyStates, MapViewViewportSuggestions,
    MinimapInteractionBuffer, ViewHandle, WorldPreviewInteractionBuffer,
};

/// Stable consumer id for map presentation instances.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MapViewInstanceId {
    WorldPreview,
    Minimap,
    SimulationMap,
    TacticalMap,
    FullscreenMap,
    CommanderMap,
    Stage7IntelMap,
}

/// Local presentation for auxiliary map consumers (not world preview / minimap).
#[derive(Clone, Debug)]
pub struct MapViewPresentationState {
    pub layers: crate::gui::editor::world_preview::layers::PreviewLayers,
    pub overlays: crate::gui::MinimapOverlayMask,
    pub follow_mode: crate::gui::MinimapFollowMode,
    pub bookmarks: Vec<crate::gui::MinimapCameraBookmark>,
    pub fit_mode: crate::gui::map_presentation_fit::MapFitMode,
    pub revision: u64,
}

impl Default for MapViewPresentationState {
    fn default() -> Self {
        let mut layers = crate::gui::editor::world_preview::layers::PreviewLayers::default();
        layers.replace_base(crate::gui::editor::world_preview::layers::PreviewLayers::BIOME);
        Self {
            layers,
            overlays: crate::gui::MinimapOverlayMask {
                fire_heat: true,
                logistics_heat: false,
                ..Default::default()
            },
            follow_mode: crate::gui::MinimapFollowMode::Free,
            bookmarks: Vec::new(),
            fit_mode: crate::gui::map_presentation_fit::MapFitMode::Contain,
            revision: 0,
        }
    }
}

impl MapViewPresentationState {
    pub fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

/// Presentation for non-primary map consumers; world preview + minimap use [`super::MapViewInstances`].
#[derive(Resource, Clone, Debug, Default)]
pub struct MapViewPresentationStates {
    simulation_map: MapViewPresentationState,
    tactical_map: MapViewPresentationState,
    fullscreen_map: MapViewPresentationState,
    commander_map: MapViewPresentationState,
    stage7_intel_map: MapViewPresentationState,
}

impl MapViewPresentationStates {
    pub fn get(&self, id: MapViewInstanceId) -> &MapViewPresentationState {
        match id {
            MapViewInstanceId::SimulationMap => &self.simulation_map,
            MapViewInstanceId::TacticalMap => &self.tactical_map,
            MapViewInstanceId::FullscreenMap => &self.fullscreen_map,
            MapViewInstanceId::CommanderMap => &self.commander_map,
            MapViewInstanceId::Stage7IntelMap => &self.stage7_intel_map,
            MapViewInstanceId::WorldPreview | MapViewInstanceId::Minimap => {
                panic!("world preview and minimap presentation live in MapViewInstances")
            }
        }
    }

    pub fn get_mut(&mut self, id: MapViewInstanceId) -> &mut MapViewPresentationState {
        match id {
            MapViewInstanceId::SimulationMap => &mut self.simulation_map,
            MapViewInstanceId::TacticalMap => &mut self.tactical_map,
            MapViewInstanceId::FullscreenMap => &mut self.fullscreen_map,
            MapViewInstanceId::CommanderMap => &mut self.commander_map,
            MapViewInstanceId::Stage7IntelMap => &mut self.stage7_intel_map,
            MapViewInstanceId::WorldPreview | MapViewInstanceId::Minimap => {
                panic!("world preview and minimap presentation live in MapViewInstances")
            }
        }
    }
}
