//! Line-of-sight style sampling (`base_fire2_smoke.md` §8).
//!
//! **DEBT-011 / ES-6-3a:** visibility samples clipmap L0 only (Field deleted).

use bevy::prelude::*;

use crate::substrate::atmosphere::{
    sample_tactical_fog_from_l0, sample_tactical_smoke_from_l0, AtmosphereClipmapStack,
    CLIPMAP_L0_RES,
};

use super::field::ATMO_GRID_SIZE;

/// Sample visibility along segment `(a,b)` in **tactical tile space**
/// ([`super::field::atmos_chunk_to_tile`] origin).
/// Returns multiplicative factor in `[0, 1]` — multiply with geometric clear-LOS as needed.
pub fn visibility_between(a: Vec2, b: Vec2, stack: &AtmosphereClipmapStack) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist <= 1e-5 {
        return sample_visibility_at(stack, a.x, a.y);
    }
    let steps = (dist.ceil() as i32).clamp(3, 48);
    let mut acc = 1.0f32;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = a.x + dx * t;
        let y = a.y + dy * t;
        acc *= sample_visibility_at(stack, x, y);
    }
    acc.clamp(0.0, 1.0)
}

/// LOS / HUD visibility from tactical smoke + fog on L0.
/// Matches fill formula: `(1 - smoke*0.7 - fog*0.45).clamp(0.05, 1)`.
#[must_use]
pub fn sample_tactical_visibility_from_l0(stack: &AtmosphereClipmapStack, x: u32, y: u32) -> f32 {
    let smoke = sample_tactical_smoke_from_l0(stack, x, y).unwrap_or(0.0);
    let fog = sample_tactical_fog_from_l0(stack, x, y).unwrap_or(0.0);
    (1.0 - smoke * 0.7 - fog * 0.45).clamp(0.05, 1.0)
}

fn sample_visibility_at(stack: &AtmosphereClipmapStack, wx: f32, wy: f32) -> f32 {
    let xi = wx.floor() as i32;
    let yi = wy.floor() as i32;
    if xi < 0 || yi < 0 {
        return 1.0;
    }
    let xu = xi as u32;
    let yu = yi as u32;
    let size = stack
        .levels
        .first()
        .map(|l| l.resolution)
        .unwrap_or(ATMO_GRID_SIZE);
    debug_assert_eq!(size, CLIPMAP_L0_RES);
    if xu >= size.x || yu >= size.y {
        return 1.0;
    }
    if sample_tactical_smoke_from_l0(stack, xu, yu).is_some() {
        return sample_tactical_visibility_from_l0(stack, xu, yu);
    }
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::atmosphere::{max_blend_l0_smoke_at, AtmosphereClipmapStack};

    #[test]
    fn uniform_clear_field_high_visibility() {
        let stack = AtmosphereClipmapStack::default();
        let v = visibility_between(Vec2::ZERO, Vec2::new(10.0, 0.0), &stack);
        assert!(v > 0.99);
    }

    #[test]
    fn opaque_band_reduces_visibility_via_l0() {
        let mut stack = AtmosphereClipmapStack::default();
        for x in 4..8 {
            max_blend_l0_smoke_at(&mut stack, x, 0, 0.9);
        }
        let v = visibility_between(Vec2::new(2.0, 0.5), Vec2::new(10.0, 0.5), &stack);
        assert!(v < 0.95);
    }
}
