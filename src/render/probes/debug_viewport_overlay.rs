//! On-screen viewport debug window + stroke overlays (RTT fill rect + ortho trace).

use bevy::prelude::*;

use crate::gui::hud::viewport_debug_overlay_enabled;

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
