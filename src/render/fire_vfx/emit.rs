//! Fire particle emission from post-LOD projection graph (frontend).

use bevy::math::{UVec2, Vec2, Vec4};
use bevy::prelude::*;

use crate::gui::GPU_FIRE_INSTANCE_BUDGET_CEILING;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::sim_visual_extract::{FireVisualGpuInstance, FIRE_VISUAL_ACTIVE_HEAT_EPS};
use crate::render::{
    trace_particle_routing, ChunkCoord, DebugRenderTraceConfig, FireChunkLodState, FireLodBand,
};
use crate::terrain::generation::chunk_world_center;

use crate::render::ExtractedCameraMetrics;
use super::frame::WorldFireParticleFrame;
use super::pack::{GpuParticleInstance, ParticleClass};
use super::witness::{
    fire_spark_witness_phase, fire_spark_zoom_scatter_gate, FireSparkWitness,
    FIRE_SPARK_BUDGET_PRESSURE, FIRE_SPARK_MIN_ZOOM_ALPHA, FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA,
    FIRE_SPARK_SCATTER_MAX,
};

/// Chunk slab size assumed when bootstrap rows lack matrix geometry (matches test harness).
const FIRE_PARTICLE_BOOTSTRAP_SLAB: u32 = 32;

/// **TRIAGE-PHASE-F-CULL-001** — sparks culled off non-tactical views / low zoom.
#[must_use]
pub fn view_aware_particle_cull_wired() -> bool {
    true
}

#[inline]
pub fn is_tactical_fire_particle_view(id: crate::gui::ViewId) -> bool {
    matches!(
        id,
        crate::gui::ViewId::WorldMain | crate::gui::ViewId::SimulationMap
    )
}

#[inline]
#[must_use]
pub fn fire_particle_scatter_count(heat: f32, zoom_alpha: f32, budget_pressure: f32) -> usize {
    let zoom_gate = fire_spark_zoom_scatter_gate(zoom_alpha);
    if zoom_gate <= 0.0 {
        return 0;
    }
    let base = if heat >= 0.72 {
        11
    } else if heat >= 0.48 {
        8
    } else if heat >= 0.28 {
        4
    } else if heat >= 0.12 {
        2
    } else {
        0
    };
    let mut slots = ((base as f32) * zoom_gate).ceil() as usize;
    if zoom_alpha >= FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA * 0.75 && heat >= 0.12 {
        slots = slots.max(2);
    }
    slots = slots.min(FIRE_SPARK_SCATTER_MAX);
    if budget_pressure >= FIRE_SPARK_BUDGET_PRESSURE && slots > 1 {
        slots = (slots / 2).max(1);
    }
    slots
}

#[inline]
fn fire_particle_scatter_offset(chunk_xy: Vec2, heat: f32, slot: u32) -> Vec2 {
    let seed = (chunk_xy.x as u32)
        .wrapping_mul(73856093)
        .wrapping_add(chunk_xy.y as u32)
        .wrapping_add(slot.wrapping_mul(19349663));
    let a = (seed & 0xffff) as f32 / 65535.0;
    let b = ((seed >> 16) & 0xffff) as f32 / 65535.0;
    let r = 10.0 + heat * 34.0;
    Vec2::new((a * 2.0 - 1.0) * r, (b * 2.0 - 1.0) * r)
}

#[inline]
fn fire_row_with_world_offset(mut row: FireVisualGpuInstance, offset: Vec2) -> FireVisualGpuInstance {
    row.world_xyz_radius.x += offset.x;
    row.world_xyz_radius.y += offset.y;
    row
}

#[inline]
fn fire_lod_band_for_instance_row(row: &FireVisualGpuInstance, chunk_lod: Option<&FireChunkLodState>) -> FireLodBand {
    let xy = row.chunk_grid_xy();
    let c = ChunkCoord::new(xy.x as i32, xy.y as i32);
    chunk_lod
        .and_then(|s| s.bands.get(&c).copied())
        .unwrap_or(FireLodBand::FullFlame)
}

/// Shapes a projected GPU row for particle emission from authoritative [`FireLodBand`].
#[must_use]
fn shape_fire_row_for_particle_lod(
    mut row: FireVisualGpuInstance,
    band: FireLodBand,
    zoom_alpha: f32,
) -> Option<(FireVisualGpuInstance, ParticleClass)> {
    match band {
        FireLodBand::None => None,
        FireLodBand::SmokeOnly => {
            row.chunk_xy_heat_lum.z *= 0.15;
            row.chunk_xy_heat_lum.w *= 0.7;
            row.smoke_ember_vis_priority.x = row.smoke_ember_vis_priority.x.max(0.22);
            row.smoke_ember_vis_priority.y *= 0.35;
            Some((row, ParticleClass::AtmosphereFx))
        }
        FireLodBand::LowFlame => {
            row.chunk_xy_heat_lum.z *= 0.62;
            row.chunk_xy_heat_lum.w *= 0.82;
            row.smoke_ember_vis_priority.y *= 0.68;
            let class = if zoom_alpha >= FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA * 0.75 {
                ParticleClass::Spark
            } else {
                ParticleClass::Ember
            };
            Some((row, class))
        }
        FireLodBand::FullFlame => Some((row, ParticleClass::Spark)),
    }
}

pub fn emit_world_fire_particles_from_projection(
    time: Res<Time>,
    cfg: Option<Res<DebugRenderTraceConfig>>,
    graph: Res<RenderProjectionGraph>,
    coherence: Option<Res<crate::render::extraction::ProjectionGraphFrameCoherence>>,
    chunk_lod: Res<FireChunkLodState>,
    cam: Res<ExtractedCameraMetrics>,
    view_manager: Option<Res<crate::gui::ViewManager>>,
    overlay: Option<Res<crate::render::SharedOverlayFieldBuffers>>,
    mut frame: ResMut<WorldFireParticleFrame>,
    mut last_trace: Local<u64>,
) {
    if coherence.as_deref().is_some_and(|c| c.evaluate_skipped) {
        frame.snapshot_stamp = graph.fire.snapshot_stamp;
        frame.anim_time_secs = time.elapsed_secs();
        return;
    }
    update_world_fire_particles_from_projection(
        graph.as_ref(),
        frame.as_mut(),
        Some(chunk_lod.as_ref()),
        *cam,
        view_manager.as_deref(),
    );
    if frame.instances.is_empty() && graph.fire.instance_buffer.is_empty() {
        if let Some(overlay) = overlay.as_ref() {
            if !overlay.chunk_fire_heat.is_empty() {
                seed_world_fire_particles_from_overlay_heat(
                    &overlay.chunk_fire_heat,
                    frame.as_mut(),
                    *cam,
                );
            }
        }
    }
    frame.anim_time_secs = time.elapsed_secs();
    if let Some(cfg) = cfg.as_deref() {
        if cfg.particle_routing_trace {
            *last_trace = last_trace.wrapping_add(1);
            if *last_trace % 30 == 0 {
                trace_particle_routing(
                    cfg,
                    &format!(
                        "world_fire_particle_frame coordinate_space=world active_count={} band={:?}",
                        frame.instances.len(),
                        frame.active_band,
                    ),
                );
            }
        }
    }
}

pub fn update_world_fire_particles_from_projection(
    graph: &RenderProjectionGraph,
    frame: &mut WorldFireParticleFrame,
    chunk_lod: Option<&FireChunkLodState>,
    cam: ExtractedCameraMetrics,
    view_manager: Option<&crate::gui::ViewManager>,
) {
    frame.snapshot_stamp = graph.fire.snapshot_stamp;
    frame.active_band = crate::gui::representation_band_from_world_lod(graph.fire.lod);
    let source_view = crate::render::view_fire_projection::projection_fire_source_view(view_manager);
    let tactical = is_tactical_fire_particle_view(source_view);
    let projection_label = match source_view {
        crate::gui::ViewId::WorldMain => "WorldMain",
        crate::gui::ViewId::SimulationMap => "SimulationMap",
        crate::gui::ViewId::WorldPreview => "WorldPreview",
        crate::gui::ViewId::Minimap => "Minimap",
    };
    let raw_cap = graph.fire.gpu_instance_capacity;
    let capacity = if raw_cap == 0 {
        if graph.fire.instance_buffer.is_empty() && graph.fire.chunk_heat.is_empty() {
            0
        } else {
            GPU_FIRE_INSTANCE_BUDGET_CEILING.min(512)
        }
    } else if raw_cap == usize::MAX {
        GPU_FIRE_INSTANCE_BUDGET_CEILING
    } else {
        raw_cap.min(GPU_FIRE_INSTANCE_BUDGET_CEILING)
    };
    frame.gpu_capacity = capacity;
    frame.instances.clear();
    let mut scatter_slots = 0usize;
    let mut budget_capped = false;

    if !tactical || capacity == 0 || cam.zoom_alpha < FIRE_SPARK_MIN_ZOOM_ALPHA {
        frame.spark_witness = FireSparkWitness {
            phase: fire_spark_witness_phase(),
            rows: 0,
            scatter_max: FIRE_SPARK_SCATTER_MAX,
            scatter_slots: 0,
            zoom_alpha: cam.zoom_alpha,
            additive_blend: true,
            budget_capped: false,
            view_culled: !tactical || cam.zoom_alpha < FIRE_SPARK_MIN_ZOOM_ALPHA,
            projection_view: projection_label,
        };
        return;
    }
    frame
        .instances
        .reserve(graph.fire.instance_buffer.len().min(capacity));
    for row in &graph.fire.instance_buffer {
        let band = fire_lod_band_for_instance_row(row, chunk_lod);
        let Some((shaped, class)) = shape_fire_row_for_particle_lod(*row, band, cam.zoom_alpha) else {
            continue;
        };
        if shaped.heat() < FIRE_VISUAL_ACTIVE_HEAT_EPS && shaped.smoke_ember_vis_priority.y < 0.02 {
            continue;
        }
        let heat = shaped.heat();
        let chunk_xy = shaped.chunk_grid_xy();
        let budget_pressure = if capacity > 0 {
            frame.instances.len() as f32 / capacity as f32
        } else {
            1.0
        };
        frame
            .instances
            .push(GpuParticleInstance::from_fire_visual(&shaped, class, cam));
        let scatter_n = fire_particle_scatter_count(heat, cam.zoom_alpha, budget_pressure);
        scatter_slots = scatter_slots.saturating_add(scatter_n);
        for slot in 0u32..scatter_n as u32 {
            if frame.instances.len() >= capacity {
                budget_capped = true;
                break;
            }
            let offset = fire_particle_scatter_offset(chunk_xy, heat, slot);
            let scattered = fire_row_with_world_offset(shaped, offset);
            let scatter_class = if heat >= 0.28 {
                ParticleClass::Spark
            } else {
                class
            };
            frame.instances.push(GpuParticleInstance::from_fire_visual(
                &scattered,
                scatter_class,
                cam,
            ));
        }
        if frame.instances.len() >= capacity {
            budget_capped = true;
            break;
        }
    }
    let mut witness_projection_view = projection_label;
    if frame.instances.is_empty()
        && graph.fire.instance_buffer.is_empty()
        && !graph.fire.chunk_heat.is_empty()
    {
        for ch in &graph.fire.chunk_heat {
            if ch.heat < 0.12 {
                continue;
            }
            let mut row = FireVisualGpuInstance::default();
            row.chunk_xy_heat_lum = Vec4::new(
                ch.chunk.x as f32,
                ch.chunk.y as f32,
                ch.heat,
                1.0,
            );
            let center = chunk_world_center(
                ChunkCoord::new(ch.chunk.x, ch.chunk.y),
                UVec2::splat(FIRE_PARTICLE_BOOTSTRAP_SLAB),
            );
            row.world_xyz_radius = Vec4::new(center.x, center.y, 0.0, 28.0);
            row.smoke_ember_vis_priority = Vec4::new(0.12, 0.55, 0.0, 1.0);
            if let Some((shaped, class)) =
                shape_fire_row_for_particle_lod(row, FireLodBand::FullFlame, cam.zoom_alpha)
            {
                frame.instances.push(GpuParticleInstance::from_fire_visual(
                    &shaped,
                    class,
                    cam,
                ));
            }
            if frame.instances.len() >= capacity.max(1) {
                break;
            }
        }
        if !frame.instances.is_empty() {
            witness_projection_view = "chunk_heat_fallback";
        }
    }
    frame.instances.sort_by_key(|row| {
        ParticleClass::from_class_id(row.ember_class_radius_smoke.y).transparent_draw_order()
    });
    frame.spark_witness = FireSparkWitness {
        phase: fire_spark_witness_phase(),
        rows: frame.instances.len(),
        scatter_max: FIRE_SPARK_SCATTER_MAX,
        scatter_slots,
        zoom_alpha: cam.zoom_alpha,
        additive_blend: true,
        budget_capped,
        view_culled: false,
        projection_view: witness_projection_view,
    };
}

/// P2-VFX-VISUAL-001 — when projection has no fire rows but overlay heat exists (visual proof).
pub fn seed_world_fire_particles_from_overlay_heat(
    chunk_fire_heat: &std::collections::HashMap<bevy::math::IVec2, f32>,
    frame: &mut WorldFireParticleFrame,
    cam: ExtractedCameraMetrics,
) {
    if cam.zoom_alpha < FIRE_SPARK_MIN_ZOOM_ALPHA {
        frame.instances.clear();
        frame.spark_witness = FireSparkWitness {
            phase: fire_spark_witness_phase(),
            rows: 0,
            scatter_max: FIRE_SPARK_SCATTER_MAX,
            scatter_slots: 0,
            zoom_alpha: cam.zoom_alpha,
            additive_blend: true,
            budget_capped: false,
            view_culled: true,
            projection_view: "overlay_bootstrap",
        };
        return;
    }
    frame.instances.clear();
    let mut scatter_slots = 0usize;
    for (&coord, &heat) in chunk_fire_heat {
        if heat < 0.12 {
            continue;
        }
        let mut row = FireVisualGpuInstance::default();
        row.chunk_xy_heat_lum = Vec4::new(coord.x as f32, coord.y as f32, heat, 1.0);
        let center = chunk_world_center(coord, UVec2::splat(FIRE_PARTICLE_BOOTSTRAP_SLAB));
        row.world_xyz_radius = Vec4::new(center.x, center.y, 0.0, 28.0);
        row.smoke_ember_vis_priority = Vec4::new(0.12, 0.55, 0.0, 1.0);
        let Some((shaped, class)) =
            shape_fire_row_for_particle_lod(row, FireLodBand::FullFlame, cam.zoom_alpha)
        else {
            continue;
        };
        frame
            .instances
            .push(GpuParticleInstance::from_fire_visual(&shaped, class, cam));
        let scatter_n = fire_particle_scatter_count(heat, cam.zoom_alpha, 0.0);
        scatter_slots = scatter_slots.saturating_add(scatter_n);
        for slot in 0..scatter_n as u32 {
            let offset = fire_particle_scatter_offset(shaped.chunk_grid_xy(), heat, slot);
            let scattered = fire_row_with_world_offset(shaped, offset);
            frame.instances.push(GpuParticleInstance::from_fire_visual(
                &scattered,
                ParticleClass::Spark,
                cam,
            ));
        }
    }
    frame.spark_witness = FireSparkWitness {
        phase: fire_spark_witness_phase(),
        rows: frame.instances.len(),
        scatter_max: FIRE_SPARK_SCATTER_MAX,
        scatter_slots,
        zoom_alpha: cam.zoom_alpha,
        additive_blend: true,
        budget_capped: false,
        view_culled: false,
        projection_view: "overlay_bootstrap",
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::gpu_buffer_registry::{BufferId, FIRE_PARTICLE_INSTANCES_BUFFER};
    use crate::gui::{
        build_representation_inputs, build_representation_result, resolution_for_band,
        LodZoneRegistry, VisualBudgetSettings,
        VisualCadence, WorldLodBand, WorldLodBands, WorldLodMap, WorldRepresentationFrame,
    };
    use crate::render::extraction::{
        ProjectionNodeTrait, RenderProjectionContext, RenderProjectionGraph,
    };
    use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame};
    use crate::render::{
        EcologyVisualSnapshot, FireChunkLodState, FireLodBand, LogisticsVisualSnapshot,
    };
    use crate::systems::sim_control::SimStepStamp;
    use bevy::math::IVec2;

    fn sample_fire_row(chunk: IVec2, heat: f32, ember: f32) -> FireVisualGpuInstance {
        let mut row = FireVisualGpuInstance::default();
        row.chunk_xy_heat_lum = Vec4::new(chunk.x as f32, chunk.y as f32, heat, 1.0);
        row.world_xyz_radius = Vec4::new(chunk.x as f32 * 64.0, chunk.y as f32 * 64.0, 0.0, 32.0);
        row.smoke_ember_vis_priority = Vec4::new(0.1, ember, 0.0, 1.0);
        row
    }

    #[test]
    fn macro_band_thins_world_fire_projection_but_keeps_rows() {
        let mut fire = FireVisualFrame::default();
        fire.stamp = SimStepStamp::new(1, 0);
        fire.instances.push(sample_fire_row(IVec2::ZERO, 0.9, 0.5));
        fire.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::ZERO,
            heat: 0.9,
            smoke: 0.1,
        });

        let mut lod = WorldRepresentationFrame::default();
        lod.bands = WorldLodBands {
            global: WorldLodBand::Macro,
            ..Default::default()
        };
        lod.resolution = resolution_for_band(WorldLodBand::Macro);
        let lod_map = WorldLodMap::default();

        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            fire.stamp,
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
            committed_stamp: fire.stamp,
        };
        graph.evaluate(&ctx);
        assert!(
            !graph.fire.instance_buffer.is_empty(),
            "macro band should still project a thinned fire instance list"
        );
        assert!(graph.fire.instance_buffer.len() <= 8);

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            None,
            ExtractedCameraMetrics::default(),
            None,
        );
        assert!(
            !particles.instances.is_empty(),
            "macro band should still route some particle rows when fire projection is non-empty"
        );
    }

    #[test]
    fn tactical_projection_feeds_world_fire_particle_rows() {
        let mut fire = FireVisualFrame::default();
        fire.instances.push(sample_fire_row(IVec2::new(2, 3), 0.8, 0.4));

        let mut lod = WorldRepresentationFrame::default();
        lod.bands.global = WorldLodBand::LocalTactical;
        lod.resolution = resolution_for_band(WorldLodBand::LocalTactical);
        let lod_map = WorldLodMap::default();

        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            fire.stamp,
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
            committed_stamp: fire.stamp,
        };
        graph.evaluate(&ctx);
        graph.fire.snapshot_stamp = 7;
        graph.fire.lod = WorldLodBand::LocalTactical;
        graph.fire.gpu_instance_capacity = lod.resolution.fire_instance_cap;

        let mut particles = WorldFireParticleFrame::default();
        let cam = ExtractedCameraMetrics {
            zoom_level: 1.0,
            zoom_alpha: 0.72,
            ..Default::default()
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        let scatter = 1 + fire_particle_scatter_count(0.8, cam.zoom_alpha, 0.0);
        assert_eq!(particles.instances.len(), scatter);
        assert_eq!(particles.snapshot_stamp, 7);
        assert_eq!(
            particles.instances[0].ember_class_radius_smoke.y,
            ParticleClass::Spark.as_f32()
        );
        assert_eq!(particles.spark_witness.projection_view, "WorldMain");
    }

    #[test]
    fn chunk_heat_fallback_only_when_instance_buffer_empty() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.chunk_heat.push(crate::render::sim_visual_extract::ChunkFireHeat {
            chunk: IVec2::ZERO,
            heat: 0.9,
            smoke: 0.0,
        });
        let cam = ExtractedCameraMetrics {
            zoom_level: 1.0,
            zoom_alpha: 0.72,
            ..Default::default()
        };
        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert_eq!(particles.spark_witness.projection_view, "chunk_heat_fallback");

        let mut row = FireVisualGpuInstance::default();
        row.chunk_xy_heat_lum = Vec4::new(0.0, 0.0, 0.85, 1.0);
        row.world_xyz_radius = Vec4::new(0.0, 0.0, 0.0, 24.0);
        row.smoke_ember_vis_priority = Vec4::new(0.1, 0.5, 0.0, 1.0);
        graph.fire.instance_buffer.push(row);
        let mut native = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(&graph, &mut native, None, cam, None);
        assert_eq!(native.spark_witness.projection_view, "WorldMain");
        assert_ne!(native.spark_witness.projection_view, "chunk_heat_fallback");
    }

    #[test]
    fn gpu_particle_maps_wide_light_radius_to_small_quad_half() {
        let mut row = FireVisualGpuInstance::default();
        row.world_xyz_radius = Vec4::new(1000.0, 2000.0, 0.0, 260.0);
        row.chunk_xy_heat_lum = Vec4::new(1.0, 2.0, 0.75, 0.6);
        let gpu = GpuParticleInstance::from_fire_visual(
            &row,
            ParticleClass::Spark,
            ExtractedCameraMetrics::default(),
        );
        assert!(
            gpu.ember_class_radius_smoke.z <= 1.51 && gpu.ember_class_radius_smoke.z >= 0.015,
            "spark half {}",
            gpu.ember_class_radius_smoke.z
        );
        assert_eq!(gpu.world_xyz_heat.w, 0.75);
    }

    #[test]
    fn particle_rows_respect_gpu_capacity_ceiling() {
        let mut fire = FireVisualFrame::default();
        for i in 0..8 {
            fire.instances
                .push(sample_fire_row(IVec2::new(i, 0), 0.8, 0.4));
        }

        let mut lod = WorldRepresentationFrame::default();
        lod.bands.global = WorldLodBand::LocalTactical;
        lod.resolution = resolution_for_band(WorldLodBand::LocalTactical);
        let lod_map = WorldLodMap::default();

        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            fire.stamp,
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
            committed_stamp: fire.stamp,
        };
        graph.evaluate(&ctx);
        graph.fire.snapshot_stamp = 3;
        graph.fire.lod = WorldLodBand::LocalTactical;
        graph.fire.gpu_instance_capacity = 3;

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            None,
            ExtractedCameraMetrics::default(),
            None,
        );
        assert_eq!(particles.instances.len(), 3);
        assert_eq!(particles.gpu_capacity, 3);
    }

    #[test]
    fn usize_max_graph_capacity_clamps_frame_gpu_capacity() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = usize::MAX;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.9, 0.5)];
        let mut particles = WorldFireParticleFrame::default();
        let cam = ExtractedCameraMetrics {
            zoom_level: 1.0,
            zoom_alpha: 0.72,
            ..Default::default()
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert_eq!(
            particles.gpu_capacity,
            crate::gui::GPU_FIRE_INSTANCE_BUDGET_CEILING
        );
        assert_eq!(
            particles.instances.len(),
            1 + fire_particle_scatter_count(0.9, cam.zoom_alpha, 0.0)
        );
    }

    #[test]
    fn authoritative_smoke_only_lod_routes_atmosphere_fx() {
        let mut fire = FireVisualFrame::default();
        fire.stamp = SimStepStamp::new(1, 0);
        fire.instances.push(sample_fire_row(IVec2::ZERO, 0.95, 0.55));

        let mut lod = WorldRepresentationFrame::default();
        lod.bands.global = WorldLodBand::LocalTactical;
        lod.resolution = resolution_for_band(WorldLodBand::LocalTactical);
        let lod_map = WorldLodMap::default();

        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            fire.stamp,
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
            committed_stamp: fire.stamp,
        };
        graph.evaluate(&ctx);
        graph.fire.snapshot_stamp = 9;
        graph.fire.lod = WorldLodBand::LocalTactical;
        graph.fire.gpu_instance_capacity = 8;

        let mut chunk_lod = FireChunkLodState::default();
        chunk_lod
            .bands
            .insert(IVec2::ZERO, FireLodBand::SmokeOnly);

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            Some(&chunk_lod),
            ExtractedCameraMetrics::default(),
            None,
        );
        assert_eq!(
            particles.instances.len(),
            1 + fire_particle_scatter_count(0.95 * 0.15, 0.72, 0.0)
        );
        assert!(
            (particles.instances[0].ember_class_radius_smoke.y - ParticleClass::AtmosphereFx.as_f32()).abs()
                < 1e-4,
            "SmokeOnly policy should route through AtmosphereFx particle class"
        );
    }

    #[test]
    fn authoritative_none_band_skips_particle_row() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 8;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::new(5, 2), 0.8, 0.4)];

        let mut chunk_lod = FireChunkLodState::default();
        chunk_lod
            .bands
            .insert(IVec2::new(5, 2), FireLodBand::None);

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            Some(&chunk_lod),
            ExtractedCameraMetrics::default(),
            None,
        );
        assert!(
            particles.instances.is_empty(),
            "None band should suppress GPU particle emission for that chunk"
        );
    }

    #[test]
    fn particle_buffer_id_is_stable() {
        assert_eq!(FIRE_PARTICLE_INSTANCES_BUFFER, BufferId(3));
    }

    #[test]
    fn hot_cell_scatter_count_at_least_three() {
        assert!(fire_particle_scatter_count(0.8, 0.72, 0.0) >= 3);
    }

    #[test]
    fn strategic_zoom_zeroes_scatter() {
        assert_eq!(fire_particle_scatter_count(0.95, 0.05, 0.0), 0);
    }

    #[test]
    fn ember_class_larger_quad_half_than_spark() {
        let mut row = FireVisualGpuInstance::default();
        row.world_xyz_radius = Vec4::new(0.0, 0.0, 0.0, 120.0);
        row.chunk_xy_heat_lum = Vec4::new(0.0, 0.0, 0.75, 0.6);
        let cam = ExtractedCameraMetrics {
            zoom_level: 1.0,
            zoom_alpha: 0.8,
            ..Default::default()
        };
        let spark = GpuParticleInstance::from_fire_visual(&row, ParticleClass::Spark, cam);
        let ember = GpuParticleInstance::from_fire_visual(&row, ParticleClass::Ember, cam);
        assert!(
            ember.ember_class_radius_smoke.z > spark.ember_class_radius_smoke.z,
            "ember {} spark {}",
            ember.ember_class_radius_smoke.z,
            spark.ember_class_radius_smoke.z
        );
    }

    #[test]
    fn low_flame_lod_routes_ember_class_when_zoomed_out() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 16;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::new(1, 1), 0.85, 0.5)];

        let mut chunk_lod = FireChunkLodState::default();
        chunk_lod
            .bands
            .insert(IVec2::new(1, 1), FireLodBand::LowFlame);

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            Some(&chunk_lod),
            ExtractedCameraMetrics {
                zoom_level: 0.2,
                zoom_alpha: 0.18,
                ..Default::default()
            },
            None,
        );
        assert!(
            (particles.instances[0].ember_class_radius_smoke.y - ParticleClass::Ember.as_f32()).abs()
                < 1e-4
        );
    }

    #[test]
    fn low_flame_lod_routes_spark_class_at_operational_zoom() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 16;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::new(1, 1), 0.85, 0.5)];

        let mut chunk_lod = FireChunkLodState::default();
        chunk_lod
            .bands
            .insert(IVec2::new(1, 1), FireLodBand::LowFlame);

        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            Some(&chunk_lod),
            ExtractedCameraMetrics {
                zoom_level: 1.0,
                zoom_alpha: FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA,
                ..Default::default()
            },
            None,
        );
        assert!(
            (particles.instances[0].ember_class_radius_smoke.y - ParticleClass::Spark.as_f32()).abs()
                < 1e-4
        );
    }

    #[test]
    fn budget_pressure_halves_scatter_slots() {
        let full = fire_particle_scatter_count(0.8, 0.72, 0.0);
        let pressured = fire_particle_scatter_count(0.8, 0.72, FIRE_SPARK_BUDGET_PRESSURE);
        assert!(pressured <= full);
        assert!(pressured >= 1);
        assert_eq!(pressured, full / 2);
    }

    #[test]
    fn p2_fire_spark_011_at_tactical_proof_zoom() {
        use crate::render::fire_vfx::witness::{fire_spark_011_green, FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA};
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 256;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.5)];
        let mut particles = WorldFireParticleFrame::default();
        let cam = ExtractedCameraMetrics {
            zoom_level: 1.0,
            zoom_alpha: FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
            ..Default::default()
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert!(
            fire_spark_011_green(&particles.spark_witness),
            "P2-FIRE-SPARK-011 witness: {:?}",
            particles.spark_witness
        );
    }

    #[test]
    fn p2_operational_play_zoom_alpha_emits_fire_spark_rows() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 64;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.4)];
        let mut particles = WorldFireParticleFrame::default();
        let cam = ExtractedCameraMetrics {
            zoom_level: 1.0,
            zoom_alpha: FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA,
            ..Default::default()
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert!(
            particles.spark_witness.rows > 0,
            "expected fire spark rows > 0 at operational zoom_alpha={}, got {}",
            FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA,
            particles.spark_witness.rows
        );
    }

    #[test]
    fn p2_tactical_zoom_alpha_08_fire_spark_rows_positive() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 64;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.4)];
        let mut particles = WorldFireParticleFrame::default();
        let cam = ExtractedCameraMetrics {
            zoom_level: 1.0,
            zoom_alpha: 0.8,
            ..Default::default()
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert!(
            particles.spark_witness.rows > 0,
            "expected fire spark rows > 0 at tactical zoom_alpha=0.8, got {}",
            particles.spark_witness.rows
        );
        assert!((particles.spark_witness.zoom_alpha - 0.8).abs() < 1e-4);
    }

    #[test]
    fn strategic_zoom_culls_fire_spark_rows() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 64;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.4)];
        let mut particles = WorldFireParticleFrame::default();
        let cam = ExtractedCameraMetrics {
            zoom_level: 0.08,
            zoom_alpha: 0.05,
            ..Default::default()
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert_eq!(particles.instances.len(), 0);
        assert_eq!(particles.spark_witness.rows, 0);
        assert!(particles.spark_witness.view_culled);
    }

    #[test]
    fn witness_tracks_scatter_slots_for_hot_cell() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 64;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.4)];
        let mut particles = WorldFireParticleFrame::default();
        let cam = ExtractedCameraMetrics {
            zoom_level: 1.0,
            zoom_alpha: 0.72,
            ..Default::default()
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert!(particles.spark_witness.scatter_slots >= 3);
        assert_eq!(particles.spark_witness.rows, particles.instances.len());
    }

    #[test]
    fn witness_phase_reflects_compute_gate() {
        use crate::render::fire_vfx::witness::fire_spark_compute_enabled;
        let enabled = fire_spark_compute_enabled();
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 8;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.4)];
        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            None,
            ExtractedCameraMetrics::default(),
            None,
        );
        if enabled {
            assert_eq!(particles.spark_witness.phase, "A+B");
        } else {
            assert_eq!(particles.spark_witness.phase, "A");
        }
    }

    #[test]
    fn witness_stamps_rows_and_zoom_alpha() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 64;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.4)];
        let mut particles = WorldFireParticleFrame::default();
        let cam = ExtractedCameraMetrics {
            zoom_level: 1.2,
            zoom_alpha: 0.66,
            ..Default::default()
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert_eq!(particles.spark_witness.rows, particles.instances.len());
        assert!((particles.spark_witness.zoom_alpha - 0.66).abs() < 1e-4);
        assert_eq!(particles.spark_witness.scatter_max, FIRE_SPARK_SCATTER_MAX);
        assert!(!particles.spark_witness.view_culled);
    }
}
