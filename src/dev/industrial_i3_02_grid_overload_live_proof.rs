//! **INDUSTRIAL-I3-02** — grid overload hook live witness
//! (`debug_runs/industrial_activation_live.json`).

use crate::economy::activation::refresh_industrial_i3_02_grid_overload_live_witness;

pub const INDUSTRIAL_I3_02_JSON: &str = "debug_runs/industrial_activation_live.json";

/// Headless seed: Portland chain + IND-E03 overload cluster → `grid_overload_hook`.
#[must_use]
pub fn refresh_industrial_i3_02_grid_overload_live_witness_proof() -> bool {
    refresh_industrial_i3_02_grid_overload_live_witness()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn industrial_i3_02_grid_overload_witness_green() {
        assert!(
            refresh_industrial_i3_02_grid_overload_live_witness_proof(),
            "INDUSTRIAL-I3-02: expected grid_overload_hook green in {INDUSTRIAL_I3_02_JSON}"
        );
    }
}
