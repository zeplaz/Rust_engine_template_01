//! Dev diagnostics plugin — passive witness ingestion on MainWorld only.

use bevy::prelude::*;

use super::events::DiagnosticEvent;
use super::models::{
    PerfAttributionWitness, RenderScheduleHandoffMs, RenderScheduleWitness, VisualReadinessWitness,
};
use super::subscribers::{ingest_diagnostic_events_system, reset_witnesses_on_enter_simulation};
use super::visual_capture_probe::VisualCaptureProbePlugin;
use crate::engine::states::BaseState;

#[must_use]
pub fn dev_diagnostics_enabled() -> bool {
    crate::dev::diagnostic_events::dev_diagnostics_enabled()
}

pub struct DevDiagnosticsPlugin;

impl Plugin for DevDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        // VisualCaptureProbePlugin self-gates on VFX_CAPTURE independently of
        // dev_diagnostics_enabled() — registered before the early-return below so
        // `VFX_CAPTURE=1` alone (without DEV_DIAGNOSTICS/RENDER_PERF/PERF) still works,
        // as long as DevDiagnosticsPlugin itself is wired into the app (see
        // engine_with_worldgen.rs — currently unconditional `add_plugins`).
        app.add_plugins(VisualCaptureProbePlugin);

        if !dev_diagnostics_enabled() {
            return;
        }

        app.init_resource::<RenderScheduleWitness>()
            .init_resource::<RenderScheduleHandoffMs>()
            .init_resource::<PerfAttributionWitness>()
            .init_resource::<VisualReadinessWitness>()
            .add_message::<DiagnosticEvent>()
            .add_systems(
                First,
                ingest_diagnostic_events_system.after(crate::render::reset_frame_perf_counters),
            )
            .add_systems(
                OnEnter(BaseState::Simulation),
                reset_witnesses_on_enter_simulation,
            );
    }
}
