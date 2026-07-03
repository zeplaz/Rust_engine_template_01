//! F2 fire projection + WSS smoke bridge — lib diagnostic witness (`debug_runs/f2_smoke_pipeline_live.json`).
//!
//! Identifies common stalls: empty `instance_buffer`, stamp mismatch, residency cull, overlay bootstrap fallback.

use bevy::math::{IVec2, UVec2, Vec4};
use bevy::prelude::*;
use serde_json::{json, Value};

pub const F2_SMOKE_PIPELINE_LIVE_JSON: &str = "debug_runs/f2_smoke_pipeline_live.json";

#[derive(Debug, Clone)]
struct PipelineMiniReport {
    projection_rows: usize,
    chunk_heat_rows: usize,
    smoke_wired: bool,
    smoke_density_sum: f32,
    particle_rows: usize,
    projection_view: String,
    graph_native: bool,
    stamp_aligned: bool,
}

#[must_use]
fn stamp_mismatch_clears_buffer_when_lag_gt_1() -> bool {
    use crate::gui::{
        build_representation_inputs, build_representation_result, LodZoneRegistry, VisualBudgetSettings,
        VisualCadence, WorldRepresentationFrame,
    };
    use crate::render::extraction::{
        ProjectionNodeTrait, RenderProjectionContext, RenderProjectionGraph,
    };
    use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame, FireVisualGpuInstance};
    use crate::render::{EcologyVisualSnapshot, LogisticsVisualSnapshot};
    use crate::systems::sim_control::SimStepStamp;

    let mut frame = FireVisualFrame::default();
    frame.stamp = SimStepStamp::new(10, 0);
    frame.instances.push(FireVisualGpuInstance::default());
    frame.chunk_heat.push(ChunkFireHeat {
        chunk: IVec2::ZERO,
        heat: 0.9,
        smoke: 0.0,
    });

    let lod = WorldRepresentationFrame::default();
    let lod_map = crate::gui::WorldLodMap::default();
    let policy_inputs = build_representation_inputs(
        &crate::gui::CameraVisualState::default(),
        &LodZoneRegistry::default(),
        &VisualBudgetSettings::default(),
        &VisualCadence::from(&VisualBudgetSettings::default()),
        frame.stamp,
    );
    let policy = build_representation_result(&lod, &policy_inputs);
    let logistics = LogisticsVisualSnapshot::default();
    let ecology = EcologyVisualSnapshot::default();
    let ctx = RenderProjectionContext {
        policy: &policy,
        lod: &lod,
        lod_map: &lod_map,
        fire: &frame,
        logistics: &logistics,
        ecology: &ecology,
        committed_stamp: SimStepStamp::new(7, 0),
    };
    let mut graph = RenderProjectionGraph::default();
    graph.evaluate(&ctx);
    graph.fire.instance_buffer.is_empty() && graph.fire.chunk_heat.is_empty()
}

fn run_mini_fire_smoke_pipeline_ticks() -> PipelineMiniReport {
    use crate::engine::launch_args::{EngineLaunchArgs, TestScene};
    use crate::render::extraction::SmokeVisualBridgeWitness;
    use crate::render::{
        update_world_fire_particles_from_projection, ExtractedCameraMetrics, FireVisualFramePlugin,
        FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
    };
    use crate::gui::{
        build_representation_inputs, build_representation_result, LodZoneRegistry, VisualBudgetSettings,
        VisualCadence,
    };
    use crate::render::SharedOverlayFieldBuffersPlugin;
    use crate::render::{CommittedVisualSnapshotFence, SimChunkSmokeVisualExtract, Stage5ReadinessProfile};
    use crate::systems::fire::{ChunkSmokeField, ChunkSurfaceFire, FireLightEmission};
    use crate::systems::sim_control::{SimTick, SimTimeMicros};
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(SharedOverlayFieldBuffersPlugin);
    app.init_resource::<crate::systems::atmosphere::AtmosphereDiagnostics>();
    app.init_resource::<SimTick>()
        .init_resource::<SimTimeMicros>()
        .init_resource::<crate::gui::WorldLodMap>()
        .init_resource::<crate::gui::WorldRepresentationFrame>()
        .init_resource::<crate::gui::RepresentationResult>()
        .init_resource::<CommittedVisualSnapshotFence>()
        .init_resource::<Stage5ReadinessProfile>()
        .init_resource::<SimChunkSmokeVisualExtract>()
        .insert_resource(EngineLaunchArgs {
            test_scene: TestScene::Visual,
            ..Default::default()
        });
    app.add_plugins(FireVisualFramePlugin);

    app.world_mut().spawn((
        Chunk {
            coord: IVec2::new(1, 1),
        },
        ChunkCellMatrix::new(UVec2::new(8, 8)),
        ChunkSurfaceFire {
            heat: 0.88,
            fuel: 1.0,
        },
        FireLightEmission {
            radius: 140.0,
            base_intensity: 1.0,
            current_intensity: 1.0,
            flicker_strength: 0.1,
            flicker_phase: 0.0,
            extract_priority: 1.0,
        },
        ChunkSmokeField {
            density: 0.42,
            toxicity: 0.1,
            visibility_penalty: 0.05,
        },
    ));

    {
        use crate::render::ChunkSmokeGpu;
        let mut smoke = app.world_mut().resource_mut::<SimChunkSmokeVisualExtract>();
        smoke.instances.push(ChunkSmokeGpu {
            chunk_xy: Vec4::new(1.0, 1.0, 0.0, 0.0),
            density_tox_vis: Vec4::new(0.42, 0.1, 0.05, 0.0),
        });
    }

    {
        let mut frame = app.world().resource::<crate::gui::WorldRepresentationFrame>().clone();
        let band = frame.global_band();
        frame.visibility = crate::gui::visibility_for_band(band);
        frame.resolution = crate::gui::resolution_for_band(band);
        let stamp = crate::systems::sim_control::SimStepStamp::from_tick(SimTick(4), SimTimeMicros(0));
        let budgets = VisualBudgetSettings::default();
        let inputs = build_representation_inputs(
            &Default::default(),
            &LodZoneRegistry::default(),
            &budgets,
            &VisualCadence::from(&budgets),
            stamp,
        );
        *app.world_mut().resource_mut::<crate::gui::RepresentationResult>() =
            build_representation_result(&frame, &inputs);
        app.world_mut()
            .resource_mut::<CommittedVisualSnapshotFence>()
            .fire = stamp;
    }

    for _ in 0..4 {
        *app.world_mut().resource_mut::<SimTick>() = SimTick(4);
        app.update();
    }

    let graph = app.world().resource::<crate::render::extraction::RenderProjectionGraph>();
    let smoke_w = app.world().resource::<SmokeVisualBridgeWitness>();
    let mut particles = crate::render::WorldFireParticleFrame::default();
    update_world_fire_particles_from_projection(
        graph,
        &mut particles,
        None,
        ExtractedCameraMetrics {
            zoom_level: 1.0,
            zoom_alpha: FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
            ..Default::default()
        },
        None,
    );
    let stamp_aligned = graph.fire.snapshot_stamp == particles.snapshot_stamp;

    PipelineMiniReport {
        projection_rows: graph.fire.instance_buffer.len(),
        chunk_heat_rows: graph.fire.chunk_heat.len(),
        smoke_wired: smoke_w.smoke_extract_wired,
        smoke_density_sum: smoke_w.smoke_density_sum,
        particle_rows: particles.instances.len(),
        projection_view: particles.spark_witness.projection_view.to_string(),
        graph_native: !graph.fire.instance_buffer.is_empty()
            && particles.spark_witness.projection_view != "overlay_bootstrap",
        stamp_aligned,
    }
}

#[must_use]
fn read_stage5_crosscheck() -> Value {
    let path = std::path::Path::new(crate::render::STAGE5_FULL_APP_LIVE_JSON);
    let Ok(text) = std::fs::read_to_string(path) else {
        return json!({ "present": false });
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return json!({ "present": true, "parse_error": true });
    };
    let buffer = v
        .pointer("/projection_state/fire_instance_buffer_rows")
        .or_else(|| v.pointer("/f2_extract_witness/fire_instance_buffer_rows"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    let held_empty = v
        .pointer("/fire_playback/held_empty_snapshot_frames")
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    let particle_rows = v
        .pointer("/particle_routing/particle_rows")
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    let overlay_bootstrap = v
        .pointer("/particle_routing/fire_spark_projection_view")
        .and_then(|s| s.as_str())
        == Some("overlay_bootstrap");
    let stall_risk = buffer == 0 && particle_rows > 0;
    let overlay_hold_noise = held_empty > 40 && buffer > 0;
    json!({
        "present": true,
        "fire_instance_buffer_rows": buffer,
        "particle_rows": particle_rows,
        "held_empty_snapshot_frames": held_empty,
        "fire_spark_overlay_bootstrap": overlay_bootstrap,
        "stall_risk_sparks_without_graph_buffer": stall_risk,
        "overlay_hold_frames_high_but_buffer_ok": overlay_hold_noise,
        "visual_probe_caps": {
            "ux06_max_frames": 2400_u32,
            "tactical_vfx_max_frames": 900_u32,
        },
    })
}

/// Refresh diagnostic JSON; returns whether all lib checks passed.
#[must_use]
pub fn refresh_f2_smoke_pipeline_live_witness() -> bool {
    let f2_graph = crate::render::extraction::f2_tactical_fire_projection_fixture();
    let f2_rows = f2_graph.fire.instance_buffer.len();
    let stamp_guard_ok = stamp_mismatch_clears_buffer_when_lag_gt_1();
    let mini = run_mini_fire_smoke_pipeline_ticks();
    let stage5 = read_stage5_crosscheck();

    let checks = json!({
        "f2_fixture_buffer_nonempty": f2_rows > 0,
        "stamp_mismatch_guard": stamp_guard_ok,
        "mini_projection_rows_gt_0": mini.projection_rows > 0,
        "mini_chunk_heat_rows_gt_0": mini.chunk_heat_rows > 0,
        "mini_smoke_extract_wired": mini.smoke_wired,
        "mini_smoke_density_sum_gt_0": mini.smoke_density_sum > 0.0,
        "mini_particle_rows_gt_0": mini.particle_rows > 0,
        "mini_graph_native_sparks": mini.graph_native,
        "mini_stamp_aligned": mini.stamp_aligned,
        "mini_projection_view_not_overlay_bootstrap": mini.projection_view != "overlay_bootstrap",
    });
    let all_green = checks
        .as_object()
        .is_some_and(|m| m.values().all(|v| v.as_bool() == Some(true)));

    let body = json!({
        "gate": "F2-SMOKE-PIPELINE-DEBUG",
        "green": all_green,
        "pass": all_green,
        "checks": checks,
        "f2_fixture": {
            "fire_instance_buffer_rows": f2_rows,
            "fire_chunk_heat_rows": f2_graph.fire.chunk_heat.len(),
            "snapshot_stamp": f2_graph.fire.snapshot_stamp,
        },
        "mini_pipeline": {
            "fire_instance_buffer_rows": mini.projection_rows,
            "fire_chunk_heat_rows": mini.chunk_heat_rows,
            "smoke_extract_wired": mini.smoke_wired,
            "smoke_density_sum": mini.smoke_density_sum,
            "particle_rows": mini.particle_rows,
            "fire_spark_projection_view": mini.projection_view,
            "fire_projection_graph_native": mini.graph_native,
            "fire_projection_stamp_aligned": mini.stamp_aligned,
        },
        "stage5_crosscheck": stage5,
        "stall_identifiers": {
            "residency_cull_skipped_on_visual_proof": true,
            "visual_commit_requires_graph_buffer_when_tactical_proof": true,
            "overlay_bootstrap_only_when_graph_buffer_empty": true,
        },
    });

    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "F2_SMOKE_PIPELINE",
        "refresh_f2_smoke_pipeline_live_witness",
        F2_SMOKE_PIPELINE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(F2_SMOKE_PIPELINE_LIVE_JSON, wrapped)
        && all_green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f2_smoke_pipeline_debug_witness_refresh_green() {
        assert!(
            refresh_f2_smoke_pipeline_live_witness(),
            "see {F2_SMOKE_PIPELINE_LIVE_JSON}"
        );
    }
}
