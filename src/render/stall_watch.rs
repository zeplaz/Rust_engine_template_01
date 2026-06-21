//! Sub-frame stall probes inside `cpu_pre_egui` (`PERF=1`, `STALL=1`, or segment ≥ threshold).
//!
//! Checkpoints bracket PreUpdate → Update → PostUpdate (readiness, egui). Logs `STALL {label}: Xms`
//! when a segment exceeds [`STALL_THRESHOLD_MS`].
//!
//! **Update span naming:** Checkpoints are wall time since the *previous* checkpoint in the frame.
//! [`FrameScheduleSpans::after_map_camera_smooth_ms`] is only the map-camera chain; the legacy
//! `to_map` total is [`FrameScheduleSpans::update_pre_map_camera_ms`] +
//! [`FrameScheduleSpans::map_camera_chain_ms`]. [`FrameScheduleSpans::before_world_repr_ms`] is
//! fire-build → world-repr, not “pre-streaming”. Streaming ends with
//! [`FrameScheduleSpans::post_streaming_spine_ms`] late in Update.

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
    /// Update: domain projection merge only (see domain_projection_frame stall checkpoints).
    pub domain_merge_ms: f32,
    /// PostUpdate start → readiness eval (map/egui prep — not domain merge).
    pub postupdate_main_ms: f32,
    /// Legacy alias — prefer [`Self::postupdate_main_ms`].
    pub postupdate_domain_merge_ms: f32,
    /// Domain merge → VT/CI matrix record.
    pub postupdate_vt_ci_ms: f32,
    /// VT/CI → readiness eval start.
    pub postupdate_to_readiness_ms: f32,
    pub readiness_ms: f32,
    pub post_readiness_to_pre_egui_ms: f32,
    /// VT/CI → readiness eval → pre-egui stamp (post-VT gap; see PERF `post_vt→egui`).
    pub post_vt_to_pre_egui_ms: f32,
    pub egui_ms: f32,
    pub post_egui_to_last_ms: f32,
    /// Update: end of streaming spine reconstruct chain (late Update, after world repr).
    pub post_streaming_spine_ms: f32,
    /// Update: PreUpdate end → first pre-repr slice (usually dominates when “pre_repr” is huge).
    pub before_world_repr_ms: f32,
    /// Update: `update_begin` → [`crate::gui::MapCameraSystemSet::ApplyInput`] (un-ordered Update work).
    pub update_pre_map_camera_ms: f32,
    /// Update: map camera ApplyInput → Smooth.
    pub map_camera_chain_ms: f32,
    /// Update: PreUpdate end → map camera smooth (legacy alias; use [`Self::update_pre_map_camera_ms`] + [`Self::map_camera_chain_ms`]).
    pub after_map_camera_smooth_ms: f32,
    /// Update: [`crate::gui::ViewAuthoritySystemSet::SyncViewManager`] → next checkpoint.
    pub after_view_sync_ms: f32,
    /// Update: [`crate::render::extraction::FireVisualFrameSet::BuildProfiles`] → world repr.
    pub after_fire_build_ms: f32,
    /// Update: world representation / LOD compute only.
    pub post_world_repr_ms: f32,
    /// Update: fire visual extract through ProjectGpu.
    pub post_fire_project_ms: f32,
    /// Update: domain projection merge.
    pub post_domain_merge_ms: f32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FrameStallWatch {
    last: Option<Instant>,
    postupdate_started: Option<Instant>,
    pub segments: Vec<(String, f32)>,
    pub spans: FrameScheduleSpans,
}

#[must_use]
pub fn stall_watch_enabled() -> bool {
    frame_perf_verbose()
        || stall_span_debug_enabled()
        || std::env::var("STALL")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || crate::dev::test_run_instrumentation::instrumentation_stall_spans()
}

/// Extra Update checkpoints + 1ms stall lines (`STALL_SPAN_DEBUG=1`). Pair with `PERF=1` + `STALL=1`.
#[must_use]
pub fn stall_span_debug_enabled() -> bool {
    std::env::var("STALL_SPAN_DEBUG")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[must_use]
fn stall_log_threshold_ms() -> f32 {
    if stall_span_debug_enabled() {
        1.0
    } else {
        STALL_THRESHOLD_MS
    }
}

impl FrameStallWatch {
    pub fn reset(&mut self, t: Instant) {
        self.last = Some(t);
        self.postupdate_started = None;
        self.segments.clear();
        self.spans = FrameScheduleSpans::default();
    }

    pub fn checkpoint(&mut self, label: &str) {
        self.checkpoint_at(label, Instant::now());
    }

    fn checkpoint_at(&mut self, label: &str, now: Instant) {
        let Some(prev) = self.last.replace(now) else {
            return;
        };
        let ms = now.duration_since(prev).as_secs_f32() * 1000.0;
        self.record_span(label, ms);
        if stall_watch_enabled() && ms >= stall_log_threshold_ms() {
            self.segments.push((label.to_string(), ms));
            info!(target: "stall", "STALL {label}: {ms:.2}ms");
        }
    }

    fn record_span(&mut self, label: &str, ms: f32) {
        match label {
            "preupdate_end" => self.spans.first_to_preupdate_ms = ms,
            "post_streaming_spine" => self.spans.post_streaming_spine_ms = ms,
            "update_begin" => {}
            "before_map_camera" => self.spans.update_pre_map_camera_ms = ms,
            "after_map_camera_smooth" => {
                self.spans.map_camera_chain_ms = ms;
            }
            "after_view_sync" => self.spans.after_view_sync_ms = ms,
            "after_fire_build" => self.spans.after_fire_build_ms = ms,
            "before_world_repr" => self.spans.before_world_repr_ms = ms,
            "post_world_repr" => self.spans.post_world_repr_ms = ms,
            "post_fire_project" => self.spans.post_fire_project_ms = ms,
            "postupdate_begin" => self.spans.update_ms = ms,
            "before_domain_merge" => {}
            "after_domain_merge" => {
                self.spans.domain_merge_ms = ms;
                self.spans.post_domain_merge_ms = ms;
            }
            "postupdate_head" => {}
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
    watch.postupdate_started = watch.last;
    watch.checkpoint("postupdate_begin");
}

pub fn stall_before_readiness(mut watch: ResMut<FrameStallWatch>) {
    let now = Instant::now();
    if let Some(t0) = watch.postupdate_started {
        let ms = now.duration_since(t0).as_secs_f32() * 1000.0;
        watch.spans.postupdate_main_ms = ms;
        watch.spans.postupdate_domain_merge_ms = ms;
    }
    watch.checkpoint_at("before_readiness", now);
}

pub fn stall_after_readiness(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("after_readiness");
}

pub fn stall_pre_egui(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("pre_egui");
    let sp = &mut watch.spans;
    sp.post_vt_to_pre_egui_ms = sp.postupdate_vt_ci_ms
        + sp.postupdate_to_readiness_ms
        + sp.readiness_ms
        + sp.post_readiness_to_pre_egui_ms;
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

/// Early PostUpdate span (not domain merge — see [`crate::render::domain_projection_frame::stall_checkpoint_after_domain_merge`]).
pub fn stall_postupdate_head(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("postupdate_head");
}

pub fn stall_checkpoint_post_streaming_spine(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("post_streaming_spine");
}

pub fn stall_checkpoint_before_world_repr(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("before_world_repr");
}

pub fn stall_checkpoint_post_world_repr(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("post_world_repr");
}

pub fn stall_checkpoint_post_fire_project(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("post_fire_project");
}

pub fn stall_checkpoint_update_begin(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("update_begin");
}

pub fn stall_checkpoint_before_map_camera(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("before_map_camera");
}

pub fn stall_checkpoint_after_map_camera_smooth(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("after_map_camera_smooth");
}

pub fn stall_checkpoint_after_view_sync(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("after_view_sync");
}

pub fn stall_checkpoint_after_fire_build(mut watch: ResMut<FrameStallWatch>) {
    watch.checkpoint("after_fire_build");
}

pub struct StallWatchPlugin;

impl Plugin for StallWatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameStallWatch>()
            .add_systems(First, reset_stall_watch.after(reset_frame_perf_counters))
            .add_systems(PreUpdate, stall_preupdate_end)
            .add_systems(PostUpdate, stall_postupdate_begin)
            .add_systems(PostUpdate, stall_postupdate_head.after(stall_postupdate_begin))
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
            )
            .add_systems(Update, stall_checkpoint_update_begin)
            .add_systems(
                Update,
                stall_checkpoint_before_map_camera
                    .before(crate::gui::MapCameraSystemSet::ApplyInput),
            )
            .add_systems(
                Update,
                stall_checkpoint_after_map_camera_smooth
                    .after(crate::gui::MapCameraSystemSet::Smooth)
                    .before(crate::gui::WorldRepresentationSystemSet::ComputeFrame),
            )
            .add_systems(
                Update,
                stall_checkpoint_after_view_sync
                    .after(crate::gui::ViewAuthoritySystemSet::SyncViewManager)
                    .after(stall_checkpoint_after_map_camera_smooth)
                    .before(crate::gui::WorldRepresentationSystemSet::ComputeFrame),
            )
            .add_systems(
                Update,
                stall_checkpoint_after_fire_build
                    .after(crate::render::extraction::FireVisualFrameSet::BuildProfiles)
                    .after(stall_checkpoint_after_view_sync)
                    .before(crate::gui::WorldRepresentationSystemSet::ComputeFrame),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_span_maps_pre_repr_slices() {
        let mut watch = FrameStallWatch::default();
        watch.record_span("after_map_camera_smooth", 2.0);
        watch.record_span("after_view_sync", 3.0);
        watch.record_span("after_fire_build", 4.0);
        watch.record_span("before_world_repr", 800.0);
        assert_eq!(watch.spans.map_camera_chain_ms, 2.0);
        assert_eq!(watch.spans.after_view_sync_ms, 3.0);
        assert_eq!(watch.spans.after_fire_build_ms, 4.0);
        assert_eq!(watch.spans.before_world_repr_ms, 800.0);
    }
}
