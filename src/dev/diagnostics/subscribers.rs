//! MainWorld subscribers — Update/First only; never ExtractSchedule or RenderApp.

use bevy::prelude::*;

use super::events::DiagnosticEvent;
use super::models::{
    PerfAttributionWitness, RenderScheduleWitness, VisualReadinessWitness,
};
use crate::dev::runtime_witness::write_enveloped_witness;

pub fn ingest_diagnostic_events_system(
    mut events: MessageReader<DiagnosticEvent>,
    mut render_witness: ResMut<RenderScheduleWitness>,
    mut perf_witness: ResMut<PerfAttributionWitness>,
    mut visual_witness: ResMut<VisualReadinessWitness>,
) {
    for event in events.read() {
        match event {
            DiagnosticEvent::RenderSchedule(e) => {
                render_witness.spans = e.spans.clone();
                render_witness.main_thread_handoff_total_ms = e.main_thread_handoff_total_ms;
                render_witness.frames_received = render_witness.frames_received.saturating_add(1);
            }
            DiagnosticEvent::PerfAttribution(e) => {
                if e.frame_ms > 0.0 {
                    perf_witness.record_frame(e.frame_ms, e.raster_b_ms, e.view_fire_ms);
                }
            }
            DiagnosticEvent::VisualReadiness(e) => {
                visual_witness.sim_valid_streak = e.sim_valid_streak;
                visual_witness.primary_window_presentable = e.primary_window_presentable;
                visual_witness.frame_budget_last_ms = e.frame_budget_last_ms;
                visual_witness.render_hole_steady_flip_count = e.render_hole_steady_flip_count;
                visual_witness.frames_sampled = visual_witness.frames_sampled.saturating_add(1);
            }
            DiagnosticEvent::MigAAudit(e) => {
                let _ = write_enveloped_witness(
                    "mig_a_audit",
                    "dev::diagnostics::ingest_diagnostic_events_system",
                    &e.relative_json_path,
                    e.body.clone(),
                );
            }
        }
    }
}

pub fn reset_witnesses_on_enter_simulation(
    mut visual: ResMut<VisualReadinessWitness>,
    mut perf: ResMut<PerfAttributionWitness>,
    mut render: ResMut<RenderScheduleWitness>,
) {
    *visual = VisualReadinessWitness::default();
    *perf = PerfAttributionWitness::default();
    *render = RenderScheduleWitness::default();
}
