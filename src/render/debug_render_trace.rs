//! Temporary focused render-routing traces (CLI `--debug-*` flags).

use bevy::prelude::*;

pub const TRACE_VIEWPORT: &str = "proc_A_dine01::gui::editor::world_preview";
pub const TRACE_CAMERA: &str = "proc_A_dine01::gui::view_representation";
pub const TRACE_RENDER_TARGET: &str = "proc_A_dine01::render::tile_world_fallback";
pub const TRACE_PARTICLES: &str = "proc_A_dine01::render::gpu_particles";

/// One-shot toggles for viewport / camera / render-target / particle routing traces.
#[derive(Resource, Debug, Clone, Copy)]
pub struct DebugRenderTraceConfig {
    pub viewport_trace: bool,
    pub camera_sync_trace: bool,
    pub render_target_trace: bool,
    pub particle_routing_trace: bool,
    /// Window vs sim-map hole vs camera scissor vs ortho fit (`sim_view_sync` target).
    pub sim_view_sync_trace: bool,
    /// Consolidated visual / viewport / spine snapshot (`visual_diag` target).
    pub visual_diag_trace: bool,
}

impl Default for DebugRenderTraceConfig {
    fn default() -> Self {
        Self {
            viewport_trace: false,
            camera_sync_trace: false,
            render_target_trace: false,
            particle_routing_trace: false,
            sim_view_sync_trace: false,
            visual_diag_trace: false,
        }
    }
}

impl DebugRenderTraceConfig {
    #[must_use]
    pub fn from_cli_flags(
        viewport_trace: bool,
        camera_sync_trace: bool,
        render_routing: bool,
        sim_view_sync_trace: bool,
        visual_diag_trace: bool,
    ) -> Self {
        Self {
            viewport_trace,
            camera_sync_trace,
            render_target_trace: render_routing,
            particle_routing_trace: render_routing,
            sim_view_sync_trace,
            visual_diag_trace,
        }
    }

    #[must_use]
    pub fn any_enabled(self) -> bool {
        self.viewport_trace
            || self.camera_sync_trace
            || self.render_target_trace
            || self.particle_routing_trace
            || self.sim_view_sync_trace
            || self.visual_diag_trace
    }
}

#[inline]
pub fn trace_viewport(cfg: &DebugRenderTraceConfig, message: &str) {
    if cfg.viewport_trace {
        bevy::log::debug!(target: TRACE_VIEWPORT, "{message}");
    }
}

#[inline]
pub fn trace_camera_sync(cfg: &DebugRenderTraceConfig, message: &str) {
    if cfg.camera_sync_trace {
        bevy::log::debug!(target: TRACE_CAMERA, "{message}");
    }
}

#[inline]
pub fn trace_render_target(cfg: &DebugRenderTraceConfig, message: &str) {
    if cfg.render_target_trace {
        bevy::log::debug!(target: TRACE_RENDER_TARGET, "{message}");
    }
}

#[inline]
pub fn trace_particle_routing(cfg: &DebugRenderTraceConfig, message: &str) {
    if cfg.particle_routing_trace {
        bevy::log::debug!(target: TRACE_PARTICLES, "{message}");
    }
}
