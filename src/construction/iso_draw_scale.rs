//! BUILD-READ-WORLD-002 — Option A iso building draw scale multiplier (post-commit visual only).

use bevy::prelude::*;

/// Designer table: [`design_build_readability_v1.md`](../../dev/design_build_readability_v1.md) §3b.
/// Mid-upper band (1.35–1.75) so primary reads ~15%+ of site stub at operational zoom.
pub const DEFAULT_ISO_DRAW_SCALE_MULTIPLIER: f32 = 1.65;
pub const ISO_DRAW_SCALE_MIN: f32 = 1.35;
pub const ISO_DRAW_SCALE_MAX: f32 = 1.75;

/// Post-commit PG-2 module draw scale — does **not** affect ghost pick / tile occupation.
#[derive(Resource, Debug, Clone)]
pub struct ConstructionIsoDrawScale {
    pub multiplier: f32,
}

impl Default for ConstructionIsoDrawScale {
    fn default() -> Self {
        Self {
            multiplier: DEFAULT_ISO_DRAW_SCALE_MULTIPLIER,
        }
    }
}

impl ConstructionIsoDrawScale {
    #[must_use]
    pub fn clamped(multiplier: f32) -> Self {
        Self {
            multiplier: multiplier.clamp(ISO_DRAW_SCALE_MIN, ISO_DRAW_SCALE_MAX),
        }
    }

    #[must_use]
    pub fn visual_scale_vec3(&self) -> Vec3 {
        Vec3::new(self.multiplier, 1.0, self.multiplier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_in_designer_band() {
        let s = ConstructionIsoDrawScale::default();
        assert!(s.multiplier >= ISO_DRAW_SCALE_MIN);
        assert!(s.multiplier <= ISO_DRAW_SCALE_MAX);
    }

    #[test]
    fn clamp_holds_band() {
        assert_eq!(
            ConstructionIsoDrawScale::clamped(0.1).multiplier,
            ISO_DRAW_SCALE_MIN
        );
        assert_eq!(
            ConstructionIsoDrawScale::clamped(9.0).multiplier,
            ISO_DRAW_SCALE_MAX
        );
    }

    #[test]
    fn visual_scale_is_xz_only() {
        let v = ConstructionIsoDrawScale::default().visual_scale_vec3();
        assert!((v.y - 1.0).abs() < 1e-5);
        assert!((v.x - v.z).abs() < 1e-5);
        assert!(v.x > 1.0);
    }
}
