//! Per-frame CPU timings for Stage 5 / representation spine diagnosis (`PERF=1` or slow frames).
//!
//! **Wall vs instrumented:** `Time::delta_secs()` is the full frame (includes GPU present/wait).
//! Spine slots (`graph`, `merge`, `atm`, …) are scoped CPU in Update/PostUpdate. Phase stamps
//! (`cpu_pre_egui`, `cpu_egui`, `cpu_post_egui`) bracket the main schedule; `gpu_gap ≈ wall − cpu_*` isolates
//! render/extract/present. HUD buckets come from [`crate::gui::hud::FrameBudgetDiagnostics`].

use std::time::Instant;

use bevy::prelude::*;
use bevy_egui::EguiPostUpdateSet;

const PERF_SLOW_MS: f32 = 16.0;

/// Wall-clock phase stamps for the current frame (see module docs).
#[derive(Resource, Clone, Debug, Default)]
pub struct FrameWallClock {
    frame_start: Option<Instant>,
    pre_egui: Option<Instant>,
    post_egui: Option<Instant>,
    last: Option<Instant>,
    /// First → before egui pass (Update + early PostUpdate).
    pub cpu_pre_egui_ms: f32,
    /// Egui context pass + texture upload in PostUpdate.
    pub cpu_egui_ms: f32,
    /// After egui → end of Last (late PostUpdate hooks not in egui set).
    pub cpu_post_egui_ms: f32,
    /// `wall − (pre_egui + egui + post_egui)` — render extract, GPU, present, idle.
    pub gpu_gap_ms: f32,
}

impl FrameWallClock {
    fn reset_stamps(&mut self) {
        self.frame_start = Some(Instant::now());
        self.pre_egui = None;
        self.post_egui = None;
        self.last = None;
        self.cpu_pre_egui_ms = 0.0;
        self.cpu_egui_ms = 0.0;
        self.cpu_post_egui_ms = 0.0;
        self.gpu_gap_ms = 0.0;
    }

    fn ms_between(from: Option<Instant>, to: Option<Instant>) -> f32 {
        match (from, to) {
            (Some(a), Some(b)) => b.duration_since(a).as_secs_f32() * 1000.0,
            _ => 0.0,
        }
    }

    fn finalize_phases(&mut self, wall_ms: f32) {
        self.cpu_pre_egui_ms = Self::ms_between(self.frame_start, self.pre_egui);
        self.cpu_egui_ms = Self::ms_between(self.pre_egui, self.post_egui);
        self.cpu_post_egui_ms = Self::ms_between(self.post_egui, self.last);
        let cpu = self.cpu_pre_egui_ms + self.cpu_egui_ms + self.cpu_post_egui_ms;
        self.gpu_gap_ms = (wall_ms - cpu).max(0.0);
    }
}

/// Sub-update CPU timings (preview / fire / streaming / map fit). Reset each frame in [`reset_frame_perf_counters`].
///
/// These explain otherwise-unattributed `cpu_pre_egui` without disturbing schedule-level [`FrameStallWatch`] checkpoints.
#[derive(Resource, Clone, Debug, Default)]
pub struct FrameUpdateAttrib {
    pub preview_cpu_raster_ms: f32,
    pub preview_gpu_present_ms: f32,
    pub fire_pipeline_ms: f32,
    pub streaming_reconstruct_ms: f32,
    pub map_fit_validate_ms: f32,
    pub tile_storage_apply_ms: f32,
    pub viewport_sync_ms: f32,
    pub map_fit_sync_ms: f32,
    pub hud_egui_ms: f32,
    pub world_gen_ui_ms: f32,
}

/// RAII scope — logs `perf_scope` when elapsed ≥ [`PERF_SLOW_MS`] (see operational perf playbook).
pub struct PerfScope {
    label: &'static str,
    start: Instant,
}

impl PerfScope {
    #[must_use]
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            start: Instant::now(),
        }
    }

    #[must_use]
    pub fn elapsed_ms(&self) -> f32 {
        self.start.elapsed().as_secs_f32() * 1000.0
    }
}

impl Drop for PerfScope {
    fn drop(&mut self) {
        let ms = self.elapsed_ms();
        if frame_perf_verbose() || ms >= PERF_SLOW_MS {
            info!(target: "perf_scope", "{} {ms:.2}ms", self.label);
        }
        intra_update_stall_log(self.label, ms);
    }
}

#[derive(Resource, Debug, Default)]
pub struct FrameAttribScratch {
    preview_cpu: Option<Instant>,
    preview_gpu: Option<Instant>,
    fire: Option<Instant>,
    streaming: Option<Instant>,
}

#[inline]
pub fn intra_update_stall_log(label: &'static str, ms: f32) {
    let stall_on = std::env::var("STALL")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    if !(frame_perf_verbose() || stall_on) || ms < PERF_SLOW_MS {
        return;
    }
    info!(target: "stall", "STALL {label}: {ms:.2}ms");
}

pub fn attrib_preview_cpu_raster_before(scratch: Option<ResMut<FrameAttribScratch>>) {
    let Some(mut s) = scratch else {
        return;
    };
    s.preview_cpu = Some(Instant::now());
}

pub fn attrib_preview_cpu_raster_after(
    scratch: Option<ResMut<FrameAttribScratch>>,
    attrib: Option<ResMut<FrameUpdateAttrib>>,
) {
    let (Some(mut s), Some(mut a)) = (scratch, attrib) else {
        return;
    };
    let Some(t0) = s.preview_cpu.take() else {
        return;
    };
    let ms = t0.elapsed().as_secs_f32() * 1000.0;
    a.preview_cpu_raster_ms = ms;
    intra_update_stall_log("upd_preview_cpu_raster", ms);
}

pub fn attrib_preview_gpu_present_before(scratch: Option<ResMut<FrameAttribScratch>>) {
    let Some(mut s) = scratch else {
        return;
    };
    s.preview_gpu = Some(Instant::now());
}

pub fn attrib_preview_gpu_present_after(
    scratch: Option<ResMut<FrameAttribScratch>>,
    attrib: Option<ResMut<FrameUpdateAttrib>>,
) {
    let (Some(mut s), Some(mut a)) = (scratch, attrib) else {
        return;
    };
    let Some(t0) = s.preview_gpu.take() else {
        return;
    };
    let ms = t0.elapsed().as_secs_f32() * 1000.0;
    a.preview_gpu_present_ms = ms;
    intra_update_stall_log("upd_preview_gpu_present", ms);
}

pub fn attrib_fire_pipeline_before(scratch: Option<ResMut<FrameAttribScratch>>) {
    let Some(mut s) = scratch else {
        return;
    };
    s.fire = Some(Instant::now());
}

pub fn attrib_fire_pipeline_after(
    scratch: Option<ResMut<FrameAttribScratch>>,
    attrib: Option<ResMut<FrameUpdateAttrib>>,
) {
    let (Some(mut s), Some(mut a)) = (scratch, attrib) else {
        return;
    };
    let Some(t0) = s.fire.take() else {
        return;
    };
    let ms = t0.elapsed().as_secs_f32() * 1000.0;
    a.fire_pipeline_ms = ms;
    intra_update_stall_log("upd_fire_pipeline", ms);
}

pub fn attrib_streaming_reconstruct_before(scratch: Option<ResMut<FrameAttribScratch>>) {
    let Some(mut s) = scratch else {
        return;
    };
    s.streaming = Some(Instant::now());
}

pub fn attrib_streaming_reconstruct_after(
    scratch: Option<ResMut<FrameAttribScratch>>,
    attrib: Option<ResMut<FrameUpdateAttrib>>,
) {
    let (Some(mut s), Some(mut a)) = (scratch, attrib) else {
        return;
    };
    let Some(t0) = s.streaming.take() else {
        return;
    };
    let ms = t0.elapsed().as_secs_f32() * 1000.0;
    a.streaming_reconstruct_ms = ms;
    intra_update_stall_log("upd_streaming_reconstruct", ms);
}

impl FrameUpdateAttrib {
    #[must_use]
    pub fn attrib_sum_ms(&self) -> f32 {
        self.preview_cpu_raster_ms
            + self.preview_gpu_present_ms
            + self.fire_pipeline_ms
            + self.streaming_reconstruct_ms
            + self.map_fit_validate_ms
            + self.tile_storage_apply_ms
            + self.viewport_sync_ms
            + self.map_fit_sync_ms
            + self.hud_egui_ms
            + self.world_gen_ui_ms
    }
}

fn record_attrib_ms(
    attrib: Option<ResMut<FrameUpdateAttrib>>,
    ms: f32,
    write: impl FnOnce(&mut FrameUpdateAttrib, f32),
    stall_label: &'static str,
) {
    if let Some(mut a) = attrib {
        write(&mut a, ms.max(0.0));
        intra_update_stall_log(stall_label, ms);
    }
}

/// Record elapsed ms into [`FrameUpdateAttrib::tile_storage_apply_ms`] when resources exist.
pub fn record_tile_storage_apply_ms(
    attrib: Option<ResMut<FrameUpdateAttrib>>,
    ms: f32,
) {
    record_attrib_ms(
        attrib,
        ms,
        |a, v| a.tile_storage_apply_ms = v,
        "streaming_tile_storage_apply",
    );
}

pub fn record_viewport_sync_ms(attrib: Option<ResMut<FrameUpdateAttrib>>, ms: f32) {
    record_attrib_ms(attrib, ms, |a, v| a.viewport_sync_ms = v, "viewport_sync");
}

pub fn record_map_fit_sync_ms(attrib: Option<ResMut<FrameUpdateAttrib>>, ms: f32) {
    record_attrib_ms(attrib, ms, |a, v| a.map_fit_sync_ms = v, "map_fit_sync");
}

/// Scoped timings accumulated each frame; emitted by [`emit_frame_perf_summary`].
#[derive(Resource, Clone, Debug, Default)]
pub struct FramePerf {
    pub frame_index: u64,
    pub world_repr_ms: f32,
    pub projection_graph_ms: f32,
    pub domain_merge_ms: f32,
    pub atmosphere_gpu_extract_ms: f32,
    pub readiness_ms: f32,
    pub tile_raster_ms: f32,
    /// Last tile raster pass only (0 when raster did not run this frame).
    pub tile_raster_ran: bool,
}

impl FramePerf {
    pub fn reset_frame_counters(&mut self) {
        self.world_repr_ms = 0.0;
        self.projection_graph_ms = 0.0;
        self.domain_merge_ms = 0.0;
        self.atmosphere_gpu_extract_ms = 0.0;
        self.readiness_ms = 0.0;
        self.tile_raster_ms = 0.0;
        self.tile_raster_ran = false;
    }

    #[must_use]
    pub fn spine_instr_ms(&self) -> f32 {
        self.projection_graph_ms + self.domain_merge_ms + self.atmosphere_gpu_extract_ms
    }

    #[must_use]
    pub fn instrumented_ms(&self) -> f32 {
        self.world_repr_ms
            + self.spine_instr_ms()
            + self.readiness_ms
            + if self.tile_raster_ran {
                self.tile_raster_ms
            } else {
                0.0
            }
    }
}

#[must_use]
pub fn frame_perf_verbose() -> bool {
    std::env::var("PERF")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || std::env::var("STAGE5_VERBOSE")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Per-frame readiness/projection trace (`READINESS_*` at info). Off by default — success path is throttled.
#[must_use]
pub fn stage5_readiness_live_verbose() -> bool {
    std::env::var("STAGE5_READINESS_VERBOSE")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

pub fn record_frame_perf_ms(perf: &mut FramePerf, ms: f32, slot: FramePerfSlot) {
    let v = ms.max(0.0);
    match slot {
        FramePerfSlot::WorldRepr => perf.world_repr_ms = v,
        FramePerfSlot::ProjectionGraph => perf.projection_graph_ms = v,
        FramePerfSlot::DomainMerge => perf.domain_merge_ms = v,
        FramePerfSlot::AtmosphereExtract => perf.atmosphere_gpu_extract_ms = v,
        FramePerfSlot::Readiness => perf.readiness_ms = v,
        FramePerfSlot::TileRaster => perf.tile_raster_ms = v,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FramePerfSlot {
    WorldRepr,
    ProjectionGraph,
    DomainMerge,
    AtmosphereExtract,
    Readiness,
    TileRaster,
}

#[must_use]
pub fn scoped_ms<F: FnOnce()>(f: F) -> f32 {
    let t0 = Instant::now();
    f();
    t0.elapsed().as_secs_f32() * 1000.0
}

/// Run `f`, record elapsed ms into `perf` for `slot`, and return `f`'s value.
pub fn timed<F, R>(slot: FramePerfSlot, perf: &mut FramePerf, f: F) -> R
where
    F: FnOnce() -> R,
{
    let t0 = Instant::now();
    let out = f();
    let ms = t0.elapsed().as_secs_f32() * 1000.0;
    record_frame_perf_ms(perf, ms, slot);
    if slot == FramePerfSlot::TileRaster {
        perf.tile_raster_ran = true;
    }
    out
}

/// Like [`timed`] when `FramePerf` is optional (systems without `ResMut<FramePerf>`).
pub fn timed_opt<F, R>(slot: FramePerfSlot, perf: Option<&mut FramePerf>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let t0 = Instant::now();
    let out = f();
    if let Some(perf) = perf {
        let ms = t0.elapsed().as_secs_f32() * 1000.0;
        record_frame_perf_ms(perf, ms, slot);
        if slot == FramePerfSlot::TileRaster {
            perf.tile_raster_ran = true;
        }
    }
    out
}

pub fn log_perf_phase(label: &str, ms: f32) {
    if frame_perf_verbose() || ms >= PERF_SLOW_MS {
        info!(target: "perf", "{label} {ms:.2}ms");
    }
}

pub fn reset_frame_perf_counters(
    mut perf: ResMut<FramePerf>,
    mut wall: ResMut<FrameWallClock>,
    mut attrib: ResMut<FrameUpdateAttrib>,
    mut scratch: ResMut<FrameAttribScratch>,
) {
    perf.reset_frame_counters();
    wall.reset_stamps();
    *attrib = FrameUpdateAttrib::default();
    *scratch = FrameAttribScratch::default();
}

pub fn stamp_frame_wall_pre_egui(mut wall: ResMut<FrameWallClock>) {
    wall.pre_egui = Some(Instant::now());
}

pub fn stamp_frame_wall_post_egui(mut wall: ResMut<FrameWallClock>) {
    wall.post_egui = Some(Instant::now());
}

pub(crate) fn stamp_frame_wall_last(mut wall: ResMut<FrameWallClock>) {
    wall.last = Some(Instant::now());
}

fn bucket_ms(budget: &crate::gui::hud::FrameBudgetDiagnostics, bucket: crate::gui::hud::FrameBudgetBucket) -> f32 {
    budget.buckets[bucket.index()].last_ms
}

pub fn emit_frame_perf_summary(
    mut perf: ResMut<FramePerf>,
    mut wall: ResMut<FrameWallClock>,
    update_attrib: Option<Res<FrameUpdateAttrib>>,
    stall: Option<Res<crate::render::FrameStallWatch>>,
    shell: Option<Res<crate::gui::hud::ProductShellDiagnostics>>,
    budget: Option<Res<crate::gui::hud::FrameBudgetDiagnostics>>,
) {
    perf.frame_index = perf.frame_index.wrapping_add(1);
    let shell_wall_ms = shell
        .as_deref()
        .map(|d| d.last_frame_delta_secs.max(0.0) * 1000.0)
        .unwrap_or(0.0);
    let stamped_wall_ms = FrameWallClock::ms_between(wall.frame_start, wall.last);
    let wall_ms = if stamped_wall_ms > 0.05 {
        stamped_wall_ms
    } else {
        shell_wall_ms
    };
    wall.finalize_phases(wall_ms);

    let attrib_snap = update_attrib
        .as_deref()
        .cloned()
        .unwrap_or_default();
    let instrumented = perf.instrumented_ms();
    let attrib_sum = attrib_snap.attrib_sum_ms();
    let budget_sum = budget.as_deref().map(budget_accounted_ms).unwrap_or(0.0);
    let gap_ms = (wall_ms - instrumented).max(0.0);

    if !frame_perf_verbose()
        && wall_ms < PERF_SLOW_MS
        && instrumented < PERF_SLOW_MS
        && wall.gpu_gap_ms < PERF_SLOW_MS
        && wall.cpu_egui_ms < PERF_SLOW_MS
        && attrib_sum < PERF_SLOW_MS
    {
        return;
    }

    let raster_ms = if perf.tile_raster_ran {
        perf.tile_raster_ms
    } else {
        0.0
    };

    let mut line = format!(
        "PERF wall={:.2} instr={:.2} gap={:.2} | cpu_pre_egui={:.2} cpu_egui={:.2} cpu_post_egui={:.2} gpu_gap={:.2} | spine={:.2} world_repr={:.2} graph={:.2} merge={:.2} atm={:.2} readiness={:.2} raster={:.2}{} | upd_attrib sum={:.2} pv_cpu={:.2} pv_gpu={:.2} fire={:.2} stream={:.2} map_fit={:.2} hud={:.2} wgen={:.2}",
        wall_ms,
        instrumented,
        gap_ms,
        wall.cpu_pre_egui_ms,
        wall.cpu_egui_ms,
        wall.cpu_post_egui_ms,
        wall.gpu_gap_ms,
        perf.spine_instr_ms(),
        perf.world_repr_ms,
        perf.projection_graph_ms,
        perf.domain_merge_ms,
        perf.atmosphere_gpu_extract_ms,
        perf.readiness_ms,
        raster_ms,
        if perf.tile_raster_ran { "" } else { " (idle)" },
        attrib_sum,
        attrib_snap.preview_cpu_raster_ms,
        attrib_snap.preview_gpu_present_ms,
        attrib_snap.fire_pipeline_ms,
        attrib_snap.streaming_reconstruct_ms,
        attrib_snap.map_fit_validate_ms,
        attrib_snap.hud_egui_ms,
        attrib_snap.world_gen_ui_ms,
    );

    if let Some(b) = budget.as_deref() {
        line.push_str(&format!(
            " | budget_sum={:.2} hud={:.2} overlay={:.2} raster_b={:.2} particles={:.2} residency={:.2} tex_reg={:.2} render_x={:.2}",
            budget_sum,
            bucket_ms(b, crate::gui::hud::FrameBudgetBucket::HudShell),
            bucket_ms(b, crate::gui::hud::FrameBudgetBucket::OverlayComposition),
            bucket_ms(b, crate::gui::hud::FrameBudgetBucket::MinimapRaster),
            bucket_ms(b, crate::gui::hud::FrameBudgetBucket::ParticleUpload),
            bucket_ms(b, crate::gui::hud::FrameBudgetBucket::ResidencyUpdates),
            bucket_ms(b, crate::gui::hud::FrameBudgetBucket::GpuTextureRegistration),
            bucket_ms(b, crate::gui::hud::FrameBudgetBucket::RenderExtraction),
        ));
        let unbudgeted_egui = (wall.cpu_egui_ms - bucket_ms(b, crate::gui::hud::FrameBudgetBucket::HudShell)).max(0.0);
        if unbudgeted_egui >= 1.0 {
            line.push_str(&format!(" | egui_unbudgeted={:.2}", unbudgeted_egui));
        }
    }

    if let Some(s) = stall.as_deref() {
        let sp = &s.spans;
        line.push_str(&format!(
            " | stall first+preupd={:.2} update={:.2} post_dom={:.2} post_vt={:.2} post→ready={:.2} ready={:.2} post→egui={:.2} egui={:.2} post_egui={:.2}",
            sp.first_to_preupdate_ms,
            sp.update_ms,
            sp.postupdate_domain_merge_ms,
            sp.postupdate_vt_ci_ms,
            sp.postupdate_to_readiness_ms,
            sp.readiness_ms,
            sp.post_readiness_to_pre_egui_ms,
            sp.egui_ms,
            sp.post_egui_to_last_ms,
        ));
        if !s.segments.is_empty() {
            let detail: String = s
                .segments
                .iter()
                .map(|(l, ms)| format!("{l}:{ms:.1}"))
                .collect::<Vec<_>>()
                .join(",");
            line.push_str(&format!(" | stall_hits=[{detail}]"));
        }
    }

    info!(target: "perf", "{line}");

    const UX_SPIKE_MS: f32 = 250.0;
    if frame_perf_verbose() || wall_ms >= UX_SPIKE_MS {
        let preview_ms = attrib_snap.preview_cpu_raster_ms + attrib_snap.preview_gpu_present_ms;
        let streaming_ms = attrib_snap.streaming_reconstruct_ms;
        let repr_ms = perf.world_repr_ms + perf.spine_instr_ms();
        let raster_ms = if perf.tile_raster_ran {
            perf.tile_raster_ms
        } else {
            0.0
        };
        let update_ms = wall.cpu_pre_egui_ms;
        let egui_ms = wall.cpu_egui_ms;
        info!(
            target: "perf",
            "PERF frame={:.1}ms update={:.1}ms egui={:.1}ms preview={:.1}ms streaming={:.1}ms tile_apply={:.1}ms viewport={:.1}ms map_fit={:.1}ms repr={:.1}ms raster={:.1}ms",
            wall_ms,
            update_ms,
            egui_ms,
            preview_ms,
            streaming_ms,
            attrib_snap.tile_storage_apply_ms,
            attrib_snap.viewport_sync_ms,
            attrib_snap.map_fit_sync_ms,
            repr_ms,
            raster_ms,
        );
        let stall_hit = stall.and_then(|s| {
            s.segments
                .iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(label, ms)| (label.to_string(), *ms))
        });
        let attrib_hit = [
            ("preview_cpu", attrib_snap.preview_cpu_raster_ms),
            ("preview_gpu", attrib_snap.preview_gpu_present_ms),
            ("streaming_apply", attrib_snap.streaming_reconstruct_ms),
            ("fire_pipeline", attrib_snap.fire_pipeline_ms),
            ("map_fit", attrib_snap.map_fit_validate_ms),
            ("tile_apply", attrib_snap.tile_storage_apply_ms),
            ("viewport_sync", attrib_snap.viewport_sync_ms),
            ("map_fit_sync", attrib_snap.map_fit_sync_ms),
            ("world_repr", perf.world_repr_ms),
            ("readiness", perf.readiness_ms),
        ]
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .filter(|(_, ms)| *ms >= UX_SPIKE_MS)
        .map(|(label, ms)| (label.to_string(), ms));
        if let Some((label, ms)) = stall_hit.or(attrib_hit) {
            info!(
                target: "stall",
                "STALL culprit={label} duration={ms:.1}ms frame={wall_ms:.1}ms",
            );
        }
    }
}

fn budget_accounted_ms(budget: &crate::gui::hud::FrameBudgetDiagnostics) -> f32 {
    crate::gui::hud::FrameBudgetBucket::ALL
        .iter()
        .map(|b| bucket_ms(budget, *b))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_attrib_sum_aggregates_buckets() {
        let a = FrameUpdateAttrib {
            preview_cpu_raster_ms: 1.0,
            preview_gpu_present_ms: 2.0,
            fire_pipeline_ms: 3.0,
            streaming_reconstruct_ms: 4.0,
            map_fit_validate_ms: 5.0,
            tile_storage_apply_ms: 1.0,
            viewport_sync_ms: 0.5,
            map_fit_sync_ms: 0.5,
            hud_egui_ms: 6.0,
            world_gen_ui_ms: 7.0,
        };
        assert!((a.attrib_sum_ms() - 30.0).abs() < f32::EPSILON);
    }
}

pub struct FramePerfPlugin;

impl Plugin for FramePerfPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FramePerf>()
            .init_resource::<FrameWallClock>()
            .init_resource::<FrameUpdateAttrib>()
            .init_resource::<FrameAttribScratch>()
            .add_systems(First, reset_frame_perf_counters)
            .add_systems(
                PostUpdate,
                (
                    stamp_frame_wall_pre_egui.before(EguiPostUpdateSet::EndPass),
                    stamp_frame_wall_post_egui.after(EguiPostUpdateSet::PostProcessOutput),
                ),
            )
            .add_systems(
                Last,
                (
                    stamp_frame_wall_last,
                    crate::gui::hud::finalize_frame_budget_diagnostics,
                    emit_frame_perf_summary,
                )
                    .chain(),
            );
    }
}
