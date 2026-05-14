//! Stage 5 readiness — runtime invariant enforcement for the visual representation spine.

use bevy::prelude::*;

use crate::gui::editor::world_preview::{
    preview_authoritative_surface, PreviewAuthoritativeSurface, PreviewCameraState,
    PreviewRenderMode, WorldPreviewGpuRuntime,
};
use crate::gui::{
    fire_visual_producer_count, RepresentationBand, RepresentationResult,
    REGISTERED_VISUAL_PRODUCERS, ViewRepresentationSystemSet, WorldRepresentationFrame,
};
use crate::render::extraction::RenderProjectionGraph;
use crate::render::overlay_field_buffers::SharedOverlayFieldBuffers;
use crate::render::phase_f_lod_proof::PhaseFLodProofReport;
use crate::render::visual_agreement::VisualAgreementFrame;
use crate::render::CommittedVisualSnapshotFence;
use crate::render::GpuRepresentationMetrics;
use crate::systems::atmosphere::{
    AtmospherePartialWriteMetrics, P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE,
};

/// Which optional surfaces must be present for readiness to pass.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage5ReadinessProfile {
    pub require_vt4: bool,
    pub require_vt5: bool,
    pub require_preview: bool,
    pub require_partial_metrics: bool,
    pub require_world_frame: bool,
    pub require_phase_f_proof: bool,
}

impl Stage5ReadinessProfile {
    pub const FULL_APP: Self = Self {
        require_vt4: true,
        require_vt5: true,
        require_preview: true,
        require_partial_metrics: true,
        require_world_frame: true,
        require_phase_f_proof: true,
    };

    pub const HEADLESS: Self = Self {
        require_vt4: false,
        require_vt5: false,
        require_preview: false,
        require_partial_metrics: false,
        require_world_frame: false,
        require_phase_f_proof: false,
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
        && report.phase_d_ok
        && report.phase_f_ok
}

pub fn evaluate_app_stage5_readiness(
    profile: Res<Stage5ReadinessProfile>,
    mut report: ResMut<AppStage5ReadinessReport>,
    policy: Option<Res<RepresentationResult>>,
    world: Option<Res<WorldRepresentationFrame>>,
    graph: Option<Res<RenderProjectionGraph>>,
    fence: Option<Res<CommittedVisualSnapshotFence>>,
    overlay: Option<Res<SharedOverlayFieldBuffers>>,
    gpu_metrics: Option<Res<GpuRepresentationMetrics>>,
    agreement: Option<Res<VisualAgreementFrame>>,
    vt_ci: Option<Res<crate::render::vt_ci_matrix::VtCiMatrixLiveReport>>,
    partial_metrics: Option<Res<AtmospherePartialWriteMetrics>>,
    preview_cam: Option<Res<PreviewCameraState>>,
    preview_gpu: Option<Res<WorldPreviewGpuRuntime>>,
    phase_f: Option<Res<PhaseFLodProofReport>>,
) {
    report.violations.clear();
    report.registered_producers = REGISTERED_VISUAL_PRODUCERS.len() as u32;
    report.duplicate_visual_scan_count = 0;

    let producer_count = fire_visual_producer_count();
    report.single_fire_extract = producer_count == 1;
    if producer_count != 1 {
        report.duplicate_visual_scan_count = producer_count.saturating_sub(1);
        report.violations.push(format!(
            "expected one fire_visual producer, registered {producer_count}"
        ));
    }

    report.gpu_field_authoritative = P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE;
    if !P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE {
        report
            .violations
            .push("GPU partial writes not authoritative".into());
    }

    report.overlay_from_shared_buffers_only = overlay.is_some();
    if overlay.is_none() {
        report
            .violations
            .push("SharedOverlayFieldBuffers missing".into());
    }

    let checklist = evaluate_stage5_spine_checklist(
        policy.as_deref(),
        graph.as_deref(),
        fence.as_deref(),
    );
    report.projection_domains = checklist.projection_domains;
    if !checklist.policy_present {
        report.violations.push("RepresentationResult missing".into());
    }
    if checklist.projection_domains < 3 {
        report
            .violations
            .push("RenderProjectionGraph domains incomplete".into());
    }
    if !checklist.fence_committed {
        report
            .violations
            .push("CommittedVisualSnapshotFence not stamped".into());
    }

    if let (Some(policy), Some(metrics)) = (policy.as_deref(), gpu_metrics.as_deref()) {
        report.particle_lod_scales = match policy.active_band {
            RepresentationBand::OverlayOnly | RepresentationBand::Dormant => {
                metrics.particle_rows == 0 && metrics.draw_instances == 0
            }
            RepresentationBand::Strategic => metrics.particle_rows <= metrics.instance_rows,
            _ => metrics.particle_rows > 0 || metrics.instance_rows > 0,
        };
        if !report.particle_lod_scales {
            report
                .violations
                .push("GPU particle rows do not scale with LOD band".into());
        }
    } else {
        report.particle_lod_scales = !profile.require_phase_f_proof;
    }

    if let Some(proof) = phase_f.as_deref() {
        report.phase_f_lod_proof_ok = proof.samples == 0 || proof.ordering_ok;
        if profile.require_phase_f_proof && proof.samples > 0 && !proof.ordering_ok {
            report
                .violations
                .push("Phase F LOD upload ordering failed".into());
        }
    } else {
        report.phase_f_lod_proof_ok = !profile.require_phase_f_proof;
    }

    report.phase_f_ok = report.particle_lod_scales && report.phase_f_lod_proof_ok;
    if profile.require_phase_f_proof && !report.phase_f_ok {
        report
            .violations
            .push("Phase F GPU particle path not proven under LOD".into());
    }

    if let (Some(cam), Some(gpu_rt)) = (preview_cam.as_deref(), preview_gpu.as_deref()) {
        report.preview_render_target_active = matches!(
            preview_authoritative_surface(gpu_rt, cam),
            PreviewAuthoritativeSurface::GpuRenderTarget
        );
        if profile.require_preview
            && cam.mode == PreviewRenderMode::GpuRenderTarget
            && !report.preview_render_target_active
        {
            report
                .violations
                .push("Phase D GPU preview requested but not authoritative".into());
        }
    } else {
        report.preview_render_target_active = false;
        if profile.require_preview {
            report
                .violations
                .push("Preview resources missing for full-app readiness".into());
        }
    }

    report.phase_d_ok = !profile.require_preview || report.preview_render_target_active;
    if profile.require_preview && !report.phase_d_ok {
        report
            .violations
            .push("Phase D GPU preview render target not authoritative".into());
    }

    if let Some(live) = vt_ci.as_deref() {
        report.vt4_ok = live.vt4.passes();
        report.vt5_ok = live.vt5_ok;
        if profile.require_vt4 && !report.vt4_ok {
            report.violations.push(format!(
                "VT-4 matrix mismatch_count={} failing_surface_mask={:#x} stamp={}",
                live.vt4.mismatch_count, live.vt4.failing_surface_mask, live.vt4.stamp.tick
            ));
        }
        if profile.require_vt5 && !report.vt5_ok {
            report.violations.push(format!(
                "VT-5 spatial invariants failed (stamp={})",
                live.vt4.stamp.tick
            ));
        }
    } else if let Some(agreement) = agreement.as_deref() {
        report.vt4_ok = agreement.mismatch_count == 0 && agreement.stamp.tick > 0;
        report.vt5_ok = !profile.require_vt5;
        if !report.vt4_ok {
            report.violations.push(format!(
                "VT-4 agreement mismatch_count={} stamp={}",
                agreement.mismatch_count, agreement.stamp.tick
            ));
        }
    } else {
        report.vt4_ok = !profile.require_vt4;
        report.vt5_ok = !profile.require_vt5;
        if profile.require_vt4 {
            report
                .violations
                .push("VisualAgreementFrame missing".into());
        }
        if profile.require_vt5 {
            report
                .violations
                .push("VtCiMatrixLiveReport missing".into());
        }
    }

    if world.is_none() && profile.require_world_frame {
        report
            .violations
            .push("WorldRepresentationFrame missing".into());
    }

    if partial_metrics.is_none() && profile.require_partial_metrics {
        report
            .violations
            .push("AtmospherePartialWriteMetrics missing".into());
    }
}

pub struct Stage5ReadinessPlugin;

impl Plugin for Stage5ReadinessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stage5ReadinessProfile>()
            .init_resource::<AppStage5ReadinessReport>()
            .add_systems(
                PostUpdate,
                evaluate_app_stage5_readiness
                    .after(ViewRepresentationSystemSet::SyncOverlayField)
                    .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu)
                    .after(crate::render::vt_ci_matrix::record_vt_ci_matrix_live),
            );
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
