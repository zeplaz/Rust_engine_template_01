//! Operator overlay — what is rendered, viewport rects, zoom, ghost tile (F3 / VfxSandbox).

use bevy::prelude::*;

use crate::construction::{BuildGhostState, BuildStripState};
use crate::engine::launch_args::{EngineLaunchArgs, TestScene};
use crate::gui::{
    MapCameraDesiredRes, MapPresentationDiagnostics, MinimapShellState,
    SimulationMapViewport, ViewManager,
};
use crate::render::{SharedOverlayFieldBuffers, TileWorldFallbackState};

/// Toggle via diagnostics or `--test renderdebug` / VfxSandbox bootstrap.
#[derive(Resource, Clone, Debug)]
pub struct RenderSurfaceDebugOverlay {
    pub enabled: bool,
}

impl Default for RenderSurfaceDebugOverlay {
    fn default() -> Self {
        Self { enabled: false }
    }
}

pub fn draw_render_surface_debug_overlay() -> Result { Ok(()) }
