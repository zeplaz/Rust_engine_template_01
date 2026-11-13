//! Dev diagnostics plugin — registers the env-gated capture probe.

use bevy::prelude::*;

use super::visual_capture_probe::VisualCaptureProbePlugin;

pub struct DevDiagnosticsPlugin;

impl Plugin for DevDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        // VisualCaptureProbePlugin self-gates on VFX_CAPTURE.
        app.add_plugins(VisualCaptureProbePlugin);
    }
}
