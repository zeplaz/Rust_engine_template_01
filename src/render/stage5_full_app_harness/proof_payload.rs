//! RGR-H3-001 split — water-surface + full FULL_APP live proof JSON payload builder.
//! Carved verbatim from `stage5_full_app_harness.rs` (pre-split monolith).

use bevy::prelude::*;

use crate::dev::diagnostics::view_authority_sample_json;
use crate::gui::{MapViewInstanceId, WorldBounds};
use crate::render::gpu_water_particles::WorldWaterParticleFrame;
use crate::render::stage5_readiness::{stage5_readiness_passes, AppStage5ReadinessReport};
use crate::render::{
    minimap_gpu_compositor_env_enabled, ui_p3_m2_minimap_acceptance_green,
    ui_p3_m3_minimap_acceptance_green,
};
use crate::render::WaterSurfaceVisualCatalog;

use super::log_e01_witness::{
    patch_log_e01_visual_confirm_witnesses, tactical_vfx_witness_json, LogE01CaptureLane,
};
use super::proof_reads::{
    map_view_consumer_payload, minimap_source_label_for_proof, stage5_finish_todo_board_snapshot,
    stage5_live_todo_board_snapshot, Stage5FullAppLiveProofReads,
};
use super::witness_gates::TacticalVfxWitnessGates;

pub(super) fn water_w1_witness_stamp(
    catalog: Option<&WaterSurfaceVisualCatalog>,
) -> (Option<bool>, Option<usize>, Option<usize>) {
    (
        catalog.map(WaterSurfaceVisualCatalog::w1_green),
        catalog.map(|c| c.river_segments.len()),
        catalog.map(|c| c.river_tiles.len()),
    )
}

pub(super) fn build_water_surface_proof_json(
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

pub(super) fn build_stage5_full_app_live_proof_payload(
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
        "perf": {
            "terrain_gpu_authoritative": reads
                .terrain_authority
                .as_ref()
                .is_some_and(|a| a.is_gpu()),
        },
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
            "perf_attribution_60s": reads
                .perf_attribution
                .as_ref()
                .map(|w| crate::render::perf_attribution_witness_json(&**w)),
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
            "view_isolation": view_authority_sample_json(
                &reads.view_isolation,
                reads.view_manager.as_deref(),
                reads.fire_witness.as_deref(),
                reads.view_projection_authority.as_deref(),
                reads.view_runtime_witness.as_deref(),
            ),
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
