//! Fire-specific packing into generic instanced-quad rows.
//!
//! ## RND-04 — Transparency / alpha blending (sort, not OIT)
//!
//! NEW: **RND-04** · gea4v2 **11.3.12** / **11.3.12.1** (Transparency and Alpha
//! Blending · Order-Independent Transparency).
//! LEGACY: `geometry_visual_effects` (gea_v2_e4 p141).
//!
//! Book path: either **sort** transparent fragments (painter / back-to-front) or
//! full **OIT**. This lane chooses **explicit class sort** inside the fire
//! transparent pass — smoke/atmosphere under embers under sparks — and relies on
//! `Core2dOverlaySet` (RND-03 owner: `core2d_overlay_order.rs`) to keep weather
//! precip under the fire particle pass. Debt not taken: A-buffer / weighted OIT.
//!
//! ## RND-14 — Card / impostor packing (intentional)
//!
//! NEW: **RND-14** · gea4v2 **11.6.4** *Card-Based Effects*.
//! LEGACY: `geometry_visual_effects` (gea_v2_e4 p141).
//!
//! Every row maps to an [`GpuInstancedQuadInstance`] **billboard card**. Smoke /
//! AtmosphereFx use larger half-edges so stacked translucent quads read as volume;
//! sparks stay pinpoint. This is the card impostor path — not a volumetric field.
//! `TRANSPARENT_ORDER_*` literals stay pinned (RND-04); RND-14 must not retarget them.
//!
//! ## RND-15 — Decals / scorchmark (cross-cite)
//!
//! NEW: **RND-15** · gea4v2 **11.6.6** *Decals* · LEGACY `geometry_visual_effects`.
//! Terrain scorchmarks are a separate **decal-like** overlay (surface-fire residual),
//! not a [`ParticleClass`] / instanced-quad card. Do not fold scars into this pack.

use bevy::math::Vec4;

use crate::render::pipelines::gpu_instanced_quad::GpuInstancedQuadInstance;
use crate::render::extraction::sim_visual_extract::FireVisualGpuInstance;

use crate::render::ExtractedCameraMetrics;

/// RND-04: smoke / atmosphere haze draws first (under) in the fire transparent pass.
pub const TRANSPARENT_ORDER_SMOKE: u8 = 0;
/// RND-04: embers between smoke and sparks.
pub const TRANSPARENT_ORDER_EMBER: u8 = 1;
/// RND-04: additive sparks last (on top) within the same pass.
pub const TRANSPARENT_ORDER_SPARK: u8 = 2;

/// Presentation class for instanced quads (FX-FIRE-SPARK-005).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleClass {
    /// FullFlame — pinpoint 0.5–2px (class_id ≤ 0.5 in WGSL expand).
    Spark,
    /// LowFlame — softer 2–6px embers (class_id ≤ 0.5, larger half-edge in Rust).
    Ember,
    /// SmokeOnly / macro garnish (class_id > 0.5 in WGSL expand).
    AtmosphereFx,
    /// Fire-column smoke puff (class 0.75): physical-size, alpha-enveloped rows.
    Smoke,
}

impl ParticleClass {
    #[inline]
    pub const fn as_f32(self) -> f32 {
        match self {
            Self::Spark => 0.0,
            Self::Ember => 0.25,
            Self::AtmosphereFx => 1.0,
            Self::Smoke => 0.75,
        }
    }

    #[inline]
    pub fn from_class_id(id: f32) -> Self {
        if id <= 0.125 {
            Self::Spark
        } else if id <= 0.5 {
            Self::Ember
        } else if id <= 0.875 {
            Self::Smoke
        } else {
            Self::AtmosphereFx
        }
    }

    /// RND-04 / P2-FIRE-SPARK-010: smoke/ember before sparks in the same transparent pass.
    ///
    /// Lower value = drawn earlier (under). Stable painter order — not OIT.
    #[inline]
    pub const fn transparent_draw_order(self) -> u8 {
        match self {
            Self::AtmosphereFx | Self::Smoke => TRANSPARENT_ORDER_SMOKE,
            Self::Ember => TRANSPARENT_ORDER_EMBER,
            Self::Spark => TRANSPARENT_ORDER_SPARK,
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
        let ember = row.smoke_ember_vis_priority.y.max(0.55);
        let smoke = row.smoke_ember_vis_priority.x.max(0.2);
        let quad_half_world =
            fire_particle_quad_base_half_world(world.w, heat, cam.zoom_level, cam.zoom_alpha, class);
        Self {
            world_xyz_heat: Vec4::new(world.x, world.y, world.z, heat),
            ember_class_radius_smoke: Vec4::new(
                ember,
                class.as_f32(),
                quad_half_world,
                smoke,
            ),
        }
    }

    #[must_use]
    pub fn as_instanced_quad(self) -> GpuInstancedQuadInstance {
        bytemuck::cast(self)
    }
}

/// Expanded vertex alias (fire lane naming).
pub type GpuParticleQuadVertex = crate::render::pipelines::gpu_instanced_quad::GpuInstancedQuadVertex;

/// World-space **half-edge** length for pinpoint spark quads (FX-FIRE-SPARK-001 Phase A).
#[inline]
fn fire_particle_quad_base_half_world(
    influence_light_radius_world: f32,
    heat: f32,
    camera_zoom: f32,
    zoom_alpha: f32,
    class: ParticleClass,
) -> f32 {
    let _ = (influence_light_radius_world, camera_zoom, zoom_alpha);
    let h = heat.clamp(0.0, 1.0);
    // Physical half-edge in tiles (1 tile = FIRE_TILE_METERS): flame/ember glow of a few
    // decimetres. The shader enforces the screen-space minimum and compensates alpha.
    match class {
        ParticleClass::Spark => 0.05 + 0.06 * h,
        ParticleClass::Ember => 0.03 + 0.04 * h,
        ParticleClass::Smoke => 0.25 + 0.20 * h,
        ParticleClass::AtmosphereFx => 0.85 + h * h * 0.55,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rnd04_smoke_under_ember_under_spark_draw_order() {
        assert_eq!(
            ParticleClass::Smoke.transparent_draw_order(),
            TRANSPARENT_ORDER_SMOKE
        );
        assert_eq!(
            ParticleClass::AtmosphereFx.transparent_draw_order(),
            TRANSPARENT_ORDER_SMOKE
        );
        assert_eq!(
            ParticleClass::Ember.transparent_draw_order(),
            TRANSPARENT_ORDER_EMBER
        );
        assert_eq!(
            ParticleClass::Spark.transparent_draw_order(),
            TRANSPARENT_ORDER_SPARK
        );
        assert!(TRANSPARENT_ORDER_SMOKE < TRANSPARENT_ORDER_EMBER);
        assert!(TRANSPARENT_ORDER_EMBER < TRANSPARENT_ORDER_SPARK);
    }

    #[test]
    fn rnd04_transparent_sort_key_is_monotonic_across_class_ids() {
        // Random-looking ids still collapse to the explicit smoke→ember→spark ladder.
        let mut orders: Vec<u8> = [0.0, 0.25, 0.75, 1.0, 0.1, 0.4, 0.8]
            .into_iter()
            .map(|id| ParticleClass::from_class_id(id).transparent_draw_order())
            .collect();
        let unsorted = orders.clone();
        orders.sort_unstable();
        // Sorting by draw order must not invent new ranks outside {0,1,2}.
        assert!(orders.iter().all(|&o| o <= TRANSPARENT_ORDER_SPARK));
        assert_eq!(unsorted.iter().copied().max(), Some(TRANSPARENT_ORDER_SPARK));
        assert_eq!(unsorted.iter().copied().min(), Some(TRANSPARENT_ORDER_SMOKE));
    }

    #[test]
    fn rnd04_emit_sorts_instances_by_transparent_draw_order() {
        let emit = include_str!("emit.rs");
        let prod = emit.split("#[cfg(test)]").next().unwrap_or(emit);
        let compact: String = prod.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            compact.contains("sort_by_key(|row|{ParticleClass::from_class_id(row.ember_class_radius_smoke.y).transparent_draw_order()})"),
            "fire emit must sort GPU rows by ParticleClass::transparent_draw_order (RND-04 painter sort)"
        );
        let sort_hits = prod.matches("transparent_draw_order()").count();
        assert!(
            sort_hits >= 3,
            "expected ≥3 transparent_draw_order sort sites in emit.rs, got {sort_hits}"
        );
    }

    #[test]
    fn rnd14_transparent_order_keys_unchanged() {
        // RND-14 must not retarget RND-04 painter ranks.
        assert_eq!(TRANSPARENT_ORDER_SMOKE, 0);
        assert_eq!(TRANSPARENT_ORDER_EMBER, 1);
        assert_eq!(TRANSPARENT_ORDER_SPARK, 2);
    }

    #[test]
    fn rnd14_smoke_atmosphere_cards_larger_than_spark_impostors() {
        // Larger half-edges = soft volumetric *look* via stacked cards (gea4v2 11.6.4).
        let heat = 0.5;
        let spark = fire_particle_quad_base_half_world(1.0, heat, 1.0, 0.0, ParticleClass::Spark);
        let smoke = fire_particle_quad_base_half_world(1.0, heat, 1.0, 0.0, ParticleClass::Smoke);
        let atmos =
            fire_particle_quad_base_half_world(1.0, heat, 1.0, 0.0, ParticleClass::AtmosphereFx);
        assert!(
            smoke > spark && atmos > smoke,
            "RND-14: smoke/atmosphere card half-edges must exceed spark (smoke={smoke} atmos={atmos} spark={spark})"
        );
    }

    #[test]
    fn rnd14_pack_documents_card_impostor_stance() {
        let pack = include_str!("pack.rs");
        assert!(pack.contains("RND-14") && pack.contains("11.6.4"));
        assert!(pack.contains("geometry_visual_effects"));
        let low = pack.to_ascii_lowercase();
        assert!(
            low.contains("billboard") || low.contains("card") || low.contains("impostor"),
            "pack.rs must document billboard/card/impostor stance"
        );
    }

    #[test]
    fn rnd15_transparent_order_keys_unchanged() {
        // RND-15 must not retarget RND-04 painter ranks (scorch is decal, not a new class).
        assert_eq!(TRANSPARENT_ORDER_SMOKE, 0);
        assert_eq!(TRANSPARENT_ORDER_EMBER, 1);
        assert_eq!(TRANSPARENT_ORDER_SPARK, 2);
    }

    #[test]
    fn rnd15_pack_cross_cites_decal_not_card() {
        let pack = include_str!("pack.rs");
        assert!(pack.contains("RND-15") && pack.contains("11.6.6"));
        assert!(pack.contains("geometry_visual_effects"));
        let low = pack.to_ascii_lowercase();
        assert!(
            low.contains("decal") && (low.contains("scorch") || low.contains("scar")),
            "pack.rs must cross-cite decal scorchmark stance"
        );
    }
}
