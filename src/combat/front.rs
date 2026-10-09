//! **Cell-level front derivation** (PLAN-FRONT-COMBAT-v1 FC-1). Pure and deterministic.
//!
//! Input: per-tile faction control (the `ChunkStrategicOverlay.faction_control` slab, gathered into a
//! sparse [`ControlGrid`]) plus a [`Hostility`] matrix. Output: per hostile pair, the front cells
//! (contested cells + owned cells touching the enemy) and the ownership boundary as oriented
//! polylines in **tile-corner** coordinates, the lower slot always on the left of travel so a
//! renderer can put FLOT teeth on the right (toward the higher slot) without re-deriving sides.

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::IVec2;

use crate::strategic::MAX_STRATEGIC_FACTION_SLOTS;

pub type SlotControl = [f32; MAX_STRATEGIC_FACTION_SLOTS];

/// Which slot pairs fight. Symmetric, irreflexive.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hostility {
    mask: [u16; MAX_STRATEGIC_FACTION_SLOTS],
}

impl Hostility {
    #[must_use]
    pub fn from_fn(mut hostile: impl FnMut(u8, u8) -> bool) -> Self {
        let mut h = Self::default();
        for a in 0..MAX_STRATEGIC_FACTION_SLOTS as u8 {
            for b in (a + 1)..MAX_STRATEGIC_FACTION_SLOTS as u8 {
                if hostile(a, b) || hostile(b, a) {
                    h.set(a, b);
                }
            }
        }
        h
    }

    /// Legacy front: slots 0 and 1 at war (what `derive_frontline_from_control_system` always assumed).
    #[must_use]
    pub fn legacy_pair() -> Self {
        let mut h = Self::default();
        h.set(0, 1);
        h
    }

    pub fn set(&mut self, a: u8, b: u8) {
        let (a, b) = (a as usize, b as usize);
        if a == b || a >= MAX_STRATEGIC_FACTION_SLOTS || b >= MAX_STRATEGIC_FACTION_SLOTS {
            return;
        }
        self.mask[a] |= 1 << b;
        self.mask[b] |= 1 << a;
    }

    #[inline]
    #[must_use]
    pub fn hostile(&self, a: u8, b: u8) -> bool {
        let (a, b) = (a as usize, b as usize);
        a < MAX_STRATEGIC_FACTION_SLOTS && b < MAX_STRATEGIC_FACTION_SLOTS && self.mask[a] & (1 << b) != 0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mask.iter().all(|m| *m == 0)
    }
}

/// Sparse tile → control. Only tiles where some slot reaches the ownership floor are kept.
#[derive(Clone, Debug, Default)]
pub struct ControlGrid {
    cells: HashMap<(i32, i32), SlotControl>,
}

impl ControlGrid {
    pub fn insert(&mut self, tile: IVec2, control: SlotControl) {
        self.cells.insert((tile.x, tile.y), control);
    }

    #[must_use]
    pub fn get(&self, tile: IVec2) -> Option<&SlotControl> {
        self.cells.get(&(tile.x, tile.y))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Gather one chunk slab (row-major cells, `size.x` wide) at `origin` tile, keeping cells ≥ `floor`.
    pub fn absorb_slab(&mut self, origin: IVec2, size_x: u32, slab: &[SlotControl], floor: f32) {
        let w = size_x.max(1) as usize;
        for (ci, row) in slab.iter().enumerate() {
            if row.iter().any(|v| *v >= floor) {
                let tile = origin + IVec2::new((ci % w) as i32, (ci / w) as i32);
                self.insert(tile, *row);
            }
        }
    }
}

/// Ownership read of one tile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellOwnership {
    pub owner: u8,
    pub owner_control: f32,
    /// Strongest slot hostile to `owner` that also clears the floor.
    pub rival: Option<(u8, f32)>,
    pub contested: bool,
}

/// Owner = strongest slot (ties → lower slot) at or above `own_min`.
#[must_use]
pub fn classify_cell(control: &SlotControl, hostility: &Hostility, own_min: f32, contest_band: f32) -> Option<CellOwnership> {
    let mut owner: Option<(u8, f32)> = None;
    for (slot, v) in control.iter().enumerate() {
        if *v >= own_min && owner.is_none_or(|(_, best)| *v > best) {
            owner = Some((slot as u8, *v));
        }
    }
    let (owner, owner_control) = owner?;
    let mut rival: Option<(u8, f32)> = None;
    for (slot, v) in control.iter().enumerate() {
        let slot = slot as u8;
        if slot != owner && *v >= own_min && hostility.hostile(owner, slot) && rival.is_none_or(|(_, best)| *v > best) {
            rival = Some((slot, *v));
        }
    }
    let contested = rival.is_some_and(|(_, rv)| owner_control - rv < contest_band);
    Some(CellOwnership { owner, owner_control, rival, contested })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrontCell {
    pub tile: IVec2,
    pub owner: u8,
    /// Hostile slot this cell faces (the other member of the segment pair).
    pub facing: u8,
    pub contested: bool,
    /// `owner_control − facing_control` (≥ 0 means owner leads).
    pub margin: f32,
}

/// Oriented boundary chain in tile-corner coordinates. `left_slot` is on the left of travel.
#[derive(Clone, Debug, PartialEq)]
pub struct FrontPolyline {
    pub points: Vec<IVec2>,
    pub left_slot: u8,
    pub right_slot: u8,
    pub closed: bool,
    /// Unit edges walked (before collinear compression).
    pub edge_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FrontSegment {
    /// `(lower, higher)` hostile slots.
    pub pair: (u8, u8),
    /// Sorted by `(y, x)`.
    pub cells: Vec<FrontCell>,
    pub polylines: Vec<FrontPolyline>,
    pub contested_cells: u32,
    pub length_m: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrontDerivation {
    /// Sorted by pair.
    pub segments: Vec<FrontSegment>,
}

impl FrontDerivation {
    #[must_use]
    pub fn total_front_cells(&self) -> usize {
        self.segments.iter().map(|s| s.cells.len()).sum()
    }

    #[must_use]
    pub fn total_length_m(&self) -> f32 {
        self.segments.iter().map(|s| s.length_m).sum()
    }
}

const NEIGHBORS4: [IVec2; 4] = [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)];

/// Directed unit edge with the cell at `tile` on its left, for the side toward `dir`.
fn boundary_edge(tile: IVec2, dir: IVec2) -> (IVec2, IVec2) {
    let (x, y) = (tile.x, tile.y);
    match (dir.x, dir.y) {
        (1, 0) => (IVec2::new(x + 1, y), IVec2::new(x + 1, y + 1)),
        (-1, 0) => (IVec2::new(x, y + 1), IVec2::new(x, y)),
        (0, 1) => (IVec2::new(x + 1, y + 1), IVec2::new(x, y + 1)),
        _ => (IVec2::new(x, y), IVec2::new(x + 1, y)),
    }
}

fn turn_rank(incoming: IVec2, candidate: IVec2) -> u8 {
    let left = IVec2::new(-incoming.y, incoming.x);
    if candidate == left {
        0
    } else if candidate == incoming {
        1
    } else {
        2
    }
}

fn compress_collinear(points: Vec<IVec2>) -> Vec<IVec2> {
    if points.len() <= 2 {
        return points;
    }
    let mut out = Vec::with_capacity(points.len());
    out.push(points[0]);
    for i in 1..points.len() - 1 {
        let a = points[i] - *out.last().unwrap();
        let b = points[i + 1] - points[i];
        if a.x * b.y - a.y * b.x != 0 {
            out.push(points[i]);
        }
    }
    out.push(*points.last().unwrap());
    out
}

/// Chain directed unit edges into polylines. Deterministic: heads (no incoming edge) first in
/// sorted order, then remaining cycles; at forks prefer left turn, then straight, then right.
#[must_use]
pub fn chain_edges(mut edges: Vec<(IVec2, IVec2)>, left_slot: u8, right_slot: u8) -> Vec<FrontPolyline> {
    let key = |p: IVec2| (p.y, p.x);
    edges.sort_by_key(|(s, e)| (key(*s), key(*e)));
    edges.dedup();
    let mut out_edges: HashMap<IVec2, Vec<usize>> = HashMap::new();
    let mut in_deg: HashMap<IVec2, u32> = HashMap::new();
    for (i, (s, e)) in edges.iter().enumerate() {
        out_edges.entry(*s).or_default().push(i);
        *in_deg.entry(*e).or_default() += 1;
    }
    let mut used = vec![false; edges.len()];
    let mut lines = Vec::new();
    let mut walk = |start: usize, used: &mut Vec<bool>| -> FrontPolyline {
        let (s, e) = edges[start];
        used[start] = true;
        let mut pts = vec![s, e];
        let mut dir = e - s;
        let mut cur = e;
        let mut n = 1u32;
        loop {
            let Some(cands) = out_edges.get(&cur) else { break };
            let next = cands
                .iter()
                .copied()
                .filter(|i| !used[*i])
                .min_by_key(|i| turn_rank(dir, edges[*i].1 - edges[*i].0));
            let Some(i) = next else { break };
            used[i] = true;
            n += 1;
            dir = edges[i].1 - edges[i].0;
            cur = edges[i].1;
            pts.push(cur);
        }
        let closed = pts.len() > 2 && pts.first() == pts.last();
        FrontPolyline { points: compress_collinear(pts), left_slot, right_slot, closed, edge_count: n }
    };
    for i in 0..edges.len() {
        if !used[i] && in_deg.get(&edges[i].0).copied().unwrap_or(0) == 0 {
            lines.push(walk(i, &mut used));
        }
    }
    for i in 0..edges.len() {
        if !used[i] {
            lines.push(walk(i, &mut used));
        }
    }
    lines
}

/// Derive fronts for every hostile pair present in `grid`.
#[must_use]
pub fn derive_fronts(grid: &ControlGrid, hostility: &Hostility, own_min: f32, contest_band: f32, tile_size_m: f32) -> FrontDerivation {
    let mut owners: HashMap<(i32, i32), CellOwnership> = HashMap::with_capacity(grid.len());
    for (k, c) in &grid.cells {
        if let Some(o) = classify_cell(c, hostility, own_min, contest_band) {
            owners.insert(*k, o);
        }
    }
    let mut tiles: Vec<(i32, i32)> = owners.keys().copied().collect();
    tiles.sort_by_key(|(x, y)| (*y, *x));

    let mut cells_by_pair: BTreeMap<(u8, u8), BTreeMap<(i32, i32), FrontCell>> = BTreeMap::new();
    let mut edges_by_pair: BTreeMap<(u8, u8), Vec<(IVec2, IVec2)>> = BTreeMap::new();
    let pair_of = |a: u8, b: u8| if a < b { (a, b) } else { (b, a) };
    let control_of = |t: (i32, i32), slot: u8| grid.cells.get(&t).map_or(0.0, |c| c[slot as usize]);

    for t in &tiles {
        let o = owners[t];
        let tile = IVec2::new(t.0, t.1);
        if o.contested {
            let (rival, rv) = o.rival.expect("contested implies rival");
            cells_by_pair.entry(pair_of(o.owner, rival)).or_default().insert(
                (t.1, t.0),
                FrontCell { tile, owner: o.owner, facing: rival, contested: true, margin: o.owner_control - rv },
            );
        }
        for d in NEIGHBORS4 {
            let nt = (t.0 + d.x, t.1 + d.y);
            let Some(no) = owners.get(&nt) else { continue };
            if no.owner == o.owner || !hostility.hostile(o.owner, no.owner) {
                continue;
            }
            let pair = pair_of(o.owner, no.owner);
            let margin = o.owner_control - control_of(*t, no.owner);
            cells_by_pair.entry(pair).or_default().entry((t.1, t.0)).or_insert(FrontCell {
                tile,
                owner: o.owner,
                facing: no.owner,
                contested: o.contested,
                margin,
            });
            if o.owner == pair.0 {
                edges_by_pair.entry(pair).or_default().push(boundary_edge(tile, d));
            }
        }
    }

    let mut segments = Vec::with_capacity(cells_by_pair.len());
    for (pair, cells) in cells_by_pair {
        let cells: Vec<FrontCell> = cells.into_values().collect();
        let contested_cells = cells.iter().filter(|c| c.contested).count() as u32;
        let polylines = chain_edges(edges_by_pair.remove(&pair).unwrap_or_default(), pair.0, pair.1);
        let edges: u32 = polylines.iter().map(|p| p.edge_count).sum();
        segments.push(FrontSegment {
            pair,
            cells,
            polylines,
            contested_cells,
            length_m: edges as f32 * tile_size_m,
        });
    }
    FrontDerivation { segments }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctl(slot: usize, v: f32) -> SlotControl {
        let mut c = [0.0; MAX_STRATEGIC_FACTION_SLOTS];
        c[slot] = v;
        c
    }

    /// `w × h` map, slot 0 on x < split, slot 1 on x ≥ split.
    fn split_grid(w: i32, h: i32, split: i32) -> ControlGrid {
        let mut g = ControlGrid::default();
        for y in 0..h {
            for x in 0..w {
                g.insert(IVec2::new(x, y), ctl(usize::from(x >= split), 0.8));
            }
        }
        g
    }

    #[test]
    fn straight_front_between_two_blocks() {
        let g = split_grid(6, 4, 3);
        let d = derive_fronts(&g, &Hostility::legacy_pair(), 0.25, 0.3, 25.0);
        assert_eq!(d.segments.len(), 1);
        let s = &d.segments[0];
        assert_eq!(s.pair, (0, 1));
        // Columns 2 (slot 0) and 3 (slot 1), 4 rows each.
        assert_eq!(s.cells.len(), 8);
        assert!(s.cells.iter().all(|c| c.tile.x == 2 || c.tile.x == 3));
        assert_eq!(s.polylines.len(), 1);
        let p = &s.polylines[0];
        assert_eq!(p.points, vec![IVec2::new(3, 0), IVec2::new(3, 4)]);
        assert_eq!((p.left_slot, p.right_slot), (0, 1));
        assert!(!p.closed);
        assert!((s.length_m - 100.0).abs() < 1e-4);
    }

    #[test]
    fn non_hostile_pair_has_no_front() {
        let g = split_grid(6, 4, 3);
        let d = derive_fronts(&g, &Hostility::default(), 0.25, 0.3, 25.0);
        assert!(d.segments.is_empty());
    }

    #[test]
    fn enclave_is_closed_ring() {
        let mut g = ControlGrid::default();
        for y in 0..5 {
            for x in 0..5 {
                let slot = usize::from(x == 2 && y == 2);
                g.insert(IVec2::new(x, y), ctl(slot, 0.9));
            }
        }
        let d = derive_fronts(&g, &Hostility::legacy_pair(), 0.25, 0.3, 25.0);
        let s = &d.segments[0];
        assert_eq!(s.cells.len(), 5, "enclave + 4 neighbours");
        assert_eq!(s.polylines.len(), 1);
        let p = &s.polylines[0];
        assert!(p.closed);
        assert_eq!(p.edge_count, 4);
        // Slot 0 surrounds the enclave and must be on the left: clockwise around the slot-1 cell.
        assert_eq!(p.left_slot, 0);
        assert!((s.length_m - 100.0).abs() < 1e-4);
    }

    #[test]
    fn contested_cell_is_front_even_without_neighbour() {
        let mut g = ControlGrid::default();
        let mut c = ctl(0, 0.6);
        c[1] = 0.5;
        g.insert(IVec2::new(10, 10), c);
        let d = derive_fronts(&g, &Hostility::legacy_pair(), 0.25, 0.3, 25.0);
        let s = &d.segments[0];
        assert_eq!(s.contested_cells, 1);
        assert_eq!(s.cells[0].owner, 0);
        assert_eq!(s.cells[0].facing, 1);
        assert!((s.cells[0].margin - 0.1).abs() < 1e-5);
        assert!(s.polylines.is_empty());
    }

    #[test]
    fn below_floor_is_unowned() {
        let o = classify_cell(&ctl(3, 0.1), &Hostility::legacy_pair(), 0.25, 0.3);
        assert!(o.is_none());
    }

    #[test]
    fn three_way_fronts_split_by_pair() {
        // 0 | 1 | 2 columns; 0-1 and 1-2 hostile, 0-2 allied.
        let mut g = ControlGrid::default();
        for y in 0..3 {
            for x in 0..9 {
                g.insert(IVec2::new(x, y), ctl((x / 3) as usize, 0.7));
            }
        }
        let mut h = Hostility::default();
        h.set(0, 1);
        h.set(1, 2);
        let d = derive_fronts(&g, &h, 0.25, 0.3, 25.0);
        let pairs: Vec<_> = d.segments.iter().map(|s| s.pair).collect();
        assert_eq!(pairs, vec![(0, 1), (1, 2)]);
    }

    #[test]
    fn derivation_is_deterministic() {
        let mut g = ControlGrid::default();
        for y in 0..12 {
            for x in 0..12 {
                let slot = usize::from((x * 7 + y * 3) % 5 < 2);
                g.insert(IVec2::new(x, y), ctl(slot, 0.5 + ((x + y) % 3) as f32 * 0.1));
            }
        }
        let a = derive_fronts(&g, &Hostility::legacy_pair(), 0.25, 0.3, 25.0);
        let b = derive_fronts(&g.clone(), &Hostility::legacy_pair(), 0.25, 0.3, 25.0);
        assert_eq!(a, b);
        // Every unit boundary edge is walked exactly once.
        let edges: u32 = a.segments[0].polylines.iter().map(|p| p.edge_count).sum();
        let mut expected = 0;
        for y in 0..12 {
            for x in 0..12 {
                let me = g.get(IVec2::new(x, y)).unwrap();
                if me[0] < me[1] {
                    continue;
                }
                for d in NEIGHBORS4 {
                    if let Some(n) = g.get(IVec2::new(x, y) + d) {
                        if n[1] > n[0] {
                            expected += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(edges, expected);
    }

    #[test]
    fn absorb_slab_keeps_only_owned_cells() {
        let mut g = ControlGrid::default();
        let slab = vec![ctl(0, 0.9), ctl(0, 0.1), ctl(1, 0.3), ctl(1, 0.0)];
        g.absorb_slab(IVec2::new(32, 64), 2, &slab, 0.25);
        assert_eq!(g.len(), 2);
        assert!(g.get(IVec2::new(32, 64)).is_some());
        assert!(g.get(IVec2::new(32, 65)).is_some());
    }
}
