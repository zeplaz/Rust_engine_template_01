//! Per-view map state — world truth stays in projection / terrain / overlay resources.
//!
//! **Viewport isolation:** [`MapViewInstances::world_preview`] and [`MapViewInstances::minimap`]
//! are independent [`MapViewState`] values. Minimap zoom/focus must not read
//! [`crate::gui::map_camera::MapCameraDesired`]; interaction is namespaced in
//! [`crate::gui::MapViewInteractionByView`] (per-surface queues) + [`MapShellPointerGate`].

use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;

use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::map_presentation_fit::MapFitMode;
use crate::gui::{MinimapCameraBookmark, MinimapFollowMode, MinimapOverlayMask};

use crate::gui::map_view::SmoothedMapInteraction;
use super::presentation::MapViewInstanceId;

/// Pointer-driven pan in a map view (middle mouse).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DragState {
    #[default]
    Idle,
    Panning,
}

/// Independent camera, fit, follow, layers, and interaction for one map consumer.
#[derive(Clone, Debug)]
pub struct MapViewState {
    pub camera_center: Vec2,
    pub zoom: f32,
    pub zoom_target: f32,
    pub viewport_size: Vec2,
    pub camera_initialized: bool,
    pub hovered_tile: Option<UVec2>,
    pub selected_tile: Option<UVec2>,
    pub drag_state: DragState,
    pub fit_mode: MapFitMode,
    pub follow_mode: MinimapFollowMode,
    pub layers: PreviewLayers,
    pub overlays: MinimapOverlayMask,
    pub bookmarks: Vec<MinimapCameraBookmark>,
    pub revision: u64,
    pub interaction: SmoothedMapInteraction,
}

impl Default for MapViewState {
    fn default() -> Self {
        Self {
            camera_center: Vec2::ZERO,
            zoom: 1.0,
            zoom_target: 1.0,
            viewport_size: Vec2::ZERO,
            camera_initialized: false,
            hovered_tile: None,
            selected_tile: None,
            drag_state: DragState::Idle,
            fit_mode: MapFitMode::Contain,
            follow_mode: MinimapFollowMode::Free,
            layers: PreviewLayers::default(),
            overlays: MinimapOverlayMask {
                fire_heat: true,
                logistics_heat: false,
                ..Default::default()
            },
            bookmarks: Vec::new(),
            revision: 0,
            interaction: SmoothedMapInteraction::default(),
        }
    }
}

impl MapViewState {
    pub fn reset_camera_for_map(&mut self, tex_w: f32, tex_h: f32) {
        self.camera_center = Vec2::new(tex_w * 0.5, tex_h * 0.5);
        self.camera_initialized = true;
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    pub fn tick_smooth_zoom(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let k = 1.0 - (-dt * 16.0).exp();
        self.zoom += (self.zoom_target - self.zoom) * k;
        self.zoom = self.zoom.clamp(0.35, 4.0);
    }

    pub fn clamp_zoom(&mut self) {
        self.zoom = self.zoom.clamp(0.35, 4.0);
        self.zoom_target = self.zoom_target.clamp(0.35, 4.0);
    }
}

/// Exactly two interactive map views; other [`super::MapViewInstanceId`] slots keep presentation-only state.
#[derive(Resource, Clone, Debug)]
pub struct MapViewInstances {
    pub world_preview: MapViewState,
    pub minimap: MapViewState,
}

impl Default for MapViewInstances {
    fn default() -> Self {
        let mut minimap = MapViewState::default();
        minimap.zoom = 0.85;
        minimap.zoom_target = 0.85;
        minimap.viewport_size = Vec2::new(260.0, 220.0);
        minimap.fit_mode = MapFitMode::Cover;
        minimap.overlays = crate::gui::simulation_minimap_overlay_defaults();
        Self {
            world_preview: MapViewState::default(),
            minimap,
        }
    }
}

impl MapViewInstances {
    #[must_use]
    pub fn get(&self, id: MapViewInstanceId) -> &MapViewState {
        match id {
            MapViewInstanceId::Minimap => &self.minimap,
            _ => &self.world_preview,
        }
    }

    pub fn get_mut(&mut self, id: MapViewInstanceId) -> &mut MapViewState {
        match id {
            MapViewInstanceId::Minimap => &mut self.minimap,
            _ => &mut self.world_preview,
        }
    }
}
