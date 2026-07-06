//! RGR-H3-001 split — headless FULL_APP readiness app assembly + probe entry point.
//! Carved verbatim from `stage5_full_app_harness.rs` (pre-split monolith).

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

#[cfg(test)]
use crate::gui::editor::world_preview::{PreviewCameraState, WorldPreviewUiState};
#[cfg(test)]
use crate::render::gpu_particle_draw::WorldFireParticleDrawDispatch;
#[cfg(test)]
use crate::render::gpu_particles::WorldFireParticleFrame;
#[cfg(test)]
use crate::render::phase_f_lod_proof::PhaseFLodProofReport;
#[cfg(test)]
use crate::render::GpuRepresentationMetrics;
use crate::render::stage5_readiness::{
    evaluate_app_stage5_readiness, AppStage5ReadinessReport, Stage5ReadinessProfile,
    Stage5ReadinessTruthInputs,
};
#[cfg(test)]
use crate::systems::sim_control::{SimTick, SimTimeMicros};

#[cfg(test)]
use bevy::asset::AssetPlugin;
#[cfg(test)]
use crate::engine::states::BaseState;
#[cfg(test)]
use crate::gui::editor::world_gen_ui::WorldGenUiState;
#[cfg(test)]
use crate::gui::editor::world_preview::{PreviewRenderMode, WorldPreviewGpuRuntime};
#[cfg(test)]
use crate::gui::{
    build_representation_inputs, build_representation_result, CameraVisualState, FxVisibilitySettings,
    LodZoneRegistry, MapCameraDesired, MapCameraSettings, OverlayFieldFrame, VisualBudgetSettings,
    VisualCadence, WorldLodBand, WorldLodBands, WorldLodMap, WorldRepresentationFrame,
};
#[cfg(test)]
use crate::gui::MapCameraDesiredRes;
#[cfg(test)]
use crate::terrain::material::WorldPreviewState;
#[cfg(test)]
use crate::render::extraction::FireVisualFramePlugin;
#[cfg(test)]
use crate::render::gpu_indirect_draw::compact_world_fire_indirect_draw;
#[cfg(test)]
use crate::render::{
    DomainProjectionFramePlugin, GpuIndirectDrawSpinePlugin, PhaseFLodProofPlugin, VtCiMatrixPlugin,
};
#[cfg(test)]
use crate::render::vt_ci_matrix::{
    build_deterministic_ci_scenario, run_vt4_ci_matrix, run_vt5_ci_spatial_matrix, Vt4CiReport,
    Vt4CiScenario, VtCiMatrixLiveReport,
};
#[cfg(test)]
use crate::render::visual_agreement::VisualAgreementFrame;
#[cfg(test)]
use crate::render::CommittedVisualSnapshotFence;
#[cfg(test)]
use crate::render::ClimateVisualAggregate;
#[cfg(test)]
use crate::render::WorldPreviewVt4Probe;
#[cfg(test)]
use crate::systems::atmosphere::{AtmosphereDiagnostics, AtmospherePartialWriteMetrics};
#[cfg(test)]
use crate::systems::sim_control::SimControlState;
#[cfg(test)]
use crate::systems::transport::TransportEdgeDirectory;
#[cfg(test)]
use crate::gui::ActiveCameraOwner;
#[cfg(test)]
use crate::render::{OverlayAgreementDebug, Stage5ReadinessPlugin};

#[cfg(test)]
/// Seed the committed VT CI scenario into the main-world spine resources.
pub(super) fn hydrate_world_from_vt_ci_scenario(world: &mut World, scenario: &Vt4CiScenario) {
    let mut lod = WorldRepresentationFrame::default();
    lod.bands = WorldLodBands {
        global: WorldLodBand::LocalTactical,
    };
    lod.resolution = crate::gui::resolution_for_band(WorldLodBand::LocalTactical);
    let policy_inputs = build_representation_inputs(
        &crate::gui::CameraVisualState::default(),
        &LodZoneRegistry::default(),
        &VisualBudgetSettings::default(),
        &VisualCadence::from(&VisualBudgetSettings::default()),
        scenario.fence.fire,
    );
    let policy = build_representation_result(&lod, &policy_inputs);
    let particle_count = scenario.particles.instances.len() as u32;
    let instance_rows = scenario.graph.fire.instance_buffer.len() as u32;
    let mut metrics = GpuRepresentationMetrics::default();
    metrics.particle_rows = particle_count;
    metrics.instance_rows = instance_rows;
    metrics.active_band = policy.active_band;
    world.insert_resource(metrics);

    let mut proof = PhaseFLodProofReport::default();
    proof.ordering_ok = true;
    proof.samples = 1;
    world.insert_resource(proof);

    if policy.particle_policy.instanced_draw {
        let mut dispatch = WorldFireParticleDrawDispatch::default();
        dispatch.instance_count = particle_count;
        world.insert_resource(compact_world_fire_indirect_draw(
            &policy,
            &scenario.particles,
            &dispatch,
        ));
        world.insert_resource(dispatch);
    }

    world.insert_resource(scenario.fire.clone());
    world.insert_resource(scenario.shared.clone());
    world.insert_resource(scenario.overlay.clone());
    world.insert_resource(scenario.graph.clone());
    world.insert_resource(scenario.particles.clone());
    world.insert_resource(scenario.fence);
    world.insert_resource(scenario.preview_probe.clone());
    world.insert_resource(lod);
    world.insert_resource(policy);
    world.insert_resource(OverlayFieldFrame {
        stamp: scenario.overlay.stamp,
        fields: scenario.overlay.fields.clone(),
        fire_heat_overlay_revision: scenario.overlay.fire_heat_overlay_revision,
    });
    world.insert_resource(scenario.graph.clone());

    let mut agreement = VisualAgreementFrame::default();
    let mut vt4 = Vt4CiReport::default();
    run_vt4_ci_matrix(scenario, &mut agreement, &mut vt4);
    world.insert_resource(agreement);
    world.insert_resource(VtCiMatrixLiveReport {
        vt4,
        vt5_ok: run_vt5_ci_spatial_matrix(scenario),
    });

    world.insert_resource(WorldPreviewUiState::default());
    if !world.contains_resource::<WorldPreviewVt4Probe>() {
        world.init_resource::<WorldPreviewVt4Probe>();
    }
    world.insert_resource(PreviewCameraState {
        center: Vec2::ZERO,
        zoom: 1.0,
        mode: PreviewRenderMode::GpuRenderTarget,
    });
    world.insert_resource(WorldPreviewGpuRuntime {
        offscreen_renderer_ready: true,
        ..default()
    });
    if !world.contains_resource::<AtmospherePartialWriteMetrics>() {
        world.init_resource::<AtmospherePartialWriteMetrics>();
    }
    if !world.contains_resource::<CommittedVisualSnapshotFence>() {
        world.insert_resource(scenario.fence);
    }
    if !world.contains_resource::<WorldFireParticleFrame>() {
        world.insert_resource(scenario.particles.clone());
    }
}

#[cfg(test)]
/// Assemble a headless app with the same readiness plugins as the live view spine.
pub(super) fn assemble_headless_full_app_readiness_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), bevy::state::app::StatesPlugin));
    app.init_resource::<Assets<Image>>();
    app.init_state::<BaseState>();
    app.init_resource::<SimControlState>();
    app.init_resource::<SimTick>();
    app.init_resource::<SimTimeMicros>();
    app.init_resource::<AtmosphereDiagnostics>();
    app.init_resource::<ClimateVisualAggregate>();
    app.init_resource::<TransportEdgeDirectory>();
    app.init_resource::<MapCameraSettings>();
    app.init_resource::<MapCameraDesiredRes>();
    app.init_resource::<WorldPreviewState>();
    app.init_resource::<WorldGenUiState>();
    app.init_resource::<VisualAgreementFrame>();
    app.init_resource::<OverlayAgreementDebug>();
    app.init_resource::<GpuRepresentationMetrics>();
    app.init_resource::<OverlayFieldFrame>();
    app.init_resource::<CameraVisualState>();
    app.init_resource::<FxVisibilitySettings>();
    app.init_resource::<VisualBudgetSettings>();
    app.init_resource::<VisualCadence>();
    app.init_resource::<ActiveCameraOwner>();
    app.init_resource::<WorldLodMap>();
    app.insert_resource(crate::terrain::generation::world_generator_enhanced::WorldGenParams::default());
    app.add_plugins((
        Stage5ReadinessPlugin,
        VtCiMatrixPlugin,
        PhaseFLodProofPlugin,
        DomainProjectionFramePlugin,
        GpuIndirectDrawSpinePlugin,
    ));
    app.add_plugins(FireVisualFramePlugin);
    app.init_resource::<WorldFireParticleDrawDispatch>();
    app.add_systems(
        Update,
        crate::render::sync_particle_draw_dispatch_from_policy
            .after(crate::render::merge_domain_projection_into_representation)
            .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu),
    );
    app.add_plugins(crate::render::SharedOverlayFieldBuffersPlugin);
    app.insert_resource(Stage5ReadinessProfile::FULL_APP);
    app
}

/// Evaluate FULL_APP readiness on the current world (no extra schedule tick).
pub fn probe_full_app_stage5_readiness(app: &mut App) -> AppStage5ReadinessReport {
    app.world_mut()
        .insert_resource(Stage5ReadinessProfile::FULL_APP);
    if !app.world().contains_resource::<AppStage5ReadinessReport>() {
        app.init_resource::<AppStage5ReadinessReport>();
    }
    if !app.world().contains_resource::<Stage5ReadinessTruthInputs>() {
        app.init_resource::<Stage5ReadinessTruthInputs>();
    }
    let _ = app
        .world_mut()
        .run_system_once(evaluate_app_stage5_readiness);
    app.world().resource::<AppStage5ReadinessReport>().clone()
}
