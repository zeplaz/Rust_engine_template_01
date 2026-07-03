//! Wall-clock brackets for Bevy [`RenderApp`] schedules (Extract + Render sets).
//!
//! Stall checkpoints on the main world measure gaps *around* the render thread; this module
//! records what the render thread actually spends in extract / prepare / draw / present.
//! Spans are sent to the main world each frame via a bounded channel and consumed in `First`.

use std::sync::{mpsc, OnceLock, Mutex};
use std::time::Instant;

use bevy::prelude::*;
use bevy::render::pipelined_rendering::RenderExtractApp;
use bevy::render::{ExtractSchedule, Render, RenderApp, RenderSystems};

use crate::render::frame_perf::frame_perf_verbose;

static RENDER_SPAN_OUTBOX: OnceLock<mpsc::SyncSender<RenderScheduleSpans>> = OnceLock::new();

/// Latest render-thread spans consumed on the main world (typically previous frame).
#[derive(Resource, Clone, Debug, Default)]
pub struct RenderScheduleWitness {
    pub spans: RenderScheduleSpans,
    /// Whole [`RenderExtractApp`] extract callback (handoff wait + extract invoke + send).
    pub main_thread_handoff_total_ms: f32,
    pub frames_received: u64,
}

/// Wall times between render schedule boundaries (ms).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderScheduleSpans {
    /// [`ExtractSchedule`] on the render world (main→render data copy).
    pub extract_schedule_ms: f32,
    pub extract_commands_ms: f32,
    pub prepare_assets_ms: f32,
    pub prepare_meshes_ms: f32,
    pub manage_views_ms: f32,
    pub queue_ms: f32,
    pub phase_sort_ms: f32,
    pub prepare_ms: f32,
    /// [`RenderSystems::Render`] — render graph + window present.
    pub render_and_present_ms: f32,
    pub cleanup_ms: f32,
    pub post_cleanup_ms: f32,
    /// First ExtractSchedule reset → after PostCleanup.
    pub total_render_app_ms: f32,
}

impl RenderScheduleSpans {
    fn record(&mut self, label: &str, ms: f32) {
        match label {
            "extract_schedule" => self.extract_schedule_ms = ms,
            "extract_commands" => self.extract_commands_ms = ms,
            "prepare_assets" => self.prepare_assets_ms = ms,
            "prepare_meshes" => self.prepare_meshes_ms = ms,
            "manage_views" => self.manage_views_ms = ms,
            "queue" => self.queue_ms = ms,
            "phase_sort" => self.phase_sort_ms = ms,
            "prepare" => self.prepare_ms = ms,
            "render_and_present" => self.render_and_present_ms = ms,
            "cleanup" => self.cleanup_ms = ms,
            "post_cleanup" => self.post_cleanup_ms = ms,
            "total_render_app" => self.total_render_app_ms = ms,
            _ => {}
        }
    }

    #[must_use]
    pub fn sum_phases_ms(&self) -> f32 {
        self.extract_schedule_ms
            + self.extract_commands_ms
            + self.prepare_assets_ms
            + self.prepare_meshes_ms
            + self.manage_views_ms
            + self.queue_ms
            + self.phase_sort_ms
            + self.prepare_ms
            + self.render_and_present_ms
            + self.cleanup_ms
            + self.post_cleanup_ms
    }
}

#[derive(Resource, Default)]
struct RenderScheduleScratch {
    frame_start: Option<Instant>,
    last: Option<Instant>,
    spans: RenderScheduleSpans,
}

impl RenderScheduleScratch {
    fn reset(&mut self) {
        let now = Instant::now();
        self.frame_start = Some(now);
        self.last = Some(now);
        self.spans = RenderScheduleSpans::default();
    }

    fn checkpoint(&mut self, label: &str) {
        let now = Instant::now();
        let Some(prev) = self.last.replace(now) else {
            return;
        };
        let ms = now.duration_since(prev).as_secs_f32() * 1000.0;
        self.spans.record(label, ms);
    }

    fn finalize_and_send(&mut self) {
        if !render_schedule_perf_enabled() {
            return;
        }
        if let (Some(start), Some(end)) = (self.frame_start, self.last) {
            let total = end.duration_since(start).as_secs_f32() * 1000.0;
            self.spans.total_render_app_ms = total;
        }
        if let Some(sender) = RENDER_SPAN_OUTBOX.get() {
            let _ = sender.try_send(self.spans.clone());
        }
    }
}

#[must_use]
pub fn render_schedule_perf_enabled() -> bool {
    frame_perf_verbose()
        || crate::render::stall_watch_enabled()
        || std::env::var("RENDER_PERF")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || crate::dev::test_run_instrumentation::instrumentation_active()
}

fn render_schedule_perf_reset(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.reset();
}

fn render_schedule_perf_begin_render(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("extract_schedule");
}

fn render_schedule_perf_after_extract_commands(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("extract_commands");
}

fn render_schedule_perf_after_prepare_assets(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("prepare_assets");
}

fn render_schedule_perf_after_prepare_meshes(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("prepare_meshes");
}

fn render_schedule_perf_after_manage_views(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("manage_views");
}

fn render_schedule_perf_after_queue(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("queue");
}

fn render_schedule_perf_after_phase_sort(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("phase_sort");
}

fn render_schedule_perf_after_prepare(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("prepare");
}

fn render_schedule_perf_after_render(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("render_and_present");
}

fn render_schedule_perf_after_cleanup(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("cleanup");
}

fn render_schedule_perf_finalize(mut scratch: ResMut<RenderScheduleScratch>) {
    scratch.checkpoint("post_cleanup");
    scratch.finalize_and_send();
}

static RENDER_SPAN_INBOX: OnceLock<Mutex<mpsc::Receiver<RenderScheduleSpans>>> = OnceLock::new();

pub fn drain_render_schedule_witness_system(mut witness: ResMut<RenderScheduleWitness>) {
    if !render_schedule_perf_enabled() {
        return;
    }
    let Some(inbox) = RENDER_SPAN_INBOX.get() else {
        return;
    };
    let Ok(guard) = inbox.lock() else {
        return;
    };
    while let Ok(spans) = guard.try_recv() {
        witness.spans = spans;
        witness.frames_received = witness.frames_received.saturating_add(1);
    }
}

fn wrap_render_extract_app(extract_sub: &mut bevy::app::SubApp) {
    let mut inner = extract_sub.take_extract();
    extract_sub.set_extract(move |main_world, sub_world| {
        let t0 = Instant::now();
        if let Some(f) = inner.as_mut() {
            f(main_world, sub_world);
        }
        let ms = t0.elapsed().as_secs_f32() * 1000.0;
        if let Some(mut witness) = main_world.get_resource_mut::<RenderScheduleWitness>() {
            witness.main_thread_handoff_total_ms = ms;
        }
    });
}

fn install_render_app_systems(render_app: &mut bevy::app::SubApp) {
    render_app.init_resource::<RenderScheduleScratch>();
    render_app.add_systems(ExtractSchedule, render_schedule_perf_reset);
    render_app.add_systems(
        Render,
        (
            render_schedule_perf_begin_render.in_set(RenderSystems::ExtractCommands),
            render_schedule_perf_after_extract_commands.after(RenderSystems::ExtractCommands),
            render_schedule_perf_after_prepare_assets.after(RenderSystems::PrepareAssets),
            render_schedule_perf_after_prepare_meshes.after(RenderSystems::PrepareMeshes),
            render_schedule_perf_after_manage_views.after(RenderSystems::ManageViews),
            render_schedule_perf_after_queue.after(RenderSystems::Queue),
            render_schedule_perf_after_phase_sort.after(RenderSystems::PhaseSort),
            render_schedule_perf_after_prepare.after(RenderSystems::Prepare),
            render_schedule_perf_after_render.after(RenderSystems::Render),
            render_schedule_perf_after_cleanup.after(RenderSystems::Cleanup),
            render_schedule_perf_finalize.after(RenderSystems::PostCleanup),
        ),
    );
}

pub struct RenderSchedulePerfPlugin;

impl Plugin for RenderSchedulePerfPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = mpsc::sync_channel(8);
        let _ = RENDER_SPAN_OUTBOX.set(tx);
        let _ = RENDER_SPAN_INBOX.set(Mutex::new(rx));

        app.init_resource::<RenderScheduleWitness>()
            .add_systems(
                First,
                drain_render_schedule_witness_system.after(crate::render::reset_frame_perf_counters),
            );

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            install_render_app_systems(render_app);
        }

        if let Some(extract_app) = app.get_sub_app_mut(RenderExtractApp) {
            wrap_render_extract_app(extract_app);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_schedule_spans_sum_matches_fields() {
        let s = RenderScheduleSpans {
            extract_schedule_ms: 1.0,
            extract_commands_ms: 2.0,
            prepare_assets_ms: 3.0,
            prepare_meshes_ms: 4.0,
            manage_views_ms: 5.0,
            queue_ms: 6.0,
            phase_sort_ms: 7.0,
            prepare_ms: 8.0,
            render_and_present_ms: 9.0,
            cleanup_ms: 1.0,
            post_cleanup_ms: 1.0,
            total_render_app_ms: 47.0,
        };
        assert!((s.sum_phases_ms() - 47.0).abs() < f32::EPSILON);
    }
}
