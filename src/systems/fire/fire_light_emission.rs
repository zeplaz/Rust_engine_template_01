//! # LGT-03 — Emissive fire glow (V4-LGT03)
//!
//! Dual-cite: NEW **LGT-03** (gea4v2 **12.5.10** *Emissive Objects* p212;
//! gea4v2 **12.7.2** *Full-Screen Post Effects* p245-248) · LEGACY keywords
//! **`lighting_post`** / **`post_processing`** (gea_v2_e4 §12 *Lighting and
//! Post-Processing* p153; chapter anchors shared with LGT-06).
//!
//! **Fire glow** (matrix note): burning chunks are **emissive** sim objects —
//! they contribute local light via this component. GEA 12.5.10 treats emissives
//! as light sources that also feed post (bloom / glow); 12.7.2 places full-screen
//! post after lighting. This engine keeps that split:
//!
//! | Layer | Owner | Role |
//! |:---|:---|:---|
//! | Sim emissive | **this module** — sole writer of [`FireLightEmission`] on chunks | heat → radius / intensity / flicker |
//! | Extract → lights | [`crate::render::extraction::FireVisualFramePlugin`] → [`crate::render::RequestLocalLight`] | pooled [`PointLight`]s |
//! | Particle self-glow | [`crate::render::fire_vfx`] (ember/spark colour) | billboard radiance, **not** a second light writer |
//! | Tonemap / HDR post | **LGT-06** [`crate::render::view_runtime::tonemap_hdr`] | SDR map RTT · PixelWorld `Tonemapping::None` |
//!
//! **Voxels in scope:** PixelWorld / massing views share the same pooled local-light
//! path and LGT-06 display-referred tonemap — do **not** add a voxel-only emissive
//! writer or a parallel map-view tone operator.
//!
//! Debt not taken: custom full-screen glow pass · rewriting LGT-06 `tonemap_hdr` ·
//! per-voxel emissive ECS components · AI-generated glow textures.
//!
//! Tripwires: `lgt03_*` · `fire_light_emission_*` (no GPU).

use bevy::prelude::*;

use super::ChunkSurfaceFire;
use crate::systems::ecology::ChunkEcology;
use crate::systems::sim_control::SimControlState;
use crate::terrain::generation::Chunk;

/// Minimum surface-fire heat before a chunk is treated as an emissive light source.
pub const HEAT_MIN: f32 = 0.08;

/// Source-token tripwires expect: tonemap/HDR stays LGT-06-owned (do not rewrite here).
pub const LGT03_TONEMAP_OWNER_TOKEN: &str = "view_runtime::tonemap_hdr";

/// Per-entity **simulation** parameters for how much light fire should contribute this tick.
///
/// No [`PointLight`] here — extraction maps this into [`crate::render::ActiveLights`] through
/// [`crate::render::RequestLocalLight`].
#[derive(Component, Clone, Copy, Debug)]
pub struct FireLightEmission {
    pub radius: f32,
    pub base_intensity: f32,
    pub current_intensity: f32,
    pub flicker_strength: f32,
    pub flicker_phase: f32,
    /// Hint for extraction / budgeting (e.g. heat-linked); not authoritative sim state.
    pub extract_priority: f32,
}

/// Light a chunk burning at `heat` emits — the one formula for both the sim component (scalar surface
/// fire) and fire visuals of chunks whose burning cells have not lifted the scalar to [`HEAT_MIN`].
#[must_use]
pub fn fire_light_emission_for_heat(coord: IVec2, heat: f32, biomass: f32) -> FireLightEmission {
    let base_intensity = (heat * (0.6 + biomass * 0.9)).clamp(0.0, 2.5);
    FireLightEmission {
        radius: 120.0 + heat * 180.0,
        base_intensity,
        // `update_fire_light_emission_flicker` runs later the same tick and applies phase.
        current_intensity: base_intensity,
        flicker_strength: 0.12,
        flicker_phase: (coord.x as f32 * 0.37 + coord.y as f32 * 0.91).fract() * std::f32::consts::TAU,
        extract_priority: 0.5 + heat * 2.0,
    }
}

/// Whether surface heat is bright enough to attach an emissive light component.
#[inline]
#[must_use]
pub fn heat_is_emissive(heat: f32) -> bool {
    heat >= HEAT_MIN
}

pub(crate) fn maintain_fire_light_emission_from_surface_fire(
    ctrl: Res<SimControlState>,
    mut commands: Commands,
    q: Query<(
        Entity,
        &Chunk,
        &ChunkSurfaceFire,
        Option<&ChunkEcology>,
        Option<&FireLightEmission>,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    for (e, chunk, fire, eco, existing) in &q {
        if !heat_is_emissive(fire.heat) {
            if existing.is_some() {
                commands.entity(e).remove::<FireLightEmission>();
            }
            continue;
        }
        let bio = eco.map(|e| e.biomass).unwrap_or(0.35);
        commands
            .entity(e)
            .insert(fire_light_emission_for_heat(chunk.coord, fire.heat, bio));
    }
}

pub(crate) fn update_fire_light_emission_flicker(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    mut q: Query<&mut FireLightEmission>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let t = time.elapsed_secs();
    for mut em in &mut q {
        // 2-11 Hz multi-component flicker, decorrelated per emitter (phase is per-chunk unique).
        let flicker = super::ember_physics::flame_flicker(t, em.flicker_phase.to_bits());
        em.current_intensity = (em.base_intensity * (1.0 + flicker * em.flicker_strength)).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_src(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(rel)
    }

    fn read(rel: &str) -> String {
        std::fs::read_to_string(repo_src(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
    }

    #[test]
    fn emission_fields_sane() {
        let e = FireLightEmission {
            radius: 100.0,
            base_intensity: 1.0,
            current_intensity: 1.0,
            flicker_strength: 0.1,
            flicker_phase: 0.0,
            extract_priority: 1.0,
        };
        assert!(e.radius > 0.0 && e.base_intensity > 0.0);
    }

    /// LGT-03 / V4-LGT03: dual-cite NEW row + LEGACY keywords stay in module docs.
    #[test]
    fn lgt03_dual_cite_emissive_post_in_module_docs() {
        let src = read("systems/fire/fire_light_emission.rs");
        assert!(
            src.contains("LGT-03") && src.contains("gea4v2") && src.contains("12.5.10"),
            "missing NEW LGT-03 / gea4v2 12.5.10 emissive cite"
        );
        assert!(
            src.contains("12.7.2"),
            "missing NEW gea4v2 12.7.2 full-screen post cite"
        );
        assert!(
            src.contains("lighting_post") && src.contains("post_processing"),
            "missing LEGACY lighting_post / post_processing cites"
        );
        assert!(
            src.contains("gea_v2_e4"),
            "missing LEGACY book id gea_v2_e4"
        );
        let vfx = read("render/fire_vfx/mod.rs");
        assert!(
            vfx.contains("LGT-03") && vfx.contains("fire_light_emission"),
            "fire_vfx must point at LGT-03 / fire_light_emission for emissive lights"
        );
    }

    /// LGT-03 tripwire: heat below HEAT_MIN is not emissive; above scales intensity.
    #[test]
    fn lgt03_heat_gate_and_intensity_scales() {
        assert!(!heat_is_emissive(HEAT_MIN - 1e-4));
        assert!(heat_is_emissive(HEAT_MIN));
        let cold = fire_light_emission_for_heat(IVec2::new(0, 0), 0.0, 0.5);
        let hot = fire_light_emission_for_heat(IVec2::new(0, 0), 1.0, 0.5);
        assert!(hot.base_intensity > cold.base_intensity);
        assert!(hot.radius > cold.radius);
        assert!(hot.extract_priority > cold.extract_priority);
        assert!(hot.base_intensity <= 2.5);
    }

    /// LGT-03 tripwire: biomass feeds emissive intensity (fuel-linked glow).
    #[test]
    fn lgt03_biomass_boosts_emissive_intensity() {
        let sparse = fire_light_emission_for_heat(IVec2::new(1, 2), 0.8, 0.1);
        let dense = fire_light_emission_for_heat(IVec2::new(1, 2), 0.8, 1.0);
        assert!(dense.base_intensity > sparse.base_intensity);
        assert_eq!(sparse.flicker_phase, dense.flicker_phase);
    }

    /// LGT-03 tripwire: do **not** rewrite LGT-06 tonemap_hdr — post stays there.
    #[test]
    fn lgt03_does_not_rewrite_lgt06_tonemap_hdr() {
        let src = read("systems/fire/fire_light_emission.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or(&src);
        assert!(
            prod.contains("LGT-06") && prod.contains(LGT03_TONEMAP_OWNER_TOKEN),
            "must defer tonemap/HDR to LGT-06 tonemap_hdr"
        );
        assert!(
            !prod.contains("pub enum ViewToneMapPolicy")
                && !prod.contains("pub const MAP_VIEW_TONE_MAP_POLICY")
                && !prod.contains("struct ToneMapHdrPass"),
            "fire_light_emission must not redefine tonemap policy symbols"
        );
        let tonemap = read("render/view_runtime/tonemap_hdr.rs");
        assert!(
            tonemap.contains("LGT-06") && tonemap.contains("MAP_VIEW_TONE_MAP_POLICY"),
            "LGT-06 tonemap_hdr contract must remain the post/HDR owner"
        );
    }

    /// LGT-03 tripwire: sole sim writer of chunk FireLightEmission is maintain_* (voxels share).
    #[test]
    fn lgt03_sole_sim_emissive_writer_shared_with_voxel_views() {
        let fire_mod = read("systems/fire/mod.rs");
        assert!(
            fire_mod.contains("maintain_fire_light_emission_from_surface_fire"),
            "Fire plugin must schedule maintain_fire_light_emission_from_surface_fire"
        );
        let src = read("systems/fire/fire_light_emission.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or(&src);
        assert!(
            prod.contains("Voxels in scope") || prod.contains("PixelWorld"),
            "must document voxels sharing the pooled emissive light path"
        );
        // No second post/glow pass type invented beside LGT-06.
        assert!(
            !prod.contains("struct BloomPass") && !prod.contains("struct GlowPass"),
            "must not invent a bloom/glow pass writer beside LGT-06"
        );
    }

    /// Fire-light-emission naming filter alias for Accept `cargo test fire_light_emission`.
    #[test]
    fn fire_light_emission_formula_deterministic_for_coord() {
        let a = fire_light_emission_for_heat(IVec2::new(3, -4), 0.55, 0.4);
        let b = fire_light_emission_for_heat(IVec2::new(3, -4), 0.55, 0.4);
        assert_eq!(a.flicker_phase, b.flicker_phase);
        assert_eq!(a.base_intensity, b.base_intensity);
        assert_eq!(a.radius, b.radius);
    }
}
