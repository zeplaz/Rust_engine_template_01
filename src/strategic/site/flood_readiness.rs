//! DF-9 — site owner applies flood readiness from water-published book (no water `ResMut`).

use bevy::prelude::*;

use super::components::{ConstructionSite, SiteFootprint};
use super::resources::SiteConstructionPhase;
use crate::systems::water::{FloodSiteReadinessBook, FLOOD_ROUTE_DEPTH_M};

/// Readiness multiplier from flood depth: 1.0 dry → ~0.15 at ≥2 m over the route threshold.
#[must_use]
pub fn flood_readiness_factor(max_depth_m: f32) -> f32 {
    if max_depth_m < FLOOD_ROUTE_DEPTH_M {
        return 1.0;
    }
    let excess = (max_depth_m - FLOOD_ROUTE_DEPTH_M).max(0.0);
    (1.0 - excess / 2.0).clamp(0.15, 1.0)
}

/// Site-domain sole writer of flood-driven `operational_readiness` derate on Operational sites.
pub fn apply_flood_site_readiness_system(
    book: Option<Res<FloodSiteReadinessBook>>,
    mut sites: Query<(&SiteFootprint, &mut ConstructionSite)>,
) {
    let Some(book) = book else {
        return;
    };
    if book.depth_by_tile.is_empty() {
        return;
    }
    for (footprint, mut site) in &mut sites {
        if site.phase != SiteConstructionPhase::Operational {
            continue;
        }
        let tiles: Vec<(i32, i32)> = footprint.tiles.iter().map(|t| (t.x, t.y)).collect();
        let depth = book.max_depth_on_tiles(&tiles);
        let factor = flood_readiness_factor(depth);
        if factor < 1.0 {
            site.operational_readiness = (site.operational_readiness.max(0.01) * factor).clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flood_readiness_derates_with_depth() {
        assert!((flood_readiness_factor(0.1) - 1.0).abs() < 1e-5);
        assert!(flood_readiness_factor(0.5) <= 1.0);
        assert!(flood_readiness_factor(2.5) < 0.5);
        assert!(flood_readiness_factor(5.0) >= 0.15);
    }
}
