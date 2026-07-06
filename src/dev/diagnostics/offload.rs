//! Producer-side trait — rendering systems emit events instead of mutating witness resources.

use bevy::prelude::*;

use super::events::{
    DiagnosticEvent, MigAAuditEvent, PerfAttributionEvent, RenderScheduleEvent,
    VisualReadinessEvent,
};

/// Implement on thin render/sim probes; subscribers in this module own witness Resources + I/O.
pub trait OffloadDiagnostics {
    fn emit_render_schedule(
        &mut self,
        writer: &mut MessageWriter<DiagnosticEvent>,
        spans: super::models::RenderScheduleSpans,
        main_thread_handoff_total_ms: f32,
    ) {
        writer.write(DiagnosticEvent::RenderSchedule(RenderScheduleEvent::new(
            spans,
            main_thread_handoff_total_ms,
        )));
    }

    fn emit_perf_attribution(
        &mut self,
        writer: &mut MessageWriter<DiagnosticEvent>,
        frame_ms: f32,
        raster_b_ms: f32,
        view_fire_ms: f32,
    ) {
        writer.write(DiagnosticEvent::PerfAttribution(PerfAttributionEvent {
            frame_ms,
            raster_b_ms,
            view_fire_ms,
        }));
    }

    fn emit_visual_readiness(
        &mut self,
        writer: &mut MessageWriter<DiagnosticEvent>,
        patch: VisualReadinessEvent,
    ) {
        writer.write(DiagnosticEvent::VisualReadiness(patch));
    }

    fn emit_mig_a_audit(
        &mut self,
        writer: &mut MessageWriter<DiagnosticEvent>,
        relative_json_path: impl Into<String>,
        body: serde_json::Value,
    ) {
        writer.write(DiagnosticEvent::MigAAudit(MigAAuditEvent {
            relative_json_path: relative_json_path.into(),
            body,
        }));
    }
}

/// Blanket impl — any system param bundle can call emit helpers.
impl<T: ?Sized> OffloadDiagnostics for T {}
