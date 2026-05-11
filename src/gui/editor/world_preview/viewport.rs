//! Camera / zoom / hover state for the world preview (editor-style viewport).

use bevy::math::{UVec2, Vec2};
use bevy::prelude::Resource;

/// Pointer-driven pan in the preview (middle mouse).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DragState {
    #[default]
    Idle,
    Panning,
}

#[derive(Resource)]
pub struct EditorViewport {
    /// World tile at the viewport center (integer corner grid: x right, y down).
    pub camera_center: Vec2,
    pub zoom: f32,
    /// Last allocated inner viewport size (screen pixels).
    pub viewport_size: Vec2,
    pub hovered_tile: Option<UVec2>,
    pub selected_tile: Option<UVec2>,
    pub drag_state: DragState,
    /// After world size changes, re-center once.
    pub camera_initialized: bool,
}

impl EditorViewport {
    pub fn reset_camera_for_map(&mut self, tex_w: f32, tex_h: f32) {
        self.camera_center = Vec2::new(tex_w * 0.5, tex_h * 0.5);
        self.camera_initialized = true;
    }
}

impl Default for EditorViewport {
    fn default() -> Self {
        Self {
            camera_center: Vec2::ZERO,
            zoom: 1.0,
            viewport_size: Vec2::ZERO,
            hovered_tile: None,
            selected_tile: None,
            drag_state: DragState::Idle,
            camera_initialized: false,
        }
    }
}
