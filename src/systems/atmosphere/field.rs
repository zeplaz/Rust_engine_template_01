//! [`AtmosphereCell`] + [`GlobalWind`] + tactical grid index (`base_fire2_smoke.md` §1, §3).
//!
//! **DEBT-011:** Fixed 128² [`AtmosphereField`] resource **deleted**. Clipmap L0 is channel
//! authority; chunk→tile indexing is the free [`atmos_chunk_to_tile`] helper (matches
//! [`crate::substrate::atmosphere::CLIPMAP_L0_RES`]).
//!
//! **DEBT-012 (kept · class C dormant):** [`AtmosphereCell`] remains the multi-channel
//! overlay / logistics **sample DTO** — not a grid. Overlays are **not** L0-direct yet
//! (`atmosphere_overlay_rgba` still takes `&AtmosphereCell`). Retire only when overlay
//! + logistics callers sample L0 channels without this struct (`unblock_when`).

use bevy::prelude::*;

use crate::substrate::atmosphere::CLIPMAP_L0_RES;

/// Tactical atmosphere grid origin (chunk coords) — aligned with clipmap L0.
pub const ATMO_GRID_ORIGIN: IVec2 = IVec2::ZERO;

/// Tactical atmosphere grid size — must match [`CLIPMAP_L0_RES`].
pub const ATMO_GRID_SIZE: UVec2 = CLIPMAP_L0_RES;

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

/// World wind for semi-Lagrangian advection on clipmap L0.
#[derive(Resource, Debug, Clone, Copy)]
pub struct GlobalWind {
    pub direction: Vec2,
    pub speed: f32,
}

impl Default for GlobalWind {
    fn default() -> Self {
        Self {
            direction: Vec2::X,
            speed: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
