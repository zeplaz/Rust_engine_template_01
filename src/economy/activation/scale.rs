//! Parametric placement scale → facility production at activation (**CONSTRUCTION-PARAM-CODER-006**).

use crate::economy::resource_flow::FacilityFlowState;
use crate::strategic::BuildingScaleParams;

/// Default production exponent when catalog omits `placement_scaling` (`construction_parametric_placement_spec_v1.md`).
pub const DEFAULT_K_PROD: f32 = 0.90;
pub const SCALE_CLAMP_MIN: f32 = 0.25;

#[must_use]
pub fn clamp_effective_scale(s: f32) -> f32 {
    s.clamp(SCALE_CLAMP_MIN, crate::construction::placement_scaling::DEFAULT_SCALE_MAX)
}

/// `prod_mult(s) = s ^ k_prod` — drives runtime production scale at activation.
#[must_use]
pub fn production_multiplier(effective_scale: f32, k_prod: f32) -> f32 {
    let s = clamp_effective_scale(effective_scale);
    s.powf(k_prod)
}

/// Apply committed [`BuildingScaleParams`] to facility flow output (single writer at activation).
pub fn apply_placement_scale_to_facility(
    scale: &BuildingScaleParams,
    flow: &mut FacilityFlowState,
) {
    flow.output_scale = production_multiplier(scale.effective_scale, DEFAULT_K_PROD);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn economy_scale_clamp() {
        assert!((production_multiplier(0.1, DEFAULT_K_PROD)
            - production_multiplier(SCALE_CLAMP_MIN, DEFAULT_K_PROD))
        .abs()
            < 1e-5);
    }
}
