//! Chunk residency helpers — Moore neighborhood rings for Wave C ghost bands.

use std::collections::{HashMap, HashSet};

use bevy::prelude::{IVec2, Resource};

use super::budget::STREAMING_RESIDENCY_UNLOAD_HYSTERESIS_CHUNKS;

/// Chunk coords in a square window around `focus` (inclusive), row-major order.
#[must_use]
pub fn chunk_window_coords(focus: IVec2, radius: i32) -> Vec<IVec2> {
    if radius < 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for y in (focus.y - radius)..=(focus.y + radius) {
        for x in (focus.x - radius)..=(focus.x + radius) {
            out.push(IVec2::new(x, y));
        }
    }
    out
}

/// Center chunk plus its eight Moore neighbors (ghost-band seed set).
#[must_use]
pub fn ghost_band_seed_coords(center: IVec2) -> Vec<IVec2> {
    chunk_window_coords(center, 1)
}

/// Chebyshev distance in chunk cells (matches square [`chunk_window_coords`] windows).
#[must_use]
pub fn chunk_chebyshev_distance(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

/// Chunk is inside the load window (radius `R`).
#[must_use]
pub fn chunk_within_load_radius(coord: IVec2, focus: IVec2, load_radius: i32) -> bool {
    if load_radius < 0 {
        return false;
    }
    chunk_chebyshev_distance(coord, focus) <= load_radius
}

/// Chunk may remain resident until outside `R + H` (unload air lock).
#[must_use]
pub fn chunk_within_retain_radius(
    coord: IVec2,
    focus: IVec2,
    load_radius: i32,
    hysteresis: i32,
) -> bool {
    if load_radius < 0 {
        return false;
    }
    let retain = load_radius.saturating_add(hysteresis.max(0));
    chunk_chebyshev_distance(coord, focus) <= retain
}

/// One hysteresis step for player-focus streaming: load inside `R`, drop pending only outside `R + H`.
///
/// Returns sorted coords, load count, unload count (chunks removed from `pending`).
#[must_use]
pub fn streaming_interest_coords_step(
    pending: &[IVec2],
    focus: IVec2,
    load_radius: i32,
    hysteresis: i32,
) -> (Vec<IVec2>, usize, usize) {
    let h = hysteresis.max(STREAMING_RESIDENCY_UNLOAD_HYSTERESIS_CHUNKS.max(1));
    let mut set = HashSet::new();
    let mut unloads = 0usize;
    for &coord in pending {
        if chunk_within_retain_radius(coord, focus, load_radius, h) {
            set.insert(coord);
        } else {
            unloads += 1;
        }
    }
    let pending_set: HashSet<IVec2> = pending.iter().copied().collect();
    let mut loads = 0usize;
    for coord in chunk_window_coords(focus, load_radius) {
        if set.insert(coord) && !pending_set.contains(&coord) {
            loads += 1;
        }
    }
    let mut coords: Vec<IVec2> = set.into_iter().collect();
    coords.sort_by_key(|coord| (coord.y, coord.x));
    (coords, loads, unloads)
}

/// Union LOD/load targets with pending chunks kept inside the unload hysteresis band around `focus`.
#[must_use]
pub fn merge_load_coords_with_residency_hysteresis(
    pending: &[IVec2],
    load_coords: &[IVec2],
    focus: IVec2,
    load_radius: i32,
    hysteresis: i32,
) -> Vec<IVec2> {
    let mut set: HashSet<IVec2> = load_coords.iter().copied().collect();
    let h = hysteresis.max(STREAMING_RESIDENCY_UNLOAD_HYSTERESIS_CHUNKS.max(1));
    for &coord in pending {
        if chunk_within_retain_radius(coord, focus, load_radius, h) {
            set.insert(coord);
        }
    }
    let mut coords: Vec<IVec2> = set.into_iter().collect();
    coords.sort_by_key(|coord| (coord.y, coord.x));
    coords
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkResidencyRole {
    Core,
    GhostBand,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkResidencyEntry {
    pub coord: IVec2,
    pub role: ChunkResidencyRole,
    pub orb_priority: u8,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct ChunkResidencyTable {
    pub entries: HashMap<IVec2, ChunkResidencyEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ghost_band_seed_is_nine_chunk_moore_window() {
        let coords = ghost_band_seed_coords(IVec2::new(2, 3));
        assert_eq!(coords.len(), 9);
        assert!(coords.contains(&IVec2::new(2, 3)));
        assert!(coords.contains(&IVec2::new(3, 4)));
    }

    #[test]
    fn focus_oscillation_across_chunk_border_does_not_thrash_after_first_step() {
        const LOAD_RADIUS: i32 = 1;
        const HYSTERESIS: i32 = STREAMING_RESIDENCY_UNLOAD_HYSTERESIS_CHUNKS;
        let focus_a = IVec2::new(10, 10);
        let focus_b = IVec2::new(11, 10);

        fn residency_churn(prev: &[IVec2], next: &[IVec2]) -> (usize, usize) {
            let prev: HashSet<IVec2> = prev.iter().copied().collect();
            let next: HashSet<IVec2> = next.iter().copied().collect();
            let loads = next.difference(&prev).count();
            let unloads = prev.difference(&next).count();
            (loads, unloads)
        }

        let load_a = chunk_window_coords(focus_a, LOAD_RADIUS);
        let retained = merge_load_coords_with_residency_hysteresis(
            &[],
            &load_a,
            focus_a,
            LOAD_RADIUS,
            HYSTERESIS,
        );
        assert!(!retained.is_empty());
        assert_eq!(residency_churn(&[], &retained), (retained.len(), 0));
        // One cross onto the neighbor chunk so the air-locked superset exists before oscillation.
        let load_b = chunk_window_coords(focus_b, LOAD_RADIUS);
        let mut retained = merge_load_coords_with_residency_hysteresis(
            &retained,
            &load_b,
            focus_b,
            LOAD_RADIUS,
            HYSTERESIS,
        );

        for i in 0..20 {
            let focus = if i % 2 == 0 { focus_a } else { focus_b };
            let load_coords = chunk_window_coords(focus, LOAD_RADIUS);
            let next = merge_load_coords_with_residency_hysteresis(
                &retained,
                &load_coords,
                focus,
                LOAD_RADIUS,
                HYSTERESIS,
            );
            assert_eq!(
                residency_churn(&retained, &next),
                (0, 0),
                "churn on oscillation step {i} focus={focus:?}"
            );
            retained = next;
        }
    }
}
