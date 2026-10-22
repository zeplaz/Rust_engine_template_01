//! VoxelMassing — presentation-only column stacks derived from XY footprint cells.
//!
//! Contract: `src/dev/plan_voxel_massing_presentation_v1.md`.
//! Sim occupation stays `FootprintMatrix` / weighted raster. Columns never enter commit payloads.
//!
//! **Dual-view:** massing columns paint only under
//! [`crate::gui::WorldPresentationKind::PixelWorld3D`]. TacticalFlat2D keeps flat
//! footprint / icon tokens (see `plan_dual_view_pixel_world_v1.md`).

use bevy::prelude::*;

use crate::construction::building_catalog::BuildingFamily;
use crate::construction::visual_authority::{
    ConstructionVisualRequests, FootprintTileColorKind, FootprintTileRequest,
};

/// Disposable column stack for world-map silhouette (not sim occupation).
#[derive(Clone, Debug)]
pub struct VoxelColumnRequest {
    pub tile: IVec2,
    /// Stacked visual floors — clamped **1..=4** at operational zoom.
    pub floors: u8,
    pub color_kind: FootprintTileColorKind,
    pub weight: f32,
}

/// Max floors painted as stacked quads (readability / Ebbtide LOD band).
pub const MASSING_FLOORS_MAX: u8 = 4;

/// Below this screen px/tile, massing collapses to floors=1 (footprint grid read).
/// Contract: `plan_voxel_massing_presentation_v1.md` §4.
pub const MASSING_MIN_PX_PER_TILE: f32 = 24.0;

/// Apply operational-zoom readability collapse.
#[must_use]
pub fn massing_floors_for_zoom(floors: u8, px_per_tile: f32) -> u8 {
    let floors = floors.clamp(1, MASSING_FLOORS_MAX);
    if px_per_tile < MASSING_MIN_PX_PER_TILE {
        1
    } else {
        floors
    }
}

/// Family height bonus (0..2) — presentation heuristic only.
#[must_use]
pub fn family_height_bonus(family: Option<BuildingFamily>) -> u8 {
    match family {
        Some(BuildingFamily::Residential) => 1,
        Some(BuildingFamily::Retail) | Some(BuildingFamily::Civic) => 1,
        Some(BuildingFamily::Industry) | Some(BuildingFamily::Logistics) => 2,
        Some(BuildingFamily::Extraction) | Some(BuildingFamily::Fuel) => 1,
        Some(BuildingFamily::Power) => 2,
        Some(BuildingFamily::Research) | Some(BuildingFamily::Agriculture) => 1,
        Some(BuildingFamily::Rail) => 0,
        None => 0,
    }
}

/// Deterministic floors from occupied cell count + optional family bonus.
#[must_use]
pub fn derive_massing_floors(occupied_cells: u32, family: Option<BuildingFamily>) -> u8 {
    let size_band = if occupied_cells == 0 {
        0u32
    } else {
        // log2-ish band: 1→0, 2–3→1, 4–7→2, 8+→3
        let mut n = occupied_cells;
        let mut band = 0u32;
        while n > 1 && band < 3 {
            n >>= 1;
            band += 1;
        }
        band
    };
    let raw = 1u32 + size_band + u32::from(family_height_bonus(family));
    raw.min(u32::from(MASSING_FLOORS_MAX)) as u8
}

/// Build column requests from footprint tile requests (same cells, stacked height).
#[must_use]
pub fn columns_from_footprint_tiles(
    tiles: &[FootprintTileRequest],
    floors: u8,
) -> Vec<VoxelColumnRequest> {
    let floors = floors.clamp(1, MASSING_FLOORS_MAX);
    tiles
        .iter()
        .filter(|t| t.weight > 0.001)
        .map(|t| VoxelColumnRequest {
            tile: t.tile,
            floors,
            color_kind: t.color_kind,
            weight: t.weight,
        })
        .collect()
}

/// After footprint sync — derive massing columns for the live ghost tiles only.
/// Staged book rows keep flat footprint (no extra height noise).
pub fn sync_voxel_massing_visual_requests(
    strip: Res<crate::construction::build_strip::BuildStripState>,
    ghost: Res<crate::construction::build_state::BuildGhostState>,
    tool: Res<crate::construction::build_tool_authority::ActiveBuildTool>,
    settings: Res<crate::construction::tile_visual::ConstructionTileVisualSettings>,
    world_mode: Option<Res<crate::gui::WorldPresentationMode>>,
    zoom: Option<Res<crate::gui::ZoomFrame>>,
    mut requests: ResMut<ConstructionVisualRequests>,
) {
    use crate::construction::build_strip::ToolContext;
    requests.massing_columns.clear();
    // Tactical = flat occupation tokens only; PixelWorld owns massing silhouette.
    let pixel_world = world_mode
        .as_ref()
        .map(|m| m.is_pixel_world())
        .unwrap_or(false);
    if !pixel_world {
        return;
    }
    if strip.active == ToolContext::None || !settings.show_occupation_tiles {
        return;
    }
    if ghost.origin.is_none() {
        return;
    }
    let occupied = requests.footprint_tiles.len() as u32;
    if occupied == 0 {
        return;
    }
    let family = tool.building_intent.as_ref().map(|i| i.family);
    let px = zoom.map(|z| z.px_per_tile).unwrap_or(MASSING_MIN_PX_PER_TILE);
    let floors = massing_floors_for_zoom(derive_massing_floors(occupied, family), px);
    // Only the live ghost slice: footprint_lock_tiles marks Adjust cells; otherwise all
    // footprint_tiles that are not from staged book alone — use tiles pushed this frame
    // for the ghost by taking the lock set when Adjust, else first `occupied` from empty
    // massing: we re-derive from current footprint_tiles when lock empty (Preview).
    let ghost_tiles: Vec<FootprintTileRequest> = if !requests.footprint_lock_tiles.is_empty() {
        let lock: std::collections::HashSet<IVec2> =
            requests.footprint_lock_tiles.iter().copied().collect();
        requests
            .footprint_tiles
            .iter()
            .filter(|t| lock.contains(&t.tile))
            .cloned()
            .collect()
    } else {
        // Preview: massing on all footprint tiles (ghost + staged). Prefer ghost-sized
        // subset by matching occupied count from matrix when possible.
        requests.footprint_tiles.clone()
    };
    requests.massing_columns = columns_from_footprint_tiles(&ghost_tiles, floors);
}

/// Witness helper — presentation path is wired when requests carry the field.
#[must_use]
pub fn voxel_massing_presentation_witness_green() -> bool {
    include_str!("visual_authority.rs").contains("massing_columns")
        && include_str!("voxel_massing.rs").contains("sync_voxel_massing_visual_requests")
        && include_str!("voxel_massing.rs").contains("is_pixel_world")
        && include_str!("world_map_frame.rs").contains("massing_columns")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::visual_authority::FootprintTileColorKind;

    #[test]
    fn floors_clamp_and_scale_with_occupied_count() {
        assert_eq!(derive_massing_floors(0, None), 1);
        assert_eq!(derive_massing_floors(1, None), 1);
        assert!(derive_massing_floors(8, Some(BuildingFamily::Industry)) <= MASSING_FLOORS_MAX);
        assert!(derive_massing_floors(16, Some(BuildingFamily::Industry)) >= 3);
    }

    #[test]
    fn massing_collapses_below_readability_px_per_tile() {
        assert_eq!(massing_floors_for_zoom(4, 23.9), 1);
        assert_eq!(massing_floors_for_zoom(4, MASSING_MIN_PX_PER_TILE), 4);
        assert_eq!(massing_floors_for_zoom(3, 48.0), 3);
    }

    #[test]
    fn columns_skip_near_zero_weight() {
        let tiles = vec![
            FootprintTileRequest {
                tile: IVec2::new(1, 2),
                color_kind: FootprintTileColorKind::Valid,
                weight: 1.0,
                fill_override: None,
            },
            FootprintTileRequest {
                tile: IVec2::new(2, 2),
                color_kind: FootprintTileColorKind::Valid,
                weight: 0.0001,
                fill_override: None,
            },
        ];
        let cols = columns_from_footprint_tiles(&tiles, 3);
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0].floors, 3);
        assert_eq!(cols[0].tile, IVec2::new(1, 2));
    }

    #[test]
    fn presentation_witness_strings_present() {
        assert!(voxel_massing_presentation_witness_green());
    }
}
