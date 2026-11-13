//! Dev diagnostics — real GPU-pixel capture probe (`VFX_CAPTURE=1`).

pub mod plugin;
pub mod visual_capture_probe;

pub use plugin::DevDiagnosticsPlugin;
pub use visual_capture_probe::{
    visual_capture_probe_enabled, VisualCaptureProbePlugin, VisualCaptureProbeState,
};
