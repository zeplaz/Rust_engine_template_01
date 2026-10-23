//! **Theaters** (PLAN-FRONT-COMBAT-v1 FC-3): each hostile pair's front cells are partitioned into
//! connected stretches (8-connectivity), long stretches are split along their principal axis into
//! sectors, and every theater keeps a **stable id** across front revisions by best overlap
//! (Jaccard) with the previous revision. Pure core + one thin system; sole writer of [`TheaterBook`].

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;

use super::balance::{FrontBalance, TheaterParams};
use super::front::{FrontCell, FrontSegment};
use crate::strategic::FrontlineState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TheaterId(pub u32);

#[derive(Clone, Debug, PartialEq)]
pub struct Theater {
    pub id: TheaterId,
    /// `(lower, higher)` hostile slots.
    pub pair: (u8, u8),
    /// Front cells of both sides, sorted by `(y, x)`.
    pub cells: Vec<FrontCell>,
    /// Mean tile position.
    pub centroid: Vec2,
    /// Extent along the principal axis, metres.
    pub length_m: f32,
    pub contested_cells: u32,
    tile_set: BTreeSet<(i32, i32)>,
}

impl Theater {
    #[must_use]
    pub fn contains_front_tile(&self, tile: IVec2) -> bool {
        self.tile_set.contains(&(tile.x, tile.y))
    }

    /// Area of responsibility: within `depth_tiles` (Euclidean) of any front cell, both sides.
    #[must_use]
    pub fn aor_contains(&self, tile: IVec2, depth_tiles: f32) -> bool {
        let r = depth_tiles.max(0.0).ceil() as i32;
        let r2 = depth_tiles * depth_tiles;
        for dy in -r..=r {
            for dx in -r..=r {
                if ((dx * dx + dy * dy) as f32) <= r2 && self.tile_set.contains(&(tile.x + dx, tile.y + dy)) {
                    return true;
                }
            }
        }
        false
    }

    #[must_use]
    pub fn cells_of(&self, owner: u8) -> impl Iterator<Item = &FrontCell> {
        self.cells.iter().filter(move |c| c.owner == owner)
    }
}

/// All theaters of the current front revision.
#[derive(Resource, Clone, Debug, Default)]
pub struct TheaterBook {
    /// Sorted by id.
    pub theaters: Vec<Theater>,
    /// `FrontlineState.revision` this was derived from.
    pub source_revision: u64,
    /// Area-of-responsibility depth in tiles (from the balance table).
    pub depth_tiles: f32,
    next_id: u32,
}

impl TheaterBook {
    #[must_use]
    pub fn get(&self, id: TheaterId) -> Option<&Theater> {
        self.theaters.iter().find(|t| t.id == id)
    }

    /// Theater whose area of responsibility covers `tile` (lowest id first).
    #[must_use]
    pub fn theater_at(&self, tile: IVec2) -> Option<&Theater> {
        self.theaters.iter().find(|t| t.aor_contains(tile, self.depth_tiles))
    }

    /// Re-partition `segments`, keeping ids of theaters that still overlap their predecessor.
    pub fn update(&mut self, segments: &[FrontSegment], params: &TheaterParams, tile_size_m: f32, revision: u64) {
        let cands = partition_segments(segments, params.max_len_m, tile_size_m);
        let previous = std::mem::take(&mut self.theaters);
        self.theaters = assign_stable_ids(&previous, cands, &mut self.next_id, params.id_keep_overlap);
        self.depth_tiles = params.depth_m / tile_size_m.max(f32::EPSILON);
        self.source_revision = revision;
    }
}

/// A partition result before it has an id.
#[derive(Clone, Debug, PartialEq)]
pub struct TheaterCandidate {
    pub pair: (u8, u8),
    pub cells: Vec<FrontCell>,
    pub length_m: f32,
}

fn key(t: IVec2) -> (i32, i32) {
    (t.x, t.y)
}

/// 8-connected components of `cells`, each sorted by `(y, x)`; components ordered by first cell.
fn components(cells: &[FrontCell]) -> Vec<Vec<FrontCell>> {
    let by_tile: BTreeMap<(i32, i32), &FrontCell> = cells.iter().map(|c| (key(c.tile), c)).collect();
    let mut seen: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut out = Vec::new();
    let mut order: Vec<(i32, i32)> = by_tile.keys().copied().collect();
    order.sort_by_key(|(x, y)| (*y, *x));
    for start in order {
        if !seen.insert(start) {
            continue;
        }
        let mut stack = vec![start];
        let mut comp = Vec::new();
        while let Some(t) = stack.pop() {
            comp.push(*by_tile[&t]);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let n = (t.0 + dx, t.1 + dy);
                    if by_tile.contains_key(&n) && seen.insert(n) {
                        stack.push(n);
                    }
                }
            }
        }
        comp.sort_by_key(|c| (c.tile.y, c.tile.x));
        out.push(comp);
    }
    out
}

/// Principal axis (unit) of the tile cloud; `(1, 0)` for degenerate clouds.
fn principal_axis(cells: &[FrontCell]) -> (f64, f64) {
    let n = cells.len() as f64;
    let (mx, my) = cells.iter().fold((0.0, 0.0), |a, c| (a.0 + f64::from(c.tile.x), a.1 + f64::from(c.tile.y)));
    let (mx, my) = (mx / n, my / n);
    let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
    for c in cells {
        let (dx, dy) = (f64::from(c.tile.x) - mx, f64::from(c.tile.y) - my);
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    if sxx + syy < 1e-9 {
        return (1.0, 0.0);
    }
    let theta = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    let (s, c) = theta.sin_cos();
    // Canonical sign so the same shape always projects the same way.
    if c < -1e-9 || (c.abs() <= 1e-9 && s < 0.0) {
        (-c, -s)
    } else {
        (c, s)
    }
}

/// Split every segment into connected stretches, then long stretches into equal sectors along the
/// principal axis (`ceil(extent / max_len_m)` sectors).
#[must_use]
pub fn partition_segments(segments: &[FrontSegment], max_len_m: f32, tile_size_m: f32) -> Vec<TheaterCandidate> {
    let mut out = Vec::new();
    for seg in segments {
        for comp in components(&seg.cells) {
            let (ax, ay) = principal_axis(&comp);
            let proj = |c: &FrontCell| f64::from(c.tile.x) * ax + f64::from(c.tile.y) * ay;
            let lo = comp.iter().map(proj).fold(f64::INFINITY, f64::min);
            let hi = comp.iter().map(proj).fold(f64::NEG_INFINITY, f64::max);
            let extent_m = ((hi - lo) + 1.0) as f32 * tile_size_m;
            let k = ((extent_m / max_len_m.max(f32::EPSILON)).ceil() as usize).max(1);
            if k == 1 {
                out.push(TheaterCandidate { pair: seg.pair, cells: comp, length_m: extent_m });
                continue;
            }
            let span = (hi - lo).max(1e-9);
            let mut bins: Vec<Vec<FrontCell>> = vec![Vec::new(); k];
            for c in comp {
                let b = (((proj(&c) - lo) / span) * k as f64).floor() as usize;
                bins[b.min(k - 1)].push(c);
            }
            for cells in bins.into_iter().filter(|b| !b.is_empty()) {
                let l = cells.iter().map(proj).fold(f64::INFINITY, f64::min);
                let h = cells.iter().map(proj).fold(f64::NEG_INFINITY, f64::max);
                out.push(TheaterCandidate {
                    pair: seg.pair,
                    cells,
                    length_m: ((h - l) + 1.0) as f32 * tile_size_m,
                });
            }
        }
    }
    out.sort_by_key(|c| (c.pair, c.cells[0].tile.y, c.cells[0].tile.x));
    out
}

fn jaccard(a: &BTreeSet<(i32, i32)>, b: &BTreeSet<(i32, i32)>) -> f32 {
    let inter = a.intersection(b).count();
    let union = a.len() + b.len() - inter;
    if union == 0 {
        0.0
    } else {
        inter as f32 / union as f32
    }
}

/// One-to-one greedy assignment of candidates to previous ids (best Jaccard ≥ `keep_overlap` first);
/// unmatched candidates get fresh ids.
#[must_use]
pub fn assign_stable_ids(
    previous: &[Theater],
    cands: Vec<TheaterCandidate>,
    next_id: &mut u32,
    keep_overlap: f32,
) -> Vec<Theater> {
    let sets: Vec<BTreeSet<(i32, i32)>> =
        cands.iter().map(|c| c.cells.iter().map(|f| key(f.tile)).collect()).collect();
    let mut pairs: Vec<(f32, usize, usize)> = Vec::new();
    for (pi, p) in previous.iter().enumerate() {
        for (ci, c) in cands.iter().enumerate() {
            if p.pair != c.pair {
                continue;
            }
            let j = jaccard(&p.tile_set, &sets[ci]);
            if j >= keep_overlap && j > 0.0 {
                pairs.push((j, pi, ci));
            }
        }
    }
    pairs.sort_by(|a, b| b.0.total_cmp(&a.0).then(previous[a.1].id.cmp(&previous[b.1].id)).then(a.2.cmp(&b.2)));
    let mut assigned: Vec<Option<TheaterId>> = vec![None; cands.len()];
    let mut used: BTreeSet<TheaterId> = BTreeSet::new();
    for (_, pi, ci) in pairs {
        if assigned[ci].is_none() && used.insert(previous[pi].id) {
            assigned[ci] = Some(previous[pi].id);
        }
    }
    let mut out: Vec<Theater> = cands
        .into_iter()
        .zip(sets)
        .zip(assigned)
        .map(|((c, tile_set), id)| {
            let id = id.unwrap_or_else(|| {
                let id = TheaterId(*next_id);
                *next_id += 1;
                id
            });
            let n = c.cells.len() as f32;
            let sum = c.cells.iter().fold(Vec2::ZERO, |a, f| a + f.tile.as_vec2());
            Theater {
                id,
                pair: c.pair,
                contested_cells: c.cells.iter().filter(|f| f.contested).count() as u32,
                centroid: sum / n.max(1.0),
                length_m: c.length_m,
                cells: c.cells,
                tile_set,
            }
        })
        .collect();
    out.sort_by_key(|t| t.id);
    out
}

/// Re-derive theaters when the front revision changes.
pub fn derive_theaters_system(front: Res<FrontlineState>, balance: Res<FrontBalance>, mut book: ResMut<TheaterBook>) {
    if front.revision == book.source_revision && book.depth_tiles > 0.0 {
        return;
    }
    book.update(&front.segments, &balance.theater, balance.tile_size_m, front.revision);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(x: i32, y: i32, owner: u8) -> FrontCell {
        FrontCell { tile: IVec2::new(x, y), owner, facing: 1 - owner, contested: false, margin: 0.5 }
    }

    /// Vertical two-column front of `len` rows starting at `(x, y0)`.
    fn column_front(x: i32, y0: i32, len: i32) -> FrontSegment {
        let mut cells = Vec::new();
        for y in y0..y0 + len {
            cells.push(cell(x, y, 0));
            cells.push(cell(x + 1, y, 1));
        }
        cells.sort_by_key(|c| (c.tile.y, c.tile.x));
        FrontSegment { pair: (0, 1), cells, polylines: vec![], contested_cells: 0, length_m: len as f32 * 25.0 }
    }

    fn merged(parts: &[FrontSegment]) -> Vec<FrontSegment> {
        let mut cells: Vec<FrontCell> = parts.iter().flat_map(|p| p.cells.clone()).collect();
        cells.sort_by_key(|c| (c.tile.y, c.tile.x));
        vec![FrontSegment { pair: (0, 1), cells, polylines: vec![], contested_cells: 0, length_m: 0.0 }]
    }

    fn params() -> TheaterParams {
        FrontBalance::default().theater
    }

    #[test]
    fn two_separate_blobs_make_two_theaters() {
        let segs = merged(&[column_front(10, 0, 10), column_front(10, 50, 10)]);
        let mut book = TheaterBook::default();
        book.update(&segs, &params(), 25.0, 1);
        assert_eq!(book.theaters.len(), 2);
        assert_ne!(book.theaters[0].id, book.theaters[1].id);
    }

    #[test]
    fn long_front_splits_into_sectors() {
        // 200 rows * 25 m = 5000 m > 2000 m max => 3 sectors.
        let segs = vec![column_front(10, 0, 200)];
        let mut book = TheaterBook::default();
        book.update(&segs, &params(), 25.0, 1);
        assert_eq!(book.theaters.len(), 3);
        let total: usize = book.theaters.iter().map(|t| t.cells.len()).sum();
        assert_eq!(total, 400);
        assert!(book.theaters.iter().all(|t| t.length_m <= 2000.0 + 25.0));
    }

    #[test]
    fn ids_are_stable_across_a_one_cell_shift() {
        let mut book = TheaterBook::default();
        book.update(&[column_front(10, 0, 100)], &params(), 25.0, 1);
        let before: Vec<TheaterId> = book.theaters.iter().map(|t| t.id).collect();
        assert!(before.len() >= 2);
        // Whole front steps one tile east and grows two rows.
        book.update(&[column_front(11, 0, 102)], &params(), 25.0, 2);
        let after: Vec<TheaterId> = book.theaters.iter().map(|t| t.id).collect();
        assert_eq!(before, after);
        assert_eq!(book.source_revision, 2);
    }

    #[test]
    fn vanished_theater_id_is_not_reused() {
        let mut book = TheaterBook::default();
        book.update(&[column_front(10, 0, 10)], &params(), 25.0, 1);
        let first = book.theaters[0].id;
        book.update(&[], &params(), 25.0, 2);
        assert!(book.theaters.is_empty());
        book.update(&[column_front(80, 80, 10)], &params(), 25.0, 3);
        assert_ne!(book.theaters[0].id, first);
    }

    #[test]
    fn area_of_responsibility_covers_depth_band_on_both_sides() {
        let mut book = TheaterBook::default();
        book.update(&[column_front(10, 0, 10)], &params(), 25.0, 1);
        assert!((book.depth_tiles - 20.0).abs() < 1e-4);
        assert!(book.theater_at(IVec2::new(30, 5)).is_some());
        assert!(book.theater_at(IVec2::new(-9, 5)).is_some());
        assert!(book.theater_at(IVec2::new(60, 5)).is_none());
    }
}
