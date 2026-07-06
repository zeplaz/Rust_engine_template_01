//! FULL_APP Stage 5 readiness probe — headless fixture + optional runtime logging.
//!
//! **RGR-H3-001** split — this used to be a single 2600+ LOC file (`stage5_full_app_harness.rs`).
//! It is now a directory module; every public item that used to live at
//! `crate::render::stage5_full_app_harness::*` is still re-exported from here (mechanical
//! split, no behavior change). Seams:
//! - [`witness_gates`] — tactical VFX zoom/witness gate flags + evaluation
//! - [`log_e01_witness`] — LOG-E01 capture-lane witness JSON + fixtures
//! - [`readiness_app`] — headless FULL_APP readiness app assembly + probe entry point
//! - [`proof_reads`] — `Stage5FullAppLiveProofReads` SystemParam + presentation-label helpers
//! - [`proof_payload`] — water-surface + full live proof JSON payload builder
//! - [`proof_commit`] — FINISH-UX-06 streak gate + proof commit system

mod log_e01_witness;
mod proof_commit;
mod proof_payload;
mod proof_reads;
mod readiness_app;
mod witness_gates;

pub use log_e01_witness::{LogE01CaptureLane, STAGE5_FULL_APP_LIVE_JSON};
#[cfg(test)]
pub use log_e01_witness::{
    log_e01_f2_combined_projection_fixture, log_e01_projection_graph_fixture,
    merge_log_e01_stage5_witness, merge_tactical_vfx_stage5_witness,
    merge_visual_perf_witness_stage5, refresh_log_e01_and_tactical_vfx_stage5_live_witness,
    refresh_log_e01_fullapp_upgrade_001_live_witness, refresh_p2_fire_spark_011_stage5_live_witness,
    refresh_stage5_visual_perf_witness_on_disk,
};
#[cfg(test)]
use log_e01_witness::{log_e01_visual_confirm_witness_json, tactical_vfx_witness_json};
pub(crate) use proof_commit::finalize_visual_full_app_live_probe;
#[cfg(test)]
use proof_payload::water_w1_witness_stamp;
#[cfg(test)]
use readiness_app::{assemble_headless_full_app_readiness_app, hydrate_world_from_vt_ci_scenario};
pub use readiness_app::probe_full_app_stage5_readiness;
pub(crate) use witness_gates::{refresh_visual_proof_fire_particles, refresh_visual_proof_water_particles};
#[cfg(test)]
use witness_gates::TacticalVfxWitnessGates;
pub use witness_gates::{
    tactical_vfx_hard_lock_enabled, vfx_sandbox_scroll_zoom_free,
    visual_tactical_vfx_camera_lock_enabled,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::proof_grade::ProofGrade;
    use crate::render::gpu_particles::WorldFireParticleFrame;
    use crate::render::gpu_water_particles::WorldWaterParticleFrame;
    use crate::render::stage5_readiness::stage5_readiness_passes;
    use crate::render::vt_ci_matrix::build_deterministic_ci_scenario;
    use bevy::math::Vec2;

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
        // FIRE-VIS-001: fire spark cull/scatter now keys on zoom_level (px-per-tile), not
        // zoom_alpha — use the full-scatter px-per-tile so this tactical-proof fixture clears
        // the cull gate; zoom_alpha stays on the proof-lock axis for fire_spark_011_green.
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            None,
            FireParticleCameraScale {
                zoom_level: crate::render::gpu_particles::FIRE_SPARK_FULL_SCATTER_PX_PER_TILE,
                zoom_alpha: crate::render::gpu_particles::FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
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
        catalog.ocean_tiles.insert((5, 5));
        let mut water = WorldWaterParticleFrame::default();
        update_world_water_particles_from_catalog(
            &catalog,
            &mut water,
            FireParticleCameraScale {
                zoom_level: 1.0,
                zoom_alpha: 0.8,
                ..Default::default()
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
        assert_eq!(
            v["veg_burn_witness"]["gate"],
            serde_json::json!("VEG-BURN-FULLAPP-006")
        );
        assert_eq!(v["veg_burn_witness"]["green"], serde_json::json!(true));
        assert!(
            v["veg_burn_witness"]["burn_active_rows"]
                .as_u64()
                .unwrap_or(0)
                > 0
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
