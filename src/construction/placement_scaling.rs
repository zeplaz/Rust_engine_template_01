//! Per-family scale defaults for parametric placement (catalog overrides in Phase 4).

use crate::construction::building_catalog::BuildingFamily;

pub const DEFAULT_SCALE_MIN: f32 = 0.35;
pub const DEFAULT_SCALE_MAX: f32 = 2.75;
pub const DEFAULT_SCALE_FACTOR: f32 = 1.0;

#[must_use]
pub fn clamp_scale_factor(scale: f32) -> f32 {
    scale.clamp(DEFAULT_SCALE_MIN, DEFAULT_SCALE_MAX)
}

#[must_use]
pub fn default_scale_factor_for_family(_family: BuildingFamily) -> f32 {
    DEFAULT_SCALE_FACTOR
}
