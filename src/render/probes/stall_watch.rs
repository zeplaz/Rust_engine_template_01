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
    /// First schedule → previous frame `last` checkpoint (render / present / GPU wait).
    pub post_render_gap_ms: f32,
    /// Sum of mid-Update wall gaps ≥100ms that no perf scope accounts for (uninstrumented
    /// CPU, blocking call, or render/present wait — not necessarily GPU).
    pub unattributed_gap_ms: f32,
    /// Update: end of streaming spine reconstruct chain (late Update, after world repr).
    pub post_streaming_spine_ms: f32,
    /// Update: PreUpdate end → first pre-repr slice (usually dominates when “pre_repr” is huge).
    pub before_world_repr_ms: f32,
    /// Update: `update_begin` → [`crate::gui::MapCameraSystemSet::ApplyInput`] (un-ordered Update work).
    pub update_pre_map_camera_ms: f32,
    /// Update: map camera ApplyInput → Smooth (same segment as [`Self::map_camera_chain_ms`]).
    pub map_camera_chain_ms: f32,
    /// Update: map camera ApplyInput → Smooth (explicit alias for triage witnesses).
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

/// PERF-INSTR-VFX-001 — wall-time between consecutive systems inside hot Update chains.
#[derive(Resource, Clone, Debug, Default)]
pub struct FrameSubstageSpans {
    pub map_apply_input_ms: f32,
    pub map_derive_ms: f32,
    pub map_smooth_ms: f32,
    pub minimap_intent_ms: f32,
    pub view_sync_ms: f32,
    /// Wall time from view_sync → fire extract chain start (scheduling / GPU bubble).
    pub fire_pre_extract_ms: f32,
    pub fire_sim_snapshot_ms: f32,
    pub fire_sync_overlay_ms: f32,
    pub fire_sync_visible_ms: f32,
    pub fire_sync_lod_ms: f32,
    pub fire_sync_active_ms: f32,
    pub fire_build_view_ms: f32,
    pub fire_emitter_sync_ms: f32,
    pub fire_commit_ms: f32,
    pub repr_decay_lod_ms: f32,
    pub repr_refresh_lod_ms: f32,
    pub repr_compute_frame_ms: f32,
    pub repr_apply_result_ms: f32,
    pub repr_proc_extract_ms: f32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FrameStallWatch {
    last: Option<Instant>,
    /// Previous frame end stamp — consumed at First to attribute GPU/present wait.
    frame_boundary_end: Option<Instant>,
    postupdate_started: Option<Instant>,
    pub segments: Vec<(String, f32)>,
    pub spans: FrameScheduleSpans,
    pub substages: FrameSubstageSpans,
}

pub fn stall_watch_enabled() -> bool {
    if std::env::var("STALL")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        return true;
    }
    if stall_span_debug_enabled() || frame_perf_verbose() {
        return true;
    }
    crate::dev::test_run_instrumentation::instrumentation_stall_spans()
        && crate::dev::test_run_instrumentation::stall_terminal_logging_enabled()
}

/// RGR-M1-004: when false, stall checkpoint systems are not registered (D3).
#[must_use]
pub fn stall_watch_plugin_latch() -> bool {
    stall_watch_enabled()
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
        self.substages = FrameSubstageSpans::default();
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
            let segment_label = if is_unattributed_gap(label, ms) {
                format!("unattributed_gap@{label}")
            } else {
                label.to_string()
            };
            self.segments.push((segment_label.clone(), ms));
            if crate::dev::test_run_instrumentation::stall_terminal_logging_enabled() {
                if is_unattributed_gap(label, ms) {
                    info!(
                        target: "stall",
                        "STALL unattributed_gap (before {label}): {ms:.2}ms — wall gap outside perf scopes (uninstrumented CPU, blocking call, or render/present wait)"
                    );
                } else {
                    info!(target: "stall", "STALL {label}: {ms:.2}ms");
                }
            }
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
                self.spans.after_map_camera_smooth_ms = ms;
            }
            "substage_map_apply_input" => self.substages.map_apply_input_ms = ms,
            "substage_map_derive" => self.substages.map_derive_ms = ms,
            "substage_map_smooth" => self.substages.map_smooth_ms = ms,
            "substage_minimap_intent" => self.substages.minimap_intent_ms = ms,
            "substage_view_sync" => self.substages.view_sync_ms = ms,
            "substage_fire_pre_extract" => self.substages.fire_pre_extract_ms = ms,
            "substage_fire_sim_snapshot" => self.substages.fire_sim_snapshot_ms = ms,
            "substage_fire_sync_overlay" => self.substages.fire_sync_overlay_ms = ms,
            "substage_fire_sync_visible" => self.substages.fire_sync_visible_ms = ms,
            "substage_fire_sync_lod" => self.substages.fire_sync_lod_ms = ms,
            "substage_fire_sync_active" => self.substages.fire_sync_active_ms = ms,
            "substage_fire_build_view" => self.substages.fire_build_view_ms = ms,
            "substage_fire_emitter_sync" => self.substages.fire_emitter_sync_ms = ms,
            "substage_fire_commit" => self.substages.fire_commit_ms = ms,
            "substage_repr_decay_lod" => self.substages.repr_decay_lod_ms = ms,
            "substage_repr_refresh_lod" => self.substages.repr_refresh_lod_ms = ms,
            "substage_repr_compute_frame" => self.substages.repr_compute_frame_ms = ms,
            "substage_repr_apply_result" => self.substages.repr_apply_result_ms = ms,
            "substage_repr_proc_extract" => self.substages.repr_proc_extract_ms = ms,
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
            "post_render_gap" => self.spans.post_render_gap_ms = ms,
            _ => {}
        }
        if is_unattributed_gap(label, ms) {
            self.spans.unattributed_gap_ms += ms;
        }
    }
}

/// Large wall gaps at schedule checkpoints that no perf scope accounts for. These can be
/// GPU/present waits, but have also been uninstrumented main-thread CPU (e.g. blocking
/// subprocess spawns in witness writers) — do not assume GPU without render-thread evidence.
#[inline]
fn is_unattributed_gap(label: &str, ms: f32) -> bool {
    ms >= 100.0
        && matches!(
            label,
            "substage_map_apply_input"
                | "substage_fire_pre_extract"
                | "post_world_repr"
                | "after_readiness"
                | "post_egui"
                | "post_render_gap"
        )
}

pub fn reset_stall_watch(mut watch: ResMut<FrameStallWatch>) {
    let now = Instant::now();
    if let Some(prev) = watch.frame_boundary_end.take() {
        let gap_ms = now.duration_since(prev).as_secs_f32() * 1000.0;
        watch.record_span("post_render_gap", gap_ms);
        if stall_watch_enabled() && gap_ms >= stall_log_threshold_ms() {
            watch
                .segments
                .push(("post_render_gap".to_string(), gap_ms));
            if crate::dev::test_run_instrumentation::stall_terminal_logging_enabled() {
                if is_unattributed_gap("post_render_gap", gap_ms) {
                    info!(
                        target: "stall",
                        "STALL unattributed_gap (before post_render_gap): {gap_ms:.2}ms — frame-boundary gap (render/present wait or uninstrumented CPU)"
                    );
                } else {
                    info!(target: "stall", "STALL post_render_gap: {gap_ms:.2}ms");
                }
            }
        }
    }
    watch.reset(now);
}

pub fn record_stall_frame_boundary(mut watch: ResMut<FrameStallWatch>) {
    watch.frame_boundary_end = watch.last;
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

macro_rules! stall_substage_fn {
    ($name:ident, $label:literal) => {
        pub fn $name(mut watch: ResMut<FrameStallWatch>) {
            watch.checkpoint($label);
        }
    };
}

stall_substage_fn!(stall_substage_map_apply_input, "substage_map_apply_input");
stall_substage_fn!(stall_substage_map_derive, "substage_map_derive");
stall_substage_fn!(stall_substage_map_smooth, "substage_map_smooth");
stall_substage_fn!(stall_substage_minimap_intent, "substage_minimap_intent");
stall_substage_fn!(stall_substage_view_sync, "substage_view_sync");
// `substage_fire_pre_extract` — view_sync → extract chain entry (often GPU wait bubble).
// `substage_fire_sim_snapshot` — isolated `extract_fire_simulation_snapshot` wall time.
stall_substage_fn!(stall_substage_fire_pre_extract, "substage_fire_pre_extract");
stall_substage_fn!(stall_substage_fire_sim_snapshot, "substage_fire_sim_snapshot");
stall_substage_fn!(stall_substage_fire_sync_overlay, "substage_fire_sync_overlay");
stall_substage_fn!(stall_substage_fire_sync_visible, "substage_fire_sync_visible");
stall_substage_fn!(stall_substage_fire_sync_lod, "substage_fire_sync_lod");
stall_substage_fn!(stall_substage_fire_sync_active, "substage_fire_sync_active");
stall_substage_fn!(stall_substage_fire_build_view, "substage_fire_build_view");
stall_substage_fn!(stall_substage_fire_emitter_sync, "substage_fire_emitter_sync");
stall_substage_fn!(stall_substage_fire_commit, "substage_fire_commit");
stall_substage_fn!(stall_substage_repr_decay_lod, "substage_repr_decay_lod");
stall_substage_fn!(stall_substage_repr_refresh_lod, "substage_repr_refresh_lod");
stall_substage_fn!(stall_substage_repr_compute_frame, "substage_repr_compute_frame");
stall_substage_fn!(stall_substage_repr_apply_result, "substage_repr_apply_result");
stall_substage_fn!(stall_substage_repr_proc_extract, "substage_repr_proc_extract");

pub struct StallWatchPlugin;

impl Plugin for StallWatchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameStallWatch>();
        if !stall_watch_plugin_latch() {
            return;
        }
        app
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
                (
                    stall_last.before(stamp_frame_wall_last),
                    record_stall_frame_boundary.after(stall_last),
                ),
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
                    .before(crate::gui::apply_minimap_camera_intent)
                    .before(crate::gui::ViewAuthoritySystemSet::SyncViewManager),
            )
            .add_systems(
                Update,
                stall_checkpoint_after_view_sync
                    .after(crate::gui::ViewAuthoritySystemSet::SyncViewManager)
                    .after(stall_checkpoint_after_map_camera_smooth)
                    .before(crate::render::extraction::FireVisualFrameSet::BuildProfiles),
            )
            .add_systems(
                Update,
                stall_checkpoint_after_fire_build
                    .after(crate::render::visual_snapshot_commit::commit_fire_visual_snapshot)
                    .after(stall_checkpoint_after_view_sync)
                    .before(stall_checkpoint_before_world_repr),
            )
            // PERF-INSTR-VFX-001 — map camera substages
            .add_systems(
                Update,
                stall_substage_map_apply_input
                    .after(crate::gui::MapCameraSystemSet::ApplyInput)
                    .before(crate::gui::MapCameraSystemSet::DeriveDesired),
            )
            .add_systems(
                Update,
                stall_substage_map_derive
                    .after(crate::gui::MapCameraSystemSet::DeriveDesired)
                    .before(crate::gui::MapCameraSystemSet::Smooth),
            )
            .add_systems(
                Update,
                stall_substage_map_smooth
                    .after(crate::gui::MapCameraSystemSet::Smooth)
                    .before(crate::gui::apply_minimap_camera_intent),
            )
            .add_systems(
                Update,
                stall_substage_minimap_intent
                    .after(crate::gui::apply_minimap_camera_intent)
                    .before(crate::gui::ViewAuthoritySystemSet::SyncViewManager),
            )
            .add_systems(
                Update,
                stall_substage_view_sync
                    .after(crate::gui::ViewAuthoritySystemSet::SyncViewManager)
                    .before(crate::render::extraction::FireVisualFrameSet::BuildProfiles),
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
        assert_eq!(watch.spans.after_map_camera_smooth_ms, 2.0);
        assert_eq!(watch.spans.after_view_sync_ms, 3.0);
        assert_eq!(watch.spans.after_fire_build_ms, 4.0);
        assert_eq!(watch.spans.before_world_repr_ms, 800.0);
    }

    #[test]
    fn stall_watch_disabled_for_quiet_demo_profile() {
        use crate::dev::test_run_instrumentation::{
            publish_test_run_instrumentation_latch, stall_terminal_logging_enabled,
            TestRunInstrumentation,
        };
        use crate::engine::{DebugManeuver, EngineLaunchArgs, TestScene};

        let profile = EngineLaunchArgs::from_cli(Some("demo".into()), false, None)
            .test_instrumentation_profile();
        publish_test_run_instrumentation_latch(&TestRunInstrumentation {
            active: profile.active,
            quiet_terminal: profile.quiet_terminal,
            frame_jsonl: profile.frame_jsonl,
            stall_spans: profile.stall_spans,
            flush_secs: profile.flush_secs,
            frame_jsonl_stride: profile.frame_jsonl_stride,
            from_test_cli: true,
            test_scene: TestScene::Visual,
            maneuver: DebugManeuver::DemoOpen,
        });
        assert!(!stall_terminal_logging_enabled());
        assert!(!stall_watch_enabled());
    }
}
