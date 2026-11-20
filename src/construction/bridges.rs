//! Bridges (PLAN-ROADS-v1): a road or rail line that crosses water is a bridge of a real structure
//! type (culvert, timber beam / trestle, pontoon, Bailey, steel girder, concrete beam, steel truss,
//! stone arch — `infrastructure::bridge_types`), built in **sections** pier to pier. Each section
//! has its own integrity: a flood loads the sections it reaches, a missile damages the sections
//! near its impact, so a strike drops a span or two, not the whole bridge. Sole writer: the
//! systems here.
//!
//! - [`register_bridges_system`]: a new line's wet stretches (depth > 0.3 m at build time) become
//!   bridges, bank to bank; the type is the cheapest that fits the line and the crossing.
//! - [`bridge_flood_damage_system`]: per section, water load (depth × speed²; deck hits ×10) over
//!   the type's strength costs integrity; pontoons fail in fast current instead and ride the water.
//! - [`bridge_blast_damage_system`]: every detonation (`BlastCraterBook.recent`) damages each
//!   section by the warhead's falloff at the section, over the type's blast toughness; the impact
//!   point is kept for the deck scorch.
//! - Routing ([`BridgeBook::edge_state`]): an intact bridge carries traffic over flooded river
//!   tiles; any fallen section or water over the deck closes it.

use std::collections::HashSet;

use bevy::prelude::*;

use super::corridor_network::{CorridorKind, CorridorNetwork};
use crate::infrastructure::bridge_types::{bridge_type_preference, choose_bridge_type, BridgeCarries, BridgeType};
use crate::systems::water::WaterTileProbe;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::WorldGenDenseTerrainCache;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Water this deep (m) under a line at build time makes that stretch a bridge.
pub const BRIDGE_WATER_DEPTH_M: f32 = 0.3;
/// Structure depth under the deck top (m): water above `deck_m - DECK_DEPTH_M` hits the deck.
pub const DECK_DEPTH_M: f32 = 1.5;
/// Integrity lost per second per unit of flood load over strength.
const DAMAGE_PER_S: f32 = 0.02;
/// Water on the deck counts this much more than on the piers (broad face, debris raft).
const DECK_LOAD_MUL: f32 = 10.0;
/// Most sections one bridge is split into.
const MAX_SECTIONS: usize = 64;
/// Impact marks kept per section (render scorch).
const MAX_HITS: usize = 3;

/// One span, pier to pier, as fractions `t0..t1` along the bridge.
#[derive(Clone, Debug, PartialEq)]
pub struct BridgeSection {
    pub t0: f32,
    pub t1: f32,
    /// 1 = whole, 0 = down.
    pub integrity: f32,
    pub fallen: bool,
    /// Impact points on this section (tile units).
    pub hits: Vec<Vec2>,
}

/// One bridge: the wet part of a line, bank to bank (tile units).
#[derive(Clone, Debug, PartialEq)]
pub struct BridgeRecord {
    pub segment_id: u32,
    pub a: Vec2,
    pub b: Vec2,
    pub bridge_type: BridgeType,
    /// Deck top (world m) as built; floating bridges ride the water instead.
    pub deck_m: f32,
    pub sections: Vec<BridgeSection>,
    /// Water over the deck right now (closed until it drops).
    pub overtopped: bool,
    /// Highest flood load seen (dump).
    pub peak_load: f32,
    pub rail: bool,
}

impl BridgeRecord {
    #[must_use]
    pub fn length_m(&self) -> f32 {
        self.a.distance(self.b) * TILE_METRES_GAMEPLAY
    }

    #[must_use]
    pub fn fallen_sections(&self) -> usize {
        self.sections.iter().filter(|s| s.fallen).count()
    }

    #[must_use]
    pub fn closed(&self) -> bool {
        self.overtopped || self.sections.iter().any(|s| s.fallen)
    }

    /// Lowest section integrity.
    #[must_use]
    pub fn integrity(&self) -> f32 {
        self.sections.iter().map(|s| s.integrity).fold(1.0, f32::min)
    }

    /// The straight stretch `p → q` (tile units) runs over this bridge.
    #[must_use]
    pub fn carries(&self, p: Vec2, q: Vec2) -> bool {
        let d = q - p;
        let l2 = d.length_squared();
        if l2 < 1e-6 {
            return false;
        }
        let mid = (self.a + self.b) * 0.5;
        let t = ((mid - p).dot(d) / l2).clamp(0.0, 1.0);
        let along = (self.b - self.a).normalize_or_zero().dot(d.normalize_or_zero()).abs() > 0.9;
        mid.distance(p + d * t) < 0.75 && along
    }

    fn point(&self, t: f32) -> Vec2 {
        self.a + (self.b - self.a) * t
    }
}

/// How routing treats a transport edge that runs over a bridge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeEdgeState {
    /// Carried over the water (flooded river tiles under it do not block).
    Open,
    /// A section is down or water is over the deck.
    Closed,
}

/// All bridges. Sole writer: this module's systems.
#[derive(Resource, Debug, Default, Clone)]
pub struct BridgeBook {
    pub bridges: Vec<BridgeRecord>,
    /// Bumps when a bridge appears, goes, or a section's state changes (routes re-plan, render).
    pub revision: u64,
}

impl BridgeBook {
    /// State of the edge `p → q` (tile centres) if it runs over a bridge.
    #[must_use]
    pub fn edge_state(&self, p: Vec2, q: Vec2) -> Option<BridgeEdgeState> {
        let mut out = None;
        for b in self.bridges.iter().filter(|b| b.carries(p, q)) {
            if b.closed() {
                return Some(BridgeEdgeState::Closed);
            }
            out = Some(BridgeEdgeState::Open);
        }
        out
    }

    /// The bridge (if any) a drawn stretch `p → q` belongs to.
    #[must_use]
    pub fn on_stretch(&self, p: Vec2, q: Vec2) -> Option<&BridgeRecord> {
        self.bridges.iter().find(|b| b.carries(p, q))
    }
}

fn centre(t: crate::strategic::BuildSiteTile) -> Vec2 {
    Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5)
}

/// What a line carries, for the bridge type choice.
#[must_use]
pub fn bridge_carries(kind: &CorridorKind) -> BridgeCarries {
    match kind {
        CorridorKind::Rail { .. } => BridgeCarries::Rail,
        CorridorKind::Road { tier, .. } => BridgeCarries::Road(*tier),
    }
}

/// A wet run along a line: bank-to-bank ends, highest water surface (world m), deepest water (m).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WetRun {
    pub a: Vec2,
    pub b: Vec2,
    pub top_m: f32,
    pub depth_m: f32,
}

/// Wet runs along `points` (tile units). `depth(tile)`, `surface(tile)`.
#[must_use]
pub fn wet_runs(points: &[Vec2], depth: &dyn Fn(IVec2) -> f32, surface: &dyn Fn(IVec2) -> Option<f32>) -> Vec<WetRun> {
    let mut out = Vec::new();
    let mut run: Option<WetRun> = None;
    let mut last_dry: Option<Vec2> = None;
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let n = (a.distance(b) / 0.5).ceil().max(1.0) as usize;
        for k in 0..=n {
            let p = a + (b - a) * (k as f32 / n as f32);
            let t = IVec2::new(p.x.floor() as i32, p.y.floor() as i32);
            let d = depth(t);
            if d > BRIDGE_WATER_DEPTH_M {
                let s = surface(t).unwrap_or(f32::NEG_INFINITY);
                let r = run.get_or_insert(WetRun { a: last_dry.unwrap_or(p), b: p, top_m: s, depth_m: d });
                r.top_m = r.top_m.max(s);
                r.depth_m = r.depth_m.max(d);
            } else {
                if let Some(mut r) = run.take() {
                    r.b = p;
                    out.push(r);
                }
                last_dry = Some(p);
            }
        }
    }
    if let (Some(mut r), Some(&end)) = (run, points.last()) {
        r.b = end;
        out.push(r);
    }
    out
}

/// Sections of a bridge of `length_m` built as `t`: one per pier span.
#[must_use]
pub fn bridge_sections(t: BridgeType, length_m: f32) -> Vec<BridgeSection> {
    let n = ((length_m / t.spec().span_m.max(1.0)).ceil() as usize).clamp(1, MAX_SECTIONS);
    (0..n)
        .map(|k| BridgeSection {
            t0: k as f32 / n as f32,
            t1: (k + 1) as f32 / n as f32,
            integrity: 1.0,
            fallen: false,
            hits: Vec::new(),
        })
        .collect()
}

/// The bridge a wet run of a line becomes.
#[must_use]
pub fn bridge_for_run(segment_id: u32, kind: &CorridorKind, run: WetRun, bank_ground_m: f32) -> BridgeRecord {
    let carries = bridge_carries(kind);
    let length_m = run.a.distance(run.b) * TILE_METRES_GAMEPLAY;
    let bridge_type = choose_bridge_type(carries, length_m, run.depth_m)
        .unwrap_or_else(|| *bridge_type_preference(carries).last().expect("every line has a type"));
    let spec = bridge_type.spec();
    let water = if run.top_m.is_finite() { run.top_m } else { bank_ground_m };
    let deck_m = if spec.floating { water + 0.5 } else { bank_ground_m.max(water + spec.clearance_m + DECK_DEPTH_M) };
    BridgeRecord {
        segment_id,
        a: run.a,
        b: run.b,
        bridge_type,
        deck_m,
        sections: bridge_sections(bridge_type, length_m),
        overtopped: false,
        peak_load: 0.0,
        rail: kind.is_rail(),
    }
}

/// New lines that cross water become bridges; removed lines lose theirs.
pub fn register_bridges_system(
    network: Res<CorridorNetwork>,
    water: WaterTileProbe,
    params: Option<Res<WorldGenParams>>,
    heights: Option<Res<WorldGenDenseTerrainCache>>,
    mut book: ResMut<BridgeBook>,
    mut seen: Local<HashSet<u32>>,
) {
    let live: HashSet<u32> = network.segments.iter().map(|s| s.id).collect();
    let before = book.bridges.len();
    book.bridges.retain(|b| live.contains(&b.segment_id));
    seen.retain(|id| live.contains(id));
    let mut changed = book.bridges.len() != before;
    if water.ready() {
        let ground = |p: Vec2| match params.as_deref() {
            Some(pr) => super::terrain_conform::sample_height_norm(p.x, p.y, pr, heights.as_deref()) * pr.elevation_range_m,
            None => 0.0,
        };
        let depth = |t: IVec2| water.depth_m(t);
        let surface = |t: IVec2| water.surface_m(t);
        let fresh: Vec<&super::corridor_network::CorridorSegment> =
            network.segments.iter().filter(|s| !seen.contains(&s.id)).collect();
        for seg in fresh {
            seen.insert(seg.id);
            let pts: Vec<Vec2> = seg.nodes.iter().copied().map(centre).collect();
            for run in wet_runs(&pts, &depth, &surface) {
                let banks = ground(run.a).max(ground(run.b));
                book.bridges.push(bridge_for_run(seg.id, &seg.kind, run, banks));
                changed = true;
            }
        }
    }
    if changed {
        book.revision = book.revision.wrapping_add(1);
    }
}

/// Flood load on the stretch `t0..t1` of a bridge and whether the water reaches its deck.
#[must_use]
pub fn section_water_load(
    b: &BridgeRecord,
    t0: f32,
    t1: f32,
    depth: &dyn Fn(IVec2) -> f32,
    surface: &dyn Fn(IVec2) -> Option<f32>,
    flow: &dyn Fn(IVec2) -> Vec2,
) -> (f32, bool, f32) {
    let spec = b.bridge_type.spec();
    let exposure = spec.pier_exposure(b.length_m());
    let soffit = b.deck_m - DECK_DEPTH_M;
    let n = (((t1 - t0) * b.a.distance(b.b)) / 0.5).ceil().max(1.0) as usize;
    let (mut load, mut over, mut vmax) = (0.0_f32, false, 0.0_f32);
    for k in 0..=n {
        let p = b.point(t0 + (t1 - t0) * (k as f32 / n as f32));
        let t = IVec2::new(p.x.floor() as i32, p.y.floor() as i32);
        let d = depth(t);
        if d <= 0.0 {
            continue;
        }
        let v2 = flow(t).length_squared();
        vmax = vmax.max(v2.sqrt());
        let mut l = d * v2 * exposure;
        if !spec.floating {
            if let Some(s) = surface(t) {
                if s > soffit {
                    over = true;
                    l += DECK_LOAD_MUL * spec.overtop_damage_mul * (s - soffit) * v2.max(1.0);
                }
            }
        }
        load = load.max(l);
    }
    (load, over, vmax)
}

/// Integrity after `dt_s` seconds of flood: load over strength (fixed), or current over the
/// pontoon's limit (floating).
#[must_use]
pub fn flood_integrity_after(t: BridgeType, integrity: f32, load: f32, current_mps: f32, dt_s: f32) -> f32 {
    let spec = t.spec();
    let over = if spec.floating {
        (current_mps / spec.max_current_mps.max(0.1) - 1.0).max(0.0) * 3.0
    } else {
        (load / spec.strength.max(1e-3) - 1.0).max(0.0)
    };
    (integrity - DAMAGE_PER_S * over * dt_s).max(0.0)
}

/// Floods wear bridge sections down (sim time).
pub fn bridge_flood_damage_system(
    water: WaterTileProbe,
    sim_time: Option<Res<crate::systems::sim_control::SimTimeMicros>>,
    mut book: ResMut<BridgeBook>,
    mut last: Local<Option<u64>>,
) {
    let Some(now) = sim_time.map(|t| t.0) else {
        return;
    };
    let dt_s = last.map_or(0.0, |l| now.saturating_sub(l) as f32 / 1_000_000.0).min(60.0);
    *last = Some(now);
    if dt_s <= 0.0 || book.bridges.is_empty() || !water.seeded() {
        return;
    }
    let depth = |t: IVec2| water.depth_m(t);
    let surface = |t: IVec2| water.surface_m(t);
    let flow = |t: IVec2| water.flow(t);
    let mut changed = false;
    for i in 0..book.bridges.len() {
        let rec = book.bridges[i].clone();
        let mut over_any = false;
        let mut peak = rec.peak_load;
        let mut sections = rec.sections.clone();
        for s in sections.iter_mut().filter(|s| !s.fallen) {
            let (load, over, v) = section_water_load(&rec, s.t0, s.t1, &depth, &surface, &flow);
            over_any |= over;
            peak = peak.max(load);
            s.integrity = flood_integrity_after(rec.bridge_type, s.integrity, load, v, dt_s);
            if s.integrity <= 0.0 {
                s.fallen = true;
                changed = true;
                bevy::log::info!(target: "construction::bridges", segment = rec.segment_id, kind = rec.bridge_type.label(), "bridge section washed out");
            }
        }
        let b = &mut book.bridges[i];
        if b.overtopped != over_any {
            b.overtopped = over_any;
            changed = true;
        }
        b.peak_load = peak;
        b.sections = sections;
    }
    if changed {
        book.revision = book.revision.wrapping_add(1);
    }
}

/// One detonation applied to every bridge section (pure core). Returns whether anything fell.
pub fn apply_blast_to_bridges(book: &mut BridgeBook, centre_tiles: Vec2, radius_m: f32, damage: f32) -> bool {
    let mut fell = false;
    for b in &mut book.bridges {
        let tough = b.bridge_type.spec().blast_toughness.max(1.0);
        let (a, d) = (b.a, b.b - b.a);
        for s in b.sections.iter_mut().filter(|s| !s.fallen) {
            let (p, q) = (a + d * s.t0, a + d * s.t1);
            let pq = q - p;
            let t = if pq.length_squared() < 1e-6 { 0.0 } else { ((centre_tiles - p).dot(pq) / pq.length_squared()).clamp(0.0, 1.0) };
            let near = p + pq * t;
            let dist_m = near.distance(centre_tiles) * TILE_METRES_GAMEPLAY;
            let fall = super::blast_radius_falloff(dist_m, radius_m);
            if fall <= 0.0 {
                continue;
            }
            s.integrity = (s.integrity - damage * fall / tough).max(0.0);
            if fall > 0.3 && s.hits.len() < MAX_HITS {
                s.hits.push(near);
            }
            if s.integrity <= 0.0 {
                s.fallen = true;
                fell = true;
            }
        }
    }
    fell
}

/// Detonations damage the bridge sections near them.
pub fn bridge_blast_damage_system(
    craters: Option<Res<super::BlastCraterBook>>,
    mut book: ResMut<BridgeBook>,
    mut cursor: Local<u64>,
) {
    let Some(craters) = craters else {
        return;
    };
    let from = *cursor;
    let fresh: Vec<(u64, Vec2, f32, f32)> = craters.recent.iter().copied().filter(|(s, ..)| *s > from).collect();
    if fresh.is_empty() || book.bridges.is_empty() {
        if let Some(max) = fresh.iter().map(|f| f.0).max() {
            *cursor = (*cursor).max(max);
        }
        return;
    }
    for (seq, centre_m, radius_m, damage) in fresh {
        *cursor = (*cursor).max(seq);
        let fell = apply_blast_to_bridges(&mut book, centre_m / TILE_METRES_GAMEPLAY, radius_m, damage);
        if fell {
            bevy::log::info!(target: "construction::bridges", ?centre_m, damage, "bridge section dropped by blast");
        }
        // Any hit changes the drawn bridge (scorch / damage tint).
        book.revision = book.revision.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::RoadTier;

    fn bridge(t: BridgeType, len_tiles: f32) -> BridgeRecord {
        let run = WetRun { a: Vec2::new(10.0, 5.5), b: Vec2::new(10.0 + len_tiles, 5.5), top_m: 100.0, depth_m: 4.0 };
        let mut b = bridge_for_run(0, &CorridorKind::Road { tier: RoadTier::Paved, lanes: 2 }, run, 100.0);
        b.bridge_type = t;
        b.sections = bridge_sections(t, b.length_m());
        b
    }

    #[test]
    fn the_river_under_a_line_is_one_wet_run_bank_to_bank() {
        let river = |t: IVec2| if (14..17).contains(&t.x) { 3.0 } else { 0.0 };
        let runs = wet_runs(&[Vec2::new(5.5, 5.5), Vec2::new(25.5, 5.5)], &river, &|_| Some(103.0));
        assert_eq!(runs.len(), 1);
        assert!(runs[0].a.x < 14.0 && runs[0].b.x >= 17.0, "{:?}", runs[0]);
        assert_eq!((runs[0].top_m, runs[0].depth_m), (103.0, 3.0));
    }

    #[test]
    fn a_long_bridge_is_built_in_pier_sections() {
        // 160 m concrete beam bridge: 35 m spans -> 5 sections; a Bailey 50 m: one span.
        assert_eq!(bridge(BridgeType::PrestressedConcrete, 10.0).sections.len(), 5);
        assert_eq!(bridge(BridgeType::Bailey, 50.0 / TILE_METRES_GAMEPLAY).sections.len(), 1);
    }

    #[test]
    fn a_missile_drops_the_span_it_hits_not_the_whole_bridge() {
        let mut book = BridgeBook::default();
        book.bridges.push(bridge(BridgeType::TimberTrestle, 10.0)); // 160 m, 27 spans of 6 m
        let hit = Vec2::new(15.0, 5.5);
        apply_blast_to_bridges(&mut book, hit, 24.0, 55.0); // small warhead, direct hit
        let b = &book.bridges[0];
        let fallen = b.fallen_sections();
        assert!(fallen >= 1 && fallen < b.sections.len() / 2, "{fallen} of {}", b.sections.len());
        assert!(b.closed(), "a gap closes the road");
        // Far end untouched.
        assert_eq!(b.sections.last().unwrap().integrity, 1.0);

        // The same hit leaves a concrete beam bridge standing but damaged where it struck.
        let mut book = BridgeBook::default();
        book.bridges.push(bridge(BridgeType::PrestressedConcrete, 10.0));
        apply_blast_to_bridges(&mut book, hit, 24.0, 55.0);
        let b = &book.bridges[0];
        assert_eq!(b.fallen_sections(), 0);
        assert!(b.integrity() < 1.0 && b.sections.iter().any(|s| !s.hits.is_empty()));
        // A large warhead on the same spot drops that span.
        apply_blast_to_bridges(&mut book, hit, 64.0, 220.0);
        assert!(book.bridges[0].fallen_sections() >= 1);
    }

    #[test]
    fn a_flood_washes_out_timber_spans_a_concrete_bridge_stands() {
        let depth = |_: IVec2| 12.0;
        let flow = |_: IVec2| Vec2::new(0.0, 5.0);
        let surface = |_: IVec2| Some(103.0); // under both decks
        let trestle = bridge(BridgeType::TimberTrestle, 10.0);
        let (load, over, v) = section_water_load(&trestle, 0.4, 0.5, &depth, &surface, &flow);
        assert!(!over && load > 250.0);
        let mut weak = 1.0;
        let mut strong = 1.0;
        for _ in 0..120 {
            weak = flood_integrity_after(BridgeType::TimberTrestle, weak, load, v, 1.0);
            strong = flood_integrity_after(BridgeType::PrestressedConcrete, strong, load, v, 1.0);
        }
        assert_eq!(weak, 0.0);
        assert_eq!(strong, 1.0);
        // Pontoons fail in fast current regardless of depth.
        assert!(flood_integrity_after(BridgeType::Pontoon, 1.0, 0.0, 5.0, 30.0) < 1.0);
        assert_eq!(flood_integrity_after(BridgeType::Pontoon, 1.0, 0.0, 1.0, 3600.0), 1.0);
    }

    #[test]
    fn routing_sees_open_and_closed_bridges() {
        let mut book = BridgeBook::default();
        book.bridges.push(bridge(BridgeType::Bailey, 10.0));
        let (p, q) = (Vec2::new(8.5, 5.5), Vec2::new(22.5, 5.5));
        assert_eq!(book.edge_state(p, q), Some(BridgeEdgeState::Open));
        assert_eq!(book.edge_state(Vec2::new(8.5, 30.5), Vec2::new(22.5, 30.5)), None);
        book.bridges[0].sections[0].fallen = true;
        assert_eq!(book.edge_state(p, q), Some(BridgeEdgeState::Closed));
    }
}
