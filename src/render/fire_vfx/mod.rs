//! Fire VFX frontend — projection → instanced-quad rows, scatter, spark tuning.
//!
//! Backend transport: [`crate::render::pipelines::gpu_instanced_quad`] + [`crate::render::pipelines::gpu_particle_draw`].
//!
//! ## RND-04 — Transparency layering
//!
//! NEW: **RND-04** · gea4v2 **11.3.12** / **11.3.12.1** · LEGACY `geometry_visual_effects`.
//! Smoke/atmosphere → ember → spark via [`ParticleClass::transparent_draw_order`]
//! (sort path; OIT deferred). Weather/cloud pass order: see `weather_vfx`.
//!
//! ## RND-14 — Card / impostor smoke (intentional)
//!
//! NEW: **RND-14** · gea4v2 **11.6.4** *Card-Based Effects* · LEGACY `geometry_visual_effects`
//! (gea_v2_e4 p141). Smoke and atmosphere rows are **instanced-quad billboard cards**
//! (camera-facing quads with soft alpha) — cheap volumetric *look*, not ray-marched
//! volumes. `TRANSPARENT_ORDER_*` stays RND-04-owned; do not retarget here.
//!
//! ## RND-15 — Decals / scorchmark (terrain-projected)
//!
//! NEW: **RND-15** · gea4v2 **11.6.6** *Decals* · LEGACY `geometry_visual_effects`
//! (gea_v2_e4 p141). Long burn scars are a **decal-like** terrain overlay (projected
//! tint / z-biased layer), fed by surface-fire residual /
//! [`crate::systems::fire::surface_fire_scorch_decal01`] and overlay heat below the
//! particle seed floor. Scorchmarks are **not** RND-14 billboard cards and must not
//! rewrite RND-04 `TRANSPARENT_ORDER_*`. See [`overlay_heat_scorch_decal01`].
//!
//! ## LGT-03 — Emissive glow vs particle self-illumination
//!
//! NEW: **LGT-03** · gea4v2 **12.5.10** / **12.7.2** · LEGACY `lighting_post` /
//! `post_processing`. **Pooled scene lights** come from sim
//! [`crate::systems::fire::fire_light_emission`] → extract → `RequestLocalLight`.
//! This module's ember/spark colour is **self-glow on cards**, not a second emissive
//! light writer. Tonemap / HDR post stays **LGT-06** `tonemap_hdr` (do not rewrite).

pub mod consumable;
pub mod emit;
pub mod frame;
pub mod pack;
pub mod particle_anim;
pub mod spark_tuning;

pub use consumable::{
    effect_spawn_hook_resolver_wired, resolve_effect_spawn, spawn_effect_consumable,
    EffectConsumable, EffectConsumableDef, EffectConsumableRegistry,
    EffectConsumableRegistryPlugin, EffectFieldId, EffectLane, EffectResolvedSpawn,
    EffectSpawnResolveContext, SpawnHookKind, EFFECT_REGISTRY_ROOT,
};
pub use emit::{
    emit_world_fire_particles_from_projection, overlay_heat_scorch_decal01,
    seed_world_fire_particles_from_overlay_heat, update_world_fire_particles_from_projection,
    view_aware_particle_cull_wired,
};
pub use frame::{WorldFireParticleFrame, WorldFireParticleGpuStorage};
pub use pack::{GpuParticleInstance, GpuParticleQuadVertex, ParticleClass};
pub use spark_tuning::{
    fire_spark_compute_enabled, FireSparkStats, FIRE_SPARK_FULL_SCATTER_PX_PER_TILE,
    FIRE_SPARK_MIN_PX_PER_TILE, FIRE_SPARK_OPERATIONAL_PLAY_PX_PER_TILE, FIRE_SPARK_SCATTER_MAX,
    FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA,
};

#[cfg(test)]
mod rnd15_decal_tests {
    //! RND-15 tripwires: scorchmark = terrain-projected decal (gea4v2 11.6.6), not particle cards.

    use std::path::PathBuf;

    use bevy::math::IVec2;

    use crate::render::CHUNK_FIRE_OVERLAY_DISPLAY_MIN;
    use crate::render::extraction::sim_visual_extract::FIRE_VISUAL_ACTIVE_HEAT_EPS;
    use crate::render::ExtractedCameraMetrics;

    use super::emit::{overlay_heat_scorch_decal01, seed_world_fire_particles_from_overlay_heat};
    use super::frame::WorldFireParticleFrame;
    use super::pack::{TRANSPARENT_ORDER_EMBER, TRANSPARENT_ORDER_SMOKE, TRANSPARENT_ORDER_SPARK};

    fn repo_src(rel: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    #[test]
    fn rnd15_docs_dual_cite_decal_path() {
        for rel in [
            "render/fire_vfx/mod.rs",
            "render/fire_vfx/emit.rs",
            "render/fire_vfx/pack.rs",
            "systems/fire/chunk_surface_fire.rs",
        ] {
            let text = repo_src(rel);
            assert!(
                text.contains("RND-15") && text.contains("11.6.6"),
                "{rel}: must dual-cite RND-15 / gea4v2 11.6.6"
            );
            assert!(
                text.contains("geometry_visual_effects"),
                "{rel}: must cite LEGACY geometry_visual_effects"
            );
            let low = text.to_ascii_lowercase();
            assert!(
                low.contains("decal"),
                "{rel}: must name decal stance"
            );
        }
    }

    #[test]
    fn rnd15_overlay_heat_scorch_band() {
        assert_eq!(overlay_heat_scorch_decal01(0.0), 0.0);
        assert_eq!(
            overlay_heat_scorch_decal01(FIRE_VISUAL_ACTIVE_HEAT_EPS * 0.5),
            0.0
        );
        let mid = FIRE_VISUAL_ACTIVE_HEAT_EPS
            + 0.5 * (CHUNK_FIRE_OVERLAY_DISPLAY_MIN - FIRE_VISUAL_ACTIVE_HEAT_EPS);
        let d = overlay_heat_scorch_decal01(mid);
        assert!((d - 0.5).abs() < 1e-4, "mid residual → ~0.5, got {d}");
        assert_eq!(
            overlay_heat_scorch_decal01(CHUNK_FIRE_OVERLAY_DISPLAY_MIN),
            1.0
        );
        assert_eq!(overlay_heat_scorch_decal01(0.9), 1.0);
    }

    #[test]
    fn rnd15_seed_skips_scorch_band_heat() {
        let mut heat = std::collections::HashMap::new();
        heat.insert(IVec2::ZERO, CHUNK_FIRE_OVERLAY_DISPLAY_MIN * 0.5);
        heat.insert(IVec2::new(1, 0), CHUNK_FIRE_OVERLAY_DISPLAY_MIN);
        let mut frame = WorldFireParticleFrame::default();
        seed_world_fire_particles_from_overlay_heat(&heat, &mut frame, ExtractedCameraMetrics::default());
        assert!(
            !frame.instances.is_empty(),
            "active-floor heat must still seed particle cards"
        );
        // Only the ≥ floor chunk contributes — residual stays decal-only.
        let mut residual_only = std::collections::HashMap::new();
        residual_only.insert(IVec2::ZERO, CHUNK_FIRE_OVERLAY_DISPLAY_MIN * 0.5);
        let mut scar_frame = WorldFireParticleFrame::default();
        seed_world_fire_particles_from_overlay_heat(
            &residual_only,
            &mut scar_frame,
            ExtractedCameraMetrics::default(),
        );
        assert!(
            scar_frame.instances.is_empty(),
            "RND-15 scorchmark band must not enter particle-card instances"
        );
    }

    #[test]
    fn rnd15_does_not_retarget_rnd04_transparent_order() {
        assert_eq!(TRANSPARENT_ORDER_SMOKE, 0);
        assert_eq!(TRANSPARENT_ORDER_EMBER, 1);
        assert_eq!(TRANSPARENT_ORDER_SPARK, 2);
    }

    #[test]
    fn rnd15_scorch_is_not_particle_class() {
        let emit = repo_src("render/fire_vfx/emit.rs");
        let prod = emit.split("#[cfg(test)]").next().unwrap_or(&emit);
        assert!(
            prod.contains("overlay_heat_scorch_decal01"),
            "emit must expose scorchmark/decal helper"
        );
        // Scorchmark must not invent a ParticleClass variant (docs may say "Decals").
        let pack = repo_src("render/fire_vfx/pack.rs");
        let pack_prod = pack.split("#[cfg(test)]").next().unwrap_or(&pack);
        let enum_block = pack_prod
            .split("enum ParticleClass")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .unwrap_or("");
        assert!(
            !enum_block.contains("Scorch") && !enum_block.contains("Decal"),
            "RND-15 must not add Scorch/Decal ParticleClass variant (decal ≠ card); got {enum_block}"
        );
    }
}

