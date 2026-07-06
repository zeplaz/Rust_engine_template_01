//! Diagnostic event bus types — render-safe import surface (**CB-MIG-003**).
//!
//! Lives at `dev/` root so `render/` can emit without pulling the full diagnostics module graph.

use bevy::prelude::*;

use crate::render::RenderScheduleSpans;

#[must_use]
pub fn dev_diagnostics_enabled() -> bool {
    std::env::var("DEV_DIAGNOSTICS")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || std::env::var("RENDER_PERF")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || std::env::var("PERF")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || std::env::var("STALL_WATCH")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || crate::dev::test_run_instrumentation::instrumentation_active()
}

/// Umbrella message for passive diagnostic offload (extend per batch).
#[derive(Message, Clone, Debug)]
pub enum DiagnosticEvent {
    RenderSchedule(RenderScheduleEvent),
    PerfAttribution(PerfAttributionEvent),
    VisualReadiness(VisualReadinessEvent),
    MigAAudit(MigAAuditEvent),
}

#[derive(Message, Clone, Debug)]
pub struct RenderScheduleEvent {
    pub spans: RenderScheduleSpans,
    pub main_thread_handoff_total_ms: f32,
}

impl RenderScheduleEvent {
    #[must_use]
    pub fn new(spans: RenderScheduleSpans, main_thread_handoff_total_ms: f32) -> Self {
        Self {
            spans,
            main_thread_handoff_total_ms,
        }
    }
}

#[derive(Message, Clone, Debug)]
pub struct PerfAttributionEvent {
    pub frame_ms: f32,
    pub raster_b_ms: f32,
    pub view_fire_ms: f32,
}

#[derive(Message, Clone, Debug, Default)]
pub struct VisualReadinessEvent {
    pub sim_valid_streak: u32,
    pub primary_window_presentable: bool,
    pub frame_budget_last_ms: f32,
    pub render_hole_steady_flip_count: u32,
}

#[derive(Message, Clone, Debug)]
pub struct MigAAuditEvent {
    pub relative_json_path: String,
    pub body: serde_json::Value,
}
