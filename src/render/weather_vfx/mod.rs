//! EFFECTS-SYSTEM ES-5 — weather precip GPU frontend + Core2d draw.
//!
//! **Authority:** `ClimateVisualAggregate` → zoom-banded streaks →
//! `WEATHER_PRECIP_INSTANCES_BUFFER` → [`draw`] vertex-pull raster.
//!
//! ES-5-001 fill · ES-5-002 upload/draw · ES-5-3 mesh retired · ES-5-4 provisional zoom.
//! Reject: Hanabi as world-scale rain (ES-7 event puffs remain parallel-OK).
//!
//! ## RND-04 — Smoke / cloud / fire transparency stack
//!
//! NEW: **RND-04** · gea4v2 **11.3.12** / **11.3.12.1** · LEGACY `geometry_visual_effects`.
//! Explicit order (sort path, not OIT): cloud sheet Z strata → precip alpha pass
//! (`WeatherPrecipRaster`) → fire particles sorted smoke→ember→spark
//! (`FireParticleRaster`). Overlay chain file owned by RND-03.
//!
//! ## RND-14 — Card / impostor volumetrics (intentional)
//!
//! NEW: **RND-14** · gea4v2 **11.6.4** *Card-Based Effects* · LEGACY `geometry_visual_effects`
//! (gea_v2_e4 p141). Cloud decks are soft **card** sprites (`Rectangle` + alpha puff
//! texture + `cloud_sheets::CLOUD_SHEET_LAYER_Z` strata); fire smoke/atmosphere are
//! **instanced-quad billboard cards** (see `fire_vfx`). This is the cheap volumetric
//! *look* — not ray-marched volumes (LGT-04 remains presentation debt). Do **not**
//! retarget `TRANSPARENT_ORDER_*` / `CLOUD_SHEET_LAYER_Z` (RND-04 locks).

pub mod cloud_sheets;
pub mod draw;
pub mod extract;
pub mod frame;

pub use draw::WeatherPrecipDrawStatus;
pub use extract::{
    apply_weather_precip_frame_from_climate, apply_weather_precip_frame_from_climate_at,
    build_precip_streak_instances, build_precip_streak_instances_at, WeatherPrecipZoomBand,
    WEATHER_PRECIP_STREAK_CAP,
};
pub use frame::WeatherPrecipFrame;


use bevy::prelude::*;

use crate::render::particle_domains::ParticleDomainRegistry;
use crate::render::ClimateVisualAggregate;

/// Registers [`WeatherPrecipFrame`] + climate fill + GPU draw.
pub struct WeatherVfxPlugin;

impl Plugin for WeatherVfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherPrecipFrame>();
        app.init_resource::<ClimateVisualAggregate>();
        app.init_resource::<ParticleDomainRegistry>();
        extract::register_weather_precip_extract(app);
        draw::register_weather_precip_draw(app);
        cloud_sheets::register_cloud_sheets(app);
    }
}

#[cfg(test)]
mod rnd04_transparency_tests {
    //! RND-04 tripwires: smoke/cloud/fire alpha layering stays explicit (sort ≠ OIT).

    use std::path::PathBuf;

    fn repo_src(rel: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn production_src(text: &str) -> &str {
        text.split("#[cfg(test)]").next().unwrap_or(text)
    }

    #[test]
    fn rnd04_weather_precip_uses_alpha_blend_without_depth_write() {
        let owned = repo_src("render/weather_vfx/draw.rs");
        let src = production_src(&owned);
        let compact: String = src.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            compact.contains("src_factor:BlendFactor::SrcAlpha")
                && compact.contains("dst_factor:BlendFactor::OneMinusSrcAlpha"),
            "weather precip must use SrcAlpha/OneMinusSrcAlpha (RND-04 alpha blend)"
        );
        assert!(
            compact.contains("depth_write_enabled:Some(false)"),
            "weather precip must disable depth write for transparent overlay (RND-04)"
        );
        assert!(
            compact.contains("Core2dOverlaySet::WeatherPrecipRaster"),
            "weather precip must sit in WeatherPrecipRaster overlay set"
        );
    }

    #[test]
    fn rnd04_overlay_chain_keeps_weather_under_fire() {
        // Read-only check of RND-03-owned file — do not dual-write order here.
        let owned = repo_src("render/pipelines/core2d_overlay_order.rs");
        let src = production_src(&owned);
        let weather = src
            .find("WeatherPrecipRaster")
            .expect("WeatherPrecipRaster missing from overlay order");
        let fire = src
            .find("FireParticleRaster")
            .expect("FireParticleRaster missing from overlay order");
        assert!(
            weather < fire,
            "RND-04: weather precip must draw before fire particles (explicit layering)"
        );
        // Chain tuple order must list weather before fire.
        let chain = src
            .split("configure_sets")
            .nth(1)
            .unwrap_or("");
        let w = chain
            .find("WeatherPrecipRaster")
            .expect("WeatherPrecipRaster not in configure_sets chain");
        let f = chain
            .find("FireParticleRaster")
            .expect("FireParticleRaster not in configure_sets chain");
        assert!(w < f, "configure_sets chain must keep Weather before Fire");
    }

    #[test]
    fn rnd04_smoke_cloud_fire_layering_contract_is_documented() {
        let pack = repo_src("render/fire_vfx/pack.rs");
        let clouds = repo_src("render/weather_vfx/cloud_sheets.rs");
        let weather_mod = repo_src("render/weather_vfx/mod.rs");
        for (label, text) in [
            ("fire_vfx/pack.rs", &pack),
            ("weather_vfx/cloud_sheets.rs", &clouds),
            ("weather_vfx/mod.rs", &weather_mod),
        ] {
            assert!(
                text.contains("RND-04") && text.contains("11.3.12"),
                "{label}: must dual-cite RND-04 / gea4v2 11.3.12"
            );
            assert!(
                text.contains("geometry_visual_effects") || text.contains("OIT"),
                "{label}: must cite LEGACY geometry_visual_effects or OIT stance"
            );
        }
        assert!(
            pack.contains("TRANSPARENT_ORDER_SMOKE")
                && pack.contains("TRANSPARENT_ORDER_SPARK"),
            "fire pack must expose explicit smoke→spark transparent order constants"
        );
        assert!(
            clouds.contains("CLOUD_SHEET_LAYER_Z"),
            "cloud sheets must expose explicit layer Z strata"
        );
    }
}

#[cfg(test)]
mod rnd14_card_effects_tests {
    //! RND-14 tripwires: cloud/smoke card impostors are intentional (not ray-marched volumes).
    //! Locked: do not retarget RND-04 `CLOUD_SHEET_LAYER_Z` / fire `TRANSPARENT_ORDER_*`.

    use std::path::PathBuf;

    use super::cloud_sheets::CLOUD_SHEET_LAYER_Z;

    fn repo_src(rel: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn production_src(text: &str) -> &str {
        text.split("#[cfg(test)]").next().unwrap_or(text)
    }

    #[test]
    fn rnd14_docs_dual_cite_card_impostor_path() {
        let weather_mod = repo_src("render/weather_vfx/mod.rs");
        let clouds = repo_src("render/weather_vfx/cloud_sheets.rs");
        let fire_mod = repo_src("render/fire_vfx/mod.rs");
        let pack = repo_src("render/fire_vfx/pack.rs");
        for (label, text) in [
            ("weather_vfx/mod.rs", &weather_mod),
            ("weather_vfx/cloud_sheets.rs", &clouds),
            ("fire_vfx/mod.rs", &fire_mod),
            ("fire_vfx/pack.rs", &pack),
        ] {
            assert!(
                text.contains("RND-14") && text.contains("11.6.4"),
                "{label}: must dual-cite RND-14 / gea4v2 11.6.4"
            );
            assert!(
                text.contains("geometry_visual_effects"),
                "{label}: must cite LEGACY geometry_visual_effects"
            );
            let stance = text.to_ascii_lowercase();
            assert!(
                stance.contains("card")
                    || stance.contains("billboard")
                    || stance.contains("impostor"),
                "{label}: must name card/billboard/impostor stance"
            );
        }
    }

    #[test]
    fn rnd14_cloud_sheets_use_rectangle_sprite_cards() {
        let owned = repo_src("render/weather_vfx/cloud_sheets.rs");
        let src = production_src(&owned);
        let compact: String = src.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            compact.contains("Rectangle::new(1.0,1.0)"),
            "cloud sheets must spawn unit Rectangle cards (RND-14)"
        );
        assert!(
            compact.contains("Mesh2d(mesh.clone())")
                && compact.contains("MeshMaterial2d(mat)")
                && src.contains("ColorMaterial"),
            "cloud sheets must be Mesh2d + ColorMaterial sprites (card impostors)"
        );
        assert!(
            src.contains("soft_puff_image"),
            "cloud cards must use soft radial-falloff puff texture"
        );
    }

    #[test]
    fn rnd14_cloud_sheet_layer_z_unchanged_from_rnd04() {
        // Pin RND-04 contract — RND-14 must not retarget strata.
        assert_eq!(CLOUD_SHEET_LAYER_Z, [400.0, 455.0, 510.0]);
        assert!(
            CLOUD_SHEET_LAYER_Z[0] < CLOUD_SHEET_LAYER_Z[1]
                && CLOUD_SHEET_LAYER_Z[1] < CLOUD_SHEET_LAYER_Z[2]
        );
    }

    #[test]
    fn rnd14_weather_vfx_rejects_raymarch_volumetric_path() {
        for rel in [
            "render/weather_vfx/mod.rs",
            "render/weather_vfx/cloud_sheets.rs",
            "render/weather_vfx/draw.rs",
            "render/weather_vfx/extract.rs",
            "render/weather_vfx/frame.rs",
        ] {
            let owned = repo_src(rel);
            let src = production_src(&owned).to_ascii_lowercase();
            assert!(
                !src.contains("ray_march")
                    && !src.contains("raymarch")
                    && !src.contains("volumetric_cloud"),
                "{rel}: RND-14 card path must not grow ray-marched volumetric clouds"
            );
        }
    }

    #[test]
    fn rnd14_fire_smoke_packs_instanced_quad_cards() {
        let pack = repo_src("render/fire_vfx/pack.rs");
        let prod = production_src(&pack);
        assert!(
            prod.contains("GpuInstancedQuadInstance") && prod.contains("as_instanced_quad"),
            "fire pack must map rows to instanced-quad billboard cards (RND-14)"
        );
        assert!(
            prod.contains("ParticleClass::Smoke") && prod.contains("ParticleClass::AtmosphereFx"),
            "fire pack must keep Smoke/AtmosphereFx card classes"
        );
        // RND-04 order keys stay pinned (RND-14 must not dual-write them).
        assert!(
            prod.contains("TRANSPARENT_ORDER_SMOKE: u8 = 0")
                && prod.contains("TRANSPARENT_ORDER_EMBER: u8 = 1")
                && prod.contains("TRANSPARENT_ORDER_SPARK: u8 = 2"),
            "RND-14 must leave TRANSPARENT_ORDER_* literals at 0/1/2"
        );
    }
}
