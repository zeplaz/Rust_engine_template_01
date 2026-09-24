//! DEBT-011 — Field→L0 consistency shim **retired** (AtmosphereField deleted).
//!
//! ES-6-2 / DEBT-010 era max-blend Field → L0 is no longer scheduled. L0 is written
//! directly by FieldFill / WindAdvect.

use crate::substrate::atmosphere::DEBT_010_WRITERS_L0;

/// Compile-time marker — Field→L0 shim removed from the live WindAdvect schedule (DEBT-011).
pub const FIELD_L0_SHIM_WIRED: bool = false;

/// DEBT-011 — shim retired because Field resource is gone.
pub const FIELD_L0_SHIM_RETIRED: bool = true;

/// Re-export DEBT-010 gate for atmosphere consumers / witnesses.
pub use crate::substrate::atmosphere::DEBT_010_WRITERS_L0 as DEBT_010_WRITERS_ON_L0;

/// No-op retained for call-site stability in tests that previously invoked the shim.
pub fn atmosphere_field_l0_shim_system() {
    let _ = (FIELD_L0_SHIM_WIRED, FIELD_L0_SHIM_RETIRED, DEBT_010_WRITERS_L0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_l0_shim_retired_after_debt_011() {
        assert!(!FIELD_L0_SHIM_WIRED);
        assert!(FIELD_L0_SHIM_RETIRED);
        assert!(DEBT_010_WRITERS_ON_L0);
    }
}
