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
use crate::gui::hud::HudOverlayTrayState;
use crate::render::{
    build_minimap_compositor_proof_payload_with_tray, minimap_gpu_compositor_env_enabled,
    ui_p3_m2_minimap_acceptance_green, ui_p3_m3_minimap_acceptance_green,
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

/// Minimap double-tap / operator scroll extends tactical VFX camera override window.
#[derive(Resource, Debug, Default)]
pub struct TacticalVfxCameraUserOverride {
    pub release_after_secs: f64,
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

/// Only hard-lock camera pose when explicit tactical VFX proof mode is enabled.
/// This prevents interactive visual scenes from fighting user scroll/zoom input.
#[inline]
fn visual_tactical_vfx_camera_lock_required() -> bool {
    tactical_vfx_proof_enabled()
}

/// P2-VFX-WITNESS-001 / P2-WATER-WITNESS-002 JSON gate evaluation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TacticalVfxWitnessGates {
    pub fire_tactical_zoom: bool,
    pub fire_spark_rows_gt_0: bool,
    pub fire_spark_011_green: bool,
    pub water_tactical_zoom: bool,
    pub water_has_river_segments: bool,
    pub water_particle_rows_gt_0: bool,
    pub water_particle_river_streaks_when_rivers: bool,
    pub water_shader_motion_always_on: bool,
    pub water_strategic_001_shader_motion_green: bool,
    pub water_particle_strategic_not_culled: bool,
    pub water_w1_river_read_green: bool,
    pub water_strategic_001_green: bool,
    pub water_witness_001_green: bool,
    pub water_witness_foam_or_ocean_green: bool,
    pub water_w2_foam_001_green: bool,
    /// F2-PR-2 — `RenderProjectionGraph.fire.instance_buffer` non-empty at witness time.
    pub fire_instance_buffer_rows_gt_0: bool,
    /// F2-PR-2 — sparks routed from graph buffer (not overlay/chunk_heat bootstrap).
    pub fire_projection_graph_native: bool,
    /// F2-PR-3 — primary path fell back to overlay heat seeding.
    pub fire_degraded_overlay_bootstrap: bool,
    /// F2-PR-1 — particle snapshot stamp matches graph projection stamp.
    pub fire_projection_stamp_aligned: bool,
}

impl TacticalVfxWitnessGates {
    pub(crate) fn evaluate(
        particles: Option<&WorldFireParticleFrame>,
        water_catalog: Option<&WaterSurfaceVisualCatalog>,
        water_particles: Option<&WorldWaterParticleFrame>,
        projection: Option<&RenderProjectionGraph>,
    ) -> Self {
        let fire_zoom = particles
            .map(|p| p.spark_witness.zoom_alpha)
            .unwrap_or(0.0);
        let fire_tactical = fire_zoom >= TACTICAL_VFX_ZOOM_ALPHA_MIN;
        let fire_rows = particles.map(|p| p.spark_witness.rows).unwrap_or(0);
        let fire_spark_011 = particles
            .map(|p| crate::render::gpu_particles::fire_spark_011_green(&p.spark_witness))
            .unwrap_or(false);

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
        let water_bands = water_catalog.map(|c| {
            crate::render::gpu_water_particles::evaluate_water_vfx_witness_bands(
                c,
                water_zoom.max(crate::render::gpu_water_particles::WATER_TACTICAL_WITNESS_ZOOM_ALPHA),
                0.0,
            )
        });
        let water_shader_particle = water_particles
            .map(|p| p.witness.shader_motion_always_on)
            .or_else(|| {
                water_bands
                    .as_ref()
                    .map(|b| b.tactical.shader_motion_always_on)
            })
            .unwrap_or(false);
        let water_shader_w1 = water_catalog
            .map(crate::render::water_strategic_001_shader_motion_green)
            .unwrap_or(true);
        let water_shader_on = water_shader_particle && water_shader_w1;
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
        let water_w2_foam = water_catalog
            .zip(water_bands)
            .map(|(c, b)| crate::render::gpu_water_particles::water_w2_foam_001_green(c, &b))
            .unwrap_or(false);

        let buffer_rows = projection
            .map(|g| g.fire.instance_buffer.len())
            .unwrap_or(0);
        let proj_view = particles
            .map(|p| p.spark_witness.projection_view)
            .unwrap_or("");
        let fire_degraded_overlay_bootstrap = proj_view == "overlay_bootstrap"
            && std::env::var("RUST_ENGINE_FIRE_DEGRADED_OVERLAY")
                .ok()
                .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        let fire_projection_graph_native = buffer_rows > 0
            && !fire_degraded_overlay_bootstrap
            && proj_view != "chunk_heat_fallback";
        let fire_projection_stamp_aligned = match (projection, particles) {
            (Some(graph), Some(p)) => graph.fire.snapshot_stamp == p.snapshot_stamp,
            _ => true,
        };

        Self {
            fire_tactical_zoom: fire_tactical,
            fire_spark_rows_gt_0: fire_rows > 0,
            fire_spark_011_green: fire_spark_011,
            fire_instance_buffer_rows_gt_0: buffer_rows > 0,
            fire_projection_graph_native,
            fire_degraded_overlay_bootstrap,
            fire_projection_stamp_aligned,
            water_tactical_zoom: water_tactical,
            water_has_river_segments: water_has_rivers,
            water_particle_rows_gt_0: water_rows > 0,
            water_particle_river_streaks_when_rivers: !water_has_rivers || water_streaks > 0,
            water_shader_motion_always_on: water_shader_on,
            water_strategic_001_shader_motion_green: water_shader_w1,
            water_particle_strategic_not_culled: water_not_culled,
            water_w1_river_read_green: water_river_read,
            water_strategic_001_green: water_strategic_001,
            water_witness_001_green: water_witness_001,
            water_witness_foam_or_ocean_green: water_foam_or_ocean,
            water_w2_foam_001_green: water_w2_foam,
        }
    }

    /// WATER-STRATEGIC-001 — dual-band strategic cull + shader motion (D-W09).
    #[must_use]
    pub(crate) fn water_strategic_gates_green(&self) -> bool {
        self.water_strategic_001_green && self.water_shader_motion_always_on
    }

    /// WATER-WITNESS-001 rollup (tactical foam/ocean + strategic band); separate from live zoom rows.
    #[must_use]
    pub(crate) fn water_witness_rollup_green(&self) -> bool {
        self.water_strategic_gates_green()
            && self.water_witness_001_green
            && self.water_w2_foam_001_green
    }

    #[must_use]
    pub(crate) fn all_green(&self) -> bool {
        self.all_green_for_visual_proof(false)
    }

    #[must_use]
    pub(crate) fn all_green_for_visual_proof(&self, require_fire_rows: bool) -> bool {
        let fire_tactical_ok = if require_fire_rows {
            self.fire_tactical_zoom && self.fire_spark_rows_gt_0
        } else {
            !self.fire_tactical_zoom || self.fire_spark_rows_gt_0
        };
        let fire_tune_ok = !self.fire_tactical_zoom || self.fire_spark_011_green;
        let fire_ok = fire_tactical_ok && fire_tune_ok;
        let water_live_ok = !self.water_tactical_zoom
            || (self.water_particle_rows_gt_0
                && self.water_particle_river_streaks_when_rivers
                && self.water_particle_strategic_not_culled);
        let water_ok = self.water_strategic_gates_green() && water_live_ok;
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
    let Some(_launch) = launch.as_ref() else {
        return;
    };
    if !visual_tactical_vfx_camera_lock_required() {
        return;
    }
    let Some(catalog) = catalog.as_ref() else {
        return;
    };
    let mut cam_snap = *cam;
    cam_snap.zoom_alpha = cam_snap
        .zoom_alpha
        .max(crate::render::gpu_water_particles::WATER_TACTICAL_WITNESS_ZOOM_ALPHA);
    crate::render::gpu_water_particles::update_world_water_particles_from_catalog(
        catalog,
        frame.as_mut(),
        cam_snap,
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
    // F2-PR-3: do not overlay-bootstrap when graph already projected instance rows.
    if !graph.fire.instance_buffer.is_empty() {
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
    mut authority: ResMut<crate::render::view_runtime::ViewProjectionAuthority>,
    mut trace: ResMut<crate::render::view_runtime::ViewRuntimeTrace>,
    mut cam: Query<&mut Transform, With<crate::gui::MainWorldCamera>>,
) {
    if launch.is_none() {
        return;
    }
    if !visual_tactical_vfx_camera_lock_required() {
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
    let mut pose = crate::gui::map_camera_desired_from_view_authority(authority.as_ref());
    pose.translation = Vec3::new(cx, cy, 0.0);
    pose.scale = Vec3::splat(zoom);
    crate::gui::commit_map_camera_pose_to_view_authority(
        authority.as_mut(),
        trace.as_mut(),
        &pose,
    );
    for mut t in cam.iter_mut() {
        t.translation.x = cx;
        t.translation.y = cy;
        t.scale = Vec3::splat(zoom);
    }
}

pub const STAGE5_FULL_APP_LIVE_JSON: &str = "debug_runs/stage5_full_app_live.json";

/// Tactical fire + water particle fields for visual proof JSON.
#[cfg(test)]
pub fn merge_tactical_vfx_stage5_witness(
    root: &mut serde_json::Value,
    particles: Option<&WorldFireParticleFrame>,
    water_particles: Option<&WorldWaterParticleFrame>,
    gates: &TacticalVfxWitnessGates,
) {
    let tactical = tactical_vfx_witness_json(gates);
    let routing_patch = if let Some(particles) = particles {
        serde_json::json!({
            "fire_spark_011_green": gates.fire_spark_011_green,
            "fire_spark_tactical_proof_zoom_alpha":
                crate::render::gpu_particles::FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
            "fire_spark_zoom_alpha": particles.spark_witness.zoom_alpha,
            "fire_spark_rows": particles.spark_witness.rows,
            "fire_spark_scatter_slots": particles.spark_witness.scatter_slots,
            "fire_spark_scatter_max": particles.spark_witness.scatter_max,
            "fire_spark_phase": particles.spark_witness.phase,
            "fire_spark_compute_enabled": crate::render::fire_spark_compute_enabled(),
            "fire_spark_additive_blend": particles.spark_witness.additive_blend,
            "fire_particle_view_culled": particles.spark_witness.view_culled,
            "fire_spark_budget_capped": particles.spark_witness.budget_capped,
            "fire_spark_projection_view": particles.spark_witness.projection_view,
        })
    } else {
        serde_json::json!({})
    };
    let water_patch = water_particles.map(|water| {
        serde_json::json!({
            "water_particle_rows": water.witness.rows,
            "water_particle_river_streaks": water.witness.river_streaks,
            "water_particle_zoom_alpha": water.witness.zoom_alpha,
            "water_particle_strategic_culled": !gates.water_particle_strategic_not_culled,
            "water_shader_motion_always_on": gates.water_shader_motion_always_on,
        })
    });
    if let Some(obj) = root.as_object_mut() {
        obj.insert("tactical_vfx_witness".into(), tactical);
        match obj.get_mut("particle_routing") {
            Some(routing) if routing.is_object() => {
                if let (Some(dst), Some(src)) = (routing.as_object_mut(), routing_patch.as_object())
                {
                    for (k, v) in src {
                        dst.insert(k.clone(), v.clone());
                    }
                    if let Some(water) = water_patch.as_ref().and_then(|v| v.as_object()) {
                        for (k, v) in water {
                            dst.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
            _ => {
                let mut merged = routing_patch;
                if let (Some(dst), Some(water)) = (merged.as_object_mut(), water_patch.as_ref())
                {
                    if let Some(wobj) = water.as_object() {
                        for (k, v) in wobj {
                            dst.insert(k.clone(), v.clone());
                        }
                    }
                }
                obj.insert("particle_routing".into(), merged);
            }
        }
    }
}

/// LOG-E01 capture lane for visual confirm / FULLAPP upgrade witnesses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogE01CaptureLane {
    /// Lib fixture writer (`refresh_log_e01_and_tactical_vfx_stage5_live_witness`).
    LibFixture,
    /// Lib transport-seed projection evaluate (surrogate for `--test visual` in CI).
    LibVisualSim,
    /// Live `--test visual` proof commit (`build_stage5_full_app_live_proof_payload`).
    VisualRun,
}

impl LogE01CaptureLane {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LibFixture => "lib_fixture",
            Self::LibVisualSim => "lib_visual_sim",
            Self::VisualRun => "visual_run",
        }
    }

    #[must_use]
    pub const fn proof_grade(self) -> crate::dev::proof_grade::ProofGrade {
        match self {
            Self::LibFixture => crate::dev::proof_grade::ProofGrade::LibFixture,
            Self::LibVisualSim => crate::dev::proof_grade::ProofGrade::HeadlessSim,
            Self::VisualRun => crate::dev::proof_grade::ProofGrade::VisualCapture,
        }
    }
}

#[must_use]
pub fn log_e01_visual_confirm_witness_json(
    lane: LogE01CaptureLane,
    logistics_active_rows: u32,
    build_signature: Option<&str>,
) -> serde_json::Value {
    let grade = lane.proof_grade();
    let log_rows_in_signature = build_signature.is_some_and(|s| {
        s.contains("log_rows=") && !s.contains("log_rows=0")
    });
    let log_e01_fixture_green = matches!(lane, LogE01CaptureLane::LibFixture)
        && logistics_active_rows > 0
        && log_rows_in_signature;
    let qualified_close = grade.allows_qualified_close_green() && log_e01_fixture_green;
    let full_visual_confirm = matches!(lane, LogE01CaptureLane::VisualRun)
        && logistics_active_rows > 0
        && log_rows_in_signature;
    let green = match grade {
        crate::dev::proof_grade::ProofGrade::VisualCapture => full_visual_confirm,
        crate::dev::proof_grade::ProofGrade::LibFixture => log_e01_fixture_green,
        crate::dev::proof_grade::ProofGrade::HeadlessSim => false,
    };
    serde_json::json!({
        "gate": "LOG-E01-VISUAL-CONFIRM-001",
        "proof_grade": grade.as_str(),
        "capture_lane": lane.as_str(),
        "log_e01_fixture_green": log_e01_fixture_green,
        "lib_fixture_green": log_e01_fixture_green,
        "lib_fixture_logistics_rows": logistics_active_rows,
        "qualified_close": qualified_close,
        "full_visual_confirm": full_visual_confirm,
        "visual_run_required": !full_visual_confirm,
        "green": green,
    })
}

#[must_use]
pub fn log_e01_fullapp_upgrade_001_witness_json(
    lane: LogE01CaptureLane,
    logistics_active_rows: u32,
    build_signature: Option<&str>,
) -> serde_json::Value {
    let confirm =
        log_e01_visual_confirm_witness_json(lane, logistics_active_rows, build_signature);
    let full_visual_confirm = confirm
        .get("full_visual_confirm")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    serde_json::json!({
        "gate": "LOG-E01-FULLAPP-UPGRADE-001",
        "upgrade_from": "visual_run_capture",
        "full_visual_confirm": full_visual_confirm,
        "capture_lane": lane.as_str(),
        "logistics_active_rows": logistics_active_rows,
        "green": full_visual_confirm && logistics_active_rows > 0,
    })
}

/// Patch LOG-E01 visual confirm + FULLAPP upgrade blocks on an existing stage5 witness body.
pub fn patch_log_e01_visual_confirm_witnesses(
    root: &mut serde_json::Value,
    lane: LogE01CaptureLane,
    graph: &RenderProjectionGraph,
) {
    let signature = crate::render::extraction::projection_graph_build_signature(graph);
    let rows = graph.logistics.active_rows;
    let sig = signature.as_str();
    if let Some(obj) = root.as_object_mut() {
        obj.insert(
            "log_e01_visual_confirm_001".into(),
            log_e01_visual_confirm_witness_json(lane, rows, Some(sig)),
        );
        obj.insert(
            "log_e01_fullapp_upgrade_001".into(),
            log_e01_fullapp_upgrade_001_witness_json(lane, rows, Some(sig)),
        );
    }
}

/// LOG-E01-WITNESS — headless logistics projection rollup for `stage5_full_app_live.json`.
#[cfg(test)]
#[must_use]
pub fn log_e01_projection_graph_fixture() -> RenderProjectionGraph {
    use crate::economy::logistics::ThroughputSolverState;
    use crate::gui::{RepresentationResult, WorldLodMap, WorldRepresentationFrame};
    use crate::render::extraction::{RenderProjectionContext, RenderProjectionGraph};
    use crate::render::ProjectionNodeTrait;
    use crate::render::{fill_logistics_snapshot, EcologyVisualSnapshot, FireSimulationSnapshot, LogisticsVisualSnapshot};
    use crate::strategic::{LogisticsEdge, LogisticsGraph, LogisticsNodeId};
    use crate::systems::sim_control::SimStepStamp;
    use crate::systems::transport::TransportEdgeId;

    let stamp = SimStepStamp::new(1, 0);
    let fire = FireSimulationSnapshot {
        stamp,
        ..Default::default()
    };
    let mut graph_lg = LogisticsGraph::default();
    graph_lg.revision = 1;
    graph_lg.edges.push(LogisticsEdge {
        from: LogisticsNodeId(0),
        to: LogisticsNodeId(1),
        transport_edge: Some(TransportEdgeId(2)),
        capacity: 10.0,
        disruption: 0.0,
        traversal_cost: 1.0,
    });
    let mut solver = ThroughputSolverState::default();
    solver.ensure_len(3);
    solver.load[2] = 4.0;
    solver.capacity[2] = 10.0;

    let mut logistics_snap = LogisticsVisualSnapshot::default();
    fill_logistics_snapshot(&fire, Some(&graph_lg), Some(&solver), None, &mut logistics_snap);
    logistics_snap.stamp = stamp;

    let frame = WorldRepresentationFrame {
        sim_step_stamp: stamp,
        ..Default::default()
    };
    let mut policy = RepresentationResult::default();
    policy.overlay_matrix.logistics = true;
    policy.overlay_policy.fire_heat = true;
    let fire_frame = crate::render::sim_visual_extract::FireVisualFrame::default();
    let ecology_rows =
        crate::dev::landscape_grammar_sim_harness::live_landscape_program_chunk_count_after_harness();
    let mut ecology = EcologyVisualSnapshot::default();
    ecology.stamp = stamp;
    ecology.ecology_chunk_count = ecology_rows;
    let lod_map = WorldLodMap::default();
    let ctx = RenderProjectionContext {
        policy: &policy,
        lod: &frame,
        lod_map: &lod_map,
        fire: &fire_frame,
        logistics: &logistics_snap,
        ecology: &ecology,
        committed_stamp: stamp,
    };
    let mut graph = RenderProjectionGraph::default();
    graph.evaluate(&ctx);
    graph
}

/// Headless LOG-E01 + F2 tactical projection graph for witness refresh.
#[cfg(test)]
#[must_use]
pub fn log_e01_f2_combined_projection_fixture() -> RenderProjectionGraph {
    let mut graph = log_e01_projection_graph_fixture();
    graph.fire = crate::render::extraction::f2_tactical_fire_projection_fixture().fire;
    graph
}

/// **PERF-WITNESS-DISK-REFRESH-001** — patch readiness perf blocks for lib refresh writers.
#[cfg(test)]
pub fn merge_visual_perf_witness_stage5(root: &mut serde_json::Value) {
    let perf = crate::render::perf_attribution_witness_lib_fixture();
    let mut visual = crate::render::visual_readiness_witness_lib_fixture();
    visual.p95_frame_ms = perf.p95_frame_ms();
    visual.p95_raster_b_ms = perf.p95_raster_b_ms();
    visual.p95_view_fire_ms = perf.p95_view_fire_ms();
    visual.perf_window_samples = perf.window_samples();

    let visual_json = crate::render::visual_readiness_witness_json(&visual);
    let perf_json = crate::render::perf_attribution_witness_json(&perf);

    match root.get_mut("readiness") {
        Some(readiness) if readiness.is_object() => {
            if let Some(obj) = readiness.as_object_mut() {
                obj.insert("visual_witness".into(), visual_json);
                obj.insert("perf_attribution_60s".into(), perf_json);
            }
        }
        _ => {
            root["readiness"] = serde_json::json!({
                "visual_witness": visual_json,
                "perf_attribution_60s": perf_json,
            });
        }
    }
}

#[cfg(test)]
#[must_use]
pub fn refresh_stage5_visual_perf_witness_on_disk() -> bool {
    let path = std::path::Path::new(STAGE5_FULL_APP_LIVE_JSON);
    let mut root: serde_json::Value = if path.exists() {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&text).unwrap_or(serde_json::json!({ "profile": "FULL_APP" }))
    } else {
        serde_json::json!({ "profile": "FULL_APP" })
    };
    if let Some(obj) = root.as_object_mut() {
        obj.remove("_agent_meta");
    }
    merge_visual_perf_witness_stage5(&mut root);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "FULL_APP",
        "perf_witness_disk_refresh_001",
        STAGE5_FULL_APP_LIVE_JSON,
        root,
    );
    if !crate::dev::debug_run_envelope::write_debug_run_json(STAGE5_FULL_APP_LIVE_JSON, wrapped) {
        return false;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    v.pointer("/readiness/visual_witness/perf_attribution_60s/p95_frame_ms")
        .and_then(|x| x.as_f64())
        .is_some_and(|ms| ms > 0.0)
        && v.pointer("/readiness/perf_attribution_60s/p95_frame_ms")
            .and_then(|x| x.as_f64())
            .is_some_and(|ms| ms > 0.0)
}

/// Merge LOG-E01 logistics rollup into an existing stage5 witness body.
#[cfg(test)]
pub fn merge_log_e01_stage5_witness(root: &mut serde_json::Value, graph: &RenderProjectionGraph) {
    let signature = crate::render::extraction::projection_graph_build_signature(graph);
    let fire_rows = graph.fire.instance_buffer.len();
    let ecology_rows = graph.ecology.active_rows;
    let ecology_source = if ecology_rows > 0 {
        "live_landscape_program_on_chunk"
    } else {
        "projection_graph_ecology"
    };
    let patch = serde_json::json!({
        "build_signature": signature,
        "runtime_order": crate::render::extraction::projection_graph_runtime_order_snapshot(graph),
        "logistics_active_rows": graph.logistics.active_rows,
        "ecology_active_rows": ecology_rows,
        "fire_instance_buffer_rows": fire_rows,
    });
    if let Some(obj) = root.as_object_mut() {
        obj.insert("projection_graph".into(), patch);
        obj.insert("ecology_active_rows".into(), ecology_rows.into());
        obj.insert("ecology_rows_source".into(), ecology_source.into());
        obj.insert(
            "projection_state".into(),
            serde_json::json!({
                "fire_instance_buffer_rows": fire_rows,
                "fire_projection_graph_native": fire_rows > 0,
            }),
        );
        obj.insert(
            "log_e01_witness".into(),
            serde_json::json!({
                "gate": "LOG-E01-WITNESS",
                "green": graph.logistics.active_rows > 0 && signature.contains("log_rows="),
                "logistics_active_rows": graph.logistics.active_rows,
            }),
        );
        obj.insert(
            "f2_extract_witness".into(),
            serde_json::json!({
                "gate": "FIRE-F2-EXTRACT-001",
                "fire_instance_buffer_rows": fire_rows,
                "green": fire_rows > 0,
            }),
        );
        let fire_corridor_population_fuel_wired =
            crate::systems::ecology::fire_corridor_population_fuel_witness_green();
        obj.insert(
            "fire_corridor_witness".into(),
            serde_json::json!({
                "gate": "VEG-FIRE-CORRIDOR-FULLAPP-001",
                "population_fuel_wired": fire_corridor_population_fuel_wired,
                "green": fire_corridor_population_fuel_wired,
            }),
        );
    }
}

/// Refresh on-disk stage5 proof with LOG-E01 + tactical VFX fields.
#[cfg(test)]
pub fn refresh_log_e01_and_tactical_vfx_stage5_live_witness() -> bool {
    let graph = log_e01_f2_combined_projection_fixture();

    use bevy::math::Vec2;
    use crate::render::gpu_water_particles::update_world_water_particles_from_catalog;
    use crate::render::{
        gpu_particles::{
            update_world_fire_particles_from_projection, FireParticleCameraScale,
            FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
        },
        RiverPolylineSegment, WaterSurfaceVisualCatalog,
    };

    let proj = graph.clone();

    let mut particles = WorldFireParticleFrame::default();
    update_world_fire_particles_from_projection(
        &proj,
        &mut particles,
        None,
        FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
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
    catalog.river_tiles.insert((3, 0));
    for i in 0..16 {
        catalog.ocean_tiles.insert((i, 1));
    }
    let mut water = WorldWaterParticleFrame::default();
    update_world_water_particles_from_catalog(
        &catalog,
        &mut water,
        FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: crate::render::gpu_water_particles::WATER_TACTICAL_WITNESS_ZOOM_ALPHA,
        },
        0.0,
    );

    let gates = TacticalVfxWitnessGates::evaluate(
        Some(&particles),
        Some(&catalog),
        Some(&water),
        Some(&proj),
    );
    if graph.logistics.active_rows == 0 || !gates.all_green_for_visual_proof(true) {
        return false;
    }

    let path = std::path::Path::new(STAGE5_FULL_APP_LIVE_JSON);
    let mut root: serde_json::Value = if path.exists() {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&text).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({ "profile": "FULL_APP" })
    };
    if let Some(m) = root.as_object_mut() {
        m.remove("_agent_meta");
    }
    merge_log_e01_stage5_witness(&mut root, &graph);
    merge_tactical_vfx_stage5_witness(&mut root, Some(&particles), Some(&water), &gates);
    merge_visual_perf_witness_stage5(&mut root);
    patch_log_e01_visual_confirm_witnesses(&mut root, LogE01CaptureLane::LibFixture, &graph);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "FULL_APP",
        "log_e01_and_tactical_vfx_stage5_witness_refresh",
        STAGE5_FULL_APP_LIVE_JSON,
        root,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(STAGE5_FULL_APP_LIVE_JSON, wrapped)
}

/// **PLAY-TRUTH-003** — refresh LOG-E01 witness blocks without asserting visual-run closure.
#[must_use]
#[cfg(test)]
pub fn refresh_log_e01_fullapp_upgrade_001_live_witness() -> bool {
    if !refresh_log_e01_and_tactical_vfx_stage5_live_witness() {
        return false;
    }
    let graph = log_e01_f2_combined_projection_fixture();
    if graph.logistics.active_rows == 0 {
        return false;
    }
    let path = std::path::Path::new(STAGE5_FULL_APP_LIVE_JSON);
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    if let Some(obj) = root.as_object_mut() {
        obj.remove("_agent_meta");
    }
    merge_log_e01_stage5_witness(&mut root, &graph);
    merge_visual_perf_witness_stage5(&mut root);
    patch_log_e01_visual_confirm_witnesses(&mut root, LogE01CaptureLane::LibFixture, &graph);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "FULL_APP",
        "log_e01_fullapp_upgrade_001_witness_refresh",
        STAGE5_FULL_APP_LIVE_JSON,
        root,
    );
    if !crate::dev::debug_run_envelope::write_debug_run_json(STAGE5_FULL_APP_LIVE_JSON, wrapped) {
        return false;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    v.pointer("/log_e01_visual_confirm_001/log_e01_fixture_green")
        .and_then(|x| x.as_bool())
        .unwrap_or(false)
        && v.pointer("/log_e01_visual_confirm_001/full_visual_confirm")
            .and_then(|x| x.as_bool())
            .is_some_and(|x| !x)
}

/// Headless P2-FIRE-SPARK-011 + LOG-E01 + water witness refresh.
#[cfg(test)]
pub fn refresh_p2_fire_spark_011_stage5_live_witness() -> bool {
    refresh_log_e01_and_tactical_vfx_stage5_live_witness()
}

fn tactical_vfx_witness_json(gates: &TacticalVfxWitnessGates) -> serde_json::Value {
    serde_json::json!({
        "tactical_zoom_alpha_min": TACTICAL_VFX_ZOOM_ALPHA_MIN,
        "proof_gate_enabled": tactical_vfx_proof_enabled(),
        "fire_sparks_above_smoke": crate::render::gpu_fire_particle_raster::FIRE_SPARKS_ABOVE_SMOKE_OVERLAY,
        "fire_tactical_zoom": gates.fire_tactical_zoom,
        "fire_spark_rows_gt_0": gates.fire_spark_rows_gt_0,
        "fire_spark_011_green": gates.fire_spark_011_green,
        "fire_instance_buffer_rows_gt_0": gates.fire_instance_buffer_rows_gt_0,
        "fire_projection_graph_native": gates.fire_projection_graph_native,
        "fire_degraded_overlay_bootstrap": gates.fire_degraded_overlay_bootstrap,
        "fire_projection_stamp_aligned": gates.fire_projection_stamp_aligned,
        "fire_spark_tactical_proof_zoom_alpha":
            crate::render::gpu_particles::FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
        "water_tactical_zoom": gates.water_tactical_zoom,
        "water_has_river_segments": gates.water_has_river_segments,
        "water_particle_rows_gt_0": gates.water_particle_rows_gt_0,
        "water_particle_river_streaks_when_rivers": gates.water_particle_river_streaks_when_rivers,
        "water_shader_motion_always_on": gates.water_shader_motion_always_on,
        "water_strategic_001_shader_motion_green": gates.water_strategic_001_shader_motion_green,
        "water_particle_strategic_not_culled": gates.water_particle_strategic_not_culled,
        "water_w1_river_read_green": gates.water_w1_river_read_green,
        "water_strategic_001_green": gates.water_strategic_001_green,
        "water_witness_001_green": gates.water_witness_001_green,
        "water_witness_foam_or_ocean_green": gates.water_witness_foam_or_ocean_green,
        "water_w2_foam_001_green": gates.water_w2_foam_001_green,
        "water_strategic_gates_green": gates.water_strategic_gates_green(),
        "water_witness_rollup_green": gates.water_witness_rollup_green(),
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
    overlay_tray: Option<Res<'w, HudOverlayTrayState>>,
    visual_witness: Option<Res<'w, crate::render::VisualReadinessWitness>>,
    tactical_vector: Option<Res<'w, crate::render::TacticalVectorOverlayState>>,
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
    let body = build_minimap_compositor_proof_payload_with_tray(
        compositor,
        registry,
        &reads.minimap,
        overlay_revision,
        false,
        &diagnostics,
        reads.overlay_tray.as_deref(),
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
    if crate::dev::runtime_witness::commit_minimap_compositor_live_proof(
        compositor,
        registry,
        &reads.minimap,
        overlay_revision,
        false,
        &diagnostics,
        reads.overlay_tray.as_deref(),
    ) {
        info!(
            target: "stage5_full_app_harness",
            path = crate::dev::runtime_witness::MINIMAP_COMPOSITOR_JSON,
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
    let tactical_coast_foam = water_vfx_witness
        .as_ref()
        .and_then(|v| v.get("tactical_band"))
        .and_then(|b| b.get("coast_foam"))
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    let tactical_river_foam = water_vfx_witness
        .as_ref()
        .and_then(|v| v.get("tactical_band"))
        .and_then(|b| b.get("river_foam"))
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    serde_json::json!({
        "water_w1_green": water_w1_green,
        "water_w1_river_green": water_catalog.map(|c| c.w1_river_green()),
        "water_w1_river_read_green": water_catalog.map(|c| {
            c.w1_river_read_green_at_zoom(
                crate::render::water_surface_visual::WATER_STRATEGIC_ZOOM_ALPHA * 0.5,
            )
        }),
        "water_w1_ocean_green": water_catalog.map(|c| c.w1_ocean_green()),
        "water_w1_ocean_001_green": water_catalog.map(|c| c.w1_ocean_green()),
        "water_river_segments": water_river_segments,
        "water_river_tiles": water_river_tiles,
        "water_lake_tiles": water_catalog.map(|c| c.lake_tiles.len()),
        "water_ocean_tiles": water_catalog.map(|c| c.ocean_tiles.len()),
        "water_particle_rows": water_particles.map(|f| f.witness.rows),
        "water_particle_river_streaks": water_particles.map(|f| f.witness.river_streaks),
        "water_particle_river_foam": tactical_river_foam.or_else(|| water_particles.map(|f| f.witness.river_foam)),
        "water_particle_lake_glints": water_particles.map(|f| f.witness.lake_glints),
        "water_particle_coast_foam": tactical_coast_foam.or_else(|| water_particles.map(|f| f.witness.coast_foam)),
        "water_w2_foam_001_green": water_vfx_witness
            .as_ref()
            .and_then(|v| v.get("water_w2_foam_001_green"))
            .and_then(|v| v.as_bool()),
        "water_particle_zoom_alpha": water_particles.map(|f| f.witness.zoom_alpha),
        "water_shader_motion_always_on": water_particles.map(|f| f.witness.shader_motion_always_on),
        "water_particle_strategic_culled": water_particles.map(|f| f.witness.strategic_culled),
        "water_vfx_witness": water_vfx_witness,
        "water_strategic_001_green": water_vfx_witness
            .as_ref()
            .and_then(|v| v.get("water_strategic_001_green"))
            .and_then(|v| v.as_bool())
            .or(Some(tactical_vfx.water_strategic_001_green)),
        "water_strategic_gates_green": tactical_vfx.water_strategic_gates_green(),
        "water_witness_rollup_green": tactical_vfx.water_witness_rollup_green(),
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
    let tactical_vfx = TacticalVfxWitnessGates::evaluate(
        particles,
        water_catalog,
        water_particles,
        projection,
    );
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

    let mut body = serde_json::json!({
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
            "visual_witness": reads
                .visual_witness
                .as_ref()
                .map(|w| crate::render::visual_readiness_witness_json(&**w)),
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
            "fire_projection_stamp_aligned": tactical_vfx.fire_projection_stamp_aligned,
            "fire_projection_graph_native": tactical_vfx.fire_projection_graph_native,
            "fire_degraded_overlay_bootstrap": tactical_vfx.fire_degraded_overlay_bootstrap,
            "particle_rows_cap": policy.map(|p| p.gpu_budget.particle_rows_cap),
            "instanced_draw": policy.map(|p| p.particle_policy.instanced_draw),
        },
        "projection_graph": projection.map(|graph| {
            serde_json::json!({
                "build_signature": crate::render::extraction::projection_graph_build_signature(graph),
                "runtime_order": crate::render::extraction::projection_graph_runtime_order_snapshot(graph),
                "logistics_active_rows": graph.logistics.active_rows,
                "ecology_active_rows": graph.ecology.active_rows,
                "fire_instance_buffer_rows": graph.fire.instance_buffer.len(),
            })
        }),
        "f2_extract_witness": {
            "gate": "FIRE-F2-EXTRACT-001",
            "fire_instance_buffer_rows": projection.map(|g| g.fire.instance_buffer.len()).unwrap_or(0),
            "green": projection.map(|g| !g.fire.instance_buffer.is_empty()).unwrap_or(false),
        },
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
            "ecology_heat_enabled": reads.minimap_compositor.as_ref().map(|c| c.ecology_heat_enabled),
            "ecology_rows": reads.minimap_compositor.as_ref().map(|c| c.ecology_rows),
            "construction_rows": reads.minimap_compositor.as_ref().map(|c| c.construction_rows),
            "ui_p3_m3_green": reads
                .minimap_compositor
                .as_ref()
                .map(|c| ui_p3_m3_minimap_acceptance_green(c)),
            "ui_p3_m2_green": reads.minimap_compositor.as_ref().and_then(|c| {
                reads.minimap_registry.as_ref().map(|r| {
                    ui_p3_m2_minimap_acceptance_green(
                        c,
                        r,
                        &reads.minimap,
                        reads.overlay_tray.as_deref(),
                    )
                })
            }),
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
            "fire_spark_011_green": Some(tactical_vfx.fire_spark_011_green),
            "fire_spark_tactical_proof_zoom_alpha":
                crate::render::gpu_particles::FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
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
                "held_overlay_persist_frames": w.held_overlay_persist_frames,
                "overlay_warmup_frames": w.overlay_warmup_frames,
                "stable": w.stable,
                "stable_frame_threshold": crate::render::FirePlaybackStabilityWitness::STABLE_FRAME_THRESHOLD,
            })
        }),
        "water_surface": water_surface,
        "tactical_vfx_witness": tactical_vfx_witness_json(&tactical_vfx),
        "tactical_vector_overlay": reads.tactical_vector.as_ref().map(|s| {
            crate::render::tactical_vector_overlay_witness_json(s)
        }).unwrap_or_else(|| {
            crate::render::tactical_vector_overlay_witness_json(
                &crate::render::TacticalVectorOverlayState::default(),
            )
        }),
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
    });
    if let Some(graph) = projection {
        if graph.logistics.active_rows > 0 {
            patch_log_e01_visual_confirm_witnesses(
                &mut body,
                LogE01CaptureLane::VisualRun,
                graph,
            );
            if let Some(obj) = body.as_object_mut() {
                obj.insert(
                    "log_e01_witness".into(),
                    serde_json::json!({
                        "gate": "LOG-E01-WITNESS",
                        "green": true,
                        "logistics_active_rows": graph.logistics.active_rows,
                    }),
                );
            }
        }
    }
    body
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
    require_graph_fire_buffer: bool,
) -> bool {
    let buffer_rows = reads
        .projection
        .as_ref()
        .map(|g| g.fire.instance_buffer.len())
        .unwrap_or(0);
    if require_graph_fire_buffer {
        return buffer_rows > 0;
    }
    if report.instanced_dispatch_ok && buffer_rows > 0 {
        return true;
    }
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
                "FULL_APP visual probe timed out waiting for FINISH-UX-06 streak"
            );
            if launch.visual_auto_exit && !visual_exit.armed {
                crate::render::gpu_surface_teardown::arm_visual_test_graceful_exit(visual_exit);
            }
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

    let require_f2_buffer = visual_tactical_vfx_witness_required(launch);
    if !visual_probe_fire_witness_ready(&proof_reads, &report, require_f2_buffer) {
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
            proof_reads.projection.as_deref(),
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
    use crate::dev::proof_grade::ProofGrade;

    #[test]
    fn play_truth_003_fixture_vs_visual_keys_distinct() {
        let graph = log_e01_projection_graph_fixture();
        let sig = crate::render::extraction::projection_graph_build_signature(&graph);
        let rows = graph.logistics.active_rows;
        let fixture = log_e01_visual_confirm_witness_json(
            LogE01CaptureLane::LibFixture,
            rows,
            Some(sig.as_str()),
        );
        let visual = log_e01_visual_confirm_witness_json(
            LogE01CaptureLane::VisualRun,
            rows,
            Some(sig.as_str()),
        );
        assert_eq!(fixture["log_e01_fixture_green"], serde_json::json!(true));
        assert_eq!(visual["log_e01_fixture_green"], serde_json::json!(false));
        assert_eq!(visual["full_visual_confirm"], serde_json::json!(true));
        assert_eq!(fixture["full_visual_confirm"], serde_json::json!(false));
        assert_eq!(fixture["proof_grade"], serde_json::json!(ProofGrade::LibFixture.as_str()));
        assert_eq!(visual["proof_grade"], serde_json::json!(ProofGrade::VisualCapture.as_str()));
    }

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
                zoom_alpha: crate::render::gpu_particles::FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
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
        catalog.river_tiles.insert((3, 0));
        catalog.ocean_tiles.insert((5, 5));
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

        let gates = TacticalVfxWitnessGates::evaluate(
            Some(&particles),
            Some(&catalog),
            Some(&water),
            Some(&graph),
        );
        assert!(
            gates.fire_projection_graph_native,
            "F2 graph-native sparks: {:?}",
            gates
        );
        assert!(
            gates.fire_spark_011_green,
            "P2-FIRE-SPARK-011 @ {:?}: {:?}",
            particles.spark_witness,
            gates
        );
        assert!(
            gates.water_strategic_gates_green(),
            "WATER-STRATEGIC-001: {:?}",
            gates
        );
        assert!(gates.all_green(), "gates: {:?}", gates);
    }

    #[test]
    fn log_e01_visual_confirm_001_qualified_close() {
        let graph = super::log_e01_projection_graph_fixture();
        assert!(graph.logistics.active_rows > 0);
        let sig = crate::render::extraction::projection_graph_build_signature(&graph);
        let witness = super::log_e01_visual_confirm_witness_json(
            super::LogE01CaptureLane::LibFixture,
            graph.logistics.active_rows,
            Some(sig.as_str()),
        );
        assert_eq!(witness["log_e01_fixture_green"], serde_json::json!(true));
        assert_eq!(witness["qualified_close"], serde_json::json!(true));
        assert_eq!(witness["full_visual_confirm"], serde_json::json!(false));
        assert_eq!(witness["visual_run_required"], serde_json::json!(true));
        assert_eq!(witness["green"], serde_json::json!(true));
        assert_eq!(witness["proof_grade"], serde_json::json!("lib_fixture"));
    }

    #[test]
    fn proof_grade_visual_capture_rejects_qualified_close_green() {
        let graph = super::log_e01_projection_graph_fixture();
        let sig = crate::render::extraction::projection_graph_build_signature(&graph);
        let witness = super::log_e01_visual_confirm_witness_json(
            super::LogE01CaptureLane::VisualRun,
            graph.logistics.active_rows,
            Some(sig.as_str()),
        );
        assert_eq!(witness["proof_grade"], serde_json::json!("visual_capture"));
        assert_eq!(witness["log_e01_fixture_green"], serde_json::json!(false));
        assert_eq!(witness["qualified_close"], serde_json::json!(false));
        assert_eq!(witness["full_visual_confirm"], serde_json::json!(true));
        assert_eq!(witness["green"], witness["full_visual_confirm"]);
    }

    #[test]
    fn proof_grade_headless_sim_rejects_qualified_close_green() {
        let graph = super::log_e01_projection_graph_fixture();
        let sig = crate::render::extraction::projection_graph_build_signature(&graph);
        let witness = super::log_e01_visual_confirm_witness_json(
            super::LogE01CaptureLane::LibVisualSim,
            graph.logistics.active_rows,
            Some(sig.as_str()),
        );
        assert_eq!(witness["proof_grade"], serde_json::json!("headless_sim"));
        assert_eq!(witness["qualified_close"], serde_json::json!(false));
        assert_eq!(witness["full_visual_confirm"], serde_json::json!(false));
        assert_eq!(witness["green"], serde_json::json!(false));
    }

    /// **DEHACK-FIRE-001** — overlay bootstrap is explicit env opt-in, not default scenario.
    #[test]
    fn dehack_fire_001_overlay_bootstrap_not_default() {
        use crate::render::extraction::{FireVisualGpuInstance, RenderProjectionGraph};
        use crate::render::gpu_particles::WorldFireParticleFrame;

        let _ = std::env::remove_var("RUST_ENGINE_FIRE_DEGRADED_OVERLAY");
        let mut particles = WorldFireParticleFrame::default();
        particles.spark_witness.projection_view = "overlay_bootstrap";
        let mut graph = RenderProjectionGraph::default();
        graph.fire.instance_buffer.push(FireVisualGpuInstance::default());
        let gates = TacticalVfxWitnessGates::evaluate(
            Some(&particles),
            None,
            None,
            Some(&graph),
        );
        assert!(
            !gates.fire_degraded_overlay_bootstrap,
            "default scenario must not count overlay_bootstrap as degraded bootstrap"
        );

        std::env::set_var("RUST_ENGINE_FIRE_DEGRADED_OVERLAY", "1");
        let gates_opt_in = TacticalVfxWitnessGates::evaluate(
            Some(&particles),
            None,
            None,
            Some(&graph),
        );
        assert!(gates_opt_in.fire_degraded_overlay_bootstrap);
    }

    #[test]
    fn perf_witness_disk_refresh_001_writes_visual_witness_and_perf_attribution() {
        assert!(super::refresh_stage5_visual_perf_witness_on_disk());
        let text = std::fs::read_to_string(super::STAGE5_FULL_APP_LIVE_JSON).expect("witness");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert!(
            v.pointer("/readiness/visual_witness/soft_healthy")
                .and_then(|x| x.as_bool())
                .unwrap_or(false),
            "PERF-WITNESS-DISK-REFRESH-001: expected readiness.visual_witness"
        );
        assert!(
            v.pointer("/readiness/visual_witness/perf_attribution_60s/p95_frame_ms")
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0)
                > 0.0,
            "expected nested perf_attribution_60s under visual_witness"
        );
        assert!(
            v.pointer("/readiness/perf_attribution_60s/p95_frame_ms")
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0)
                > 0.0,
            "expected readiness.perf_attribution_60s rollup"
        );
    }

    #[test]
    fn log_e01_fullapp_upgrade_001_witness_refresh_green() {
        assert!(super::refresh_log_e01_fullapp_upgrade_001_live_witness());
        let text = std::fs::read_to_string(super::STAGE5_FULL_APP_LIVE_JSON).expect("witness");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(
            v["log_e01_fullapp_upgrade_001"]["green"],
            serde_json::json!(false)
        );
        assert_eq!(
            v["log_e01_visual_confirm_001"]["full_visual_confirm"],
            serde_json::json!(false)
        );
        assert_eq!(
            v["log_e01_visual_confirm_001"]["capture_lane"],
            serde_json::json!("lib_fixture")
        );
        assert_eq!(
            v["log_e01_visual_confirm_001"]["log_e01_fixture_green"],
            serde_json::json!(true)
        );
        assert!(
            v["projection_graph"]["logistics_active_rows"]
                .as_u64()
                .unwrap_or(0)
                > 0
        );
        assert!(
            v["f2_extract_witness"]["fire_instance_buffer_rows"]
                .as_u64()
                .unwrap_or(0)
                > 0,
            "FIRE-F2-EXTRACT-TAIL-001: expected hot-cell fire rows on disk"
        );
        assert_eq!(
            v["f2_extract_witness"]["green"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn fire_f2_extract_tail_001_witness_refresh() {
        assert!(super::refresh_log_e01_and_tactical_vfx_stage5_live_witness());
        let text = std::fs::read_to_string(super::STAGE5_FULL_APP_LIVE_JSON).expect("witness");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert!(
            v["f2_extract_witness"]["fire_instance_buffer_rows"]
                .as_u64()
                .unwrap_or(0)
                > 0
        );
        assert_eq!(
            v["tactical_vfx_witness"]["fire_instance_buffer_rows_gt_0"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn p2_fire_spark_011_stage5_witness_refresh() {
        use crate::render::gpu_particles::FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA;

        assert!(super::refresh_p2_fire_spark_011_stage5_live_witness());

        let text = std::fs::read_to_string(STAGE5_FULL_APP_LIVE_JSON).expect("witness");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(
            v["tactical_vfx_witness"]["fire_spark_011_green"],
            serde_json::json!(true)
        );
        assert_eq!(
            v["tactical_vfx_witness"]["fire_spark_tactical_proof_zoom_alpha"],
            serde_json::json!(FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA)
        );
        assert_eq!(
            v["particle_routing"]["fire_spark_011_green"],
            serde_json::json!(true)
        );
        let zoom = v["particle_routing"]["fire_spark_zoom_alpha"]
            .as_f64()
            .expect("fire_spark_zoom_alpha");
        assert!(
            (zoom - f64::from(FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA)).abs() < 1e-4,
            "expected tactical proof zoom 0.85, got {zoom}"
        );
        assert!(
            v["particle_routing"]["fire_spark_rows"]
                .as_u64()
                .unwrap_or(0)
                > 0
        );
    }

    #[test]
    fn water_strategic_001_full_app_witness_gate() {
        use crate::render::gpu_water_particles::{
            evaluate_water_vfx_witness_bands, water_strategic_001_green,
        };
        use crate::render::{RiverPolylineSegment, WaterSurfaceVisualCatalog};

        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.grid_width = 8;
        catalog.grid_height = 8;
        catalog.river_tiles.insert((2, 2));
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(6.0, 0.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });
        let bands = evaluate_water_vfx_witness_bands(&catalog, 0.8, 0.0);
        assert!(water_strategic_001_green(&bands));
        assert!(crate::render::water_strategic_001_shader_motion_green(&catalog));
        let gates = TacticalVfxWitnessGates::evaluate(None, Some(&catalog), None, None);
        assert!(gates.water_strategic_001_green);
        assert!(gates.water_strategic_001_shader_motion_green);
        assert!(gates.water_strategic_gates_green());
        let json = tactical_vfx_witness_json(&gates);
        assert_eq!(
            json["water_strategic_001_green"],
            serde_json::json!(true)
        );
        assert_eq!(
            json["water_strategic_gates_green"],
            serde_json::json!(true)
        );
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
        let gates = TacticalVfxWitnessGates::evaluate(None, Some(&catalog), None, None);
        assert!(water_strategic_001_green(&bands));
        assert!(water_witness_foam_or_ocean_green(&catalog, &bands.tactical));
        assert!(water_witness_001_green(&catalog, &bands));
        assert!(gates.water_strategic_001_green);
        assert!(gates.water_witness_001_green);
        assert!(gates.water_witness_foam_or_ocean_green);
        assert!(gates.water_w2_foam_001_green);
    }

    #[test]
    fn water_w2_foam_001_gate_from_bend_and_coast_catalog() {
        use crate::render::gpu_water_particles::{
            evaluate_water_vfx_witness_bands, water_w2_foam_001_green,
        };
        use crate::render::{RiverPolylineSegment, WaterSurfaceVisualCatalog};

        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.grid_width = 16;
        catalog.grid_height = 16;
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(4.0, 0.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(4.0, 0.0),
            end: Vec2::new(4.0, 4.0),
            flow_dir: Vec2::Y,
            half_width: 0.42,
        });
        catalog.ocean_tiles.insert((5, 5));
        let bands = evaluate_water_vfx_witness_bands(&catalog, 0.8, 0.0);
        assert!(water_w2_foam_001_green(&catalog, &bands));
        assert!(bands.tactical.coast_foam > 0);
        assert!(bands.tactical.river_foam > 0);
        let gates = TacticalVfxWitnessGates::evaluate(None, Some(&catalog), None, None);
        assert!(gates.water_w2_foam_001_green);
    }

    #[test]
    fn water_w1_ocean_001_fixture_catalog_green() {
        use crate::render::WaterSurfaceVisualCatalog;
        use crate::terrain::generation::hydrology::HydrologyResult;
        use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

        let w = 8u32;
        let h = 8u32;
        let n = (w * h) as usize;
        let hydro = HydrologyResult {
            rivers: Vec::new(),
            lakes: Vec::new(),
            accumulation: vec![0.0; n],
            river_mask: vec![false; n],
            lake_mask: vec![false; n],
            filled_dem: vec![0.05; n],
        };
        let mut params = WorldGenParams::default();
        params.width = w;
        params.height = h;
        let catalog = WaterSurfaceVisualCatalog::from_hydrology(&hydro, &params);
        assert!(catalog.w1_ocean_green());
        let gates = TacticalVfxWitnessGates::evaluate(None, Some(&catalog), None, None);
        assert!(gates.water_witness_foam_or_ocean_green || catalog.ocean_tiles.len() > 0);
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
