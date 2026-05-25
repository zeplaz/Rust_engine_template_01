//! CI-level VT-4 / VT-5 scene matrix — one deterministic spine across visual surfaces.
//!
//! Run in CI via `cargo test --lib vt_ci_matrix`.

use bevy::math::{IVec2, Vec4};
use bevy::prelude::*;

use crate::gui::{
    build_representation_inputs, build_representation_result, LodZoneRegistry, OverlayFieldFrame,
    VisualBudgetSettings, VisualCadence, WorldLodBand, WorldLodBands, WorldLodMap,
    WorldRepresentationFrame,
};
use crate::render::extraction::{
    ProjectionNodeTrait, RenderProjectionContext, RenderProjectionGraph,
};
use crate::render::gpu_particles::{
    update_world_fire_particles_from_projection, WorldFireParticleFrame,
};
use crate::render::overlay_field_buffers::SharedOverlayFieldBuffers;
use crate::render::{
    fire_chunk_lod_state_from_simulation, tactical_fire_visual, FireSimulationSnapshot,
    FireVisualFramesByView,
};
use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame, FireVisualGpuInstance};
use crate::render::visual_agreement::{
    hash_shared_overlay_heat, update_visual_agreement_frame, OverlayAgreementDebug,
    VisualAgreementFrame, WorldPreviewVt4Probe,
};
use crate::render::visual_snapshot_commit::CommittedVisualSnapshotFence;
use crate::render::{EcologyVisualSnapshot, LogisticsVisualSnapshot};
use crate::render::vt_spatial_invariants::passes_vt5_spatial_invariants;
use crate::systems::sim_control::SimStepStamp;

/// VT-4 consumer surface id (bit index for [`Vt4CiReport::failing_surface_mask`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Vt4SurfaceId {
    MinimapOverlay = 0,
    WorldPreview = 1,
    GpuFireField = 2,
    ParticleProjection = 3,
}

impl Vt4SurfaceId {
    #[must_use]
    pub const fn bit(self) -> u32 {
        1u32 << (self as u8)
    }
}

/// Strict VT-4 CI outcome for one committed scenario tick.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Vt4CiReport {
    pub stamp: SimStepStamp,
    pub mismatch_count: u32,
    pub failing_surface_mask: u32,
}

impl Vt4CiReport {
    #[must_use]
    pub fn passes(&self) -> bool {
        self.mismatch_count == 0 && self.failing_surface_mask == 0
    }

    pub fn record_surface_mismatch(&mut self, surface: Vt4SurfaceId) {
        self.mismatch_count = self.mismatch_count.saturating_add(1);
        self.failing_surface_mask |= surface.bit();
    }
}

/// Deterministic multi-surface fixture for CI (extract → fence → projection → particles).
#[derive(Debug, Clone)]
pub struct Vt4CiScenario {
    pub fire: FireVisualFrame,
    pub sim: FireSimulationSnapshot,
    pub shared: SharedOverlayFieldBuffers,
    pub overlay: OverlayFieldFrame,
    pub graph: RenderProjectionGraph,
    pub particles: WorldFireParticleFrame,
    pub preview_probe: WorldPreviewVt4Probe,
    pub fence: CommittedVisualSnapshotFence,
}

fn sample_fire_instance(chunk: IVec2, heat: f32, ember: f32) -> FireVisualGpuInstance {
    let mut row = FireVisualGpuInstance::default();
    row.chunk_xy_heat_lum = Vec4::new(chunk.x as f32, chunk.y as f32, heat, 1.0);
    row.world_xyz_radius = Vec4::new(chunk.x as f32 * 64.0, chunk.y as f32 * 64.0, 0.0, 32.0);
    row.smoke_ember_vis_priority = Vec4::new(0.1, ember, 0.0, 1.0);
    row
}

/// Build the canonical CI scene: multi-chunk heat + instances, committed fence, projection, particles.
#[must_use]
pub fn build_deterministic_ci_scenario() -> Vt4CiScenario {
    let stamp = SimStepStamp::new(42, 9_000);
    let rows = vec![
        ChunkFireHeat {
            chunk: IVec2::new(0, 0),
            heat: 0.5,
            smoke: 0.0,
        },
        ChunkFireHeat {
            chunk: IVec2::new(12, 4),
            heat: 0.7,
            smoke: 0.1,
        },
    ];
    let instances = vec![
        sample_fire_instance(IVec2::new(0, 0), 0.9, 0.4),
        sample_fire_instance(IVec2::new(16, 4), 0.8, 0.35),
    ];
    let fire = FireVisualFrame {
        stamp,
        instances,
        chunk_heat: rows,
    };

    let mut shared = SharedOverlayFieldBuffers::default();
    shared.stamp = stamp;
    for row in &fire.chunk_heat {
        shared.chunk_fire_heat.insert(row.chunk, row.heat);
    }

    let overlay = OverlayFieldFrame {
        stamp,
        fields: std::collections::HashMap::new(),
        fire_heat_overlay_revision: 7,
    };

    let mut lod = WorldRepresentationFrame::default();
    lod.bands = WorldLodBands {
        global: WorldLodBand::LocalTactical,
    };
    lod.resolution = crate::gui::resolution_for_band(WorldLodBand::LocalTactical);
    let lod_map = WorldLodMap::default();
    let policy_inputs = build_representation_inputs(
        &crate::gui::CameraVisualState::default(),
        &LodZoneRegistry::default(),
        &VisualBudgetSettings::default(),
        &VisualCadence::from(&VisualBudgetSettings::default()),
        stamp,
    );
    let policy = build_representation_result(&lod, &policy_inputs);
    let mut graph = RenderProjectionGraph::default();
    let logistics = LogisticsVisualSnapshot::default();
    let ecology = EcologyVisualSnapshot::default();
    let ctx = RenderProjectionContext {
        policy: &policy,
        lod: &lod,
        lod_map: &lod_map,
        fire: &fire,
        logistics: &logistics,
        ecology: &ecology,
        committed_stamp: stamp,
    };
    graph.evaluate(&ctx);

    let sim = FireSimulationSnapshot {
        stamp: fire.stamp,
        instances: fire.instances.clone(),
        chunk_heat: fire.chunk_heat.clone(),
    };
    let chunk_lod = fire_chunk_lod_state_from_simulation(&sim);

    let mut particles = WorldFireParticleFrame::default();
    update_world_fire_particles_from_projection(
        &graph,
        &mut particles,
        Some(&chunk_lod),
        crate::render::gpu_particles::FireParticleCameraScale::default(),
        None,
    );

    let preview_probe = WorldPreviewVt4Probe {
        stamp,
        overlay_heat_hash: hash_shared_overlay_heat(&shared.chunk_fire_heat),
        overlay_revision: overlay.fire_heat_overlay_revision,
        consumer_active: true,
    };

    let fence = CommittedVisualSnapshotFence { fire: stamp };

    Vt4CiScenario {
        fire: fire.clone(),
        sim: sim.clone(),
        shared,
        overlay,
        graph,
        particles,
        preview_probe,
        fence,
    }
}

/// Build a live VT-4 scenario from the current committed visual spine resources.
#[must_use]
pub fn build_live_vt4_scenario(
    fire: &FireVisualFrame,
    sim: &FireSimulationSnapshot,
    shared: &SharedOverlayFieldBuffers,
    overlay: &OverlayFieldFrame,
    graph: &RenderProjectionGraph,
    particles: &WorldFireParticleFrame,
    preview_probe: &WorldPreviewVt4Probe,
    fence: &CommittedVisualSnapshotFence,
) -> Vt4CiScenario {
    Vt4CiScenario {
        fire: fire.clone(),
        sim: sim.clone(),
        shared: shared.clone(),
        overlay: overlay.clone(),
        graph: graph.clone(),
        particles: particles.clone(),
        preview_probe: preview_probe.clone(),
        fence: *fence,
    }
}

/// Apply VT-4 surface bitmask checks on top of an already-updated agreement frame.
pub fn apply_vt4_ci_surface_checks(
    scenario: &Vt4CiScenario,
    agreement: &VisualAgreementFrame,
    report: &mut Vt4CiReport,
) {
    report.stamp = scenario.fire.stamp;
    report.mismatch_count = agreement.mismatch_count.min(u32::MAX as u64) as u32;

    if scenario.fence.fire != scenario.fire.stamp {
        report.record_surface_mismatch(Vt4SurfaceId::MinimapOverlay);
        report.record_surface_mismatch(Vt4SurfaceId::WorldPreview);
        report.record_surface_mismatch(Vt4SurfaceId::GpuFireField);
        report.record_surface_mismatch(Vt4SurfaceId::ParticleProjection);
    }

    if scenario.shared.stamp != scenario.fire.stamp {
        report.record_surface_mismatch(Vt4SurfaceId::MinimapOverlay);
    }

    if scenario.preview_probe.participates_in_vt4() {
        if scenario.preview_probe.stamp != scenario.fire.stamp {
            report.record_surface_mismatch(Vt4SurfaceId::WorldPreview);
        }

        if scenario.preview_probe.overlay_revision != scenario.overlay.fire_heat_overlay_revision {
            report.record_surface_mismatch(Vt4SurfaceId::WorldPreview);
        }

        if scenario.preview_probe.overlay_heat_hash != agreement.sim_overlay_heat_hash {
            report.record_surface_mismatch(Vt4SurfaceId::WorldPreview);
        }
    }

    if agreement.projected_fire_heat_hash != agreement.fire_heat_hash {
        report.record_surface_mismatch(Vt4SurfaceId::GpuFireField);
    }

    if scenario.particles.snapshot_stamp != scenario.fence.fire.tick {
        report.record_surface_mismatch(Vt4SurfaceId::ParticleProjection);
    }
}

/// Run VT-4 strict checks across minimap, preview, GPU fire field, and particle projection rows.
pub fn run_vt4_ci_matrix(
    scenario: &Vt4CiScenario,
    agreement: &mut VisualAgreementFrame,
    report: &mut Vt4CiReport,
) {
    *report = Vt4CiReport::default();

    update_visual_agreement_frame(
        &scenario.fire,
        &scenario.sim,
        &scenario.shared,
        &scenario.overlay,
        Some(&scenario.graph),
        Some(&scenario.preview_probe),
        agreement,
    );
    apply_vt4_ci_surface_checks(scenario, agreement, report);
}

/// Apply CI report fields to runtime VT-4 debug resources.
pub fn apply_vt4_ci_report_to_overlay_debug(
    report: &Vt4CiReport,
    agreement: &VisualAgreementFrame,
    overlay_debug: &mut OverlayAgreementDebug,
) {
    overlay_debug.stamp = agreement.stamp;
    overlay_debug.compared_stamp = report.stamp;
    overlay_debug.mismatch_count = report.mismatch_count;
    overlay_debug.overlay_revision = agreement.overlay_revision;
    overlay_debug.gpu_row_count = agreement.projected_instance_count as u32;
    overlay_debug.preview_revision = agreement.preview_overlay_revision;
    overlay_debug.failing_surface_mask = report.failing_surface_mask;
}

/// VT-5 spatial invariants on extract, projection, and particle rows.
#[must_use]
pub fn run_vt5_ci_spatial_matrix(scenario: &Vt4CiScenario) -> bool {
    if scenario.fire.instances.len() < 2 {
        return true;
    }
    passes_vt5_spatial_invariants(&scenario.fire.instances)
        && passes_vt5_spatial_invariants(&scenario.graph.fire.instance_buffer)
        && particle_rows_pass_vt5(&scenario.particles)
}

/// Latest live VT-4 / VT-5 matrix outcome for runtime readiness and diagnostics.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct VtCiMatrixLiveReport {
    pub vt4: Vt4CiReport,
    pub vt5_ok: bool,
}

pub fn record_vt_ci_matrix_live(
    fire_by_view: Res<FireVisualFramesByView>,
    sim: Res<FireSimulationSnapshot>,
    shared: Res<SharedOverlayFieldBuffers>,
    overlay: Res<OverlayFieldFrame>,
    graph: Res<RenderProjectionGraph>,
    particles: Res<WorldFireParticleFrame>,
    preview_probe: Option<Res<WorldPreviewVt4Probe>>,
    fence: Res<CommittedVisualSnapshotFence>,
    agreement: Res<VisualAgreementFrame>,
    mut overlay_debug: ResMut<OverlayAgreementDebug>,
    mut live: ResMut<VtCiMatrixLiveReport>,
) {
    let probe = preview_probe.as_deref().cloned().unwrap_or_default();
    let fire = tactical_fire_visual(fire_by_view.as_ref());
    let scenario = build_live_vt4_scenario(
        fire,
        sim.as_ref(),
        shared.as_ref(),
        overlay.as_ref(),
        graph.as_ref(),
        particles.as_ref(),
        &probe,
        fence.as_ref(),
    );
    let mut report = Vt4CiReport::default();
    apply_vt4_ci_surface_checks(&scenario, agreement.as_ref(), &mut report);
    apply_vt4_ci_report_to_overlay_debug(&report, agreement.as_ref(), overlay_debug.as_mut());
    live.vt4 = report;
    live.vt5_ok = run_vt5_ci_spatial_matrix(&scenario);
    if !live.vt4.passes() {
        warn!(
            "VT-4 live matrix mismatch_count={} failing_surface_mask={:#x} stamp={}",
            live.vt4.mismatch_count, live.vt4.failing_surface_mask, live.vt4.stamp.tick
        );
    }
    if !live.vt5_ok {
        warn!(
            "VT-5 spatial invariants failed on extract/projection/particle rows (stamp={})",
            scenario.fire.stamp.tick
        );
    }
}

/// Deterministic full-app VT fixture used by readiness and integration tests.
#[must_use]
pub fn full_app_vt_ci_fixture_passes() -> bool {
    let scenario = build_deterministic_ci_scenario();
    let mut agreement = VisualAgreementFrame::default();
    let mut report = Vt4CiReport::default();
    run_vt4_ci_matrix(&scenario, &mut agreement, &mut report);
    report.passes() && run_vt5_ci_spatial_matrix(&scenario)
}

pub struct VtCiMatrixPlugin;

impl Plugin for VtCiMatrixPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VtCiMatrixLiveReport>().add_systems(
            PostUpdate,
            record_vt_ci_matrix_live
                .after(crate::render::visual_agreement::record_visual_agreement_frame),
        );
    }
}

fn particle_rows_pass_vt5(particles: &WorldFireParticleFrame) -> bool {
    if particles.instances.len() < 2 {
        return false;
    }
    let mut rows = Vec::with_capacity(particles.instances.len());
    for inst in &particles.instances {
        let mut row = FireVisualGpuInstance::default();
        row.chunk_xy_heat_lum = Vec4::new(
            inst.world_xyz_heat.x / 64.0,
            inst.world_xyz_heat.y / 64.0,
            inst.world_xyz_heat.w,
            1.0,
        );
        rows.push(row);
    }
    passes_vt5_spatial_invariants(&rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    #[test]
    fn vt4_ci_matrix_agreement_green() {
        let scenario = build_deterministic_ci_scenario();
        let mut agreement = VisualAgreementFrame::default();
        let mut report = Vt4CiReport::default();
        run_vt4_ci_matrix(&scenario, &mut agreement, &mut report);
        assert!(report.passes(), "mismatches={} mask={:#x}", report.mismatch_count, report.failing_surface_mask);
        assert_eq!(report.stamp, scenario.fire.stamp);
    }

    #[test]
    fn vt4_ci_matrix_stamp_mismatch_fails_with_surface_mask() {
        let mut scenario = build_deterministic_ci_scenario();
        scenario.overlay.stamp = SimStepStamp::new(99, 0);
        let mut agreement = VisualAgreementFrame::default();
        let mut report = Vt4CiReport::default();
        run_vt4_ci_matrix(&scenario, &mut agreement, &mut report);
        assert!(!report.passes());
        assert!(report.mismatch_count > 0);
    }

    #[test]
    fn vt4_ci_matrix_fence_mismatch_flags_all_surfaces() {
        let mut scenario = build_deterministic_ci_scenario();
        scenario.fence.fire = SimStepStamp::new(1, 0);
        let mut agreement = VisualAgreementFrame::default();
        let mut report = Vt4CiReport::default();
        run_vt4_ci_matrix(&scenario, &mut agreement, &mut report);
        assert!(!report.passes());
        assert_ne!(report.failing_surface_mask & Vt4SurfaceId::ParticleProjection.bit(), 0);
    }

    #[test]
    fn vt4_ci_matrix_preview_revision_mismatch_fails() {
        let mut scenario = build_deterministic_ci_scenario();
        scenario.preview_probe.overlay_revision = 0;
        let mut agreement = VisualAgreementFrame::default();
        let mut report = Vt4CiReport::default();
        run_vt4_ci_matrix(&scenario, &mut agreement, &mut report);
        assert!(!report.passes());
        assert_ne!(report.failing_surface_mask & Vt4SurfaceId::WorldPreview.bit(), 0);
    }

    #[test]
    fn vt5_ci_matrix_spatial_passes_on_extract_projection_particles() {
        let scenario = build_deterministic_ci_scenario();
        assert!(run_vt5_ci_spatial_matrix(&scenario));
    }

    #[test]
    fn vt5_ci_matrix_collapsed_extract_fails() {
        let mut scenario = build_deterministic_ci_scenario();
        scenario.fire.instances = vec![
            sample_fire_instance(IVec2::ZERO, 0.9, 0.4),
            sample_fire_instance(IVec2::ZERO, 0.8, 0.3),
        ];
        assert!(!passes_vt5_spatial_invariants(&scenario.fire.instances));
    }

    #[test]
    fn vt4_ci_overlay_debug_receives_surface_mask() {
        let scenario = build_deterministic_ci_scenario();
        let mut agreement = VisualAgreementFrame::default();
        let mut report = Vt4CiReport::default();
        let mut overlay_debug = OverlayAgreementDebug::default();
        run_vt4_ci_matrix(&scenario, &mut agreement, &mut report);
        apply_vt4_ci_report_to_overlay_debug(&report, &agreement, &mut overlay_debug);
        assert_eq!(overlay_debug.mismatch_count, 0);
        assert_eq!(overlay_debug.compared_stamp, scenario.fire.stamp);
        assert_eq!(overlay_debug.failing_surface_mask, 0);
    }

    #[test]
    fn vt_ci_app_bootstraps_agreement_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<VisualAgreementFrame>();
        app.init_resource::<OverlayAgreementDebug>();
        app.init_resource::<WorldPreviewVt4Probe>();
        app.init_resource::<CommittedVisualSnapshotFence>();
        app.update();
        assert!(app.world().contains_resource::<VisualAgreementFrame>());
        assert!(app.world().contains_resource::<OverlayAgreementDebug>());
    }

    #[test]
    fn live_vt4_scenario_matches_deterministic_fixture() {
        let scenario = build_deterministic_ci_scenario();
        let live = build_live_vt4_scenario(
            &scenario.fire,
            &scenario.sim,
            &scenario.shared,
            &scenario.overlay,
            &scenario.graph,
            &scenario.particles,
            &scenario.preview_probe,
            &scenario.fence,
        );
        let mut agreement = VisualAgreementFrame::default();
        let mut report = Vt4CiReport::default();
        run_vt4_ci_matrix(&live, &mut agreement, &mut report);
        assert!(report.passes());
        assert!(run_vt5_ci_spatial_matrix(&live));
    }

    #[test]
    fn full_app_vt_ci_fixture_passes_deterministic_scene() {
        assert!(full_app_vt_ci_fixture_passes());
    }

    #[test]
    fn vt4_ci_matrix_preview_consumer_inactive_skips_surface() {
        let scenario = build_deterministic_ci_scenario();
        let mut agreement = VisualAgreementFrame::default();
        let mut report = Vt4CiReport::default();
        let mut inactive = scenario.clone();
        inactive.preview_probe.consumer_active = false;
        inactive.preview_probe.stamp = SimStepStamp::default();
        inactive.preview_probe.overlay_revision = 0;
        inactive.preview_probe.overlay_heat_hash = 0;
        run_vt4_ci_matrix(&inactive, &mut agreement, &mut report);
        assert_eq!(report.failing_surface_mask & Vt4SurfaceId::WorldPreview.bit(), 0);
    }

    #[test]
    fn stage5_ci_core_readiness_fixture_passes() {
        assert!(crate::render::vt_ci_matrix::full_app_vt_ci_fixture_passes());
        use crate::render::stage5_readiness::{stage5_readiness_passes, AppStage5ReadinessReport};
        use crate::systems::atmosphere::P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE;

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
            registered_producers: 1,
            duplicate_visual_scan_count: 0,
            violations: Vec::new(),
        };
        assert!(stage5_readiness_passes(&report));
    }
}
