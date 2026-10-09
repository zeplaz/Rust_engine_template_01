//! Ghost preview vs the single commit funnel.
//!
//! Preview and occupancy rejection leave occupation and the player pending queue untouched.
//! Gameplay sites are created only when [`classify_ghost_place`] returns [`GhostPlaceAction::Commit`],
//! which the Enter / second-LMB path turns into one [`crate::strategic::CommitConstructionSiteEvent`].

use bevy::prelude::IVec2;

use crate::strategic::{SitePlacementValidation, TileOccupationBook};

/// Marks a zero-size footprint. Does not write occupation or pending rows.
#[must_use]
pub fn stamp_empty_footprint(report: &mut SitePlacementValidation, width: u32, depth: u32) -> bool {
    if footprint_is_placeable(width, depth) {
        return false;
    }
    report.valid = false;
    report.allows_commit = false;
    if !report.errors.iter().any(|e| e == EMPTY_FOOTPRINT_ERROR) {
        report.errors.push(EMPTY_FOOTPRINT_ERROR.to_string());
    }
    true
}

#[must_use]
pub fn footprint_is_placeable(width: u32, depth: u32) -> bool {
    width >= 1 && depth >= 1
}

pub const EMPTY_FOOTPRINT_ERROR: &str = "empty_footprint";
pub const FOOTPRINT_OCCUPIED_ERROR: &str = "footprint_occupied";
pub const WATER_FOOTPRINT_ERROR: &str = "footprint_on_water";

/// Port and oil rig are the only buildings that may occupy river, lake, or ocean.
#[must_use]
pub fn building_may_occupy_water(catalog_id: Option<&str>) -> bool {
    let Some(id) = catalog_id.map(str::trim).filter(|id| !id.is_empty()) else {
        return false;
    };
    let normalized = id.to_ascii_lowercase().replace('-', "_");
    let tokens: Vec<&str> = normalized
        .split(|c: char| matches!(c, '_' | '/' | ':' | '.' | ' '))
        .filter(|token| !token.is_empty())
        .collect();
    tokens.iter().any(|token| token_is_port(token))
        || tokens.windows(2).any(|pair| pair == ["oil", "rig"])
        || tokens.iter().any(|token| *token == "oilrig")
}

fn token_is_port(token: &str) -> bool {
    matches!(token, "port" | "seaport" | "harbor" | "harbour")
}

/// True when a non-water building covers at least one water cell.
#[must_use]
pub fn land_building_blocked_on_water(
    catalog_id: Option<&str>,
    tiles: &[IVec2],
    is_water: impl Fn(i32, i32) -> bool,
) -> bool {
    !building_may_occupy_water(catalog_id) && tiles.iter().any(|tile| is_water(tile.x, tile.y))
}

/// Marks the preview report. Does not write occupation or pending rows.
pub fn apply_water_footprint_reject(
    report: &mut SitePlacementValidation,
    catalog_id: Option<&str>,
    tiles: &[IVec2],
    is_water: impl Fn(i32, i32) -> bool,
) -> bool {
    if !land_building_blocked_on_water(catalog_id, tiles, is_water) {
        return false;
    }
    report.valid = false;
    report.allows_commit = false;
    if !report.errors.iter().any(|e| e == WATER_FOOTPRINT_ERROR) {
        report.errors.push(WATER_FOOTPRINT_ERROR.to_string());
    }
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GhostPlaceAction {
    Preview,
    RejectOccupied,
    Commit,
}

/// `commit_requested` is the validation gate (`allows_commit`). Occupancy wins.
#[must_use]
pub fn classify_ghost_place(commit_requested: bool, footprint_occupied: bool) -> GhostPlaceAction {
    if footprint_occupied {
        GhostPlaceAction::RejectOccupied
    } else if commit_requested {
        GhostPlaceAction::Commit
    } else {
        GhostPlaceAction::Preview
    }
}

#[must_use]
pub fn commits_gameplay(action: GhostPlaceAction) -> bool {
    matches!(action, GhostPlaceAction::Commit)
}

#[must_use]
pub fn may_queue_commit(width: u32, depth: u32, allows_commit: bool, occupied: bool) -> bool {
    footprint_is_placeable(width, depth)
        && commits_gameplay(classify_ghost_place(allows_commit, occupied))
}

/// Odd quarter-turns swap the axis-aligned footprint. A zero side stays unplaceable.
#[must_use]
pub fn oriented_footprint(width: u32, depth: u32, turns: u8) -> Option<(u32, u32)> {
    if !footprint_is_placeable(width, depth) {
        return None;
    }
    if turns % 2 == 1 {
        Some((depth, width))
    } else {
        Some((width, depth))
    }
}

/// Tiles the occupancy probe reads. Odd quarter-turns use the swapped footprint. Empty sides list nothing.
#[must_use]
pub fn oriented_rect_tiles(
    origin_x: i32,
    origin_z: i32,
    width: u32,
    depth: u32,
    turns: u8,
) -> Vec<IVec2> {
    match oriented_footprint(width, depth, turns) {
        Some((w, d)) => footprint_rect_tiles(origin_x, origin_z, w, d),
        None => Vec::new(),
    }
}

#[must_use]
pub fn footprint_rect_tiles(origin_x: i32, origin_z: i32, width: u32, depth: u32) -> Vec<IVec2> {
    if !footprint_is_placeable(width, depth) {
        return Vec::new();
    }
    let w = width;
    let d = depth;
    let mut tiles = Vec::with_capacity((w as usize) * (d as usize));
    for dz in 0..d {
        for dx in 0..w {
            tiles.push(IVec2::new(origin_x + dx as i32, origin_z + dz as i32));
        }
    }
    tiles
}

#[must_use]
pub fn rect_hits_occupied(book: &TileOccupationBook, tiles: &[IVec2]) -> bool {
    let weights: Vec<(IVec2, f32)> = tiles.iter().copied().map(|tile| (tile, 1.0)).collect();
    book.would_overlap(&weights)
}

/// Marks the preview report when an unweighted footprint overlaps committed tiles.
/// Does not write the occupation book.
pub fn apply_rect_occupancy_reject(
    report: &mut SitePlacementValidation,
    book: &TileOccupationBook,
    tiles: &[IVec2],
) -> bool {
    if !rect_hits_occupied(book, tiles) {
        return false;
    }
    report.valid = false;
    report.allows_commit = false;
    if !report.errors.iter().any(|e| e == FOOTPRINT_OCCUPIED_ERROR) {
        report.errors.push(FOOTPRINT_OCCUPIED_ERROR.to_string());
    }
    true
}

#[must_use]
pub fn report_is_footprint_occupied(report: &SitePlacementValidation) -> bool {
    report
        .errors
        .iter()
        .any(|e| e == FOOTPRINT_OCCUPIED_ERROR || e == "weighted_overlap")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::SiteId;
    use crate::construction::pending_construction::{
        PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
    };
    use crate::strategic::{
        BuildSiteTile, FootprintTiles, LayerType, SiteArchetype, SitePlacementValidation,
    };

    fn open_report() -> SitePlacementValidation {
        SitePlacementValidation {
            valid: true,
            allows_commit: true,
            terrain_score: 1.0,
            logistics_score: 1.0,
            strategic_score: 1.0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn pending_row() -> PendingBuildBlueprint {
        PendingBuildBlueprint {
            kind: PendingEntryKind::BuildSite,
            label: "player-row".into(),
            archetype: SiteArchetype::CivilHousing,
            origin: BuildSiteTile { x: 1, z: 1 },
            footprint: FootprintTiles { width: 1, depth: 1 },
            layer: LayerType::Surface,
            rotation_quarter_turns: 0,
            mirror_x: false,
            approved: false,
            catalog_id: None,
        }
    }

    #[test]
    fn sweep_preview_move_does_not_change_occupation_or_pending() {
        let mut book = TileOccupationBook::default();
        book.apply_site(SiteId(3), &[(IVec2::new(2, 3), 1.0)]);
        let before = book.clone();
        let mut pending = PendingConstructionQueue::default();
        pending.push(pending_row());
        let action = classify_ghost_place(false, false);
        assert_eq!(action, GhostPlaceAction::Preview);
        assert!(!commits_gameplay(action));
        let mut report = open_report();
        let tiles = footprint_rect_tiles(8, 8, 1, 1);
        assert!(!apply_rect_occupancy_reject(&mut report, &book, &tiles));
        assert!(report.allows_commit);
        assert_eq!(book, before);
        assert_eq!(pending.entries.len(), 1);
        assert!(!pending.entries[0].approved);
    }

    #[test]
    fn sweep_occupied_rect_blocks_commit_and_keeps_pending() {
        let mut book = TileOccupationBook::default();
        book.apply_site(SiteId(9), &[(IVec2::new(4, 5), 1.0)]);
        let before = book.clone();
        let mut pending = PendingConstructionQueue::default();
        pending.push(pending_row());
        let tiles = footprint_rect_tiles(4, 5, 2, 1);
        let mut report = open_report();
        assert!(apply_rect_occupancy_reject(&mut report, &book, &tiles));
        assert!(!report.allows_commit);
        assert!(!report.valid);
        assert!(report_is_footprint_occupied(&report));
        let action = classify_ghost_place(report.allows_commit, true);
        assert_eq!(action, GhostPlaceAction::RejectOccupied);
        assert!(!commits_gameplay(action));
        assert_eq!(book, before);
        assert_eq!(pending.entries.len(), 1);
        assert_eq!(pending.entries[0].label, "player-row");
    }

    #[test]
    fn drain_empty_footprint_does_not_commit() {
        let mut book = TileOccupationBook::default();
        book.apply_site(SiteId(2), &[(IVec2::new(1, 1), 1.0)]);
        let before = book.clone();
        let mut pending = PendingConstructionQueue::default();
        pending.push(pending_row());
        let mut report = open_report();
        assert!(stamp_empty_footprint(&mut report, 0, 2));
        assert!(!report.allows_commit);
        assert!(report.errors.iter().any(|e| e == EMPTY_FOOTPRINT_ERROR));
        let action = classify_ghost_place(report.allows_commit, false);
        assert!(!commits_gameplay(action));
        assert_eq!(pending.entries.len(), 1);
        assert_eq!(book, before);
        assert!(!stamp_empty_footprint(&mut open_report(), 1, 1));
    }

    #[test]
    fn live_unplaceable_footprint_cannot_queue_commit() {
        assert!(!may_queue_commit(0, 2, true, false));
        assert!(!may_queue_commit(2, 2, true, true));
        assert!(!may_queue_commit(2, 2, false, false));
        assert!(may_queue_commit(2, 2, true, false));
        let mut pending = PendingConstructionQueue::default();
        pending.push(pending_row());
        assert_eq!(pending.entries.len(), 1);
        assert!(!pending.entries[0].approved);
    }

    #[test]
    fn push_odd_turn_swaps_the_footprint() {
        assert_eq!(oriented_footprint(2, 5, 1), Some((5, 2)));
        assert_eq!(oriented_footprint(2, 5, 2), Some((2, 5)));
        assert_eq!(oriented_footprint(0, 5, 1), None);
    }

    #[test]
    fn cont_empty_footprint_lists_no_tiles() {
        assert!(footprint_rect_tiles(4, 5, 0, 2).is_empty());
        assert!(footprint_rect_tiles(4, 5, 2, 0).is_empty());
        assert_eq!(footprint_rect_tiles(4, 5, 1, 1), vec![IVec2::new(4, 5)]);
    }

    #[test]
    fn cont_occupancy_rect_follows_the_quarter_turn() {
        let flat = oriented_rect_tiles(0, 0, 2, 1, 0);
        let turned = oriented_rect_tiles(0, 0, 2, 1, 1);
        assert_eq!(flat, vec![IVec2::new(0, 0), IVec2::new(1, 0)]);
        assert_eq!(turned, vec![IVec2::new(0, 0), IVec2::new(0, 1)]);
        assert!(oriented_rect_tiles(0, 0, 0, 2, 1).is_empty());
    }

    #[test]
    fn over_even_quarter_keeps_the_axes() {
        assert_eq!(oriented_footprint(6, 3, 4), Some((6, 3)));
        assert_eq!(
            oriented_rect_tiles(0, 0, 6, 3, 4),
            oriented_rect_tiles(0, 0, 6, 3, 0)
        );
        assert_ne!(
            oriented_rect_tiles(0, 0, 6, 3, 4),
            oriented_rect_tiles(0, 0, 6, 3, 1)
        );
    }

    fn river(x: i32, z: i32) -> bool {
        (x, z) == (3, 4)
    }

    #[test]
    fn wet_house_on_water_cannot_commit() {
        let tiles = vec![IVec2::new(3, 4)];
        let mut report = open_report();
        assert!(apply_water_footprint_reject(
            &mut report,
            Some("house"),
            &tiles,
            river
        ));
        assert!(!report.allows_commit);
        assert!(report.errors.iter().any(|e| e == WATER_FOOTPRINT_ERROR));
        assert!(!may_queue_commit(1, 1, report.allows_commit, false));
    }

    #[test]
    fn wet_port_and_oil_rig_may_occupy_water() {
        let tiles = vec![IVec2::new(3, 4), IVec2::new(4, 4)];
        let mut port = open_report();
        let mut rig = open_report();
        assert!(!apply_water_footprint_reject(
            &mut port,
            Some("sea_port"),
            &tiles,
            river
        ));
        assert!(!apply_water_footprint_reject(
            &mut rig,
            Some("oil-rig"),
            &tiles,
            river
        ));
        assert!(port.allows_commit && rig.allows_commit);
        assert!(building_may_occupy_water(Some("harbor")));
        assert!(building_may_occupy_water(Some("catalog/oil_rig")));
    }

    #[test]
    fn wet_transport_and_water_plant_are_not_water_buildings() {
        assert!(!building_may_occupy_water(Some("transport")));
        assert!(!building_may_occupy_water(Some("support")));
        assert!(!building_may_occupy_water(Some("water_plant")));
        assert!(!building_may_occupy_water(Some("airport")));
        assert!(!building_may_occupy_water(None));
        let mut report = open_report();
        assert!(land_building_blocked_on_water(
            Some("warehouse"),
            &[IVec2::new(2, 2), IVec2::new(3, 4)],
            river
        ));
        assert!(!apply_water_footprint_reject(
            &mut report,
            Some("warehouse"),
            &[IVec2::new(2, 2)],
            river
        ));
        assert!(report.allows_commit);
    }

}
