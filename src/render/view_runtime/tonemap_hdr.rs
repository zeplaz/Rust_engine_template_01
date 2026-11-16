//! # LGT-06 — Tone mapping / HDR for map views (V4-LGT06)
//!
//! Dual-cite: NEW **LGT-06** (gea4v2 12.7.1 *Tone Mapping for High Dynamic Range
//! Rendering* p244-245) · LEGACY keywords **`lighting_post`** /
//! **`post_processing`** (gea_v2_e4 §12 *Lighting and Post-Processing* p153;
//! §12.7 *Post-Processing* p244). No dedicated `tone_mapping` / `hdr` keyword
//! hit in the legacy matrix — cite the chapter anchors.
//!
//! **Fire-at-night readability** (matrix note): keep the shared simulation-map
//! RTT display-referred and readable while bright fire overlays composite.
//! GEA 12.7 places tone map / post **after** lighting; this engine mirrors that
//! by running Core2d overlays before `Core2dSystems::EarlyPostProcess` (RND-03)
//! and by **not** inventing a second filmic writer on the map cameras.
//!
//! ## Locked existing path (sole contract)
//!
//! | Surface | Policy |
//! |:---|:---|
//! | Simulation map RTT | **SDR** [`CORE2D_OVERLAY_SDR_FORMAT`] (`Rgba8UnormSrgb`) — display-referred |
//! | Core2d overlay PSO table | `[SDR=0, HDR=1]` via [`core2d_overlay_pipeline_hdr_index`] |
//! | PixelWorld3D (voxels **in scope**) | Bevy `Tonemapping::None` — authored palette, no filmic crush |
//! | Overlay → post order | MainPass → overlays → `EarlyPostProcess` |
//!
//! Debt not taken: custom ACES/Reinhard operator · HDR float `SimulationMapTexture` ·
//! auto-exposure · dual-writing Bevy `Tonemapping` from UI/sim.
//!
//! Tripwires: `lgt06_*` · `tonemap_*` · `hdr_*` (no GPU).

use bevy::render::render_resource::TextureFormat;

use crate::render::pipelines::core2d_overlay_order::{
    core2d_overlay_pipeline_hdr_index, CORE2D_OVERLAY_SDR_FORMAT,
};

/// Display-referred map / voxel view: skip filmic tone map (LGT-06).
///
/// PixelWorld terrain colours and Core2d fire overlays are authored for the SDR
/// RTT. A filmic curve would crush night contrast on the shared target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ViewToneMapPolicy {
    /// No operator — leave values as composed (Bevy `Tonemapping::None`).
    DisplayReferredNone,
    /// Reserved: float HDR target + operator (not adopted this slice).
    HdrFloatTarget,
}

/// Simulation-map + PixelWorld tone-map policy (sole LGT-06 authority).
pub const MAP_VIEW_TONE_MAP_POLICY: ViewToneMapPolicy = ViewToneMapPolicy::DisplayReferredNone;

/// Voxel PixelWorld3D uses the same display-referred policy (voxels in scope).
pub const PIXEL_WORLD_TONE_MAP_POLICY: ViewToneMapPolicy = ViewToneMapPolicy::DisplayReferredNone;

/// Source token tripwires expect on the PixelWorld camera spawn site.
pub const PIXEL_WORLD_TONEMAPPING_NONE_TOKEN: &str =
    "bevy::core_pipeline::tonemapping::Tonemapping::None";

/// SDR display format for the shared map RTT (must match overlay SDR PSOs).
#[inline]
#[must_use]
pub const fn map_view_display_format() -> TextureFormat {
    CORE2D_OVERLAY_SDR_FORMAT
}

/// `true` when the target is the locked 8-bit sRGB display path (HDR index 0).
#[inline]
#[must_use]
pub fn is_display_referred_sdr_format(format: TextureFormat) -> bool {
    matches!(
        format,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Bgra8UnormSrgb
    )
}

/// Index into overlay `[SDR, HDR]` PSO tables — LGT-06 HDR path selector.
#[inline]
#[must_use]
pub fn hdr_pipeline_index_for_target(target_format: TextureFormat) -> usize {
    core2d_overlay_pipeline_hdr_index(target_format)
}

/// Whether the locked map-view policy skips filmic tone mapping.
#[inline]
#[must_use]
pub const fn map_view_skips_filmic_tonemap() -> bool {
    matches!(MAP_VIEW_TONE_MAP_POLICY, ViewToneMapPolicy::DisplayReferredNone)
}

#[cfg(test)]
mod lgt06_tonemap_hdr_tests {
    use super::*;
    use bevy::render::render_resource::TextureFormat;
    use std::path::PathBuf;

    fn repo_src(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(rel)
    }

    fn read(rel: &str) -> String {
        std::fs::read_to_string(repo_src(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
    }

    /// LGT-06 / V4-LGT06 tripwire: dual-cite (NEW row + LEGACY keywords) stays in module docs.
    #[test]
    fn lgt06_dual_cite_tone_mapping_hdr_in_module_docs() {
        let src = read("render/view_runtime/tonemap_hdr.rs");
        assert!(
            src.contains("LGT-06") && src.contains("gea4v2 12.7.1"),
            "missing NEW LGT-06 / gea4v2 12.7.1 cite"
        );
        assert!(
            src.contains("lighting_post") && src.contains("post_processing"),
            "missing LEGACY lighting_post / post_processing cites"
        );
        assert!(
            src.contains("gea_v2_e4"),
            "missing LEGACY book id gea_v2_e4"
        );
        let passes = read("render/view_runtime/passes.rs");
        assert!(
            passes.contains("LGT-06") && passes.contains("ToneMapHdrPass"),
            "passes.rs must dual-cite LGT-06 and expose ToneMapHdrPass"
        );
    }

    /// LGT-06 tripwire: map RTT stays on the SDR display format (fire-at-night readability).
    #[test]
    fn lgt06_simulation_map_rtt_stays_display_referred_sdr() {
        assert_eq!(map_view_display_format(), TextureFormat::Rgba8UnormSrgb);
        assert!(is_display_referred_sdr_format(map_view_display_format()));
        assert!(map_view_skips_filmic_tonemap());
        let rtt = read("gui/tactical/sim_map_rtt.rs");
        assert!(
            rtt.contains("CORE2D_OVERLAY_SDR_FORMAT"),
            "simulation_map_rtt_image must keep CORE2D_OVERLAY_SDR_FORMAT"
        );
        assert!(
            !rtt.contains("Rgba16Float"),
            "SimulationMapTexture must not silently move to HDR float"
        );
    }

    /// Tonemap tripwire: PixelWorld (voxel) camera keeps `Tonemapping::None`.
    #[test]
    fn tonemap_pixel_world_camera_uses_none() {
        assert_eq!(PIXEL_WORLD_TONE_MAP_POLICY, ViewToneMapPolicy::DisplayReferredNone);
        let cam = read("render/pixel_world/camera.rs");
        assert!(
            cam.contains(PIXEL_WORLD_TONEMAPPING_NONE_TOKEN)
                || cam.contains("Tonemapping::None"),
            "PixelWorld camera must keep Tonemapping::None (LGT-06 display-referred)"
        );
        assert!(
            cam.contains("LGT-06") || cam.contains("tonemap_hdr"),
            "pixel_world/camera.rs must point at LGT-06 / tonemap_hdr contract"
        );
    }

    /// HDR tripwire: overlay PSO index SDR→0, float HDR→1 (existing path).
    #[test]
    fn hdr_overlay_pipeline_index_contract() {
        assert_eq!(hdr_pipeline_index_for_target(CORE2D_OVERLAY_SDR_FORMAT), 0);
        assert_eq!(
            hdr_pipeline_index_for_target(TextureFormat::Bgra8UnormSrgb),
            0
        );
        assert_eq!(
            hdr_pipeline_index_for_target(TextureFormat::Rgba16Float),
            1
        );
        assert!(!is_display_referred_sdr_format(TextureFormat::Rgba16Float));
    }

    /// HDR / LGT-06 tripwire: overlays stay before EarlyPostProcess (GEA 12.7 order).
    #[test]
    fn hdr_overlay_chain_before_early_post_process() {
        let order = read("render/pipelines/core2d_overlay_order.rs");
        assert!(
            order.contains("EarlyPostProcess"),
            "overlay order must name EarlyPostProcess"
        );
        assert!(
            order.contains("before(Core2dSystems::EarlyPostProcess)"),
            "overlays must run before EarlyPostProcess (tone map / post slot)"
        );
    }

    /// Tonemap tripwire: policy enum has exactly one adopted variant in production use.
    #[test]
    fn tonemap_map_view_policy_is_display_referred_none() {
        assert_eq!(MAP_VIEW_TONE_MAP_POLICY, ViewToneMapPolicy::DisplayReferredNone);
        assert_eq!(PIXEL_WORLD_TONE_MAP_POLICY, MAP_VIEW_TONE_MAP_POLICY);
        // Reserved variant exists for future HDR float adoption — not selected.
        let _reserved = ViewToneMapPolicy::HdrFloatTarget;
        assert_ne!(MAP_VIEW_TONE_MAP_POLICY, _reserved);
    }
}
