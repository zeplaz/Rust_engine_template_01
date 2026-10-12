//! Ghost / commit validation — shared entry for buildings + site ghosts (BUILD-P1-02).
//!
//! Authority: strategic [`evaluate_site_placement_at_world_tile`] (same path as AI).
//! Road/rail path tools write [`crate::construction::BuildPlacementPreview`] via their
//! segment validators; [`GhostValid`] mirrors that shared preview report.

use bevy::prelude::Query;

use crate::strategic::{
    evaluate_site_placement_at_world_tile, evaluate_site_placement_stubs, BuildSiteTile,
    FootprintTiles, SitePlacementValidation, StrategicRasterConfig,
};
use crate::strategic::ChunkStrategicOverlay;

#[inline]
pub fn validate_planned_site_stubs() -> SitePlacementValidation {
    evaluate_site_placement_stubs()
}

/// Shared production entry for building / site ghost validity.
#[inline]
pub fn evaluate_ghost_site_placement(
    origin: BuildSiteTile,
    footprint: FootprintTiles,
    config: Option<&StrategicRasterConfig>,
    overlays: &Query<&ChunkStrategicOverlay>,
) -> SitePlacementValidation {
    evaluate_site_placement_at_world_tile(origin, footprint, config, overlays)
}

/// [`GhostValid`] flag from a placement report (preview truth, not commit policy alone).
#[inline]
#[must_use]
pub fn ghost_valid_flag(report: &SitePlacementValidation) -> bool {
    report.valid
}
