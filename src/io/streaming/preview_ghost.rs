//! Preview / streaming ghost-band coord expansion for tag sampling.

use std::collections::HashSet;

use bevy::prelude::IVec2;

use super::residency::{chunk_window_coords, ghost_band_seed_coords};

#[must_use]
pub fn preview_coords_with_ghost_bands(core: &[IVec2]) -> Vec<IVec2> {
    let mut merged = HashSet::new();
    for center in core {
        merged.extend(ghost_band_seed_coords(*center));
    }
    let mut coords: Vec<IVec2> = merged.into_iter().collect();
    coords.sort_by_key(|coord| (coord.y, coord.x));
    coords
}

#[must_use]
pub fn ghost_band_neighbor_coords_for_preview(focus: IVec2, radius: i32) -> Vec<IVec2> {
    let core = chunk_window_coords(focus, radius);
    preview_coords_with_ghost_bands(&core)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ghost_band_preview_expands_core_window() {
        let core = chunk_window_coords(IVec2::ZERO, 0);
        let expanded = preview_coords_with_ghost_bands(&core);
        assert_eq!(expanded.len(), 9);
    }
}
