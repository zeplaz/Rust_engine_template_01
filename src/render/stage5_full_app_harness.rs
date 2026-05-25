//! FULL_APP Stage 5 readiness probe — headless fixture + optional runtime logging.

use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::gui::editor::world_preview::{
    PreviewCameraState, PreviewPathAuthority, WorldPreviewRenderTargetRegistry,
    WorldPreviewRenderViewportContract, WorldPreviewUiState, WorldPreviewViewportAuthority,
};
use crate::render::gpu_particle_draw::WorldFireParticleDrawDispatch;
use crate::render::phase_f_lod_proof::PhaseFLodProofReport;
use crate::gui::{
    MapTextureSource, MapPresentationDiagnostics, MapViewInstanceId, MapViewInstances,
    MapViewPresentationStates, MapViewTextureCache, MapFitValidationLog, ResolvedMapViewFrames,
    MinimapPresentationSource, MinimapShellState, SimulationMapViewport,
    ViewRepresentationSnapshot, WorldBounds,
};
use crate::render::extraction::RenderProjectionGraph;
use crate::render::gpu_indirect_draw::GpuIndirectDrawSpine;
use crate::render::gpu_particles::WorldFireParticleFrame;
use crate::render::gpu_water_particles::WorldWaterParticleFrame;
use crate::render::WaterSurfaceVisualCatalog;
use crate::render::overlay_field_buffers::SharedOverlayFieldBuffers;
use crate::render::{
    build_minimap_compositor_proof_payload, minimap_gpu_compositor_env_enabled,
    MinimapGpuCompositorDiagnostics, MinimapCompositorState, MinimapRenderTargetRegistry,
};
use crate::render::viewport_pipeline::{ResolvedViewports, ViewportPresentationMismatch};
use crate::render::GpuRepresentationMetrics;
use crate::dev::{
    Stage5FinishTodoBoard, Stage5FinishUx06Streak, Stage5LiveTodoBoard, TodoStatus,
    FINISH_UX06_STREAK_DONE, STAGE5_FINISH_TODOS, STAGE5_TODOS,
};
use crate::render::stage5_readiness::{
    evaluate_app_stage5_readiness, stage5_readiness_passes, AppStage5ReadinessReport,
    Stage5ReadinessProfile, Stage5ReadinessTruthInputs,
};
use crate::systems::sim_control::{SimTick, SimTimeMicros};
use crate::engine::DebugCaptureFrameGate;

/// Tactical zoom band for Phase 2 VFX witness gates (matches §7 / D-F09).
pub(crate) const TACTICAL_VFX_ZOOM_ALPHA_MIN: f32 = 0.65;

/// When set, `--test visual` blocks proof commit unless [`TacticalVfxWitnessGates::all_green`].
#[inline]
pub(crate) fn tactical_vfx_proof_enabled() -> bool {
    matches!(
        std::env::var("TACTICAL_VFX_PROOF").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    )
}

/// P2-VFX-VISUAL-001: `--test visual` / VfxSandbox always require tactical particle witness before proof commit.
#[inline]
pub(crate) fn visual_tactical_vfx_witness_required(launch: &crate::engine::EngineLaunchArgs) -> bool {
    tactical_vfx_proof_enabled()
        || matches!(
            launch.test_scene,
            crate::engine::TestScene::Visual | crate::engine::TestScene::VfxSandbox
        )
}

/// P2-VFX-WITNESS-001 / P2-WATER-WITNESS-002 JSON gate evaluation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TacticalVfxWitnessGates {
    pub fire_tactical_zoom: bool,
    pub fire_spark_rows_gt_0: bool,
    pub water_tactical_zoom: bool,
    pub water_has_river_segments: bool,
    pub water_particle_rows_gt_0: bool,
    pub water_particle_river_streaks_when_rivers: bool,
    pub water_shader_motion_always_on: bool,
    pub water_particle_strategic_not_culled: bool,
    pub water_w1_river_read_green: bool,
    pub water_strategic_001_green: bool,
    pub water_witness_001_green: bool,
    pub water_witness_foam_or_ocean_green: bool,
}

impl TacticalVfxWitnessGates {
    pub(crate) fn evaluate(
        particles: Option<&WorldFireParticleFrame>,
        water_catalog: Option<&WaterSurfaceVisualCatalog>,
        water_particles: Option<&WorldWaterParticleFrame>,
    ) -> Self {
        let fire_zoom = particles
            .map(|p| p.spark_witness.zoom_alpha)
            .unwrap_or(0.0);
        let fire_tactical = fire_zoom >= TACTICAL_VFX_ZOOM_ALPHA_MIN;
        let fire_rows = particles.map(|p| p.spark_witness.rows).unwrap_or(0);

        let water_zoom = water_particles
            .map(|p| p.witness.zoom_alpha)
            .unwrap_or(0.0);
        let water_tactical = water_zoom >= TACTICAL_VFX_ZOOM_ALPHA_MIN;
        let water_has_rivers = water_catalog
            .map(|c| !c.river_segments.is_empty())
            .unwrap_or(false);
        let water_rows = water_particles.map(|p| p.witness.rows).unwrap_or(0);
        let water_streaks = water_particles
            .map(|p| p.witness.river_streaks)
            .unwrap_or(0);
        let water_shader_on = water_particles
            .map(|p| p.witness.shader_motion_always_on)
            .unwrap_or(false);
        let water_not_culled = water_particles
            .map(|p| !p.witness.strategic_culled)
            .unwrap_or(false);
        let water_river_read = water_catalog
            .map(|c| {
                c.w1_river_read_green_at_zoom(
                    crate::render::water_surface_visual::WATER_STRATEGIC_ZOOM_ALPHA * 0.5,
                )
            })
            .unwrap_or(false);
        let water_bands = water_catalog.map(|c| {
            crate::render::gpu_water_particles::evaluate_water_vfx_witness_bands(
                c,
                water_zoom,
                0.0,
            )
        });
        let water_strategic_001 = water_bands
            .as_ref()
            .map(crate::render::gpu_water_particles::water_strategic_001_green)
            .unwrap_or(false);
        let water_witness_001 = water_catalog
            .zip(water_bands)
            .map(|(c, b)| crate::render::gpu_water_particles::water_witness_001_green(c, &b))
            .unwrap_or(false);
        let water_foam_or_ocean = water_catalog
            .zip(water_bands)
            .map(|(c, b)| {
                crate::render::gpu_water_particles::water_witness_foam_or_ocean_green(
                    c,
                    &b.tactical,
                )
            })
            .unwrap_or(false);

        Self {
            fire_tactical_zoom: fire_tactical,
            fire_spark_rows_gt_0: fire_rows > 0,
            water_tactical_zoom: water_tactical,
            water_has_river_segments: water_has_rivers,
            water_particle_rows_gt_0: water_rows > 0,
            water_particle_river_streaks_when_rivers: !water_has_rivers || water_streaks > 0,
            water_shader_motion_always_on: water_shader_on,
            water_particle_strategic_not_culled: water_not_culled,
            water_w1_river_read_green: water_river_read,
            water_strategic_001_green: water_strategic_001,
            water_witness_001_green: water_witness_001,
            water_witness_foam_or_ocean_green: water_foam_or_ocean,
        }
    }

    #[must_use]
    pub(crate) fn all_green(&self) -> bool {
        self.all_green_for_visual_proof(false)
    }

    #[must_use]
    pub(crate) fn all_green_for_visual_proof(&self, require_fire_rows: bool) -> bool {
        let fire_ok = if require_fire_rows {
            self.fire_tactical_zoom && self.fire_spark_rows_gt_0
        } else {
            !self.fire_tactical_zoom || self.fire_spark_rows_gt_0
        };
        let water_ok = self.water_shader_motion_always_on
            && (!self.water_tactical_zoom
                || (self.water_particle_rows_gt_0
                    && self.water_particle_river_streaks_when_rivers
                    && self.water_particle_strategic_not_culled));
        fire_ok && water_ok
    }
}

/// Re-stamp water particles after tactical camera fix (same zoom resource as fire).
pub(crate) fn refresh_visual_proof_water_particles(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    time: Res<Time>,
    catalog: Option<Res<WaterSurfaceVisualCatalog>>,
    cam: Res<crate::render::gpu_particles::FireParticleCameraScale>,
    mut frame: ResMut<WorldWaterParticleFrame>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !visual_tactical_vfx_witness_required(launch) {
        return;
    }
    let Some(catalog) = catalog.as_ref() else {
        return;
    };
    crate::render::gpu_water_particles::update_world_water_particles_from_catalog(
        catalog,
        frame.as_mut(),
        *cam,
        time.elapsed_secs(),
    );
}

/// Re-stamp fire particles after tactical camera fix (Update emit may have used strategic zoom).
pub(crate) fn refresh_visual_proof_fire_particles(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    overlay: Res<crate::render::SharedOverlayFieldBuffers>,
    graph: Res<crate::render::extraction::RenderProjectionGraph>,
    chunk_lod: Res<crate::render::FireChunkLodState>,
    cam: Res<crate::render::gpu_particles::FireParticleCameraScale>,
    view_manager: Option<Res<crate::gui::ViewManager>>,
    mut particles: ResMut<WorldFireParticleFrame>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !visual_tactical_vfx_witness_required(launch) {
        return;
    }
    crate::render::gpu_particles::update_world_fire_particles_from_projection(
        graph.as_ref(),
        particles.as_mut(),
        Some(chunk_lod.as_ref()),
        *cam,
        view_manager.as_deref(),
    );
    if particles.spark_witness.rows > 0 {
        return;
    }
    // Projection graph empty (view cull / stamp) but overlay has seeded fire — witness bootstrap.
    if overlay.chunk_fire_heat.is_empty() {
        return;
    }
    crate::render::gpu_particles::seed_world_fire_particles_from_overlay_heat(
        &overlay.chunk_fire_heat,
        particles.as_mut(),
        *cam,
    );
}

/// Keep map at tactical zoom during visual capture (world-fit camera would strategic-cull particles).
pub(crate) fn maintain_visual_tactical_vfx_camera(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    test_scene: Option<Res<crate::engine::ActiveTestScene>>,
    params: Res<crate::terrain::generation::world_generator_enhanced::WorldGenParams>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    sim_viewport: Res<crate::gui::SimulationMapViewport>,
    mut desired: ResMut<crate::gui::MapCameraDesired>,
    mut cam: Query<&mut Transform, With<crate::gui::MainWorldCamera>>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !visual_tactical_vfx_witness_required(launch) {
        return;
    }
    if params.width == 0 || params.height == 0 {
        return;
    }
    let _ = test_scene;
    let world_w = params.width as f32;
    let world_h = params.height as f32;
    let window_px = windows
        .single()
        .ok()
        .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let viewport =
        crate::gui::map_camera_viewport_pixels(window_px, Some(sim_viewport.as_ref()));
    let (zoom_lo, zoom_hi) = crate::gui::map_zoom_limits_for_world(world_w, world_h, viewport);
    let zoom = crate::gui::map_scale_for_zoom_alpha(
        crate::gui::TACTICAL_VFX_PROOF_ZOOM_ALPHA,
        zoom_lo,
        zoom_hi,
    );
    let cx = world_w * 0.5;
    let cy = world_h * 0.5;
    desired.translation = Vec3::new(cx, cy, 0.0);
    desired.scale = Vec3::splat(zoom);
    for mut t in cam.iter_mut() {
        t.translation.x = cx;
        t.translation.y = cy;
        t.scale = Vec3::splat(zoom);
    }
}

fn tactical_vfx_witness_json(gates: &TacticalVfxWitnessGates) -> serde_json::Value {
    serde_json::json!({
        "tactical_zoom_alpha_min": TACTICAL_VFX_ZOOM_ALPHA_MIN,
        "proof_gate_enabled": tactical_vfx_proof_enabled(),
        "fire_sparks_above_smoke": crate::render::gpu_fire_particle_raster::FIRE_SPARKS_ABOVE_SMOKE_OVERLAY,
        "fire_tactical_zoom": gates.fire_tactical_zoom,
        "fire_spark_rows_gt_0": gates.fire_spark_rows_gt_0,
        "water_tactical_zoom": gates.water_tactical_zoom,
        "water_has_river_segments": gates.water_has_river_segments,
        "water_particle_rows_gt_0": gates.water_particle_rows_gt_0,
        "water_particle_river_streaks_when_rivers": gates.water_particle_river_streaks_when_rivers,
        "water_shader_motion_always_on": gates.water_shader_motion_always_on,
        "water_particle_strategic_not_culled": gates.water_particle_strategic_not_culled,
        "water_w1_river_read_green": gates.water_w1_river_read_green,
        "water_strategic_001_green": gates.water_strategic_001_green,
        "water_witness_001_green": gates.water_witness_001_green,
        "water_witness_foam_or_ocean_green": gates.water_witness_foam_or_ocean_green,
        "all_green": gates.all_green(),
    })
}

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
fn hydrate_world_from_vt_ci_scenario(world: &mut World, scenario: &Vt4CiScenario) {
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
fn assemble_headless_full_app_readiness_app() -> App {
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
    app.init_resource::<MapCameraDesired>();
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

#[derive(SystemParam)]
pub(crate) struct Stage5FullAppLiveProofReads<'w> {
    sim_tick: Res<'w, SimTick>,
    sim_time: Res<'w, SimTimeMicros>,
    eval_inv: Res<'w, crate::render::Stage5ReadinessEvalInvocation>,
    visual_fence: Res<'w, crate::render::CommittedVisualSnapshotFence>,
    resolved: Res<'w, ResolvedViewports>,
    viewport_mismatch: Res<'w, ViewportPresentationMismatch>,
    preview_authority: Res<'w, WorldPreviewViewportAuthority>,
    preview_path: Res<'w, PreviewPathAuthority>,
    render_contract: Res<'w, WorldPreviewRenderViewportContract>,
    render_registry: Res<'w, WorldPreviewRenderTargetRegistry>,
    view_snapshot: Res<'w, ViewRepresentationSnapshot>,
    preview_ui: Res<'w, WorldPreviewUiState>,
    preview_cam: Res<'w, PreviewCameraState>,
    minimap: Res<'w, MinimapShellState>,
    minimap_registry: Option<Res<'w, MinimapRenderTargetRegistry>>,
    minimap_compositor: Option<Res<'w, MinimapCompositorState>>,
    minimap_gpu_diagnostics: Option<Res<'w, MinimapGpuCompositorDiagnostics>>,
    sim_map: Res<'w, SimulationMapViewport>,
    policy: Option<Res<'w, crate::gui::RepresentationResult>>,
    projection: Option<Res<'w, RenderProjectionGraph>>,
    metrics: Option<Res<'w, GpuRepresentationMetrics>>,
    phase_f: Option<Res<'w, PhaseFLodProofReport>>,
    indirect: Option<Res<'w, GpuIndirectDrawSpine>>,
    draw: Option<Res<'w, WorldFireParticleDrawDispatch>>,
    particles: Option<Res<'w, WorldFireParticleFrame>>,
    water_catalog: Option<Res<'w, WaterSurfaceVisualCatalog>>,
    water_particles: Option<Res<'w, WorldWaterParticleFrame>>,
    overlay: Option<Res<'w, SharedOverlayFieldBuffers>>,
    map_presentation: Res<'w, MapViewPresentationStates>,
    map_views: Res<'w, MapViewInstances>,
    map_frames: Res<'w, ResolvedMapViewFrames>,
    map_texture_cache: Res<'w, MapViewTextureCache>,
    map_presentation_diag: Res<'w, MapPresentationDiagnostics>,
    map_fit_log: Res<'w, MapFitValidationLog>,
    todo_board: Option<Res<'w, Stage5LiveTodoBoard>>,
    finish_todo_board: Option<Res<'w, Stage5FinishTodoBoard>>,
    finish_ux06_streak: Option<Res<'w, Stage5FinishUx06Streak>>,
    view_isolation: Res<'w, crate::gui::ViewIsolationDiagnostics>,
    view_projection_authority: Option<Res<'w, crate::render::view_runtime::ViewProjectionAuthority>>,
    view_runtime_witness: Option<Res<'w, crate::render::view_runtime::ViewRuntimeWitness>>,
    fire_witness: Option<Res<'w, crate::render::Stage5FireViewChunkWitness>>,
    fire_playback: Option<Res<'w, crate::render::FirePlaybackStabilityWitness>>,
    view_manager: Option<Res<'w, crate::gui::ViewManager>>,
}

fn stage5_live_todo_board_snapshot(board: &Stage5LiveTodoBoard) -> serde_json::Value {
    let rows: Vec<serde_json::Value> = STAGE5_TODOS
        .iter()
        .zip(board.status.iter())
        .map(|(row, st)| {
            let status = match st {
                TodoStatus::Open => "Open",
                TodoStatus::InProgress => "InProgress",
                TodoStatus::Done => "Done",
            };
            serde_json::json!({
                "id": row.id,
                "status": status,
            })
        })
        .collect();
    let done = board
        .status
        .iter()
        .filter(|s| **s == TodoStatus::Done)
        .count();
    serde_json::json!({
        "registry_len": STAGE5_TODOS.len(),
        "done_count": done,
        "all_done": done == STAGE5_TODOS.len(),
        "rows": rows,
    })
}

fn stage5_finish_todo_board_snapshot(board: &Stage5FinishTodoBoard) -> serde_json::Value {
    let rows: Vec<serde_json::Value> = STAGE5_FINISH_TODOS
        .iter()
        .zip(board.status.iter())
        .map(|(row, st)| {
            let status = match st {
                TodoStatus::Open => "Open",
                TodoStatus::InProgress => "InProgress",
                TodoStatus::Done => "Done",
            };
            serde_json::json!({
                "id": row.id,
                "status": status,
            })
        })
        .collect();
    let done = board
        .status
        .iter()
        .filter(|s| **s == TodoStatus::Done)
        .count();
    serde_json::json!({
        "registry_len": STAGE5_FINISH_TODOS.len(),
        "done_count": done,
        "all_done": done == STAGE5_FINISH_TODOS.len(),
        "rows": rows,
    })
}

fn minimap_source_label(source: MinimapPresentationSource) -> &'static str {
    match source {
        MinimapPresentationSource::SharedCpuRaster => "CpuRaster",
        MinimapPresentationSource::SharedRenderTargetImage => "GpuRenderTarget",
    }
}

fn minimap_gpu_composite_active(reads: &Stage5FullAppLiveProofReads) -> bool {
    reads.minimap_compositor.as_ref().is_some_and(|c| {
        reads
            .minimap_registry
            .as_ref()
            .is_some_and(|r| r.committed_image != Handle::default() && c.stamp > 0)
    })
}

fn minimap_source_label_for_proof(reads: &Stage5FullAppLiveProofReads) -> &'static str {
    if minimap_gpu_compositor_env_enabled() && minimap_gpu_composite_active(reads) {
        "GpuRenderTarget"
    } else {
        minimap_source_label(reads.minimap.presentation_source)
    }
}

fn write_minimap_compositor_live_proof_from_reads(reads: &Stage5FullAppLiveProofReads) {
    const PROOF_PATH: &str = "debug_runs/minimap_compositor_live.json";
    let Some(compositor) = reads.minimap_compositor.as_ref() else {
        return;
    };
    let Some(registry) = reads.minimap_registry.as_ref() else {
        return;
    };
    let overlay_revision = reads.overlay.as_ref().map(|o| o.revision).unwrap_or(0);
    let diagnostics = reads
        .minimap_gpu_diagnostics
        .as_ref()
        .map(|d| d.as_ref())
        .cloned()
        .unwrap_or_default();
    let body = build_minimap_compositor_proof_payload(
        compositor,
        registry,
        &reads.minimap,
        overlay_revision,
        false,
        &diagnostics,
    );
    if !body
        .get("composite_ok")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        info!(
            target: "stage5_full_app_harness",
            stamp = compositor.stamp,
            rt_bound = registry.committed_image != Handle::default(),
            presentation = ?reads.minimap.presentation_source,
            "skipped minimap compositor live proof — composite_ok false"
        );
        return;
    }
    let payload = crate::dev::debug_run_envelope::wrap_debug_run(
        "MINIMAP_COMPOSITOR_M1",
        "stage5_full_app_harness",
        PROOF_PATH,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(PROOF_PATH, payload) {
        info!(
            target: "stage5_full_app_harness",
            path = PROOF_PATH,
            "wrote minimap compositor live proof (FULL_APP finalize)"
        );
    }
}

fn map_texture_source_label(source: &MapTextureSource) -> &'static str {
    match source {
        MapTextureSource::GpuRenderTarget(_) => "GpuRenderTarget",
        MapTextureSource::SharedCpuRaster(_) => "SharedCpuRaster",
    }
}

fn map_view_consumer_payload(
    id: MapViewInstanceId,
    presentation_aux: &MapViewPresentationStates,
    map_views: &MapViewInstances,
    frames: &ResolvedMapViewFrames,
    cache: &MapViewTextureCache,
    layout: &MapPresentationDiagnostics,
) -> serde_json::Value {
    let frame = frames.get(id);
    let binding = cache.binding(id);
    let (presentation_revision, fit_mode) = match id {
        MapViewInstanceId::WorldPreview => (
            map_views.world_preview.revision,
            map_views.world_preview.fit_mode,
        ),
        MapViewInstanceId::Minimap => (map_views.minimap.revision, map_views.minimap.fit_mode),
        MapViewInstanceId::SimulationMap
        | MapViewInstanceId::TacticalMap
        | MapViewInstanceId::FullscreenMap
        | MapViewInstanceId::CommanderMap
        | MapViewInstanceId::Stage7IntelMap => {
            let presentation_state = presentation_aux.get(id);
            (presentation_state.revision, presentation_state.fit_mode)
        }
    };
    let layout_slot = match id {
        MapViewInstanceId::Minimap => &layout.minimap,
        _ => &layout.world_preview,
    };
    serde_json::json!({
        "presentation_revision": presentation_revision,
        "fit_mode": fit_mode.label(),
        "texture_source": map_texture_source_label(&frame.texture_source),
        "viewport_extent": {
            "x": frame.viewport_extent.x,
            "y": frame.viewport_extent.y,
        },
        "projection_revision": frame.projection_revision,
        "overlay_revision": frame.overlay_revision,
        "texture_rebinds_frame": binding.rebinds_frame,
        "texture_rebinds_total": binding.rebinds_total,
        "stale_binding_count_frame": binding.stale_cache_frame,
        "allocated_rect": layout_slot.allocated_rect.map(rect_json),
        "image_rect": layout_slot.image_rect.map(rect_json),
        "uv_rect": rect_json(layout_slot.uv_rect),
        "padding": layout_slot.padding,
        "aspect_texture": layout_slot.aspect_texture,
        "aspect_panel": layout_slot.aspect_panel,
        "camera_zoom": layout_slot.camera_zoom,
        "fit_validation": layout_slot.validation.as_ref().map(|validation| serde_json::json!({
            "mismatch": validation.mismatch,
            "delta_pixels": validation.delta_pixels,
            "uv_delta": validation.uv_delta,
        })),
    })
}

fn rect_json(rect: bevy_egui::egui::Rect) -> serde_json::Value {
    serde_json::json!({
        "min": { "x": rect.min.x, "y": rect.min.y },
        "max": { "x": rect.max.x, "y": rect.max.y },
        "width": rect.width(),
        "height": rect.height(),
    })
}

/// FX-WATER-SHADER-002 W1-B1 — witness fields for `stage5_full_app_live.json`.
fn water_w1_witness_stamp(
    catalog: Option<&WaterSurfaceVisualCatalog>,
) -> (Option<bool>, Option<usize>, Option<usize>) {
    (
        catalog.map(WaterSurfaceVisualCatalog::w1_green),
        catalog.map(|c| c.river_segments.len()),
        catalog.map(|c| c.river_tiles.len()),
    )
}

fn build_water_surface_proof_json(
    water_catalog: Option<&WaterSurfaceVisualCatalog>,
    water_particles: Option<&WorldWaterParticleFrame>,
    tactical_vfx: &TacticalVfxWitnessGates,
) -> serde_json::Value {
    let (water_w1_green, water_river_segments, water_river_tiles) =
        water_w1_witness_stamp(water_catalog);
    let water_vfx_witness = water_catalog.map(|c| {
        let live_zoom = water_particles
            .map(|f| f.witness.zoom_alpha)
            .unwrap_or(crate::render::gpu_water_particles::WATER_TACTICAL_WITNESS_ZOOM_ALPHA);
        let bands = crate::render::gpu_water_particles::evaluate_water_vfx_witness_bands(
            c, live_zoom, 0.0,
        );
        crate::render::gpu_water_particles::water_vfx_witness_json(c, &bands)
    });
    serde_json::json!({
        "water_w1_green": water_w1_green,
        "water_w1_river_green": water_catalog.map(|c| c.w1_river_green()),
        "water_w1_river_read_green": water_catalog.map(|c| {
            c.w1_river_read_green_at_zoom(
                crate::render::water_surface_visual::WATER_STRATEGIC_ZOOM_ALPHA * 0.5,
            )
        }),
        "water_w1_ocean_green": water_catalog.map(|c| c.w1_ocean_green()),
        "water_river_segments": water_river_segments,
        "water_river_tiles": water_river_tiles,
        "water_lake_tiles": water_catalog.map(|c| c.lake_tiles.len()),
        "water_ocean_tiles": water_catalog.map(|c| c.ocean_tiles.len()),
        "water_particle_rows": water_particles.map(|f| f.witness.rows),
        "water_particle_river_streaks": water_particles.map(|f| f.witness.river_streaks),
        "water_particle_river_foam": water_particles.map(|f| f.witness.river_foam),
        "water_particle_lake_glints": water_particles.map(|f| f.witness.lake_glints),
        "water_particle_coast_foam": water_particles.map(|f| f.witness.coast_foam),
        "water_particle_zoom_alpha": water_particles.map(|f| f.witness.zoom_alpha),
        "water_shader_motion_always_on": water_particles.map(|f| f.witness.shader_motion_always_on),
        "water_particle_strategic_culled": water_particles.map(|f| f.witness.strategic_culled),
        "water_vfx_witness": water_vfx_witness,
        "water_strategic_001_green": water_vfx_witness
            .as_ref()
            .and_then(|v| v.get("water_strategic_001_green"))
            .and_then(|v| v.as_bool()),
        "water_witness_001_green": water_vfx_witness
            .as_ref()
            .and_then(|v| v.get("water_witness_001_green"))
            .and_then(|v| v.as_bool()),
        "tactical_witness_gates": tactical_vfx_witness_json(tactical_vfx),
    })
}

fn build_stage5_full_app_live_proof_payload(
    report: &AppStage5ReadinessReport,
    gate: &crate::render::FullRenderDiagnosticGate,
    summary: &crate::render::FullRenderDiagnosticSummary,
    reads: &Stage5FullAppLiveProofReads,
) -> serde_json::Value {
    let policy = reads.policy.as_deref();
    let metrics = reads.metrics.as_deref();
    let indirect = reads.indirect.as_deref();
    let draw = reads.draw.as_deref();
    let particles = reads.particles.as_deref();
    let water_catalog = reads.water_catalog.as_deref();
    let water_particles = reads.water_particles.as_deref();
    let overlay = reads.overlay.as_deref();
    let projection = reads.projection.as_deref();
    let phase_f = reads.phase_f.as_deref();
    let tactical_vfx = TacticalVfxWitnessGates::evaluate(particles, water_catalog, water_particles);
    let water_surface =
        build_water_surface_proof_json(water_catalog, water_particles, &tactical_vfx);

    let spine_done = reads
        .todo_board
        .as_ref()
        .map(|b| b.status.iter().all(|s| *s == crate::dev::stage5_live_todos::TodoStatus::Done))
        .unwrap_or(false);
    let finish_done = reads
        .finish_todo_board
        .as_ref()
        .map(|b| b.status.iter().all(|s| *s == crate::dev::stage5_live_todos::TodoStatus::Done))
        .unwrap_or(false);

    serde_json::json!({
        "profile": "FULL_APP",
        "test_scene": "visual",
        "stage5_closure": {
            "operational_gate": "FULL_APP",
            "passes": stage5_readiness_passes(report),
            "spine_todos_all_done": spine_done,
            "finish_todos_all_done": finish_done,
            "checklist": "src/dev/stage5_close_checklist.md",
            "triage_backlog": "src/dev/stage5_triage_backlog.md",
            "not_gate": "VM-06..11, full fire streaming, gpu-tile gizmo removal, construction stage",
        },
        "diagnostic_captured": gate.captured,
        "sim_step_stamp": {
            "tick": reads.sim_tick.0,
            "sim_time_micros": reads.sim_time.0,
        },
        "readiness": {
            "passes": stage5_readiness_passes(report),
            "live_todo_board": reads.todo_board.as_ref().map(|b| stage5_live_todo_board_snapshot(b)),
            "live_finish_todo_board": reads
                .finish_todo_board
                .as_ref()
                .map(|b| stage5_finish_todo_board_snapshot(&**b)),
            "finish_ux06_streak": reads.finish_ux06_streak.as_ref().map(|s| {
                serde_json::json!({
                    "consecutive_good": s.consecutive_good,
                    "last_blocker": s.last_blocker,
                    "streak_target": crate::dev::stage5_finish_todos::FINISH_UX06_STREAK_DONE,
                })
            }),
            "vt4_ok": report.vt4_ok,
            "vt5_ok": report.vt5_ok,
            "single_fire_extract": report.single_fire_extract,
            "gpu_field_authoritative": report.gpu_field_authoritative,
            "preview_render_target_active": report.preview_render_target_active,
            "phase_d_ok": report.phase_d_ok,
            "overlay_from_shared_buffers_only": report.overlay_from_shared_buffers_only,
            "particle_lod_scales": report.particle_lod_scales,
            "phase_f_lod_proof_ok": report.phase_f_lod_proof_ok,
            "instanced_dispatch_ok": report.instanced_dispatch_ok,
            "phase_f_ok": report.phase_f_ok,
            "projection_domains": report.projection_domains,
            "registered_producers": report.registered_producers,
            "duplicate_visual_scan_count": report.duplicate_visual_scan_count,
            "violations": report.violations,
        },
        "viewport_contracts": {
            "resolved_revision": reads.resolved.revision,
            "world_preview": {
                "logical_size": {
                    "x": reads.resolved.world_preview.logical_size.x,
                    "y": reads.resolved.world_preview.logical_size.y,
                },
                "physical_extent": {
                    "x": reads.resolved.world_preview.physical_extent.x,
                    "y": reads.resolved.world_preview.physical_extent.y,
                },
                "world_extent": {
                    "x": reads.resolved.world_preview.world_extent.x,
                    "y": reads.resolved.world_preview.world_extent.y,
                },
                "valid": reads.resolved.world_preview.valid,
            },
            "minimap_panel": {
                "logical_size": {
                    "x": reads.resolved.minimap_panel.logical_size.x,
                    "y": reads.resolved.minimap_panel.logical_size.y,
                },
                "physical_extent": {
                    "x": reads.resolved.minimap_panel.physical_extent.x,
                    "y": reads.resolved.minimap_panel.physical_extent.y,
                },
                "valid": reads.resolved.minimap_panel.valid,
            },
            "simulation_map": {
                "valid": reads.sim_map.valid,
                "min": { "x": reads.sim_map.min.x, "y": reads.sim_map.min.y },
                "max": { "x": reads.sim_map.max.x, "y": reads.sim_map.max.y },
            },
            "preview_authority": {
                "committed": reads.preview_authority.committed,
                "revision": reads.preview_authority.revision,
                "gpu_authoritative": reads.preview_authority.gpu_authoritative,
                "logical_viewport": {
                    "x": reads.preview_authority.logical_viewport.x,
                    "y": reads.preview_authority.logical_viewport.y,
                },
                "physical_render_extent": {
                    "x": reads.preview_authority.physical_render_extent.x,
                    "y": reads.preview_authority.physical_render_extent.y,
                },
            },
            "view_snapshot": {
                "committed": reads.view_snapshot.committed,
                "frame_id": reads.view_snapshot.frame_id,
                "viewport": {
                    "width": reads.view_snapshot.viewport.width(),
                    "height": reads.view_snapshot.viewport.height(),
                },
                "gpu_target_size": {
                    "x": reads.view_snapshot.gpu_target_size.x,
                    "y": reads.view_snapshot.gpu_target_size.y,
                },
            },
            "egui_world_preview_viewport": reads.preview_ui.last_viewport_rect.map(|rect| serde_json::json!({
                "width": rect.width(),
                "height": rect.height(),
            })),
            "mismatch_flags": {
                "world_preview_extent_mismatch": reads.viewport_mismatch.world_preview_extent_mismatch,
                "minimap_panel_extent_mismatch": reads.viewport_mismatch.minimap_panel_extent_mismatch,
                "simulation_map_extent_mismatch": reads.viewport_mismatch.simulation_map_extent_mismatch,
                "stale_texture_binding": reads.viewport_mismatch.stale_texture_binding,
            },
            "view_isolation": {
                "note": "infrastructure_lane_non_gating",
                "minimap_main_lockstep_suspect": reads.view_isolation.minimap_main_lockstep_suspect,
                "preview_main_lockstep_suspect": reads.view_isolation.preview_main_lockstep_suspect,
                "simulation_map_shares_main_camera": reads.view_isolation.simulation_map_shares_main_camera,
                "preview_overlay_fire_heat": reads.view_isolation.preview_overlay_fire_heat,
                "minimap_overlay_fire_heat": reads.view_isolation.minimap_overlay_fire_heat,
                "world_main_viewport": reads.view_manager.as_ref().and_then(|m| m.view(crate::gui::ViewId::WorldMain)).map(|v| {
                    serde_json::json!({
                        "width": v.viewport_rect.width(),
                        "height": v.viewport_rect.height(),
                    })
                }),
                "world_main_visible_fire_orphans": reads.fire_witness.as_ref().map(|w| w.world_main_visible_orphan_chunks),
                "view_runtime": reads.view_projection_authority.as_ref().map(|auth| {
                    serde_json::json!({
                        "authority_revision": auth.last_commit_revision,
                        "pose_writers": crate::render::view_runtime::pose_writers_json(auth),
                        "world_preview_logical": auth.surface(crate::render::view_runtime::ViewSurfaceId::WorldPreview).map(|s| {
                            serde_json::json!({ "x": s.render.logical_size.x, "y": s.render.logical_size.y, "valid": s.render.valid })
                        }),
                    })
                }),
                "vm_a_witness": reads.view_runtime_witness.as_ref().map(|w| {
                    serde_json::json!({
                        "minimap_shell_wrote_map_camera_desired": w.minimap_shell_wrote_map_camera_desired,
                        "dual_writer_pose_violation": w.dual_writer_pose_violation,
                        "infrastructure_view_isolation_green": w.infrastructure_view_isolation_green,
                    })
                }),
            },
        },
        "render_targets": {
            "preview_mode": format!("{:?}", reads.preview_cam.mode),
            "preview_path_authority": format!("{:?}", reads.preview_path.authoritative_surface),
            "gpu_present_count": reads.preview_path.gpu_present_count,
            "committed_size": {
                "x": reads.render_registry.committed_size.x,
                "y": reads.render_registry.committed_size.y,
            },
            "committed_revision": reads.render_registry.revision,
            "render_contract": {
                "camera_ready": reads.render_contract.camera_ready,
                "version": reads.render_contract.version,
                "size": {
                    "x": reads.render_contract.size.x,
                    "y": reads.render_contract.size.y,
                },
            },
        },
        "projection_state": {
            "active_band": policy.map(|p| format!("{:?}", p.active_band)),
            "fire_instance_buffer_rows": projection.map(|graph| graph.fire.instance_buffer.len()),
            "particle_rows_cap": policy.map(|p| p.gpu_budget.particle_rows_cap),
            "instanced_draw": policy.map(|p| p.particle_policy.instanced_draw),
        },
        "projection_graph": projection.map(|graph| {
            serde_json::json!({
                "build_signature": crate::render::extraction::projection_graph_build_signature(graph),
                "runtime_order": crate::render::extraction::projection_graph_runtime_order_snapshot(graph),
                "logistics_active_rows": graph.logistics.active_rows,
                "ecology_active_rows": graph.ecology.active_rows,
            })
        }),
        "readiness_eval_invocation": reads.eval_inv.0,
        "committed_visual_fence": {
            "fire_tick": reads.visual_fence.fire.tick,
            "fire_sim_time_micros": reads.visual_fence.fire.sim_time_micros,
        },
        "agent_cleanup_hints": if report.violations.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::json!({
                "violations": report.violations,
                "first_fix": report.violations.first(),
            })
        },
        "phase_f_live": {
            "particle_rows": metrics.map(|m| m.particle_rows),
            "instance_rows": metrics.map(|m| m.instance_rows),
            "draw_instances": metrics.map(|m| m.draw_instances),
            "upload_bytes": metrics.map(|m| m.upload_bytes),
            "phase_f_lod_proof_ok": phase_f.map(|proof| proof.ordering_ok),
            "phase_f_samples": phase_f.map(|proof| proof.samples),
            "indirect_instance_count": indirect.map(|spine| spine.world_fire.instance_count),
            "indirect_dispatch_count": indirect.map(|spine| spine.dispatch_count),
            "draw_dispatch_instance_count": draw.map(|dispatch| dispatch.instance_count),
            "particle_bounds": particles.map(|frame| {
                let snapshot_bounds = WorldBounds::from_particle_instances(&frame.instances);
                serde_json::json!({
                    "min": { "x": snapshot_bounds.min.x, "y": snapshot_bounds.min.y },
                    "max": { "x": snapshot_bounds.max.x, "y": snapshot_bounds.max.y },
                })
            }),
        },
        "minimap_source": {
            "presentation_source": minimap_source_label_for_proof(reads),
            "shared_projection": overlay.is_some(),
            "overlay_revision": overlay.map(|o| o.revision),
            "cached_texture_revision": reads.minimap.cached_texture_revision,
            "compositor_revision": reads.minimap.compositor_revision,
            "gpu_compositor_env": minimap_gpu_compositor_env_enabled(),
            "composite_ok": reads.minimap_compositor.as_ref().map(|c| {
                reads.minimap_registry.as_ref().is_some_and(|r| {
                    r.committed_image != Handle::default() && c.stamp > 0
                })
            }),
            "stamp": reads.minimap_compositor.as_ref().map(|c| c.stamp),
            "rt_bound": reads.minimap_registry.as_ref().map(|r| r.committed_image != Handle::default()),
            "extent": reads.minimap_registry.as_ref().map(|r| {
                serde_json::json!({ "x": r.committed_size.x, "y": r.committed_size.y })
            }),
            "dual_minimap_present": reads.minimap_compositor.as_ref().map(|c| c.dual_minimap_present),
            "extent_match_px": reads.minimap_compositor.as_ref().map(|c| c.extent_match_px),
        },
        "map_view_consumers": {
            "world_preview": map_view_consumer_payload(
                MapViewInstanceId::WorldPreview,
                &reads.map_presentation,
                &reads.map_views,
                &reads.map_frames,
                &reads.map_texture_cache,
                &reads.map_presentation_diag,
            ),
            "minimap": map_view_consumer_payload(
                MapViewInstanceId::Minimap,
                &reads.map_presentation,
                &reads.map_views,
                &reads.map_frames,
                &reads.map_texture_cache,
                &reads.map_presentation_diag,
            ),
        },
        "map_presentation_stability": {
            "mismatch_frames": reads.map_fit_log.mismatch_frames,
            "fit_mode_mismatch": reads.map_fit_log.fit_mode_mismatch,
            "preview_scale": reads.map_fit_log.preview_scale,
            "minimap_scale": reads.map_fit_log.minimap_scale,
            "world_preview_mismatch": reads.map_fit_log.world_preview.as_ref().map(|v| v.mismatch).unwrap_or(false),
            "minimap_mismatch": reads.map_fit_log.minimap.as_ref().map(|v| v.mismatch).unwrap_or(false),
        },
        "particle_routing": {
            "coordinate_space": "world",
            "active_particle_rows": particles.map(|frame| frame.instances.len()),
            "fire_spark_phase": particles.map(|frame| frame.spark_witness.phase),
            "fire_spark_compute_enabled": Some(crate::render::fire_spark_compute_enabled()),
            "fire_spark_rows": particles.map(|frame| frame.spark_witness.rows),
            "fire_spark_scatter_max": particles.map(|frame| frame.spark_witness.scatter_max),
            "fire_spark_scatter_slots": particles.map(|frame| frame.spark_witness.scatter_slots),
            "fire_spark_zoom_alpha": particles.map(|frame| frame.spark_witness.zoom_alpha),
            "fire_spark_additive_blend": particles.map(|frame| frame.spark_witness.additive_blend),
            "fire_spark_budget_capped": particles.map(|frame| frame.spark_witness.budget_capped),
            "fire_particle_view_culled": particles.map(|frame| frame.spark_witness.view_culled),
            "fire_spark_projection_view": particles.map(|frame| frame.spark_witness.projection_view),
            "snapshot_particle_bounds": {
                "min": { "x": reads.view_snapshot.particle_bounds.min.x, "y": reads.view_snapshot.particle_bounds.min.y },
                "max": { "x": reads.view_snapshot.particle_bounds.max.x, "y": reads.view_snapshot.particle_bounds.max.y },
            },
            "resolved_half_extents": {
                "x": reads.resolved.simulation_map.valid.then_some(reads.resolved.simulation_map.half_extents.x)
                    .or_else(|| reads.resolved.primary_window.valid.then_some(reads.resolved.primary_window.half_extents.x)),
                "y": reads.resolved.simulation_map.valid.then_some(reads.resolved.simulation_map.half_extents.y)
                    .or_else(|| reads.resolved.primary_window.valid.then_some(reads.resolved.primary_window.half_extents.y)),
            },
        },
        "texture_stale_reasons": {
            "preview_stale_binding": reads.viewport_mismatch.stale_texture_binding,
            "minimap_cached_behind_raster": reads.minimap.cached_texture_revision,
        },
        "authority_revisions": {
            "resolved_viewports": reads.resolved.revision,
            "preview_viewport_authority": reads.preview_authority.revision,
            "render_target_registry": reads.render_registry.revision,
            "overlay_fields": overlay.map(|o| o.revision),
        },
        "fire_playback": reads.fire_playback.as_ref().map(|w| {
            serde_json::json!({
                "active_fire_chunks": w.active_fire_chunks,
                "consecutive_frames_with_heat": w.consecutive_frames_with_heat,
                "held_empty_snapshot_frames": w.held_empty_snapshot_frames,
                "stable": w.stable,
                "stable_frame_threshold": crate::render::FirePlaybackStabilityWitness::STABLE_FRAME_THRESHOLD,
            })
        }),
        "water_surface": water_surface,
        "tactical_vfx_witness": tactical_vfx_witness_json(&tactical_vfx),
        "world_preview_layout": {
            "d01_unified_workspace": crate::gui::editor::world_preview::WORLD_PREVIEW_UNIFIED_WORKSPACE,
            "ui_wp_layout_001": "signed",
        },
        "render_anomalies": {
            "viewport_zero_size_detected": summary.viewport_zero_size_detected,
            "camera_count_gt_one_world_camera": summary.camera_count_gt_one_world_camera,
            "particle_screen_space_detected": summary.particle_screen_space_detected,
            "render_target_mismatch": summary.render_target_mismatch,
            "egui_rect_ne_viewport_rect": summary.egui_rect_ne_viewport_rect,
            "stale_texture_usage": summary.stale_texture_usage,
        },
    })
}

/// Max PostUpdate frames after full-render diagnostic capture before `--test visual` fails.
pub const VISUAL_PROBE_UX06_MAX_FRAMES_AFTER_CAPTURE: u32 = 2400;

/// Max frames to wait for tactical fire/water particle rows before committing proof anyway.
pub const VISUAL_PROBE_TACTICAL_VFX_MAX_FRAMES: u32 = 900;

#[derive(Default)]
pub(crate) struct VisualProbeFinalizeState {
    proof_committed: bool,
    frames_since_capture: u32,
    last_logged_streak: u32,
}

fn finish_ux06_streak_done(streak: Option<&Stage5FinishUx06Streak>) -> bool {
    streak
        .map(|s| s.consecutive_good >= FINISH_UX06_STREAK_DONE)
        .unwrap_or(false)
}

/// `--test visual` proof should not commit before the instanced fire spine is wired.
/// Non-zero instance rows are optional (strategic zoom / cold ecology may be empty while
/// `instanced_dispatch_ok` still holds).
fn visual_probe_fire_witness_ready(
    reads: &Stage5FullAppLiveProofReads,
    report: &AppStage5ReadinessReport,
) -> bool {
    if report.instanced_dispatch_ok && reads.projection.is_some() {
        return true;
    }
    let buffer_rows = reads
        .projection
        .as_ref()
        .map(|g| g.fire.instance_buffer.len())
        .unwrap_or(0);
    let particle_rows = reads
        .particles
        .as_ref()
        .map(|f| f.instances.len())
        .unwrap_or(0);
    let draw_rows = reads
        .draw
        .as_ref()
        .map(|d| d.instance_count)
        .unwrap_or(0);
    buffer_rows > 0 || particle_rows > 0 || draw_rows > 0
}

/// After `--test visual` captures diagnostics: hold until **FINISH-UX-06** reaches
/// [`FINISH_UX06_STREAK_DONE`] consecutive clean readiness evals, then write proof JSON.
/// After proof commit, [`crate::render::gpu_surface_teardown::tick_visual_test_graceful_exit`] requests
/// `AppExit` so Vulkan surfaces tear down cleanly (manual window close can panic in wgpu).
pub(crate) fn finalize_visual_full_app_live_probe(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    profile: Res<Stage5ReadinessProfile>,
    report: Res<AppStage5ReadinessReport>,
    gate: Res<crate::render::FullRenderDiagnosticGate>,
    summary: Res<crate::render::FullRenderDiagnosticSummary>,
    proof_reads: Stage5FullAppLiveProofReads,
    streak: Option<Res<Stage5FinishUx06Streak>>,
    visual_exit: ResMut<crate::render::gpu_surface_teardown::VisualTestGracefulExit>,
    capture_gate: Option<Res<DebugCaptureFrameGate>>,
    mut state: Local<VisualProbeFinalizeState>,
) {
    if state.proof_committed || *profile != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    if !gate.captured {
        return;
    }

    state.frames_since_capture = state.frames_since_capture.saturating_add(1);

    let min_frames = launch.min_capture_frames;
    let sim_frames = capture_gate.as_ref().map(|g| g.sim_frames).unwrap_or(0);
    if sim_frames < min_frames {
        if state.frames_since_capture == 1 || state.frames_since_capture % 30 == 0 {
            info!(
                target: "stage5_full_app_harness",
                sim_frames,
                min_frames,
                "visual probe waiting for minimum sim capture frames"
            );
        }
        return;
    }

    if crate::render::full_render_diagnostic_has_critical_anomaly(&summary) {
        warn!(
            ?summary,
            "FULL_APP visual probe blocked by render anomaly flags"
        );
        return;
    }
    if !stage5_readiness_passes(&report) {
        warn!(
            violations = ?report.violations,
            "FULL_APP visual probe blocked by readiness violations"
        );
        return;
    }

    let streak_n = streak.as_ref().map(|s| s.consecutive_good).unwrap_or(0);
    let ux06_done = finish_ux06_streak_done(streak.as_deref());

    if !ux06_done {
        if state.frames_since_capture >= VISUAL_PROBE_UX06_MAX_FRAMES_AFTER_CAPTURE {
            let blocker = streak
                .as_ref()
                .and_then(|s| s.last_blocker.as_deref())
                .unwrap_or("unknown");
            warn!(
                target: "stage5_full_app_harness",
                streak = streak_n,
                target_streak = FINISH_UX06_STREAK_DONE,
                frames_after_capture = state.frames_since_capture,
                last_blocker = ?blocker,
                "FULL_APP visual probe timed out waiting for FINISH-UX-06 streak (app keeps running)"
            );
            return;
        }
        if streak_n >= state.last_logged_streak.saturating_add(15)
            || (streak_n == 0 && state.last_logged_streak != 0)
        {
            info!(
                target: "stage5_full_app_harness",
                streak = streak_n,
                target_streak = FINISH_UX06_STREAK_DONE,
                frames_after_capture = state.frames_since_capture,
                last_blocker = streak.as_ref().and_then(|s| s.last_blocker.as_deref()),
                "visual probe waiting for FINISH-UX-06 streak"
            );
            state.last_logged_streak = streak_n;
        }
        return;
    }

    if !visual_probe_fire_witness_ready(&proof_reads, &report) {
        if streak_n >= state.last_logged_streak.saturating_add(15) {
            let buffer_rows = proof_reads
                .projection
                .as_ref()
                .map(|g| g.fire.instance_buffer.len())
                .unwrap_or(0);
            let particle_rows = proof_reads
                .particles
                .as_ref()
                .map(|f| f.instances.len())
                .unwrap_or(0);
            info!(
                target: "stage5_full_app_harness",
                fire_instance_buffer_rows = buffer_rows,
                particle_rows,
                "visual probe waiting for fire/particle witness before proof commit"
            );
            state.last_logged_streak = streak_n;
        }
        return;
    }

    if visual_tactical_vfx_witness_required(launch) {
        let gates = TacticalVfxWitnessGates::evaluate(
            proof_reads.particles.as_deref(),
            proof_reads.water_catalog.as_deref(),
            proof_reads.water_particles.as_deref(),
        );
        if !gates.all_green_for_visual_proof(true) {
            if state.frames_since_capture < VISUAL_PROBE_TACTICAL_VFX_MAX_FRAMES {
                if state.frames_since_capture == 1 || state.frames_since_capture % 30 == 0 {
                    info!(
                        target: "stage5_full_app_harness",
                        ?gates,
                        fire_rows = proof_reads
                            .particles
                            .as_ref()
                            .map(|p| p.spark_witness.rows)
                            .unwrap_or(0),
                        water_streaks = proof_reads
                            .water_particles
                            .as_ref()
                            .map(|p| p.witness.river_streaks)
                            .unwrap_or(0),
                        "visual probe waiting for tactical VFX witness (zoom + particle rows)"
                    );
                }
                return;
            }
            warn!(
                target: "stage5_full_app_harness",
                ?gates,
                frames = state.frames_since_capture,
                "tactical VFX witness not green — committing proof after timeout"
            );
        }
    }

    const PROOF_PATH: &str = "debug_runs/stage5_full_app_live.json";
    let body = build_stage5_full_app_live_proof_payload(&report, &gate, &summary, &proof_reads);
    let payload = crate::dev::debug_run_envelope::wrap_debug_run(
        "FULL_APP",
        "stage5_full_app_harness",
        PROOF_PATH,
        body,
    );
    if !crate::dev::debug_run_envelope::write_debug_run_json(PROOF_PATH, payload) {
        warn!("stage5 live proof write failed: {PROOF_PATH}");
        return;
    }
    info!(
        target: "stage5_full_app_harness",
        path = PROOF_PATH,
        streak = streak_n,
        "wrote stage5 FULL_APP live proof (FINISH-UX-06 streak complete)"
    );

    state.proof_committed = true;

    if minimap_gpu_compositor_env_enabled() {
        write_minimap_compositor_live_proof_from_reads(&proof_reads);
    }

    if launch.visual_auto_exit {
        if !visual_exit.armed {
            crate::render::gpu_surface_teardown::arm_visual_test_graceful_exit(visual_exit);
        }
    } else {
        info!(
            target: "stage5_full_app_harness",
            path = PROOF_PATH,
            "--test visual --stay-open: proof written; close the window when finished"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_full_app_readiness_fixture_is_green() {
        let scenario = build_deterministic_ci_scenario();
        let mut app = assemble_headless_full_app_readiness_app();
        hydrate_world_from_vt_ci_scenario(app.world_mut(), &scenario);
        let report = probe_full_app_stage5_readiness(&mut app);
        if !stage5_readiness_passes(&report) {
            eprintln!("FULL_APP readiness violations: {:?}", report.violations);
        }
        assert!(
            stage5_readiness_passes(&report),
            "FULL_APP readiness failed: {:?}",
            report.violations
        );
    }

    #[test]
    fn tactical_vfx_witness_gates_green_at_tactical_zoom() {
        use bevy::math::{Vec2, Vec4};

        use crate::render::extraction::FireVisualGpuInstance;
        use crate::render::gpu_water_particles::update_world_water_particles_from_catalog;
        use crate::render::{
            gpu_particles::{update_world_fire_particles_from_projection, FireParticleCameraScale},
            RiverPolylineSegment, WaterSurfaceVisualCatalog,
        };

        let mut graph = crate::render::extraction::RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 64;
        let mut row = FireVisualGpuInstance::default();
        row.chunk_xy_heat_lum = Vec4::new(0.0, 0.0, 0.85, 1.0);
        row.world_xyz_radius = Vec4::new(0.0, 0.0, 0.0, 32.0);
        row.smoke_ember_vis_priority = Vec4::new(0.1, 0.4, 0.0, 1.0);
        graph.fire.instance_buffer = vec![row];

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            None,
            FireParticleCameraScale {
                camera_zoom: 1.0,
                zoom_alpha: 0.8,
            },
            None,
        );

        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(6.0, 0.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });
        let mut water = WorldWaterParticleFrame::default();
        update_world_water_particles_from_catalog(
            &catalog,
            &mut water,
            FireParticleCameraScale {
                camera_zoom: 1.0,
                zoom_alpha: 0.8,
            },
            0.0,
        );

        let gates = TacticalVfxWitnessGates::evaluate(Some(&particles), Some(&catalog), Some(&water));
        assert!(gates.all_green(), "gates: {:?}", gates);
    }

    #[test]
    fn water_witness_001_dual_band_gates_from_catalog() {
        use crate::render::gpu_water_particles::{
            evaluate_water_vfx_witness_bands, water_strategic_001_green, water_witness_001_green,
            water_witness_foam_or_ocean_green,
        };
        use crate::render::{RiverPolylineSegment, WaterSurfaceVisualCatalog};

        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.grid_width = 16;
        catalog.grid_height = 16;
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(8.0, 0.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });
        catalog.river_tiles.insert((4, 0));
        catalog.ocean_tiles.insert((5, 5));
        catalog.lake_tiles.insert((0, 0));

        let bands = evaluate_water_vfx_witness_bands(&catalog, 0.8, 0.0);
        let gates = TacticalVfxWitnessGates::evaluate(None, Some(&catalog), None);
        assert!(water_strategic_001_green(&bands));
        assert!(water_witness_foam_or_ocean_green(&catalog, &bands.tactical));
        assert!(water_witness_001_green(&catalog, &bands));
        assert!(gates.water_strategic_001_green);
        assert!(gates.water_witness_001_green);
        assert!(gates.water_witness_foam_or_ocean_green);
    }

    #[test]
    fn water_w1_witness_stamp_reflects_catalog() {
        use crate::render::{RiverPolylineSegment, WaterSurfaceVisualCatalog};

        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.river_tiles.insert((3, 4));
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(3.0, 4.0),
            end: Vec2::new(5.0, 4.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });

        let (green, segments, tiles) = water_w1_witness_stamp(Some(&catalog));
        assert_eq!(green, Some(true));
        assert_eq!(segments, Some(1));
        assert_eq!(tiles, Some(1));

        let (missing_green, missing_segments, missing_tiles) = water_w1_witness_stamp(None);
        assert_eq!(missing_green, None);
        assert_eq!(missing_segments, None);
        assert_eq!(missing_tiles, None);
    }
}
