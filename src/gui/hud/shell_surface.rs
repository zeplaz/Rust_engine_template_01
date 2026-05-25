//! Shell surface presentation modes — egui immediate vs cached texture vs native stub.

use bevy::prelude::*;

use super::shell_framework::ProductShellWidgetId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShellSurfaceMode {
    #[default]
    ImmediateEgui,
    CachedTextureSurface,
    NativeStub,
}

#[derive(Resource, Clone, Debug)]
pub struct ShellSurfacePolicy {
    pub default_mode: ShellSurfaceMode,
    pub minimap_legend: ShellSurfaceMode,
    pub telemetry_graph: ShellSurfaceMode,
    pub explainability_pane: ShellSurfaceMode,
    pub overlay_key_panel: ShellSurfaceMode,
}

impl Default for ShellSurfacePolicy {
    fn default() -> Self {
        Self {
            default_mode: ShellSurfaceMode::ImmediateEgui,
            minimap_legend: ShellSurfaceMode::CachedTextureSurface,
            telemetry_graph: ShellSurfaceMode::CachedTextureSurface,
            explainability_pane: ShellSurfaceMode::CachedTextureSurface,
            overlay_key_panel: ShellSurfaceMode::CachedTextureSurface,
        }
    }
}

impl ShellSurfacePolicy {
    #[must_use]
    pub const fn mode_for_widget(&self, id: ProductShellWidgetId) -> ShellSurfaceMode {
        match id {
            ProductShellWidgetId::Minimap => self.minimap_legend,
            ProductShellWidgetId::Explainability | ProductShellWidgetId::IntelTimeline => {
                self.explainability_pane
            }
            ProductShellWidgetId::OverlaysPanel | ProductShellWidgetId::OverlayTray => {
                self.overlay_key_panel
            }
            ProductShellWidgetId::CommandShell => self.telemetry_graph,
            _ => self.default_mode,
        }
    }

    #[must_use]
    pub const fn uses_cached_texture(&self, mode: ShellSurfaceMode) -> bool {
        matches!(mode, ShellSurfaceMode::CachedTextureSurface)
    }
}
