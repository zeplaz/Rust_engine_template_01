//! **Pressure points** (PLAN-FRONT-COMBAT-v1 FC-4): explainable per-cell scores on each theater's
//! front, top-N per theater per side per kind after non-max suppression. Pure core + one system;
//! sole writer of [`PressurePointBook`].
//!
//! * `Threatened` (side S defends a cell of its own): enemy force ratio, enemy artillery, thin margin,
//!   S's supply shortfall, easy approach, minus S's fortification cover there.
//! * `Opportunity` (S attacks an enemy front cell): S's local force edge, thin enemy margin, enemy supply
//!   shortfall, easy going, minus enemy artillery and enemy cover there.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::balance::{FrontBalance, PressureWeights};
use super::fortification::FortificationIndex;
use super::front::SlotControl;
use super::theater::{Theater, TheaterBook, TheaterId};
use crate::strategic::{ChunkStrategicOverlay, FrontlineState};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PressureKind {
    Threatened,
    Opportunity,
}

/// Signed, already-weighted contribution of each term (sum = score).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PressureDrivers {
    pub threat: f32,
    pub artillery: f32,
    pub contest: f32,
    pub supply: f32,
    pub mobility: f32,
    pub fortification: f32,
}

impl PressureDrivers {
    #[must_use]
    pub fn total(&self) -> f32 {
        self.threat + self.artillery + self.contest + self.supply + self.mobility + self.fortification
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PressurePoint {
    pub theater: TheaterId,
    /// The faction the point is for (defends it / attacks through it).
    pub side: u8,
    pub kind: PressureKind,
    pub tile: IVec2,
    pub score: f32,
    pub drivers: PressureDrivers,
}

/// Overlay fields the score reads at one tile.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CellFields {
    pub threat: SlotControl,
    pub artillery: SlotControl,
    pub logistics: SlotControl,
    pub mobility_cost: f32,
}

pub type FieldGrid = HashMap<(i32, i32), CellFields>;

#[derive(Resource, Clone, Debug, Default)]
pub struct PressurePointBook {
    /// Sorted by `(theater, side, kind)`, strongest first within each group.
    pub points: Vec<PressurePoint>,
    pub source_tick: u64,
    pub source_theater_revision: u64,
}

impl PressurePointBook {
    pub fn top(&self, theater: TheaterId, side: u8, kind: PressureKind) -> impl Iterator<Item = &PressurePoint> {
        self.points.iter().filter(move |p| p.theater == theater && p.side == side && p.kind == kind)
    }

    /// Strongest point for a side across all theaters.
    #[must_use]
    pub fn best_for(&self, side: u8, kind: PressureKind) -> Option<&PressurePoint> {
        self.points
            .iter()
            .filter(|p| p.side == side && p.kind == kind)
            .max_by(|a, b| a.score.total_cmp(&b.score).then(b.tile.y.cmp(&a.tile.y)).then(b.tile.x.cmp(&a.tile.x)))
    }
}

fn slot(v: &SlotControl, s: u8) -> f32 {
    v.get(s as usize).copied().unwrap_or(0.0)
}

/// Drivers for one candidate. `own` is the side the point is for, `enemy` its opponent.
#[must_use]
pub fn score_cell(
    kind: PressureKind,
    own: u8,
    enemy: u8,
    margin: f32,
    f: &CellFields,
    cover: f32,
    w: &PressureWeights,
) -> PressureDrivers {
    let (e, o) = (slot(&f.threat, enemy), slot(&f.threat, own));
    let contest = w.w_contest * (1.0 - margin.abs().clamp(0.0, 1.0));
    let mobility = w.w_mobility * (1.0 - f.mobility_cost.clamp(0.0, 1.0));
    match kind {
        PressureKind::Threatened => PressureDrivers {
            threat: w.w_threat * e * (e / (e + o + 1e-3)),
            artillery: w.w_artillery * slot(&f.artillery, enemy),
            contest,
            supply: w.w_supply * (1.0 - slot(&f.logistics, own).clamp(0.0, 1.0)),
            mobility,
            fortification: -w.w_fortification * cover,
        },
        PressureKind::Opportunity => PressureDrivers {
            threat: w.w_threat * (o - e).clamp(-1.0, 1.0),
            artillery: -w.w_artillery * slot(&f.artillery, enemy),
            contest,
            supply: w.w_supply * (1.0 - slot(&f.logistics, enemy).clamp(0.0, 1.0)),
            mobility,
            fortification: -w.w_fortification * cover,
        },
    }
}

/// Greedy non-max suppression: best first (ties by tile order), skip anything within `radius_tiles`.
fn nms(mut pts: Vec<PressurePoint>, radius_tiles: f32, k: usize) -> Vec<PressurePoint> {
    pts.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.tile.y.cmp(&b.tile.y)).then(a.tile.x.cmp(&b.tile.x)));
    let mut kept: Vec<PressurePoint> = Vec::new();
    for p in pts {
        if kept.len() >= k {
            break;
        }
        if kept.iter().all(|q| (q.tile - p.tile).as_vec2().length() >= radius_tiles) {
            kept.push(p);
        }
    }
    kept
}

/// Score one theater: both sides, both kinds. `cover(tile, slot)` is that slot's fortification cover.
#[must_use]
pub fn score_theater(
    theater: &Theater,
    fields: &FieldGrid,
    cover: &dyn Fn(IVec2, u8) -> f32,
    w: &PressureWeights,
    tile_size_m: f32,
) -> Vec<PressurePoint> {
    let radius = w.nms_radius_m / tile_size_m.max(f32::EPSILON);
    let mut out = Vec::new();
    for (side, other) in [(theater.pair.0, theater.pair.1), (theater.pair.1, theater.pair.0)] {
        for kind in [PressureKind::Threatened, PressureKind::Opportunity] {
            let cell_owner = match kind {
                PressureKind::Threatened => side,
                PressureKind::Opportunity => other,
            };
            let cands: Vec<PressurePoint> = theater
                .cells_of(cell_owner)
                .filter_map(|c| {
                    let f = fields.get(&(c.tile.x, c.tile.y)).copied().unwrap_or_default();
                    let drivers = score_cell(kind, side, other, c.margin, &f, cover(c.tile, cell_owner), w);
                    let score = drivers.total();
                    (score > 0.0).then_some(PressurePoint { theater: theater.id, side, kind, tile: c.tile, score, drivers })
                })
                .collect();
            out.extend(nms(cands, radius, w.top_k));
        }
    }
    out
}

/// Rescore on the front cadence (or when theaters changed).
pub fn score_pressure_points_system(
    front: Res<FrontlineState>,
    theaters: Res<TheaterBook>,
    balance: Res<FrontBalance>,
    forts: Option<Res<FortificationIndex>>,
    overlays: Query<&ChunkStrategicOverlay>,
    mut book: ResMut<PressurePointBook>,
) {
    if front.derived_at_tick == book.source_tick
        && theaters.source_revision == book.source_theater_revision
        && !(book.points.is_empty() && !theaters.theaters.is_empty())
    {
        return;
    }
    book.source_tick = front.derived_at_tick;
    book.source_theater_revision = theaters.source_revision;
    let needed: HashSet<(i32, i32)> =
        theaters.theaters.iter().flat_map(|t| t.cells.iter().map(|c| (c.tile.x, c.tile.y))).collect();
    let mut fields = FieldGrid::with_capacity(needed.len());
    for ov in overlays.iter() {
        let w = ov.size.x.max(1) as usize;
        let ox = ov.chunk_coord.x.saturating_mul(ov.size.x as i32);
        let oy = ov.chunk_coord.y.saturating_mul(ov.size.y as i32);
        for ci in 0..ov.len_cells() {
            let t = (ox + (ci % w) as i32, oy + (ci / w) as i32);
            if needed.contains(&t) {
                fields.insert(
                    t,
                    CellFields {
                        threat: ov.threat[ci],
                        artillery: ov.artillery_danger[ci],
                        logistics: ov.logistics_strength[ci],
                        mobility_cost: ov.mobility_cost[ci],
                    },
                );
            }
        }
    }
    let cover = |tile: IVec2, owner: u8| forts.as_ref().map_or(0.0, |f| f.cover_at(tile, owner, &balance));
    let mut points = Vec::new();
    for t in &theaters.theaters {
        points.extend(score_theater(t, &fields, &cover, &balance.pressure, balance.tile_size_m));
    }
    book.points = points;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::front::{FrontCell, FrontSegment};

    fn weights() -> PressureWeights {
        FrontBalance::default().pressure
    }

    fn theater_with_cells(cells: Vec<FrontCell>) -> Theater {
        let seg = FrontSegment { pair: (0, 1), cells, polylines: vec![], contested_cells: 0, length_m: 0.0 };
        let mut book = TheaterBook::default();
        book.update(&[seg], &FrontBalance::default().theater, 25.0, 1);
        book.theaters.remove(0)
    }

    fn own_cell(x: i32, y: i32) -> FrontCell {
        FrontCell { tile: IVec2::new(x, y), owner: 0, facing: 1, contested: false, margin: 0.5 }
    }

    fn fields(threat1: f32, logi0: f32) -> CellFields {
        let mut f = CellFields::default();
        f.threat[1] = threat1;
        f.logistics[0] = logi0;
        f
    }

    #[test]
    fn starved_cell_outranks_supplied_cell() {
        let t = theater_with_cells((0..=40).map(|y| own_cell(0, y)).collect());
        let mut g = FieldGrid::new();
        for y in 1..40 {
            g.insert((0, y), fields(0.0, 1.0));
        }
        g.insert((0, 0), fields(0.4, 1.0));
        g.insert((0, 40), fields(0.4, 0.0));
        let pts = score_theater(&t, &g, &|_, _| 0.0, &weights(), 25.0);
        let thr: Vec<_> = pts.iter().filter(|p| p.side == 0 && p.kind == PressureKind::Threatened).collect();
        assert_eq!(thr[0].tile, IVec2::new(0, 40));
        assert!(thr[0].score > thr[1].score);
        assert!(thr[0].drivers.supply > thr[1].drivers.supply);
    }

    #[test]
    fn fortification_lowers_threatened_score() {
        let t = theater_with_cells(vec![own_cell(0, 0)]);
        let mut g = FieldGrid::new();
        g.insert((0, 0), fields(0.6, 0.5));
        let bare = score_theater(&t, &g, &|_, _| 0.0, &weights(), 25.0);
        let dug = score_theater(&t, &g, &|_, _| 0.7, &weights(), 25.0);
        assert!(dug[0].score < bare[0].score);
    }

    #[test]
    fn nms_keeps_top_k_spaced_apart() {
        let w = weights();
        let cells: Vec<FrontCell> = (0..60).map(|y| own_cell(0, y)).collect();
        let t = theater_with_cells(cells);
        let mut g = FieldGrid::new();
        for y in 0..60 {
            g.insert((0, y), fields(0.1 + y as f32 * 0.01, 0.5));
        }
        let pts = score_theater(&t, &g, &|_, _| 0.0, &w, 25.0);
        let thr: Vec<_> = pts.iter().filter(|p| p.side == 0 && p.kind == PressureKind::Threatened).collect();
        assert_eq!(thr.len(), w.top_k);
        let r = w.nms_radius_m / 25.0;
        for a in &thr {
            for b in &thr {
                if a.tile != b.tile {
                    assert!((a.tile - b.tile).as_vec2().length() >= r);
                }
            }
        }
        assert_eq!(thr[0].tile, IVec2::new(0, 59), "highest enemy threat wins");
    }
}
