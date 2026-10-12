//! On-screen viewport debug window + stroke overlays (RTT fill rect + ortho trace).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::gui::hud::{
    stroke_viewport_debug_rect, viewport_debug_overlay_enabled, CameraProjectionInfo,
    UiViewportRect,
};
use crate::gui::{
    MainWorldCameraOrthoTrace, SimulationMapTexture, SimulationMapViewport,
    SimulationMapViewportDebug, SimulationMapViewportTrace,
};
use crate::gui::simulation_map_texture_extent;
use crate::render::ResolvedViewports;

#[inline]
pub fn debug_viewport_overlay_enabled() -> bool {
    viewport_debug_overlay_enabled()
}

/// F4-adjacent: always on when `VIEWPORT_DEBUG_OVERLAY=1` / `VISUAL_DIAG=1`.
pub fn draw_debug_viewport_overlay() {}

pub struct DebugViewportOverlayPlugin;

impl Plugin for DebugViewportOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_debug_viewport_overlay);
    }
}
