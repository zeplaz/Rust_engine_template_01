//! **Phase F** — world-anchored GPU particle instances from post-LOD [`RenderProjectionGraph`].
//!
//! One upload path: per-view tactical [`crate::render::sim_visual_extract::FireVisualFrame`] → projection →
//! [`WorldFireParticleFrame`] (filtered by [`crate::render::FireChunkLodState`] when present) → registry buffer.

use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bytemuck::{Pod, Zeroable};

use crate::gui::{
    camera_zoom, map_zoom_alpha_with_limits, map_zoom_limits_for_world,
    GPU_FIRE_INSTANCE_BUDGET_CEILING, MapCameraDesired, RepresentationBand, ViewId, ViewManager,
};
use crate::render::{trace_particle_routing, DebugRenderTraceConfig};
use crate::render::extraction::RenderProjectionGraph;
use crate::render::{
    ChunkCoord, FireChunkLodState, FireLodBand,
};
use crate::render::sim_visual_extract::{FireVisualGpuInstance, FIRE_VISUAL_ACTIVE_HEAT_EPS};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use bevy::window::PrimaryWindow;

/// Presentation class for instanced quads (FX-FIRE-SPARK-005).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleClass {
    /// FullFlame — pinpoint 0.5–2px (class_id ≤ 0.5 in WGSL expand).
    Spark,
    /// LowFlame — softer 2–6px embers (class_id ≤ 0.5, larger half-edge in Rust).
    Ember,
    /// SmokeOnly / macro garnish (class_id > 0.5 in WGSL expand).
    AtmosphereFx,
}

impl ParticleClass {
    #[inline]
    const fn as_f32(self) -> f32 {
        match self {
            Self::Spark => 0.0,
            Self::Ember => 0.25,
            Self::AtmosphereFx => 1.0,
        }
    }

    #[inline]
    fn from_class_id(id: f32) -> Self {
        if id <= 0.125 {
            Self::Spark
        } else if id <= 0.5 {
            Self::Ember
        } else {
            Self::AtmosphereFx
        }
    }

    /// P2-FIRE-SPARK-010: smoke/ember draws before sparks in the same transparent pass.
    #[inline]
    const fn transparent_draw_order(self) -> u8 {
        match self {
            Self::AtmosphereFx => 0,
            Self::Ember => 1,
            Self::Spark => 2,
        }
    }
}

/// Live witness for FX-FIRE-SPARK-003 (stage5 / diagnostic JSON).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FireSparkWitness {
    pub phase: &'static str,
    pub rows: usize,
    pub scatter_max: usize,
    pub scatter_slots: usize,
    pub zoom_alpha: f32,
    pub additive_blend: bool,
    pub budget_capped: bool,
    pub view_culled: bool,
    pub projection_view: &'static str,
}

pub const FIRE_SPARK_SCATTER_MAX: usize = 14;
/// Below this zoom band, emit no GPU spark rows (strategic map — no spark flood).
pub const FIRE_SPARK_MIN_ZOOM_ALPHA: f32 = 0.28;
/// Full scatter density by this zoom band (tactical burn read).
pub const FIRE_SPARK_FULL_SCATTER_ZOOM_ALPHA: f32 = 0.58;
/// P2-FIRE-SPARK-011 / `--test visual` proof band (matches [`crate::gui::TACTICAL_VFX_PROOF_ZOOM_ALPHA`]).
pub const FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA: f32 = 0.85;
/// Legacy alias — tile fallback uses this for CPU heat boost cutoff.
pub const FIRE_SPARK_STRATEGIC_ZOOM_ALPHA: f32 = FIRE_SPARK_MIN_ZOOM_ALPHA;
/// When filled rows exceed this fraction of capacity, halve scatter per hot cell.
const FIRE_SPARK_BUDGET_PRESSURE: f32 = 0.85;

/// Phase B compute advection gate (`FIRE_SPARK_COMPUTE=0|false|off` disables).
#[inline]
#[must_use]
pub fn fire_spark_compute_enabled() -> bool {
    !matches!(
        std::env::var("FIRE_SPARK_COMPUTE").as_deref(),
        Ok("0") | Ok("false") | Ok("off")
    )
}

/// P2-FIRE-SPARK-011 — tactical shower read @ proof zoom (D-F07 / F-T03).
#[must_use]
pub fn fire_spark_011_green(w: &FireSparkWitness) -> bool {
    w.rows > 0
        && w.scatter_slots >= 3
        && w.zoom_alpha >= FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA * 0.75
        && !w.view_culled
        && w.additive_blend
}

#[inline]
fn fire_spark_witness_phase() -> &'static str {
    if fire_spark_compute_enabled() {
        "A+B"
    } else {
        "A"
    }
}

/// Camera scale for world-fire particles (synced before emission each frame).
#[derive(Resource, Clone, Copy, Debug)]
pub struct FireParticleCameraScale {
    pub camera_zoom: f32,
    pub zoom_alpha: f32,
}

impl Default for FireParticleCameraScale {
    fn default() -> Self {
        Self {
            camera_zoom: 1.0,
            zoom_alpha: 0.5,
        }
    }
}

/// World-space **half-edge** length for pinpoint spark quads (FX-FIRE-SPARK-001 Phase A).
///
/// [`FireVisualGpuInstance::world_xyz_radius`]`w` mirrors light falloff radius (often huge). Sparks use a
/// **screen-stable** 0.5–2 px half-edge — many low-α points, not large soft blobs (D-F01, D-F07).
#[inline]
fn fire_particle_quad_base_half_world(
    influence_light_radius_world: f32,
    heat: f32,
    camera_zoom: f32,
    zoom_alpha: f32,
    class: ParticleClass,
) -> f32 {
    let h = heat.clamp(0.0, 1.0);
    let _influence_cap =
        influence_light_radius_world.mul_add(0.004, h * h * 0.35).clamp(0.04, 1.2);
    let z = camera_zoom.max(0.06);
    let za = zoom_alpha.clamp(0.0, 1.0);
    let screen_half_spark = (0.5 + za * 1.5) * (0.38 + h * 0.62);
    let screen_half = match class {
        ParticleClass::Spark => screen_half_spark,
        ParticleClass::Ember => (screen_half_spark * 2.8).clamp(1.0, 6.0),
        ParticleClass::AtmosphereFx => screen_half_spark * 0.72,
    };
    let world_half = (screen_half / z).clamp(0.015, 1.5);
    world_half.max(0.015)
}

/// Packed instanced-quad row for the GPU particle buffer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct GpuParticleInstance {
    /// World `xyz`, `w` = heat `[0,1]`.
    pub world_xyz_heat: Vec4,
    /// `x` ember rate, `y` class ordinal, `z` **world half-edge** base for the billboard (Phase-F), `w` smoke density.
    pub ember_class_radius_smoke: Vec4,
}

impl GpuParticleInstance {
    #[must_use]
    pub fn from_fire_visual(
        row: &FireVisualGpuInstance,
        class: ParticleClass,
        cam: FireParticleCameraScale,
    ) -> Self {
        let world = row.world_xyz_radius;
        let heat = row.heat();
        let quad_half_world =
            fire_particle_quad_base_half_world(world.w, heat, cam.camera_zoom, cam.zoom_alpha, class);
        Self {
            world_xyz_heat: Vec4::new(world.x, world.y, world.z, heat),
            ember_class_radius_smoke: Vec4::new(
                row.smoke_ember_vis_priority.y,
                class.as_f32(),
                quad_half_world,
                row.smoke_ember_vis_priority.x,
            ),
        }
    }
}

/// One expanded billboard vertex for instanced world-fire quads (`WorldFireFx`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct GpuParticleQuadVertex {
    /// World `xy`, `z` = heat, `w` = ember.
    pub world_xy_heat_ember: Vec4,
}

impl GpuParticleQuadVertex {
    #[must_use]
    pub fn from_corner(world_x: f32, world_y: f32, heat: f32, ember: f32) -> Self {
        Self {
            world_xy_heat_ember: Vec4::new(world_x, world_y, heat, ember),
        }
    }
}

/// CPU-side particle snapshot for one committed sim step (LOD-shaped).
#[derive(Resource, Debug, Clone, ExtractResource)]
pub struct WorldFireParticleFrame {
    pub snapshot_stamp: u64,
    /// Wall clock for shader pulse (avoid using `snapshot_stamp` — it advances with sim, not frames).
    pub anim_time_secs: f32,
    pub active_band: RepresentationBand,
    pub gpu_capacity: usize,
    pub instances: Vec<GpuParticleInstance>,
    pub spark_witness: FireSparkWitness,
}

impl Default for WorldFireParticleFrame {
    fn default() -> Self {
        Self {
            snapshot_stamp: 0,
            anim_time_secs: 0.0,
            active_band: RepresentationBand::Full,
            gpu_capacity: usize::MAX,
            instances: Vec::new(),
            spark_witness: FireSparkWitness::default(),
        }
    }
}

/// Render-world view of the latest particle upload (count only).
#[derive(Resource, Default)]
pub struct WorldFireParticleGpuStorage {
    pub instance_count: u32,
    pub expanded_vertex_count: u32,
}

pub fn sync_fire_particle_camera_scale(
    desired: Option<Res<MapCameraDesired>>,
    view_manager: Option<Res<ViewManager>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    params: Option<Res<WorldGenParams>>,
    mut out: ResMut<FireParticleCameraScale>,
) {
    // INFRA-VM09-001: prefer ViewManager WorldMain zoom over raw MapCameraDesired.
    let z = view_manager
        .as_deref()
        .and_then(|m| camera_zoom(m, ViewId::WorldMain))
        .or_else(|| desired.as_deref().map(|d| d.scale.x))
        .unwrap_or(1.0)
        .max(0.06);
    let vp = windows
        .single()
        .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or(Vec2::ONE);
    let (lo, hi) = params
        .as_deref()
        .map(|p| map_zoom_limits_for_world(p.width as f32, p.height as f32, vp))
        .unwrap_or((0.08, 4.0));
    out.camera_zoom = z;
    out.zoom_alpha = map_zoom_alpha_with_limits(z, lo, hi);
}

pub fn emit_world_fire_particles_from_projection(
    time: Res<Time>,
    cfg: Option<Res<DebugRenderTraceConfig>>,
    graph: Res<RenderProjectionGraph>,
    chunk_lod: Res<FireChunkLodState>,
    cam: Res<FireParticleCameraScale>,
    view_manager: Option<Res<crate::gui::ViewManager>>,
    mut frame: ResMut<WorldFireParticleFrame>,
    mut last_trace: Local<u64>,
) {
    update_world_fire_particles_from_projection(
        graph.as_ref(),
        frame.as_mut(),
        Some(chunk_lod.as_ref()),
        *cam,
        view_manager.as_deref(),
    );
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

#[inline]
fn fire_lod_band_for_instance_row(row: &FireVisualGpuInstance, chunk_lod: Option<&FireChunkLodState>) -> FireLodBand {
    let xy = row.chunk_grid_xy();
    let c = ChunkCoord::new(xy.x as i32, xy.y as i32);
    chunk_lod
        .and_then(|s| s.bands.get(&c).copied())
        .unwrap_or(FireLodBand::FullFlame)
}

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
fn fire_spark_zoom_scatter_gate(zoom_alpha: f32) -> f32 {
    let za = zoom_alpha.clamp(0.0, 1.0);
    let span = (FIRE_SPARK_FULL_SCATTER_ZOOM_ALPHA - FIRE_SPARK_MIN_ZOOM_ALPHA).max(1e-4);
    ((za - FIRE_SPARK_MIN_ZOOM_ALPHA) / span).clamp(0.0, 1.0)
}

#[inline]
fn fire_particle_scatter_count(heat: f32, zoom_alpha: f32, budget_pressure: f32) -> usize {
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

/// Shapes a projected GPU row for particle emission from authoritative [`FireLodBand`].
#[must_use]
fn shape_fire_row_for_particle_lod(
    mut row: FireVisualGpuInstance,
    band: FireLodBand,
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
            Some((row, ParticleClass::Ember))
        }
        FireLodBand::FullFlame => Some((row, ParticleClass::Spark)),
    }
}

pub fn update_world_fire_particles_from_projection(
    graph: &RenderProjectionGraph,
    frame: &mut WorldFireParticleFrame,
    chunk_lod: Option<&FireChunkLodState>,
    cam: FireParticleCameraScale,
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
        0
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
        let Some((shaped, class)) = shape_fire_row_for_particle_lod(*row, band) else {
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
    // F2-PR-3: chunk_heat bootstrap only when projection graph instance_buffer is empty.
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
            row.world_xyz_radius = Vec4::new(
                ch.chunk.x as f32 * 64.0 + 32.0,
                ch.chunk.y as f32 * 64.0 + 32.0,
                0.0,
                28.0,
            );
            row.smoke_ember_vis_priority = Vec4::new(0.12, 0.55, 0.0, 1.0);
            if let Some((shaped, class)) =
                shape_fire_row_for_particle_lod(row, FireLodBand::FullFlame)
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
    cam: FireParticleCameraScale,
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
        row.world_xyz_radius = Vec4::new(
            coord.x as f32 * 64.0 + 32.0,
            coord.y as f32 * 64.0 + 32.0,
            0.0,
            28.0,
        );
        row.smoke_ember_vis_priority = Vec4::new(0.12, 0.55, 0.0, 1.0);
        let Some((shaped, class)) = shape_fire_row_for_particle_lod(row, FireLodBand::FullFlame) else {
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
            FireParticleCameraScale::default(),
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
        let cam = FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: 0.72,
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
        let cam = FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: 0.72,
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
            FireParticleCameraScale::default(),
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
            FireParticleCameraScale::default(),
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
        let cam = FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: 0.72,
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
            FireParticleCameraScale::default(),
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
            FireParticleCameraScale::default(),
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
        assert_eq!(fire_particle_scatter_count(0.95, 0.2, 0.0), 0);
    }

    #[test]
    fn ember_class_larger_quad_half_than_spark() {
        let mut row = FireVisualGpuInstance::default();
        row.world_xyz_radius = Vec4::new(0.0, 0.0, 0.0, 120.0);
        row.chunk_xy_heat_lum = Vec4::new(0.0, 0.0, 0.75, 0.6);
        let cam = FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: 0.8,
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
    fn low_flame_lod_routes_ember_class() {
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
            FireParticleCameraScale::default(),
            None,
        );
        assert!(
            (particles.instances[0].ember_class_radius_smoke.y - ParticleClass::Ember.as_f32()).abs()
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

    /// P2-FIRE-SPARK-011 — shower read at tactical proof zoom (0.85).
    #[test]
    fn p2_fire_spark_011_at_tactical_proof_zoom() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 256;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.5)];
        let mut particles = WorldFireParticleFrame::default();
        let cam = FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert!(
            fire_spark_011_green(&particles.spark_witness),
            "P2-FIRE-SPARK-011 witness: {:?}",
            particles.spark_witness
        );
    }

    /// P2-VFX-WITNESS-001 W-1 — tactical zoom must emit particle rows.
    #[test]
    fn p2_tactical_zoom_alpha_08_fire_spark_rows_positive() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 64;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.4)];
        let mut particles = WorldFireParticleFrame::default();
        let cam = FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: 0.8,
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
        let cam = FireParticleCameraScale {
            camera_zoom: 0.08,
            zoom_alpha: 0.05,
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
        let cam = FireParticleCameraScale {
            camera_zoom: 1.0,
            zoom_alpha: 0.72,
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert!(particles.spark_witness.scatter_slots >= 3);
        assert_eq!(particles.spark_witness.rows, particles.instances.len());
    }

    #[test]
    fn witness_phase_reflects_compute_gate() {
        let enabled = fire_spark_compute_enabled();
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 8;
        graph.fire.instance_buffer = vec![sample_fire_row(IVec2::ZERO, 0.85, 0.4)];
        let mut particles = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(
            &graph,
            &mut particles,
            None,
            FireParticleCameraScale::default(),
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
        let cam = FireParticleCameraScale {
            camera_zoom: 1.2,
            zoom_alpha: 0.66,
        };
        update_world_fire_particles_from_projection(&graph, &mut particles, None, cam, None);
        assert_eq!(particles.spark_witness.rows, particles.instances.len());
        assert!((particles.spark_witness.zoom_alpha - 0.66).abs() < 1e-4);
        assert_eq!(particles.spark_witness.scatter_max, FIRE_SPARK_SCATTER_MAX);
        assert!(!particles.spark_witness.view_culled);
    }
}
