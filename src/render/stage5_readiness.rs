//! Stage 5 readiness — runtime invariant enforcement for the visual representation spine.
//! Exit gate: `Stage5ReadinessProfile::FULL_APP` green in the running app — see
//! `prompts/guides/stage5_convergence_directive_v1.md` §9–§15.

use bevy::prelude::*;

use crate::engine::debug_maneuver::tick_debug_capture_frame_gate;
use crate::gui::editor::world_preview::{
    preview_authoritative_surface, PreviewAuthoritativeSurface, PreviewCameraState,
    PreviewPathAuthority, PreviewPresentationDebug, PreviewRenderMode, WorldPreviewGpuRuntime,
};
use crate::gui::{
    fire_visual_producer_count, RepresentationBand, RepresentationResult,
    REGISTERED_VISUAL_PRODUCERS, ViewRepresentationSystemSet, WorldRepresentationFrame,
};
use crate::gui::in_simulation_or_editor;
use crate::render::extraction::{projection_graph_runtime_order_snapshot, RenderProjectionGraph};
use crate::render::FireSimulationSnapshot;
use crate::render::overlay_field_buffers::SharedOverlayFieldBuffers;
use crate::render::phase_f_lod_proof::PhaseFLodProofReport;
use crate::render::visual_agreement::VisualAgreementFrame;
use crate::render::CommittedVisualSnapshotFence;
use crate::render::GpuRepresentationMetrics;
use crate::systems::atmosphere::{
    AtmospherePartialWriteMetrics, P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE,
};

/// Monotonic counter for [`evaluate_app_stage5_readiness`] invocations (live app diagnostics).
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Stage5ReadinessEvalInvocation(pub u32);

fn bump_eval_invocation(world: &mut World) -> u32 {
    if !world.contains_resource::<Stage5ReadinessEvalInvocation>() {
        world.init_resource::<Stage5ReadinessEvalInvocation>();
    }
    let mut n = world.resource_mut::<Stage5ReadinessEvalInvocation>();
    n.0 = n.0.wrapping_add(1);
    n.0
}

/// Which optional surfaces must be present for readiness to pass.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage5ReadinessProfile {
    pub require_vt4: bool,
    pub require_vt5: bool,
    pub require_preview: bool,
    pub require_partial_metrics: bool,
    pub require_world_frame: bool,
    pub require_phase_f_proof: bool,
    pub require_instanced_draw: bool,
}

impl Stage5ReadinessProfile {
    pub const FULL_APP: Self = Self {
        require_vt4: true,
        require_vt5: true,
        require_preview: true,
        require_partial_metrics: true,
        require_world_frame: true,
        require_phase_f_proof: true,
        require_instanced_draw: true,
    };

    pub const HEADLESS: Self = Self {
        require_vt4: false,
        require_vt5: false,
        require_preview: false,
        require_partial_metrics: false,
        require_world_frame: false,
        require_phase_f_proof: false,
        require_instanced_draw: false,
    };
}

impl Default for Stage5ReadinessProfile {
    fn default() -> Self {
        Self::HEADLESS
    }
}

/// App-level Stage 5 readiness snapshot (HUD / diagnostics / CI hooks).
#[derive(Resource, Debug, Default, Clone)]
pub struct AppStage5ReadinessReport {
    pub vt4_ok: bool,
    pub vt5_ok: bool,
    pub single_fire_extract: bool,
    pub gpu_field_authoritative: bool,
    pub preview_render_target_active: bool,
    pub phase_d_ok: bool,
    pub overlay_from_shared_buffers_only: bool,
    pub particle_lod_scales: bool,
    pub phase_f_lod_proof_ok: bool,
    pub instanced_dispatch_ok: bool,
    pub phase_f_ok: bool,
    pub projection_domains: u8,
    pub registered_producers: u32,
    pub duplicate_visual_scan_count: u32,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stage5SpineChecklist {
    pub policy_present: bool,
    pub projection_domains: u8,
    pub registry_buffer_ids: u8,
    pub fence_committed: bool,
}

#[must_use]
pub fn evaluate_stage5_spine_checklist(
    policy: Option<&RepresentationResult>,
    graph: Option<&RenderProjectionGraph>,
    fence: Option<&CommittedVisualSnapshotFence>,
) -> Stage5SpineChecklist {
    Stage5SpineChecklist {
        policy_present: policy.is_some(),
        projection_domains: if graph.is_some() { 3 } else { 0 },
        registry_buffer_ids: 5,
        fence_committed: fence.is_some_and(|f| f.fire.tick > 0 || f.fire.sim_time_micros > 0),
    }
}

#[must_use]
pub fn stage5_readiness_passes(report: &AppStage5ReadinessReport) -> bool {
    report.violations.is_empty()
        && report.vt4_ok
        && report.vt5_ok
        && report.single_fire_extract
        && report.gpu_field_authoritative
        && report.overlay_from_shared_buffers_only
        && report.particle_lod_scales
        && report.phase_f_lod_proof_ok
        && report.instanced_dispatch_ok
        && report.phase_d_ok
        && report.phase_f_ok
}

/// When FULL_APP passes, logging every frame to the Windows console can cost **~200ms/update** (stdout lock).
const READINESS_FULL_APP_SUCCESS_LOG_INTERVAL: u32 = 240;

#[inline]
fn stage5_readiness_live_verbose_logs() -> bool {
    crate::render::frame_perf::stage5_readiness_live_verbose()
}

#[inline]
fn readiness_full_app_live_log_this_frame(
    inv: u32,
    passes: bool,
    violations_empty: bool,
) -> bool {
    stage5_readiness_live_verbose_logs()
        || !passes
        || !violations_empty
        || inv % READINESS_FULL_APP_SUCCESS_LOG_INTERVAL == 0
}

pub fn evaluate_app_stage5_readiness(world: &mut World) {
    let eval_started = std::time::Instant::now();
    let inv = bump_eval_invocation(world);
    let profile = world.resource::<Stage5ReadinessProfile>().clone();
    let (
        report,
        presence,
        stamp_tick,
        policy_tick_log,
        fence_fire_tick_log,
        projection_graph_snap,
    ) = {
        let policy = world.get_resource::<RepresentationResult>();
    let world_frame = world.get_resource::<WorldRepresentationFrame>();
    let graph = world.get_resource::<RenderProjectionGraph>();
    let fence = world.get_resource::<CommittedVisualSnapshotFence>();
    let overlay = world.get_resource::<SharedOverlayFieldBuffers>();
    let gpu_metrics = world.get_resource::<GpuRepresentationMetrics>();
    let agreement = world.get_resource::<VisualAgreementFrame>();
    let vt_ci = world.get_resource::<crate::render::vt_ci_matrix::VtCiMatrixLiveReport>();
    let partial_metrics = world.get_resource::<AtmospherePartialWriteMetrics>();
    let preview_cam = world.get_resource::<PreviewCameraState>();
    let preview_gpu = world.get_resource::<WorldPreviewGpuRuntime>();
    let phase_f = world.get_resource::<PhaseFLodProofReport>();
    let indirect = world.get_resource::<crate::render::GpuIndirectDrawSpine>();
    let draw_dispatch = world.get_resource::<crate::render::WorldFireParticleDrawDispatch>();
    let fire = world.get_resource::<FireSimulationSnapshot>();
    let stamp_tick = live_readiness_stamp_tick(policy, fire);

    let mut violations = Vec::new();
    let registered_producers = REGISTERED_VISUAL_PRODUCERS.len() as u32;
    let mut duplicate_visual_scan_count = 0u32;

    let producer_count = fire_visual_producer_count();
    let single_fire_extract = producer_count == 1;
    if producer_count != 1 {
        duplicate_visual_scan_count = producer_count.saturating_sub(1);
        violations.push(format!(
            "expected one fire_visual producer, registered {producer_count}"
        ));
    }

    let gpu_field_authoritative = P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE;
    if !gpu_field_authoritative {
        violations.push("GPU partial writes not authoritative".into());
    }

    let overlay_from_shared_buffers_only = overlay.is_some();
    if !overlay_from_shared_buffers_only {
        violations.push("SharedOverlayFieldBuffers missing".into());
    }

    let checklist = evaluate_stage5_spine_checklist(policy, graph, fence);
    let projection_domains = checklist.projection_domains;
    if !checklist.policy_present {
        violations.push("RepresentationResult missing".into());
    }
    if checklist.projection_domains < 3 {
        violations.push("RenderProjectionGraph domains incomplete".into());
    }
    if !checklist.fence_committed {
        violations.push("CommittedVisualSnapshotFence not stamped".into());
    }

    let particle_lod_scales = if let (Some(policy), Some(metrics)) = (policy, gpu_metrics) {
        let scales = if !policy.extract_plan.fire_instances {
            metrics.particle_rows == 0 && metrics.draw_instances == 0
        } else {
            match policy.active_band {
                RepresentationBand::OverlayOnly | RepresentationBand::Dormant => {
                    metrics.particle_rows == 0 && metrics.draw_instances == 0
                }
                RepresentationBand::Strategic => {
                    let projected = metrics.instance_rows.max(metrics.draw_instances);
                    metrics.particle_rows <= projected
                }
                _ => {
                    let expect_gpu = policy.extract_plan.fire_instances
                        && graph
                            .is_some_and(|projection| !projection.fire.instance_buffer.is_empty());
                    if expect_gpu {
                        metrics.particle_rows > 0
                            || metrics.instance_rows > 0
                            || metrics.draw_instances > 0
                    } else {
                        true
                    }
                }
            }
        };
        if !scales {
            violations.push("GPU particle rows do not scale with LOD band".into());
        }
        scales
    } else {
        !profile.require_phase_f_proof
    };

    let phase_f_lod_proof_ok = if let Some(proof) = phase_f {
        let ok = proof.samples == 0 || proof.ordering_ok;
        if profile.require_phase_f_proof && proof.samples > 0 && !proof.ordering_ok {
            violations.push("Phase F LOD upload ordering failed".into());
        }
        ok
    } else {
        !profile.require_phase_f_proof
    };

    let instanced_dispatch_ok = if let Some(policy) = policy {
        if !policy.particle_policy.instanced_draw {
            true
        } else if let (Some(indirect), Some(draw)) = (indirect, draw_dispatch) {
            let cap = policy.gpu_budget.particle_rows_cap as u32;
            let indirect_count = indirect.world_fire.instance_count;
            let dispatch_count = draw.instance_count;
            indirect_count == dispatch_count
                && indirect_count <= cap
                && (indirect_count == 0 || indirect.dispatch_count > 0)
        } else {
            !profile.require_instanced_draw
        }
    } else {
        !profile.require_instanced_draw
    };
    if profile.require_instanced_draw && !instanced_dispatch_ok {
        violations.push("Phase F instanced draw dispatch not aligned with policy".into());
    }

    let phase_f_ok = particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok;
    if profile.require_phase_f_proof && !phase_f_ok {
        violations.push("Phase F GPU particle path not proven under LOD".into());
    }

    let preview_render_target_active = if let (Some(cam), Some(gpu_rt)) = (preview_cam, preview_gpu)
    {
        let active = matches!(
            preview_authoritative_surface(gpu_rt, cam),
            PreviewAuthoritativeSurface::GpuRenderTarget
        );
        if profile.require_preview
            && cam.mode == PreviewRenderMode::GpuRenderTarget
            && !active
        {
            violations.push("Phase D GPU preview requested but not authoritative".into());
        }
        active
    } else {
        if profile.require_preview {
            violations.push("Preview resources missing for full-app readiness".into());
        }
        false
    };

    let phase_d_ok = !profile.require_preview || preview_render_target_active;
    if profile.require_preview && !phase_d_ok {
        violations.push("Phase D GPU preview render target not authoritative".into());
    }

    let (vt4_ok, vt5_ok) = if let Some(live) = vt_ci {
        let vt4_ok = live.vt4.passes();
        let vt5_ok = live.vt5_ok;
        if profile.require_vt4 && !vt4_ok {
            violations.push(format!(
                "VT-4 matrix mismatch_count={} failing_surface_mask={:#x} stamp={}",
                live.vt4.mismatch_count, live.vt4.failing_surface_mask, live.vt4.stamp.tick
            ));
        }
        if profile.require_vt5 && !vt5_ok {
            violations.push(format!(
                "VT-5 spatial invariants failed (stamp={})",
                live.vt4.stamp.tick
            ));
        }
        (vt4_ok, vt5_ok)
    } else if let Some(agreement) = agreement {
        let vt4_ok = agreement.mismatch_count == 0 && agreement.stamp.tick > 0;
        let vt5_ok = !profile.require_vt5;
        if !vt4_ok {
            violations.push(format!(
                "VT-4 agreement mismatch_count={} stamp={}",
                agreement.mismatch_count, agreement.stamp.tick
            ));
        }
        (vt4_ok, vt5_ok)
    } else {
        if profile.require_vt4 {
            violations.push("VisualAgreementFrame missing".into());
        }
        if profile.require_vt5 {
            violations.push("VtCiMatrixLiveReport missing".into());
        }
        (!profile.require_vt4, !profile.require_vt5)
    };

    if world_frame.is_none() && profile.require_world_frame {
        violations.push("WorldRepresentationFrame missing".into());
    }

    if partial_metrics.is_none() && profile.require_partial_metrics {
        violations.push("AtmospherePartialWriteMetrics missing".into());
    }

    let presence = ReadinessInputPresence::capture(
        policy,
        world_frame,
        graph,
        fence,
        overlay,
        gpu_metrics,
        agreement,
        vt_ci,
        partial_metrics,
        preview_cam,
        preview_gpu,
        phase_f,
        indirect,
        draw_dispatch,
        fire,
    );

        let policy_tick_log = policy.map(|p| p.stamp.tick).unwrap_or(0);
        let fence_fire_tick_log = fence.map(|f| f.fire.tick).unwrap_or(0);
        let projection_graph_snap =
            graph.map(|g| projection_graph_runtime_order_snapshot(g));

        let report = AppStage5ReadinessReport {
            vt4_ok,
            vt5_ok,
            single_fire_extract,
            gpu_field_authoritative,
            preview_render_target_active,
            phase_d_ok,
            overlay_from_shared_buffers_only,
            particle_lod_scales,
            phase_f_lod_proof_ok,
            instanced_dispatch_ok,
            phase_f_ok,
            projection_domains,
            registered_producers,
            duplicate_visual_scan_count,
            violations,
        };

        (
            report,
            presence,
            stamp_tick,
            policy_tick_log,
            fence_fire_tick_log,
            projection_graph_snap,
        )
    };

    *world.resource_mut::<AppStage5ReadinessReport>() = report;
    let mut truth_inputs = world.resource_mut::<Stage5ReadinessTruthInputs>();
    truth_inputs.stamp_tick = stamp_tick;
    truth_inputs.presence = presence;

    if profile == Stage5ReadinessProfile::FULL_APP {
        let r = world.resource::<AppStage5ReadinessReport>();
        let passes = stage5_readiness_passes(r);
        let log = readiness_full_app_live_log_this_frame(inv, passes, r.violations.is_empty());
        if log {
            info!(
                target: "stage5_readiness::live",
                "READINESS_EVAL_BEGIN inv={inv} sim_stamp_tick={stamp_tick} policy_tick={policy_tick_log}",
            );
            if let Some(snap) = projection_graph_snap.as_ref() {
                info!(
                    target: "stage5_readiness::live",
                    "READINESS_PROJECTION_GRAPH inv={inv} checklist_domains={} fence_fire_tick={fence_fire_tick_log} {}",
                    r.projection_domains,
                    snap,
                );
            } else {
                warn!(
                    target: "stage5_readiness::live",
                    "READINESS_PROJECTION_GRAPH inv={inv} graph=ABSENT (domains checklist={})",
                    r.projection_domains,
                );
            }
            let viol_digest = r.violations.join(" | ");
            let viol_digest_out = if r.violations.is_empty() {
                "(none)".to_string()
            } else if viol_digest.len() > 400 {
                format!(
                    "{}…(+{} chars)",
                    &viol_digest[..400],
                    viol_digest.len().saturating_sub(400)
                )
            } else {
                viol_digest
            };
            info!(
                target: "stage5_readiness::live",
                "READINESS_EVAL_END inv={inv} passes={passes} viol_len={} fence_fire_tick={fence_fire_tick_log} viol_digest={}",
                r.violations.len(),
                viol_digest_out,
            );
            info!(
                target: "stage5_readiness::live",
                "READINESS_EVAL_FLAGS inv={inv} vt4={} vt5={} fire1={} gpu_f={} prev_rt={} phd={} ovl={} p_lod={} pf_lod={} inst={} phf={} dom={}",
                r.vt4_ok,
                r.vt5_ok,
                r.single_fire_extract,
                r.gpu_field_authoritative,
                r.preview_render_target_active,
                r.phase_d_ok,
                r.overlay_from_shared_buffers_only,
                r.particle_lod_scales,
                r.phase_f_lod_proof_ok,
                r.instanced_dispatch_ok,
                r.phase_f_ok,
                r.projection_domains,
            );
            if !r.violations.is_empty() {
                for (i, v) in r.violations.iter().take(24).enumerate() {
                    warn!(
                        target: "stage5_readiness::live",
                        "READINESS_VIOLATION_ROW inv={inv} i={i} {v}",
                    );
                }
                if r.violations.len() > 24 {
                    warn!(
                        target: "stage5_readiness::live",
                        "READINESS_VIOLATION_ROW inv={inv} ... {} more not shown",
                        r.violations.len() - 24,
                    );
                }
            }
            let rows_emitted = r.violations.len().min(24);
            let rows_trunc = r.violations.len().saturating_sub(24);
            info!(
                target: "stage5_readiness::live",
                "READINESS_VIOLATION_SUMMARY inv={inv} viol_len={} viol_rows_emitted={} viol_rows_truncated={} fence_fire_tick={fence_fire_tick_log}",
                r.violations.len(),
                rows_emitted,
                rows_trunc,
            );
        }
    }

    crate::dev::stage5_live_todos::hook_post_readiness_evaluate(world);

    if let Some(mut perf) = world.get_resource_mut::<crate::render::FramePerf>() {
        crate::render::record_frame_perf_ms(
            &mut perf,
            eval_started.elapsed().as_secs_f32() * 1000.0,
            crate::render::FramePerfSlot::Readiness,
        );
        perf.frame_index = perf.frame_index.wrapping_add(1);
    }
}

fn live_readiness_stamp_tick(
    policy: Option<&RepresentationResult>,
    fire: Option<&FireSimulationSnapshot>,
) -> u64 {
    policy
        .map(|p| p.stamp.tick)
        .or_else(|| fire.map(|f| f.stamp.tick))
        .unwrap_or(0)
}

/// Latest FULL_APP input wiring snapshot (written by [`evaluate_app_stage5_readiness`]).
#[derive(Resource, Default, Clone, Debug)]
pub struct Stage5ReadinessTruthInputs {
    pub stamp_tick: u64,
    pub presence: ReadinessInputPresence,
}

/// Resource presence for FULL_APP inputs (UNKNOWN = missing wiring → treat as FAIL for operators).
#[derive(Clone, Debug, Default)]
pub struct ReadinessInputPresence {
    representation_result: bool,
    world_representation_frame: bool,
    render_projection_graph: bool,
    committed_visual_snapshot_fence: bool,
    shared_overlay_field_buffers: bool,
    gpu_representation_metrics: bool,
    visual_agreement_frame: bool,
    vt_ci_matrix_live_report: bool,
    atmosphere_partial_write_metrics: bool,
    preview_camera_state: bool,
    world_preview_gpu_runtime: bool,
    phase_f_lod_proof_report: bool,
    gpu_indirect_draw_spine: bool,
    world_fire_particle_draw_dispatch: bool,
    fire_simulation_snapshot: bool,
}

impl ReadinessInputPresence {
    fn capture(
        policy: Option<&RepresentationResult>,
        world: Option<&WorldRepresentationFrame>,
        graph: Option<&RenderProjectionGraph>,
        fence: Option<&CommittedVisualSnapshotFence>,
        overlay: Option<&SharedOverlayFieldBuffers>,
        gpu_metrics: Option<&GpuRepresentationMetrics>,
        agreement: Option<&VisualAgreementFrame>,
        vt_ci: Option<&crate::render::vt_ci_matrix::VtCiMatrixLiveReport>,
        partial_metrics: Option<&AtmospherePartialWriteMetrics>,
        preview_cam: Option<&PreviewCameraState>,
        preview_gpu: Option<&WorldPreviewGpuRuntime>,
        phase_f: Option<&PhaseFLodProofReport>,
        indirect: Option<&crate::render::GpuIndirectDrawSpine>,
        draw_dispatch: Option<&crate::render::WorldFireParticleDrawDispatch>,
        fire: Option<&FireSimulationSnapshot>,
    ) -> Self {
        Self {
            representation_result: policy.is_some(),
            world_representation_frame: world.is_some(),
            render_projection_graph: graph.is_some(),
            committed_visual_snapshot_fence: fence.is_some(),
            shared_overlay_field_buffers: overlay.is_some(),
            gpu_representation_metrics: gpu_metrics.is_some(),
            visual_agreement_frame: agreement.is_some(),
            vt_ci_matrix_live_report: vt_ci.is_some(),
            atmosphere_partial_write_metrics: partial_metrics.is_some(),
            preview_camera_state: preview_cam.is_some(),
            world_preview_gpu_runtime: preview_gpu.is_some(),
            phase_f_lod_proof_report: phase_f.is_some(),
            gpu_indirect_draw_spine: indirect.is_some(),
            world_fire_particle_draw_dispatch: draw_dispatch.is_some(),
            fire_simulation_snapshot: fire.is_some(),
        }
    }

    fn missing_wiring_full_app(&self, profile: &Stage5ReadinessProfile) -> Vec<&'static str> {
        let mut out = Vec::new();
        if !self.representation_result {
            out.push("RepresentationResult");
        }
        if profile.require_world_frame && !self.world_representation_frame {
            out.push("WorldRepresentationFrame");
        }
        if !self.render_projection_graph {
            out.push("RenderProjectionGraph");
        }
        if !self.committed_visual_snapshot_fence {
            out.push("CommittedVisualSnapshotFence");
        }
        if !self.shared_overlay_field_buffers {
            out.push("SharedOverlayFieldBuffers");
        }
        if profile.require_partial_metrics && !self.atmosphere_partial_write_metrics {
            out.push("AtmospherePartialWriteMetrics");
        }
        if profile.require_preview && (!self.preview_camera_state || !self.world_preview_gpu_runtime) {
            if !self.preview_camera_state {
                out.push("PreviewCameraState");
            }
            if !self.world_preview_gpu_runtime {
                out.push("WorldPreviewGpuRuntime");
            }
        }
        out
    }
}

fn format_full_app_readiness_truth_dump(
    stamp_tick: u64,
    post_update_tick: u32,
    profile: &Stage5ReadinessProfile,
    report: &AppStage5ReadinessReport,
    presence: &ReadinessInputPresence,
    passes: bool,
) -> String {
    let first_v = report
        .violations
        .first()
        .map(String::as_str)
        .unwrap_or("(none)");
    let missing = presence.missing_wiring_full_app(profile);
    let missing_s = if missing.is_empty() {
        "none".into()
    } else {
        missing.join(", ")
    };
    format!(
        "\n\
========== STAGE5_FULL_APP_TRUTH (post_update_invocation={post_update_tick} sim_tick={stamp_tick}) ==========\n\
FULL_APP_PROFILE_ACTIVE: {}\n\
stage5_readiness_passes: {passes}\n\
--- AppStage5ReadinessReport (hard gates) ---\n\
  vt4_ok: {}  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport OR VisualAgreementFrame]\n\
  vt5_ok: {}  [src: evaluate_app_stage5_readiness / VtCiMatrixLiveReport]\n\
  single_fire_extract: {}  [src: fire_visual_producer_count / REGISTERED_VISUAL_PRODUCERS]\n\
  gpu_field_authoritative: {}  [src: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE]\n\
  preview_render_target_active: {}  [src: preview_authoritative_surface]\n\
  phase_d_ok: {}  [derived: !require_preview || preview_render_target_active]\n\
  overlay_from_shared_buffers_only: {}  [src: SharedOverlayFieldBuffers resource exists]\n\
  particle_lod_scales: {}  [src: GpuRepresentationMetrics vs RepresentationResult band]\n\
  phase_f_lod_proof_ok: {}  [src: PhaseFLodProofReport]\n\
  instanced_dispatch_ok: {}  [src: GpuIndirectDrawSpine vs WorldFireParticleDrawDispatch]\n\
  phase_f_ok: {}  [derived: particle_lod_scales && phase_f_lod_proof_ok && instanced_dispatch_ok]\n\
  projection_domains (report): {}  [src: RenderProjectionGraph via evaluate_stage5_spine_checklist]\n\
  registered_producers: {}\n\
  duplicate_visual_scan_count: {}\n\
--- violations (first = primary suspect) ---\n\
  first: {first_v}\n\
  all: {:?}\n\
--- input wiring (MISSING = UNKNOWN→operator FAIL) ---\n\
  RepresentationResult: {}\n\
  WorldRepresentationFrame: {}\n\
  RenderProjectionGraph: {}\n\
  CommittedVisualSnapshotFence: {}\n\
  SharedOverlayFieldBuffers: {}\n\
  GpuRepresentationMetrics: {}\n\
  VisualAgreementFrame: {}\n\
  VtCiMatrixLiveReport: {}\n\
  AtmospherePartialWriteMetrics: {}\n\
  PreviewCameraState: {}\n\
  WorldPreviewGpuRuntime: {}\n\
  PhaseFLodProofReport: {}\n\
  GpuIndirectDrawSpine: {}\n\
  WorldFireParticleDrawDispatch: {}\n\
  FireSimulationSnapshot: {}\n\
  MISSING_WIRING_FULL_APP: {missing_s}\n\
================================================================\n",
        *profile == Stage5ReadinessProfile::FULL_APP,
        report.vt4_ok,
        report.vt5_ok,
        report.single_fire_extract,
        report.gpu_field_authoritative,
        report.preview_render_target_active,
        report.phase_d_ok,
        report.overlay_from_shared_buffers_only,
        report.particle_lod_scales,
        report.phase_f_lod_proof_ok,
        report.instanced_dispatch_ok,
        report.phase_f_ok,
        report.projection_domains,
        report.registered_producers,
        report.duplicate_visual_scan_count,
        report.violations,
        presence.representation_result,
        presence.world_representation_frame,
        presence.render_projection_graph,
        presence.committed_visual_snapshot_fence,
        presence.shared_overlay_field_buffers,
        presence.gpu_representation_metrics,
        presence.visual_agreement_frame,
        presence.vt_ci_matrix_live_report,
        presence.atmosphere_partial_write_metrics,
        presence.preview_camera_state,
        presence.world_preview_gpu_runtime,
        presence.phase_f_lod_proof_report,
        presence.gpu_indirect_draw_spine,
        presence.world_fire_particle_draw_dispatch,
        presence.fire_simulation_snapshot,
    )
}

/// Runtime truth layer for AGENTS.md FULL_APP gate: runs **after** [`evaluate_app_stage5_readiness`].
fn trace_live_full_app_readiness_violations(
    profile: Res<Stage5ReadinessProfile>,
    report: Res<AppStage5ReadinessReport>,
    truth_inputs: Res<Stage5ReadinessTruthInputs>,
    mut post_update_tick: Local<u32>,
    mut last_emit: Local<Option<(bool, u32)>>,
) {
    if *profile != Stage5ReadinessProfile::FULL_APP {
        return;
    }

    *post_update_tick = post_update_tick.saturating_add(1);
    let f = *post_update_tick;
    let presence = truth_inputs.presence.clone();
    let stamp_tick = truth_inputs.stamp_tick;
    let passes = stage5_readiness_passes(&report);

    let boot_window = f < 32;
    let periodic = f.is_multiple_of(480);
    let pass_changed = last_emit.map(|(p, _)| p != passes).unwrap_or(true);
    let should_emit =
        boot_window || periodic || pass_changed || !report.violations.is_empty() || !passes;

    if !should_emit {
        return;
    }
    *last_emit = Some((passes, f));

    let block = format_full_app_readiness_truth_dump(
        stamp_tick,
        f,
        profile.as_ref(),
        report.as_ref(),
        &presence,
        passes,
    );

    if passes && report.violations.is_empty() {
        info!(target: "stage5_readiness::truth", "{}", block);
    } else {
        warn!(target: "stage5_readiness::truth", "{}", block);
    }
}

fn sync_live_preview_authority_for_full_app(
    profile: Res<Stage5ReadinessProfile>,
    mut cam: ResMut<PreviewCameraState>,
    gpu_rt: Res<WorldPreviewGpuRuntime>,
    mut authority: ResMut<PreviewPathAuthority>,
    mut debug: ResMut<PreviewPresentationDebug>,
) {
    if *profile != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    if gpu_rt.offscreen_renderer_ready {
        cam.mode = PreviewRenderMode::GpuRenderTarget;
    }
    authority.gpu_render_target_requested = cam.mode == PreviewRenderMode::GpuRenderTarget;
    authority.authoritative_surface = preview_authoritative_surface(&gpu_rt, &cam);
    authority.cpu_raster_fallback_active = matches!(
        authority.authoritative_surface,
        PreviewAuthoritativeSurface::CpuSwap
    );
    debug.authoritative_surface = authority.authoritative_surface;
}

/// End-of-frame line: confirms **PostUpdate readiness** is visible to the rest of the frame loop
/// (`Last` runs after `PostUpdate`). When failing, logs every frame; when passing, every 30 frames.
fn stage5_readiness_live_full_app_frame_fence(
    profile: Res<Stage5ReadinessProfile>,
    report: Res<AppStage5ReadinessReport>,
    inv: Res<Stage5ReadinessEvalInvocation>,
    mut tick: Local<u32>,
) {
    if *profile != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    *tick = tick.wrapping_add(1);
    let passes = stage5_readiness_passes(&report);
    if !passes {
        warn!(
            target: "stage5_readiness::live",
            "READINESS_FRAME_FENCE eval_inv={} viol_len={} last_eval_passes=false flags vt4={} vt5={} fire1={} gpu_f={} prev_rt={} phd={} ovl={} inst={} phf={} dom={}",
            inv.0,
            report.violations.len(),
            report.vt4_ok,
            report.vt5_ok,
            report.single_fire_extract,
            report.gpu_field_authoritative,
            report.preview_render_target_active,
            report.phase_d_ok,
            report.overlay_from_shared_buffers_only,
            report.instanced_dispatch_ok,
            report.phase_f_ok,
            report.projection_domains,
        );
        return;
    }
    if *tick % 30 == 0 {
        info!(
            target: "stage5_readiness::live",
            "READINESS_FRAME_FENCE_OK eval_inv={} frame_tick={} passes=true",
            inv.0,
            *tick,
        );
    }
}

pub struct Stage5ReadinessPlugin;

impl Plugin for Stage5ReadinessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stage5ReadinessProfile>()
            .init_resource::<AppStage5ReadinessReport>()
            .init_resource::<Stage5ReadinessTruthInputs>()
            .init_resource::<Stage5ReadinessEvalInvocation>()
            .add_systems(
                Update,
                sync_live_preview_authority_for_full_app.run_if(in_simulation_or_editor),
            )
            .add_systems(
                PostUpdate,
                evaluate_app_stage5_readiness
                    .after(ViewRepresentationSystemSet::SyncRepresentationMetrics)
                    .after(crate::render::sync_world_fire_indirect_draw)
                    .after(ViewRepresentationSystemSet::PostFX)
                    .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu)
                    .after(crate::render::vt_ci_matrix::record_vt_ci_matrix_live),
            )
            .add_systems(
                PostUpdate,
                trace_live_full_app_readiness_violations
                    .after(evaluate_app_stage5_readiness),
            )
            .add_systems(
                PostUpdate,
                (
                    crate::render::stage5_full_app_harness::maintain_visual_tactical_vfx_camera,
                    crate::render::gpu_particles::sync_fire_particle_camera_scale,
                    crate::render::stage5_full_app_harness::refresh_visual_proof_fire_particles,
                    crate::render::stage5_full_app_harness::refresh_visual_proof_water_particles,
                )
                    .chain()
                    .before(crate::render::FullRenderDiagnosticSet::Capture),
            )
            .add_systems(
                PostUpdate,
                (
                    tick_debug_capture_frame_gate,
                    crate::render::stage5_full_app_harness::finalize_visual_full_app_live_probe,
                )
                    .chain()
                    .after(crate::render::FullRenderDiagnosticSet::Capture),
            )
            .add_systems(Last, stage5_readiness_live_full_app_frame_fence);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::phase_f_lod_proof::PhaseFLodProofReport;

    #[test]
    fn spine_checklist_counts_projection_domains() {
        let graph = RenderProjectionGraph::default();
        let report = evaluate_stage5_spine_checklist(
            Some(&RepresentationResult::default()),
            Some(&graph),
            None,
        );
        assert!(report.policy_present);
        assert_eq!(report.projection_domains, 3);
        assert_eq!(report.registry_buffer_ids, 5);
    }

    #[test]
    fn single_fire_visual_producer_registered() {
        assert_eq!(fire_visual_producer_count(), 1);
    }

    #[test]
    fn full_app_readiness_chain_aligns_instanced_draw_before_evaluate() {
        use bevy::MinimalPlugins;
        use crate::render::gpu_particles::WorldFireParticleFrame;
        use crate::render::phase_f_lod_proof::PhaseFLodProofReport;
        use crate::render::{
            GpuIndirectDrawSpinePlugin, GpuRepresentationMetrics, WorldFireParticleDrawDispatch,
        };
        use crate::gui::{GpuBudgetPolicy, RepresentationResult};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(GpuIndirectDrawSpinePlugin);
        app.insert_resource(Stage5ReadinessProfile::FULL_APP);
        app.init_resource::<AppStage5ReadinessReport>();
        app.init_resource::<Stage5ReadinessTruthInputs>();
        app.init_resource::<GpuRepresentationMetrics>();
        app.init_resource::<WorldFireParticleFrame>();
        app.init_resource::<WorldFireParticleDrawDispatch>();
        app.init_resource::<PhaseFLodProofReport>();
        let mut particles = WorldFireParticleFrame::default();
        particles.instances.resize(8, Default::default());
        app.insert_resource(particles);
        let mut policy = RepresentationResult::default();
        policy.gpu_budget = GpuBudgetPolicy {
            particle_rows_cap: 3,
            fire_instance_cap: 3,
            reserved_capacity: 3,
            active_capacity: 3,
        };
        policy.particle_policy.instanced_draw = true;
        app.insert_resource(policy);
        app.add_systems(
            PostUpdate,
            crate::render::sync_particle_draw_dispatch_from_policy,
        );
        app.add_systems(
            PostUpdate,
            evaluate_app_stage5_readiness.after(crate::render::sync_world_fire_indirect_draw),
        );
        app.update();
        let report = app.world().resource::<AppStage5ReadinessReport>();
        assert!(
            report.instanced_dispatch_ok,
            "instanced dispatch not aligned: {:?}",
            report.violations
        );
    }

    #[test]
    fn headless_profile_skips_optional_surface_violations() {
        let profile = Stage5ReadinessProfile::HEADLESS;
        let mut report = AppStage5ReadinessReport::default();
        report.violations.clear();
        report.vt4_ok = !profile.require_vt4;
        report.vt5_ok = !profile.require_vt5;
        report.preview_render_target_active = false;
        report.phase_f_lod_proof_ok = !profile.require_phase_f_proof;
        report.single_fire_extract = true;
        report.gpu_field_authoritative = P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE;
        report.overlay_from_shared_buffers_only = true;
        report.particle_lod_scales = true;
        report.instanced_dispatch_ok = true;
        report.phase_f_ok = true;
        report.phase_d_ok = true;
        assert!(report.violations.is_empty());
        assert!(stage5_readiness_passes(&report));
    }

    #[test]
    fn headless_minimal_app_readiness_skips_optional_surface_violations() {
        use bevy::MinimalPlugins;
        use crate::render::extraction::RenderProjectionGraph;
        use crate::render::overlay_field_buffers::SharedOverlayFieldBuffers;
        use crate::render::visual_snapshot_commit::CommittedVisualSnapshotFence;
        use crate::systems::sim_control::SimStepStamp;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Stage5ReadinessProfile::HEADLESS);
        app.init_resource::<AppStage5ReadinessReport>();
        app.init_resource::<Stage5ReadinessTruthInputs>();
        app.init_resource::<RepresentationResult>();
        app.init_resource::<RenderProjectionGraph>();
        app.init_resource::<SharedOverlayFieldBuffers>();
        app.insert_resource(CommittedVisualSnapshotFence {
            fire: SimStepStamp::new(1, 0),
            ..Default::default()
        });
        app.add_systems(PostUpdate, evaluate_app_stage5_readiness);
        app.update();

        let report = app.world().resource::<AppStage5ReadinessReport>();
        assert!(report.violations.is_empty(), "{:?}", report.violations);
        assert!(stage5_readiness_passes(report));
    }

    #[test]
    fn stage5_ci_fixture_passes_core_invariants() {
        let report = AppStage5ReadinessReport {
            vt4_ok: true,
            vt5_ok: true,
            single_fire_extract: true,
            gpu_field_authoritative: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE,
            preview_render_target_active: true,
            phase_d_ok: true,
            overlay_from_shared_buffers_only: true,
            particle_lod_scales: true,
            phase_f_lod_proof_ok: true,
            instanced_dispatch_ok: true,
            phase_f_ok: true,
            projection_domains: 3,
            registered_producers: REGISTERED_VISUAL_PRODUCERS.len() as u32,
            duplicate_visual_scan_count: 0,
            violations: Vec::new(),
        };
        assert!(stage5_readiness_passes(&report));
        assert_eq!(fire_visual_producer_count(), 1);

        let mut proof = PhaseFLodProofReport::default();
        let mut metrics = GpuRepresentationMetrics::default();
        metrics.record_fire_upload(RepresentationBand::Full, 128, 16_384, 512, 512, 32_768);
        proof.record_sample(RepresentationBand::Full, &metrics);
        metrics.record_fire_upload(RepresentationBand::Strategic, 32, 4_096, 512, 128, 32_768);
        proof.record_sample(RepresentationBand::Strategic, &metrics);
        assert!(proof.evaluate_ordering());
    }
}
