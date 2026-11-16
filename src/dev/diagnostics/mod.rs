//! Dev diagnostics — real GPU-pixel capture probe (`VFX_CAPTURE=1`).
//!
//! **V4-DBG04** · **DBG-04** / LEGACY `render_debug_tools` — see [`visual_capture_probe`] and
//! [`crate::render::probes::vfx_capture_env`].

pub mod plugin;
pub mod visual_capture_probe;

pub use plugin::DevDiagnosticsPlugin;
pub use visual_capture_probe::{
    visual_capture_probe_enabled, VisualCaptureProbePlugin, VisualCaptureProbeState,
};
