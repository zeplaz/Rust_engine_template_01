//! Ghost / commit validation — delegates to strategic site scoring (same as AI).

use crate::strategic::{evaluate_site_placement_stubs, SitePlacementValidation};

#[inline]
pub fn validate_planned_site_stubs() -> SitePlacementValidation {
    evaluate_site_placement_stubs()
}
