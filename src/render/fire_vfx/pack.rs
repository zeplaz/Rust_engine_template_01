//! Fire-specific packing into generic instanced-quad rows.

use bevy::math::Vec4;

use crate::render::gpu_instanced_quad::GpuInstancedQuadInstance;
use crate::render::sim_visual_extract::FireVisualGpuInstance;

use crate::render::ExtractedCameraMetrics;

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
    pub const fn as_f32(self) -> f32 {
        match self {
            Self::Spark => 0.0,
            Self::Ember => 0.25,
            Self::AtmosphereFx => 1.0,
        }
    }

    #[inline]
    pub fn from_class_id(id: f32) -> Self {
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
    pub const fn transparent_draw_order(self) -> u8 {
        match self {
            Self::AtmosphereFx => 0,
            Self::Ember => 1,
            Self::Spark => 2,
        }
    }
}

/// Fire-packed alias — same 32-byte layout as [`GpuInstancedQuadInstance`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuParticleInstance {
    pub world_xyz_heat: Vec4,
    pub ember_class_radius_smoke: Vec4,
}

impl GpuParticleInstance {
    #[must_use]
    pub fn from_fire_visual(
        row: &FireVisualGpuInstance,
        class: ParticleClass,
        cam: ExtractedCameraMetrics,
    ) -> Self {
        let world = row.world_xyz_radius;
        let heat = row.heat();
        let quad_half_world =
            fire_particle_quad_base_half_world(world.w, heat, cam.zoom_level, cam.zoom_alpha, class);
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

    #[must_use]
    pub fn as_instanced_quad(self) -> GpuInstancedQuadInstance {
        bytemuck::cast(self)
    }
}

/// Expanded vertex alias (fire lane naming).
pub type GpuParticleQuadVertex = crate::render::gpu_instanced_quad::GpuInstancedQuadVertex;

/// World-space **half-edge** length for pinpoint spark quads (FX-FIRE-SPARK-001 Phase A).
#[inline]
fn fire_particle_quad_base_half_world(
    influence_light_radius_world: f32,
    heat: f32,
    camera_zoom: f32,
    zoom_alpha: f32,
    class: ParticleClass,
) -> f32 {
    let h = heat.clamp(0.0, 1.0);
    let _ = (camera_zoom, zoom_alpha);
    // World-locked half-edge — scales with tactical zoom like chunk footprints (not screen-stabilized).
    let chunk_span = influence_light_radius_world.max(8.0);
    let base = chunk_span * 0.045 + h * h * 0.22;
    match class {
        ParticleClass::Spark => base.clamp(0.35, chunk_span * 0.55),
        ParticleClass::Ember => (base * 1.85).clamp(0.6, chunk_span * 0.85),
        ParticleClass::AtmosphereFx => (base * 0.72).clamp(0.25, chunk_span * 0.45),
    }
}
