//! **Fortification index** (PLAN-FRONT-COMBAT-v1 FC-7): tile -> owner + fortification family.
//!
//! Read model for pressure scoring and combat cover.
//!
//! **Single writer:** [`rebuild_fortification_index_system`], derived from `Operational` defense
//! `ConstructionSite`s. This lane never spawns a site: [`plan_fortification_rows`] only proposes
//! construction rows, and the executor queues them as `PendingConstructionQueue` entries so they go
//! through the one construction funnel (validate stock, commit, build with delivered materials).

use std::collections::HashMap;

use bevy::prelude::*;

use super::balance::{FortKind, FrontBalance};
use super::front::FrontCell;
use super::pressure::{PressureKind, PressurePoint};
use super::theater::Theater;
use crate::construction::DefenseKind;
use crate::economy::logistics::orders::{FactionSlot, LogisticsOrderSite};
use crate::strategic::{ConstructionSite, SiteArchetype, SiteConstructionPhase, SiteFootprint};

#[derive(Resource, Clone, Debug, Default)]
pub struct FortificationIndex {
    tiles: HashMap<(i32, i32), (u8, FortKind)>,
}

impl FortificationIndex {
    pub fn set(&mut self, tile: IVec2, owner: u8, kind: FortKind) {
        self.tiles.insert((tile.x, tile.y), (owner, kind));
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
    }

    #[must_use]
    pub fn get(&self, tile: IVec2) -> Option<(u8, FortKind)> {
        self.tiles.get(&(tile.x, tile.y)).copied()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    /// Cover (0..1) `owner` enjoys on `tile`; 0 when the tile is unfortified or fortified by another slot.
    #[must_use]
    pub fn cover_at(&self, tile: IVec2, owner: u8, balance: &FrontBalance) -> f32 {
        match self.get(tile) {
            Some((o, kind)) if o == owner => balance.fortification(kind).map_or(0.0, |f| f.cover),
            _ => 0.0,
        }
    }
}

/// Fortification family of a defense site archetype (`None` for everything else).
#[must_use]
pub fn fort_kind_of(archetype: SiteArchetype) -> Option<FortKind> {
    match archetype {
        SiteArchetype::TrenchLine => Some(FortKind::Trench),
        SiteArchetype::BunkerComplex => Some(FortKind::Bunker),
        SiteArchetype::DefensiveWall => Some(FortKind::Wall),
        SiteArchetype::DragonTeeth => Some(FortKind::DragonTeeth),
        SiteArchetype::Minefield => Some(FortKind::Minefield),
        _ => None,
    }
}

/// Rebuild the index from Operational defense sites. Owner slot =
/// (`FactionSlot` of the owner entity, else the site's logistics owner, else the player). The better-cover site wins a shared tile.
pub fn rebuild_fortification_index_system(
    balance: Res<FrontBalance>,
    changed: Query<
        (),
        Or<(Changed<ConstructionSite>, Changed<SiteFootprint>, Changed<LogisticsOrderSite>, Changed<FactionSlot>)>,
    >,
    slots: Query<&FactionSlot>,
    mut removed: RemovedComponents<ConstructionSite>,
    sites: Query<(&ConstructionSite, &SiteFootprint, Option<&LogisticsOrderSite>)>,
    mut index: ResMut<FortificationIndex>,
    mut built: Local<bool>,
) {
    let gone = removed.read().count() > 0;
    if *built && changed.is_empty() && !gone {
        return;
    }
    *built = true;
    let mut rows: Vec<(IVec2, u8, FortKind)> = Vec::new();
    for (site, fp, owner) in &sites {
        if site.phase != SiteConstructionPhase::Operational {
            continue;
        }
        let Some(kind) = fort_kind_of(site.archetype) else { continue };
        let slot = slots.get(site.owner).map(|s| s.0).unwrap_or_else(|_| owner.map_or(0, |o| o.owner.0));
        rows.extend(fp.tiles.iter().map(|t| (*t, slot, kind)));
    }
    let cover = |k: FortKind| balance.fortification(k).map_or(0.0, |f| f.cover);
    // Deterministic: strongest cover is written last and wins; ties by slot, kind, tile.
    rows.sort_by(|a, b| {
        cover(a.2)
            .total_cmp(&cover(b.2))
            .then(a.1.cmp(&b.1))
            .then(a.2.cmp(&b.2))
            .then((a.0.y, a.0.x).cmp(&(b.0.y, b.0.x)))
    });
    index.clear();
    for (tile, slot, kind) in rows {
        index.set(tile, slot, kind);
    }
}

/// One proposed defense row (a construction origin + family).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FortifyRow {
    pub kind: DefenseKind,
    pub origin: IVec2,
}

fn snap_axis(v: Vec2) -> IVec2 {
    if v.x.abs() >= v.y.abs() {
        IVec2::new(if v.x >= 0.0 { 1 } else { -1 }, 0)
    } else {
        IVec2::new(0, if v.y >= 0.0 { 1 } else { -1 })
    }
}

/// Plan rows for `side` in `theater`: bunkers behind its Threatened pressure points, dragon's teeth on
/// the enemy's attack axes (the enemy's Opportunity points land on our cells), trench strips along its
/// front set back from the line. Deterministic, at most `fortify_max_rows`, no overlaps, non-negative
/// origins only (construction origins are unsigned tiles).
#[must_use]
pub fn plan_fortification_rows(theater: &Theater, side: u8, points: &[PressurePoint], balance: &FrontBalance) -> Vec<FortifyRow> {
    let own: Vec<&FrontCell> = theater.cells_of(side).collect();
    let enemy: Vec<&FrontCell> = theater.cells.iter().filter(|c| c.owner != side).collect();
    if own.is_empty() {
        return Vec::new();
    }
    let mean = |cs: &[&FrontCell]| cs.iter().fold(Vec2::ZERO, |a, c| a + c.tile.as_vec2()) / cs.len().max(1) as f32;
    // Forward = toward the enemy.
    let forward = if enemy.is_empty() { IVec2::new(1, 0) } else { snap_axis(mean(&enemy) - mean(&own)) };
    let back = -forward;
    let setback = (balance.combat.fortify_setback_m / balance.tile_size_m.max(f32::EPSILON)).round().max(1.0) as i32;
    let max = balance.combat.fortify_max_rows;
    let mut rows: Vec<FortifyRow> = Vec::new();
    let mut taken: Vec<(IVec2, IVec2)> = Vec::new();
    let mut try_push = |kind: DefenseKind, origin: IVec2, rows: &mut Vec<FortifyRow>| {
        if rows.len() >= max || origin.x < 0 || origin.y < 0 {
            return;
        }
        let fp = kind.footprint();
        let (lo, hi) = (origin, origin + IVec2::new(fp.width as i32 - 1, fp.depth as i32 - 1));
        if taken.iter().any(|(a, b)| lo.x <= b.x && hi.x >= a.x && lo.y <= b.y && hi.y >= a.y) {
            return;
        }
        taken.push((lo, hi));
        rows.push(FortifyRow { kind, origin });
    };
    let ranked = |kind: PressureKind, mine: bool| {
        let mut v: Vec<&PressurePoint> = points
            .iter()
            .filter(|p| p.theater == theater.id && p.kind == kind && (p.side == side) == mine)
            .collect();
        v.sort_by(|a, b| b.score.total_cmp(&a.score).then((a.tile.y, a.tile.x).cmp(&(b.tile.y, b.tile.x))));
        v
    };
    for p in ranked(PressureKind::Threatened, true) {
        try_push(DefenseKind::Bunker, p.tile + back * setback, &mut rows);
    }
    for p in ranked(PressureKind::Opportunity, false) {
        if theater.contains_front_tile(p.tile) {
            try_push(DefenseKind::DragonTeeth, p.tile + forward, &mut rows);
        }
    }
    let stride = DefenseKind::TrenchLine.footprint().width.max(1) as i32;
    let mut seen: Vec<(i32, i32)> = Vec::new();
    for c in &own {
        let key = (c.tile.x.div_euclid(stride), c.tile.y);
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        try_push(DefenseKind::TrenchLine, IVec2::new(key.0 * stride, c.tile.y) + back * setback, &mut rows);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::LayerType;

    fn site(world: &mut World, archetype: SiteArchetype, phase: SiteConstructionPhase, tiles: Vec<IVec2>) -> Entity {
        let owner = world.spawn_empty().id();
        world
            .spawn((
                ConstructionSite { site_id: 1, owner, archetype, phase, operational_readiness: 1.0 },
                SiteFootprint { tiles, layer: LayerType::Surface },
            ))
            .id()
    }

    #[test]
    fn operational_defense_site_feeds_cover_and_unfinished_does_not() {
        let mut app = App::new();
        app.insert_resource(FrontBalance::default())
            .init_resource::<FortificationIndex>()
            .add_systems(Update, rebuild_fortification_index_system);
        let done = site(
            app.world_mut(),
            SiteArchetype::BunkerComplex,
            SiteConstructionPhase::Operational,
            vec![IVec2::new(4, 4), IVec2::new(5, 4)],
        );
        site(app.world_mut(), SiteArchetype::TrenchLine, SiteConstructionPhase::UnderConstruction, vec![IVec2::new(9, 9)]);
        app.update();
        let b = FrontBalance::default();
        let idx = app.world().resource::<FortificationIndex>();
        assert!((idx.cover_at(IVec2::new(4, 4), 0, &b) - 0.7).abs() < 1e-5);
        assert_eq!(idx.cover_at(IVec2::new(4, 4), 1, &b), 0.0, "other slot gets no cover");
        assert_eq!(idx.cover_at(IVec2::new(9, 9), 0, &b), 0.0, "still under construction");
        app.world_mut().despawn(done);
        app.update();
        assert_eq!(app.world().resource::<FortificationIndex>().cover_at(IVec2::new(4, 4), 0, &b), 0.0);
    }

    #[test]
    fn plan_rows_sit_behind_the_front_and_respect_the_cap() {
        use crate::combat::front::FrontSegment;
        use crate::combat::theater::TheaterBook;
        let mut cells = Vec::new();
        for y in 0..12 {
            cells.push(FrontCell { tile: IVec2::new(30, y), owner: 0, facing: 1, contested: false, margin: 0.5 });
            cells.push(FrontCell { tile: IVec2::new(31, y), owner: 1, facing: 0, contested: false, margin: 0.5 });
        }
        let seg = FrontSegment { pair: (0, 1), cells, polylines: vec![], contested_cells: 0, length_m: 0.0 };
        let b = FrontBalance::default();
        let mut book = TheaterBook::default();
        book.update(&[seg], &b.theater, b.tile_size_m, 1);
        let rows = plan_fortification_rows(&book.theaters[0], 0, &[], &b);
        assert!(!rows.is_empty() && rows.len() <= b.combat.fortify_max_rows);
        assert!(rows.iter().all(|r| r.origin.x < 30), "behind our line (we are west of the enemy)");
        assert_eq!(rows, plan_fortification_rows(&book.theaters[0], 0, &[], &b), "deterministic");
    }
}
