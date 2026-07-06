//! RGR-H3-001 split — LOG-E01 capture-lane witness JSON + fixtures.
//! Carved verbatim from `stage5_full_app_harness.rs` (pre-split monolith).

use crate::render::extraction::RenderProjectionGraph;
#[cfg(test)]
use crate::render::gpu_particles::WorldFireParticleFrame;
#[cfg(test)]
use crate::render::gpu_water_particles::WorldWaterParticleFrame;

use super::witness_gates::{tactical_vfx_proof_enabled, TacticalVfxWitnessGates, TACTICAL_VFX_ZOOM_ALPHA_MIN};

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
        obj.insert(
            "veg_burn_witness".into(),
            crate::dev::landscape_grammar_burn_live_proof::veg_burn_stage5_witness_json(),
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
    // FIRE-VIS-001: fire spark cull/scatter keys on zoom_level (px-per-tile), not zoom_alpha —
    // use full-scatter px-per-tile to clear the cull gate for this tactical-proof witness refresh.
    update_world_fire_particles_from_projection(
        &proj,
        &mut particles,
        None,
        FireParticleCameraScale {
            zoom_level: crate::render::gpu_particles::FIRE_SPARK_FULL_SCATTER_PX_PER_TILE,
            zoom_alpha: FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
            ..Default::default()
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
            zoom_level: 1.0,
            zoom_alpha: crate::render::gpu_water_particles::WATER_TACTICAL_WITNESS_ZOOM_ALPHA,
            ..Default::default()
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

pub(super) fn tactical_vfx_witness_json(gates: &TacticalVfxWitnessGates) -> serde_json::Value {
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
