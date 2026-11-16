//! # LGT-05 — Participating media (V4-LGT05)
//!
//! Dual-cite: NEW **LGT-05** (gea4v2 **12.1.2.2** *Interactions with Participating
//! Media* p155-156) · LEGACY keywords **`lighting_post`** / **`post_processing`**
//! (gea_v2_e4 §12 *Lighting and Post-Processing* p153; chapter anchors shared with
//! LGT-03/LGT-06 — no dedicated `participating_media` legacy keyword).
//!
//! **Physical basis** (matrix note): smoke / fog / ash are **participating media** —
//! they absorb/scatter along a path and reduce visibility. This module owns the
//! **unit transmittance** sample for those media channels on [`AtmosphereCell`].
//! Metre-scale weather÷tracer combine stays on
//! [`super::frame::combine_visibility_m`] (AtmosphereFrame sole writer) — do **not**
//! dual-write that path from here.
//!
//! | Layer | Owner | Role |
//! |:---|:---|:---|
//! | Media sample DTO | **this module** — [`AtmosphereCell`] + [`participating_media_visibility`] | unit clearness in `[floor, 1]` |
//! | L0 tracer storage | [`crate::substrate::atmosphere::AtmosphereClipmapStack`] | smoke/ash/ember channels |
//! | Metre visibility | [`super::frame::combine_visibility_m`] | weather_m / (1+k·tracers) |
//! | LOS sample | [`super::visibility`] | multiplies unit clearness along a segment |
//!
//! **Voxels in scope:** PixelWorld / massing consumers share the same AtmosphereCell
//! sample DTO and L0-backed unit transmittance — do **not** add a voxel-only fog
//! writer or a parallel extinction table.
//!
//! Debt not taken: ray-march volumetrics (LGT-04) · resurrecting fixed 128²
//! `AtmosphereField` · dual-writing `combine_visibility_m` · AI haze textures.
//!
//! Tripwires: `lgt05_*` · `participating_media_*` (no GPU).
//!
//! **DEBT-011:** Fixed 128² [`AtmosphereField`] resource **deleted**. Clipmap L0 is channel
//! authority; chunk→tile indexing is the free [`atmos_chunk_to_tile`] helper (matches
//! [`crate::substrate::atmosphere::CLIPMAP_L0_RES`]).
//!
//! **DEBT-012 (kept · class C dormant):** [`AtmosphereCell`] remains the multi-channel
//! overlay / logistics **sample DTO** — not a grid. Overlays are **not** L0-direct yet
//! (`atmosphere_overlay_rgba` still takes `&AtmosphereCell`). Retire only when overlay
//! + logistics callers sample L0 channels without this struct (`unblock_when`).
//! Wind authority is [`crate::systems::weather::WeatherField`] (W2 deleted `GlobalWind`).

use bevy::prelude::*;

use crate::substrate::atmosphere::CLIPMAP_L0_RES;

/// Tactical atmosphere grid origin (chunk coords) — aligned with clipmap L0.
pub const ATMO_GRID_ORIGIN: IVec2 = IVec2::ZERO;

/// Tactical atmosphere grid size — must match [`CLIPMAP_L0_RES`].
pub const ATMO_GRID_SIZE: UVec2 = CLIPMAP_L0_RES;

/// Smoke optical weight for unit transmittance (LOS / cell sample; LGT-05).
pub const PARTICIPATING_SMOKE_OPTICAL_K: f32 = 0.7;

/// Fog optical weight for unit transmittance (weather fog on the sample DTO).
pub const PARTICIPATING_FOG_OPTICAL_K: f32 = 0.5;

/// Ash optical weight for unit transmittance (fire-borne ash on the sample DTO).
pub const PARTICIPATING_ASH_OPTICAL_K: f32 = 0.4;

/// Floor so totally opaque media never fully zeros LOS multipliers.
pub const PARTICIPATING_VIS_FLOOR: f32 = 0.05;

/// One logical atmosphere column (chunk- or tile-mapped).
///
/// DEBT-012 keep: sample contract for overlays/logistics — not storage authority.
#[derive(Clone, Copy, Debug, Default)]
pub struct AtmosphereCell {
    pub smoke_density: f32,
    pub fog_density: f32,
    pub toxicity: f32,
    pub heat_distortion: f32,
    pub ash_density: f32,
    pub ember_density: f32,
    /// Combined visibility in `[0, 1]` — 1 = clear.
    pub visibility: f32,
}

/// Unit transmittance through participating media densities (GEA 12.1.2.2).
///
/// Linear optical-depth approximation: `1 − Σ kᵢρᵢ`, clamped to
/// [`PARTICIPATING_VIS_FLOOR`]..`1`. Metre-scale weather combine remains
/// [`super::frame::combine_visibility_m`].
#[inline]
#[must_use]
pub fn participating_media_visibility(smoke: f32, fog: f32, ash: f32) -> f32 {
    let optical = PARTICIPATING_SMOKE_OPTICAL_K * smoke.clamp(0.0, 1.0)
        + PARTICIPATING_FOG_OPTICAL_K * fog.clamp(0.0, 1.0)
        + PARTICIPATING_ASH_OPTICAL_K * ash.clamp(0.0, 1.0);
    (1.0 - optical).clamp(PARTICIPATING_VIS_FLOOR, 1.0)
}

impl AtmosphereCell {
    /// Derive [`Self::visibility`] from smoke / fog / ash (participating media).
    #[inline]
    #[must_use]
    pub fn with_derived_visibility(mut self) -> Self {
        self.visibility = participating_media_visibility(
            self.smoke_density,
            self.fog_density,
            self.ash_density,
        );
        self
    }
}

/// Map chunk coord → L0 / tactical tile index.
#[inline]
#[must_use]
pub fn atmos_chunk_to_tile(chunk: IVec2) -> Option<(u32, u32)> {
    atmos_chunk_to_tile_in(ATMO_GRID_ORIGIN, ATMO_GRID_SIZE, chunk)
}

/// Map chunk coord → tile under an explicit origin/size (tests / tool callers).
#[inline]
#[must_use]
pub fn atmos_chunk_to_tile_in(origin: IVec2, size: UVec2, chunk: IVec2) -> Option<(u32, u32)> {
    let lx = chunk.x - origin.x;
    let ly = chunk.y - origin.y;
    if lx < 0 || ly < 0 {
        return None;
    }
    let ux = lx as u32;
    let uy = ly as u32;
    if ux >= size.x || uy >= size.y {
        return None;
    }
    Some((ux, uy))
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
    fn chunk_to_tile_roundtrip_origin_zero() {
        assert_eq!(atmos_chunk_to_tile(IVec2::new(0, 0)), Some((0, 0)));
        assert_eq!(atmos_chunk_to_tile(IVec2::new(127, 0)), Some((127, 0)));
        assert_eq!(atmos_chunk_to_tile(IVec2::new(-1, 0)), None);
        assert_eq!(atmos_chunk_to_tile(IVec2::new(128, 0)), None);
    }

    #[test]
    fn grid_size_matches_clipmap_l0() {
        assert_eq!(ATMO_GRID_SIZE, CLIPMAP_L0_RES);
    }

    /// LGT-05 tripwire: denser participating media lowers unit visibility.
    #[test]
    fn lgt05_denser_media_lowers_unit_visibility() {
        let clear = participating_media_visibility(0.0, 0.0, 0.0);
        let smoke = participating_media_visibility(0.8, 0.0, 0.0);
        let fog = participating_media_visibility(0.0, 0.8, 0.0);
        let ash = participating_media_visibility(0.0, 0.0, 0.8);
        let stacked = participating_media_visibility(0.5, 0.4, 0.3);
        assert!((clear - 1.0).abs() < 1e-5);
        assert!(smoke < clear && fog < clear && ash < clear);
        assert!(stacked < smoke);
        assert!(smoke >= PARTICIPATING_VIS_FLOOR);
    }

    /// LGT-05 tripwire: AtmosphereCell derives visibility from media channels.
    #[test]
    fn lgt05_cell_with_derived_visibility_matches_formula() {
        let cell = AtmosphereCell {
            smoke_density: 0.6,
            fog_density: 0.2,
            ash_density: 0.1,
            ..Default::default()
        }
        .with_derived_visibility();
        let expect =
            participating_media_visibility(cell.smoke_density, cell.fog_density, cell.ash_density);
        assert!((cell.visibility - expect).abs() < 1e-6);
    }

    /// LGT-05 tripwire: LOS smoke weight shares the optical constant (no dual formula).
    #[test]
    fn lgt05_visibility_los_shares_smoke_optical_k() {
        let vis = read("systems/atmosphere/visibility.rs");
        assert!(
            vis.contains("PARTICIPATING_SMOKE_OPTICAL_K")
                || vis.contains("participating_media_visibility"),
            "visibility.rs must use field.rs participating-media optical constant/fn"
        );
    }

    /// LGT-05 tripwire: metre combine stays on AtmosphereFrame; no ray-march invent.
    #[test]
    fn lgt05_does_not_dual_write_frame_or_invent_raymarch() {
        let src = read("systems/atmosphere/field.rs");
        let prod = src.split("#[cfg(test)]").next().unwrap_or(&src);
        assert!(
            prod.contains("combine_visibility_m") || prod.contains("AtmosphereFrame"),
            "must defer metre-scale combine to AtmosphereFrame / combine_visibility_m"
        );
        assert!(
            prod.contains("Voxels in scope") || prod.contains("PixelWorld"),
            "must document voxels sharing the AtmosphereCell / L0 sample path"
        );
        assert!(
            !prod.contains("struct RayMarchPass") && !prod.contains("pub fn ray_march"),
            "must not invent LGT-04 ray-march volumetrics here"
        );
        assert!(
            !prod.contains("pub struct AtmosphereField"),
            "must not resurrect deleted AtmosphereField grid resource"
        );
    }

    /// Naming filter alias for Accept `cargo test participating_media`.
    #[test]
    fn participating_media_visibility_deterministic() {
        let a = participating_media_visibility(0.33, 0.12, 0.08);
        let b = participating_media_visibility(0.33, 0.12, 0.08);
        assert_eq!(a.to_bits(), b.to_bits());
    }
}
