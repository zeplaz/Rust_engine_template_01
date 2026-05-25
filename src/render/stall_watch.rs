//! Sub-frame stall probes inside `cpu_pre_egui` (`PERF=1`, `STALL=1`, or segment ≥ threshold).
//!
//! Checkpoints bracket PreUpdate → Update → PostUpdate (readiness, egui). Logs `STALL {label}: Xms`
//! when a segment exceeds [`STALL_THRESHOLD_MS`].

use std::time::Instant;

use bevy::prelude::*;
use bevy_egui::EguiPostUpdateSet;

use crate::render::evaluate_app_stage5_readiness;
use crate::render::frame_perf::{
    frame_perf_verbose, reset_frame_perf_counters, stamp_frame_wall_last,
};

/// Log and retain segments at or above this duration (ms).
pub const STALL_THRESHOLD_MS: f32 = 5.0;

/// Wall-clock spans between major schedule boundaries (ms).
#[derive(Resource, Clone, Debug, Default)]
pub struct FrameScheduleSpans {
    pub first_to_preupdate_ms: f32,
    pub update_ms: f32,
    /// PostUpdate start → domain projection merge.
    pub postupdate_domain_merge_ms: f32,
    /// Domain merge → VT/CI matrix record.
    pub postupdate_vt_ci_ms: f32,
    /// VT/CI → readiness eval start.
    pub postupdate_to_readiness_ms: f32,
    pub readiness_ms: f32,
    pub post_readiness_to_pre_egui_ms: f32,
    pub egui_ms: f32,
    pub post_egui_to_last_ms: f32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FrameStallWatch {
    last: Option<Instant>,
    pub segments: Vec<(String, f32)>,
    pub spans: FrameScheduleSpans,
}

#[must_use]
pub fn stall_watch_enabled() -> bool {
    frame_perf_verbose()
        || std::env::var("STALL")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

impl FrameStallWatch {
    pub fn reset(&mut self, t: Instant) {
        self.last = Some(t);
        self.segments.clear();
        self.spans = FrameScheduleSpans::default();
    }

    pub fn checkpoint(&mut self, label: &str) {
        let now = Instant::now();
        let Some(prev) = self.last.replace(now) else {
            return;
        };
        let ms = now.duration_since(prev).as_secs_f32() * 1000.0;
        self.record_span(label, ms);
        if stall_watch_enabled() && ms >= STALL_THRESHOLD_MS {
            self.segments.push((label.to_string(), ms));
            info!(target: "stall", "STALL {label}: {ms:.2}ms");
        }
    }

    fn record_span(&mut self, label: &str, ms: f32) {
        match label {
            "preupdate_end" => self.spans.first_to_preupdate_ms = ms,
            "postupdate_begin" => self.spans.update_ms = ms,
            "after_domain_merge" => self.spans.postupdate_domain_merge_ms = ms,
            "after_vt_ci" => self.spans.postupdate_vt_ci_ms = ms,
            "before_readiness" => self.spans.postupdate_to_readiness_ms = ms,
            "after_readiness" => self.spans.readiness_ms = ms,
            "pre_egui" => self.spans.post_readiness_to_pre_egui_ms = ms,
            "post_egui" => self.spans.egui_ms = ms,
            "last" => self.spans.post_egui_to_last_ms = ms,
            _ => {}
        }
    }
}

pub fn reset_stall_watch(mut watch: ResMut<FrameStallWatch>) {
    watch.reset(Instant::now());
}

pub fn stall_preupdate_end(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("preupdate_end");
}

pub fn stall_postupdate_begin(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("postupdate_begin");
}

pub fn stall_before_readiness(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("before_readiness");
}

pub fn stall_after_readiness(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("after_readiness");
}

pub fn stall_pre_egui(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("pre_egui");
}

pub fn stall_post_egui(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("post_egui");
}

pub fn stall_last(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("last");
}

pub fn stall_after_vt_ci(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("after_vt_ci");
}

pub fn stall_after_domain_merge(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("after_domain_merge");
}

pub struct StallWatchPlugin;

impl Plugin for StallWatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameStallWatch>()
            .add_systems(First, reset_stall_watch.after(reset_frame_perf_counters))
            .add_systems(PreUpdate, stall_preupdate_end)
            .add_systems(PostUpdate, stall_postupdate_begin)
            .add_systems(
                PostUpdate,
                stall_after_domain_merge.after(crate::render::merge_domain_projection_into_representation),
            )
            .add_systems(
                PostUpdate,
                stall_after_vt_ci.after(crate::render::vt_ci_matrix::record_vt_ci_matrix_live),
            )
            .add_systems(
                PostUpdate,
                stall_before_readiness.before(evaluate_app_stage5_readiness),
            )
            .add_systems(
                PostUpdate,
                stall_after_readiness.after(evaluate_app_stage5_readiness),
            )
            .add_systems(
                PostUpdate,
                (
                    stall_pre_egui.before(EguiPostUpdateSet::EndPass),
                    stall_post_egui.after(EguiPostUpdateSet::PostProcessOutput),
                ),
            )
            .add_systems(
                Last,
                stall_last.before(stamp_frame_wall_last),
            );
    }
}
