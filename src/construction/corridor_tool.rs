//! One corridor placement tool for roads (every tier), rail and firebreaks (PLAN-ROADS-v1 RD-6).
//!
//! The mode adapter ([`CorridorModeSpec`]) is the only thing that differs between a gravel road,
//! a motorway and a railway: width, steepest grade, how much cut/fill the works may take, tightest
//! curve, longest bridge. Everything else is shared:
//!
//! - [`check_corridor_path`] judges each stretch of a drawn path: **Ok**, **Works** (buildable
//!   with a bridge, cut/fill, level crossing or overpass, each labelled) or **Blocked** (with the
//!   reason). Pure: ground height and water depth come in as closures, so the live preview, the
//!   scenario step and any AI builder run the same rules.
//! - [`summarize_path`] is the readout (length, works, trip time, reasons, bridge material bill).
//! - [`assist_route`]: "follow terrain" — an auto path from the last point to the cursor that
//!   winds within the mode's grade and bridges water only where it can.
//! - [`apply_bridge_choice`] / [`bridge_choices`]: the player may pick another bridge type that
//!   spans every crossing of the path (the cheapest that fits is the default).
//! - [`commit_corridor_path`] queues the buildable stretches into the construction funnel; blocked
//!   stretches are skipped, never half-built.

use bevy::prelude::*;

use super::construction_pipeline::{ConstructionIntent, ConstructionPlanQueue, ConstructionType};
use super::corridor_network::CorridorNetwork;
use crate::infrastructure::bridge_types::{bridge_type_preference, BridgeCarries, BridgeType};
use crate::infrastructure::RoadTier;
use crate::strategic::BuildSiteTile;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// What the tool lays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorridorToolKind {
    Road(RoadTier),
    Rail,
    Firebreak,
}

/// Per-mode placement rules.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CorridorModeSpec {
    /// Drawn width (preview ribbon units).
    pub width: f32,
    /// Steepest grade the finished line may have (rise / run).
    pub max_grade: f32,
    /// Deepest cut or highest fill (m) the works may take to hold `max_grade` locally.
    pub max_cut_m: f32,
    /// Tightest curve radius (m); 0 = no limit.
    pub min_radius_m: f32,
    /// Longest bridge (m); 0 = cannot cross water.
    pub max_bridge_m: f32,
    /// Speed the finished line allows (m/s), for the trip-time readout.
    pub cap_mps: f32,
    pub rail: bool,
    pub limited_access: bool,
}

/// Rules for each kind (first pass; tiers' speeds come from the tier table).
#[must_use]
pub fn mode_spec(kind: CorridorToolKind) -> CorridorModeSpec {
    let road = |width: f32, grade: f32, cut: f32, radius: f32, bridge: f32, tier: RoadTier| CorridorModeSpec {
        width,
        max_grade: grade,
        max_cut_m: cut,
        min_radius_m: radius,
        max_bridge_m: bridge,
        cap_mps: tier.spec().speed_mps(),
        rail: false,
        limited_access: tier.limited_access(),
    };
    match kind {
        CorridorToolKind::Road(t @ (RoadTier::Track | RoadTier::Dirt)) => road(6.0, 0.14, 4.0, 0.0, 60.0, t),
        CorridorToolKind::Road(t @ RoadTier::Gravel) => road(8.0, 0.12, 8.0, 0.0, 200.0, t),
        CorridorToolKind::Road(t @ RoadTier::Paved) => road(9.0, 0.10, 12.0, 0.0, 400.0, t),
        CorridorToolKind::Road(t @ RoadTier::Concrete) => road(10.0, 0.10, 14.0, 0.0, 500.0, t),
        CorridorToolKind::Road(t @ RoadTier::Motorway) => road(16.0, 0.06, 20.0, 400.0, 800.0, t),
        CorridorToolKind::Rail => CorridorModeSpec {
            width: 6.0,
            max_grade: 0.03,
            max_cut_m: 18.0,
            min_radius_m: 300.0,
            max_bridge_m: 600.0,
            cap_mps: 80.0 / 3.6,
            rail: true,
            limited_access: false,
        },
        CorridorToolKind::Firebreak => CorridorModeSpec {
            width: 4.0,
            max_grade: 1.0,
            max_cut_m: 0.0,
            min_radius_m: 0.0,
            max_bridge_m: 0.0,
            cap_mps: 0.0,
            rail: false,
            limited_access: false,
        },
    }
}

/// Verdict on one stretch.
#[derive(Clone, Debug, PartialEq)]
pub enum StretchVerdict {
    Ok,
    /// Buildable with these works (each a short label: "bridge 96 m", "cut/fill 6 m", ...).
    Works(Vec<String>),
    /// Not buildable, and why.
    Blocked(String),
}

impl StretchVerdict {
    #[must_use]
    pub fn buildable(&self) -> bool {
        !matches!(self, StretchVerdict::Blocked(_))
    }
}

/// One checked stretch of the drawn path (tile units).
#[derive(Clone, Debug, PartialEq)]
pub struct StretchCheck {
    pub a: Vec2,
    pub b: Vec2,
    pub length_m: f32,
    pub verdict: StretchVerdict,
    /// The bridge this stretch needs, if it crosses water.
    pub bridge: Option<StretchBridge>,
}

/// A water crossing on a stretch and the structure chosen for it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchBridge {
    pub carries: BridgeCarries,
    pub span_m: f32,
    pub depth_m: f32,
    /// Cheapest type that fits (the default).
    pub cheapest: BridgeType,
    /// Type that will be built.
    pub kind: BridgeType,
}

impl StretchBridge {
    /// `t` can carry this line over this crossing.
    #[must_use]
    pub fn fits(&self, t: BridgeType) -> bool {
        let s = t.spec();
        bridge_type_preference(self.carries).contains(&t)
            && self.span_m <= s.max_length_m
            && (s.floating || self.depth_m <= s.max_depth_m)
    }
}

fn carries_of(kind: CorridorToolKind) -> BridgeCarries {
    match kind {
        CorridorToolKind::Road(tier) => BridgeCarries::Road(tier),
        _ => BridgeCarries::Rail,
    }
}

/// Bridge types that span every crossing of the path (cheapest first); empty without water.
#[must_use]
pub fn bridge_choices(checks: &[StretchCheck]) -> Vec<BridgeType> {
    let bridges: Vec<StretchBridge> = checks.iter().filter_map(|c| c.bridge).collect();
    let Some(first) = bridges.first() else {
        return Vec::new();
    };
    bridge_type_preference(first.carries)
        .iter()
        .copied()
        .filter(|t| bridges.iter().all(|b| b.fits(*t)))
        .collect()
}

/// Builds `choice` on every crossing it fits (others keep the cheapest) and relabels the works.
pub fn apply_bridge_choice(checks: &mut [StretchCheck], choice: Option<BridgeType>) {
    for c in checks.iter_mut() {
        let Some(b) = c.bridge.as_mut() else {
            continue;
        };
        let want = choice.filter(|t| b.fits(*t)).unwrap_or(b.cheapest);
        if want == b.kind {
            continue;
        }
        let (old, new) = (format!("{} {:.0} m", b.kind.label(), b.span_m), format!("{} {:.0} m", want.label(), b.span_m));
        b.kind = want;
        if let StretchVerdict::Works(w) = &mut c.verdict {
            for x in w.iter_mut().filter(|x| **x == old) {
                x.clone_from(&new);
            }
        }
    }
}

/// Water this deep (m) needs a bridge.
const BRIDGE_DEPTH_M: f32 = 0.3;
/// Terrain sample spacing along a stretch (tiles).
const SAMPLE_TILES: f32 = 0.5;

fn crosses(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let r = b - a;
    let s = d - c;
    let den = r.perp_dot(s);
    if den.abs() < 1e-6 {
        return false;
    }
    let t = (c - a).perp_dot(s) / den;
    let u = (c - a).perp_dot(r) / den;
    (0.02..=0.98).contains(&t) && (0.02..=0.98).contains(&u)
}

fn centre(t: BuildSiteTile) -> Vec2 {
    Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5)
}

/// Judge every stretch of `points` (tile units, ≥ 2 points) for `kind`.
/// `ground_m(p)` = terrain height (m) at tile-space `p`; `water_m(tile)` = water depth (m).
#[must_use]
pub fn check_corridor_path(
    points: &[Vec2],
    kind: CorridorToolKind,
    ground_m: &dyn Fn(Vec2) -> f32,
    water_m: &dyn Fn(IVec2) -> f32,
    network: &CorridorNetwork,
    map_tiles: UVec2,
) -> Vec<StretchCheck> {
    let spec = mode_spec(kind);
    let inside = |p: Vec2| p.x >= 0.0 && p.y >= 0.0 && p.x < map_tiles.x as f32 && p.y < map_tiles.y as f32;
    let mut out = Vec::with_capacity(points.len().saturating_sub(1));
    for (i, w) in points.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let len_tiles = a.distance(b);
        let length_m = len_tiles * TILE_METRES_GAMEPLAY;
        let mut bridge = None;
        let verdict = (|| {
            if !inside(a) || !inside(b) {
                return StretchVerdict::Blocked("outside the map".into());
            }
            if len_tiles < 0.5 {
                return StretchVerdict::Blocked("too short".into());
            }
            if kind == CorridorToolKind::Firebreak {
                return StretchVerdict::Ok;
            }
            let mut works = Vec::new();
            // Curve at the bend into this stretch (rail, motorway).
            if spec.min_radius_m > 0.0 && i > 0 {
                let prev = points[i] - points[i - 1];
                let next = b - a;
                let turn = prev.angle_to(next).abs();
                if turn > 0.02 {
                    let shorter = prev.length().min(next.length()) * TILE_METRES_GAMEPLAY;
                    let radius = (shorter * 0.5) / (turn * 0.5).tan().max(1e-4);
                    if radius < spec.min_radius_m {
                        return StretchVerdict::Blocked(format!(
                            "curve {radius:.0} m tighter than {:.0} m: spread the bend",
                            spec.min_radius_m
                        ));
                    }
                }
            }
            // Samples along the stretch: ground height and whether it is water.
            let n = (len_tiles / SAMPLE_TILES).ceil().max(1.0) as usize;
            let mut dry: Vec<(f32, f32)> = Vec::with_capacity(n + 1); // (distance m, height m)
            let (mut wet_run, mut wet_max) = (0usize, 0usize);
            let mut deepest = 0.0_f32;
            for k in 0..=n {
                let p = a + (b - a) * (k as f32 / n as f32);
                let tile = IVec2::new(p.x.floor() as i32, p.y.floor() as i32);
                let depth = water_m(tile);
                if depth > BRIDGE_DEPTH_M {
                    deepest = deepest.max(depth);
                    wet_run += 1;
                    wet_max = wet_max.max(wet_run);
                } else {
                    wet_run = 0;
                    dry.push((k as f32 / n as f32 * length_m, ground_m(p)));
                }
            }
            if wet_max > 0 {
                let span_m = (wet_max as f32 + 1.0) * SAMPLE_TILES * TILE_METRES_GAMEPLAY;
                if spec.max_bridge_m <= 0.0 {
                    return StretchVerdict::Blocked("crosses water".into());
                }
                // The cheapest real structure that spans it for this line (culvert, timber,
                // pontoon, Bailey, girder, concrete beam, truss, stone arch).
                let carries = carries_of(kind);
                match crate::infrastructure::bridge_types::choose_bridge_type(carries, span_m, deepest) {
                    Some(t) => {
                        works.push(format!("{} {span_m:.0} m", t.label()));
                        bridge = Some(StretchBridge { carries, span_m, depth_m: deepest, cheapest: t, kind: t });
                    }
                    None => {
                        return StretchVerdict::Blocked(format!(
                            "water {span_m:.0} m wide, {deepest:.0} m deep: no bridge type spans it"
                        ))
                    }
                }
            }
            // Grade: the whole climb must be within the limit; local bumps take cut/fill.
            if let (Some(&(d0, h0)), Some(&(d1, h1))) = (dry.first(), dry.last()) {
                let run = (d1 - d0).max(1.0);
                let climb = (h1 - h0).abs() / run;
                if climb > spec.max_grade {
                    return StretchVerdict::Blocked(format!(
                        "climb {:.1}% steeper than {:.1}%: add a bend to wind up the slope",
                        climb * 100.0,
                        spec.max_grade * 100.0
                    ));
                }
                let local = dry
                    .windows(2)
                    .map(|p| (p[1].1 - p[0].1).abs() / (p[1].0 - p[0].0).max(1.0))
                    .fold(0.0_f32, f32::max);
                if local > spec.max_grade {
                    let cut = dry
                        .iter()
                        .map(|&(d, h)| (h - (h0 + (h1 - h0) * (d - d0) / run)).abs())
                        .fold(0.0_f32, f32::max);
                    if cut > spec.max_cut_m {
                        return StretchVerdict::Blocked(format!(
                            "needs {cut:.0} m of cut/fill, at most {:.0} m",
                            spec.max_cut_m
                        ));
                    }
                    works.push(format!("cut/fill {cut:.0} m"));
                }
            }
            // Crossings with other lines.
            for seg in &network.segments {
                let other_rail = seg.kind.is_rail();
                let other_limited = seg.kind.road_tier().is_some_and(RoadTier::limited_access);
                if !seg.spans().any(|(p, q)| crosses(a, b, centre(p), centre(q))) {
                    continue;
                }
                let label = match (spec.rail, other_rail) {
                    (true, false) | (false, true) => "level crossing",
                    (false, false) if spec.limited_access || other_limited => "overpass",
                    (true, true) => "rail junction",
                    _ => continue,
                };
                if !works.iter().any(|w| w == label) {
                    works.push(label.into());
                }
            }
            if works.is_empty() {
                StretchVerdict::Ok
            } else {
                StretchVerdict::Works(works)
            }
        })();
        let bridge = bridge.filter(|_| verdict.buildable());
        out.push(StretchCheck { a, b, length_m, verdict, bridge });
    }
    out
}

/// Readout for the sheet and hint line.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CorridorPathSummary {
    pub length_m: f32,
    pub stretches_ok: usize,
    pub stretches_works: usize,
    pub stretches_blocked: usize,
    /// Truck (12 m/s) time over the buildable stretches at the line's speed.
    pub trip_s: f32,
    /// Up to four "stretch n: reason" lines (blocked first).
    pub notes: Vec<String>,
    /// Bridge materials for the buildable stretches (cargo, units), largest first.
    pub bridge_materials: Vec<(String, f32)>,
    /// Bridge type being built (when the path crosses water) and how many types fit.
    pub bridge: Option<(BridgeType, usize)>,
}

impl CorridorPathSummary {
    /// "Steel 140, Concrete 300" (empty without bridges).
    #[must_use]
    pub fn materials_line(&self) -> String {
        self.bridge_materials.iter().map(|(c, n)| format!("{c} {n:.0}")).collect::<Vec<_>>().join(", ")
    }
}

#[must_use]
pub fn summarize_path(checks: &[StretchCheck], kind: CorridorToolKind) -> CorridorPathSummary {
    let cap = mode_spec(kind).cap_mps;
    let mut s = CorridorPathSummary::default();
    let mut blocked = Vec::new();
    let mut works = Vec::new();
    for (i, c) in checks.iter().enumerate() {
        match &c.verdict {
            StretchVerdict::Ok => s.stretches_ok += 1,
            StretchVerdict::Works(w) => {
                s.stretches_works += 1;
                works.push(format!("stretch {}: {}", i + 1, w.join(", ")));
            }
            StretchVerdict::Blocked(why) => {
                s.stretches_blocked += 1;
                blocked.push(format!("stretch {} blocked: {why}", i + 1));
            }
        }
        if let (true, Some(b)) = (c.verdict.buildable(), c.bridge) {
            for (cargo, per_m) in &b.kind.spec().materials_per_m {
                match s.bridge_materials.iter_mut().find(|(n, _)| n == cargo) {
                    Some((_, q)) => *q += per_m * b.span_m,
                    None => s.bridge_materials.push((cargo.clone(), per_m * b.span_m)),
                }
            }
            s.bridge = Some((b.kind, 0));
        }
        if c.verdict.buildable() {
            s.length_m += c.length_m;
            if cap > 0.0 {
                s.trip_s += c.length_m / cap.min(12.0);
            }
        }
    }
    s.notes = blocked.into_iter().chain(works).take(4).collect();
    s.bridge_materials.sort_by(|a, b| b.1.total_cmp(&a.1));
    if let Some((t, _)) = s.bridge {
        s.bridge = Some((t, bridge_choices(checks).len()));
    }
    s
}

fn tile_of(p: Vec2) -> BuildSiteTile {
    BuildSiteTile { x: p.x.floor().max(0.0) as u32, z: p.y.floor().max(0.0) as u32 }
}

/// Queue every buildable stretch (one network line each) into the construction funnel. Returns
/// how many were queued.
pub fn commit_corridor_path(checks: &[StretchCheck], kind: CorridorToolKind, queue: &mut ConstructionPlanQueue) -> usize {
    let mut n = 0;
    // The whole path undoes as one step.
    let group = queue.begin_group();
    for c in checks.iter().filter(|c| c.verdict.buildable()) {
        let (head, tail) = (tile_of(c.a), tile_of(c.b));
        if head == tail {
            continue;
        }
        let entity_type = match kind {
            CorridorToolKind::Road(tier) => ConstructionType::RoadSegment { head, tail, tier },
            CorridorToolKind::Rail => ConstructionType::RailSegment { head, tail },
            CorridorToolKind::Firebreak => ConstructionType::FirebreakSegment { head, tail },
        };
        queue.enqueue_in_group(ConstructionIntent { entity_type, world_position: (c.a + c.b) * 0.5, rotation: 0.0 }, group);
        // The bridge type the tool showed is the one built.
        if let (Some(b), Some(plan)) = (c.bridge, queue.plans.last_mut()) {
            plan.bridge = Some(b.kind);
        }
        n += 1;
    }
    n
}

/// Route assist grid step (tiles).
const ASSIST_STEP_TILES: f32 = 2.0;
/// Most grid nodes the route assist expands before giving up.
const ASSIST_MAX_EXPANDED: usize = 60_000;
/// Water cells cost this much more per metre (bridges are dear; cross where it is narrow).
const ASSIST_WATER_MUL: f32 = 4.0;

/// Route assist (RD-6 "follow terrain"): a path from `from` to `to` (tile units) that holds the
/// mode's grade (winding up slopes instead of straight at them), crosses water only where the
/// mode can bridge it, and is simplified so every stretch passes [`check_corridor_path`].
/// Returns the points after `from` (ending at `to`), or `None` when no such path is found.
#[must_use]
pub fn assist_route(
    from: Vec2,
    to: Vec2,
    kind: CorridorToolKind,
    ground_m: &dyn Fn(Vec2) -> f32,
    water_m: &dyn Fn(IVec2) -> f32,
    network: &CorridorNetwork,
    map_tiles: UVec2,
) -> Option<Vec<Vec2>> {
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, HashMap};
    let spec = mode_spec(kind);
    if kind == CorridorToolKind::Firebreak || from.distance(to) < ASSIST_STEP_TILES {
        return Some(vec![to]);
    }
    let s = ASSIST_STEP_TILES;
    let step_m = s * TILE_METRES_GAMEPLAY;
    // Search box: the two ends with a margin for detours.
    let margin = (from.distance(to) * 0.5).max(24.0);
    let lo = (from.min(to) - Vec2::splat(margin)).max(Vec2::ZERO);
    let hi = (from.max(to) + Vec2::splat(margin)).min(map_tiles.as_vec2() - Vec2::splat(0.5));
    let gw = ((hi.x - lo.x) / s).ceil() as i32 + 1;
    let gh = ((hi.y - lo.y) / s).ceil() as i32 + 1;
    let pos = |c: IVec2| lo + c.as_vec2() * s;
    let cell = |p: Vec2| ((p - lo) / s).round().as_ivec2().clamp(IVec2::ZERO, IVec2::new(gw - 1, gh - 1));
    let (start, goal) = (cell(from), cell(to));
    let wet = |p: Vec2| water_m(IVec2::new(p.x.floor() as i32, p.y.floor() as i32)) > BRIDGE_DEPTH_M;
    let mut height: HashMap<IVec2, f32> = HashMap::new();
    let mut h_at = |c: IVec2| *height.entry(c).or_insert_with(|| ground_m(pos(c)));
    const DIRS: [(i32, i32); 16] = [
        (1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1),
        (2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (1, -2), (-1, 2), (-1, -2),
    ];
    let key = |c: f32| (c * 16.0) as u64;
    let mut open = BinaryHeap::new();
    let mut best: HashMap<IVec2, f32> = HashMap::new();
    let mut came: HashMap<IVec2, IVec2> = HashMap::new();
    best.insert(start, 0.0);
    open.push(Reverse((0_u64, start.x, start.y)));
    let mut expanded = 0;
    let mut found = false;
    while let Some(Reverse((_, x, y))) = open.pop() {
        let c = IVec2::new(x, y);
        if c == goal {
            found = true;
            break;
        }
        expanded += 1;
        if expanded > ASSIST_MAX_EXPANDED {
            break;
        }
        let g = best[&c];
        let hc = h_at(c);
        for (dx, dy) in DIRS {
            let n = c + IVec2::new(dx, dy);
            if n.x < 0 || n.y < 0 || n.x >= gw || n.y >= gh {
                continue;
            }
            let dist_m = (IVec2::new(dx, dy).as_vec2().length()) * step_m;
            let p = pos(n);
            let water = wet(p) || wet((pos(c) + p) * 0.5);
            if water && spec.max_bridge_m <= 0.0 {
                continue;
            }
            // Over water the deck is level; on land hold the grade with a little slack for cut/fill.
            let cost = if water {
                dist_m * ASSIST_WATER_MUL
            } else {
                let grade = (h_at(n) - hc).abs() / dist_m;
                if grade > spec.max_grade * 0.95 {
                    continue;
                }
                dist_m * (1.0 + grade / spec.max_grade.max(1e-3))
            };
            let ng = g + cost;
            if best.get(&n).is_none_or(|&b| ng < b) {
                best.insert(n, ng);
                came.insert(n, c);
                let f = ng + pos(n).distance(pos(goal)) * TILE_METRES_GAMEPLAY;
                open.push(Reverse((key(f), n.x, n.y)));
            }
        }
    }
    if !found {
        return None;
    }
    let mut cells = vec![goal];
    while let Some(&p) = came.get(cells.last().expect("non-empty")) {
        cells.push(p);
    }
    cells.reverse();
    let mut raw: Vec<Vec2> = cells.iter().map(|&c| pos(c)).collect();
    raw[0] = from;
    *raw.last_mut().expect("non-empty") = to;
    // Simplify: from each kept point reach as far along the raw path as one buildable stretch
    // goes (the check also sees the bend from the previous kept point).
    let buildable = |prev: Option<Vec2>, a: Vec2, b: Vec2| {
        let pts: Vec<Vec2> = prev.into_iter().chain([a, b]).collect();
        check_corridor_path(&pts, kind, ground_m, water_m, network, map_tiles)
            .last()
            .is_some_and(|c| c.verdict.buildable())
    };
    let mut out: Vec<Vec2> = Vec::new();
    let (mut i, mut prev) = (0usize, None::<Vec2>);
    while i + 1 < raw.len() {
        let mut j = i + 1;
        while j + 1 < raw.len() && buildable(prev, raw[i], raw[j + 1]) {
            j += 1;
        }
        prev = Some(raw[i]);
        out.push(raw[j]);
        i = j;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(_: Vec2) -> f32 {
        100.0
    }
    fn dry(_: IVec2) -> f32 {
        0.0
    }
    const MAP: UVec2 = UVec2::new(256, 256);

    fn check(points: &[Vec2], kind: CorridorToolKind, ground: &dyn Fn(Vec2) -> f32, water: &dyn Fn(IVec2) -> f32) -> Vec<StretchCheck> {
        check_corridor_path(points, kind, ground, water, &CorridorNetwork::default(), MAP)
    }

    #[test]
    fn a_river_gets_a_labelled_bridge_and_a_wide_one_is_blocked() {
        let river = |t: IVec2| if (50..54).contains(&t.x) { 3.0 } else { 0.0 };
        let c = check(&[Vec2::new(40.5, 10.5), Vec2::new(64.5, 10.5)], CorridorToolKind::Road(RoadTier::Gravel), &flat, &river);
        // 72 m of 3 m deep river on a gravel road: a Bailey bridge (too long for timber beam).
        match &c[0].verdict {
            StretchVerdict::Works(w) => assert!(w.iter().any(|x| x.starts_with("Bailey bridge")), "{w:?}"),
            v => panic!("expected a bridge, got {v:?}"),
        }
        // A 1.6 km lake: nothing a dirt road builds spans it.
        let lake = |t: IVec2| if (20..120).contains(&t.x) { 3.0 } else { 0.0 };
        let c = check(&[Vec2::new(10.5, 10.5), Vec2::new(130.5, 10.5)], CorridorToolKind::Road(RoadTier::Dirt), &flat, &lake);
        assert!(matches!(c[0].verdict, StretchVerdict::Blocked(ref why) if why.contains("no bridge type")));
        assert!(c[0].bridge.is_none(), "a blocked stretch builds no bridge");
    }

    #[test]
    fn route_assist_winds_up_a_slope_a_straight_line_cannot_climb() {
        // A ridge rising 40 m over 10 tiles (160 m, 25%) across the straight line, fading along y.
        let ridge = |p: Vec2| {
            let x = p.x - 40.0;
            let along = (p.y / 120.0).clamp(0.0, 1.0);
            100.0 + if x < 0.0 { 0.0 } else if x < 10.0 { 4.0 * x } else { 40.0 } * (1.0 - along)
        };
        let kind = CorridorToolKind::Road(RoadTier::Gravel);
        let (a, b) = (Vec2::new(30.5, 10.5), Vec2::new(60.5, 10.5));
        let straight = check(&[a, b], kind, &ridge, &dry);
        assert!(matches!(straight[0].verdict, StretchVerdict::Blocked(_)), "straight up is too steep");
        let route = assist_route(a, b, kind, &ridge, &dry, &CorridorNetwork::default(), MAP).expect("a way round");
        assert_eq!(route.last(), Some(&b));
        let pts: Vec<Vec2> = std::iter::once(a).chain(route.iter().copied()).collect();
        let checks = check(&pts, kind, &ridge, &dry);
        assert!(checks.iter().all(|c| c.verdict.buildable()), "{checks:?}");
        assert!(pts.len() > 2, "it bends");
    }

    #[test]
    fn the_player_can_pick_another_bridge_type_and_sees_its_bill() {
        let river = |t: IVec2| if (50..54).contains(&t.x) { 3.0 } else { 0.0 };
        let kind = CorridorToolKind::Road(RoadTier::Gravel);
        let mut c = check(&[Vec2::new(40.5, 10.5), Vec2::new(64.5, 10.5)], kind, &flat, &river);
        let choices = bridge_choices(&c);
        assert_eq!(choices.first(), Some(&BridgeType::Bailey), "cheapest that fits first");
        assert!(choices.contains(&BridgeType::SteelGirder), "{choices:?}");
        assert!(!choices.contains(&BridgeType::TimberBeam), "too short a type is not offered");
        apply_bridge_choice(&mut c, Some(BridgeType::SteelGirder));
        assert_eq!(c[0].bridge.map(|b| b.kind), Some(BridgeType::SteelGirder));
        match &c[0].verdict {
            StretchVerdict::Works(w) => assert!(w.iter().any(|x| x.starts_with("steel girder bridge")), "{w:?}"),
            v => panic!("expected a bridge, got {v:?}"),
        }
        let s = summarize_path(&c, kind);
        assert!(s.bridge_materials.iter().any(|(n, q)| n == "Steel" && *q > 0.0), "{:?}", s.bridge_materials);
        // A type that does not fit falls back to the cheapest.
        apply_bridge_choice(&mut c, Some(BridgeType::TimberBeam));
        assert_eq!(c[0].bridge.map(|b| b.kind), Some(BridgeType::Bailey));
        let mut q = ConstructionPlanQueue::default();
        apply_bridge_choice(&mut c, Some(BridgeType::SteelGirder));
        assert_eq!(commit_corridor_path(&c, kind, &mut q), 1);
        assert_eq!(q.plans[0].bridge, Some(BridgeType::SteelGirder), "the shown type is the one built");
    }

    #[test]
    fn rail_refuses_a_climb_a_road_takes_and_says_why() {
        // 6% climb: fine for a gravel road (12%), too steep for rail (3%).
        let slope = |p: Vec2| p.x * TILE_METRES_GAMEPLAY * 0.06;
        let pts = [Vec2::new(10.5, 10.5), Vec2::new(40.5, 10.5)];
        assert_eq!(check(&pts, CorridorToolKind::Road(RoadTier::Gravel), &slope, &dry)[0].verdict, StretchVerdict::Ok);
        match &check(&pts, CorridorToolKind::Rail, &slope, &dry)[0].verdict {
            StretchVerdict::Blocked(why) => assert!(why.contains("climb") && why.contains("3.0%"), "{why}"),
            v => panic!("rail should be blocked, got {v:?}"),
        }
    }

    #[test]
    fn a_bump_on_a_level_line_is_cut_and_filled() {
        let bump = |p: Vec2| if (20.0..22.0).contains(&p.x) { 105.0 } else { 100.0 };
        let c = check(&[Vec2::new(10.5, 10.5), Vec2::new(40.5, 10.5)], CorridorToolKind::Road(RoadTier::Paved), &bump, &dry);
        assert!(matches!(&c[0].verdict, StretchVerdict::Works(w) if w.iter().any(|x| x.starts_with("cut/fill"))), "{:?}", c[0].verdict);
    }

    #[test]
    fn rail_rejects_a_sharp_corner_but_takes_a_wide_curve() {
        let sharp = [Vec2::new(10.5, 10.5), Vec2::new(40.5, 10.5), Vec2::new(40.5, 40.5)];
        let c = check(&sharp, CorridorToolKind::Rail, &flat, &dry);
        assert!(matches!(&c[1].verdict, StretchVerdict::Blocked(why) if why.contains("curve")));
        let gentle = [Vec2::new(10.5, 10.5), Vec2::new(60.5, 10.5), Vec2::new(110.5, 15.5)];
        let c = check(&gentle, CorridorToolKind::Rail, &flat, &dry);
        assert!(c.iter().all(|s| s.verdict.buildable()), "{c:?}");
    }

    #[test]
    fn road_over_rail_is_a_level_crossing_and_commit_skips_blocked_stretches() {
        let mut net = CorridorNetwork::default();
        net.add_segment(
            crate::construction::CorridorKind::Rail { profile: "default_rail".into() },
            BuildSiteTile { x: 20, z: 0 },
            BuildSiteTile { x: 20, z: 40 },
        );
        let pts = [Vec2::new(10.5, 20.5), Vec2::new(30.5, 20.5), Vec2::new(300.5, 20.5)];
        let kind = CorridorToolKind::Road(RoadTier::Gravel);
        let c = check_corridor_path(&pts, kind, &flat, &dry, &net, MAP);
        assert!(matches!(&c[0].verdict, StretchVerdict::Works(w) if w.contains(&"level crossing".to_string())));
        assert!(matches!(&c[1].verdict, StretchVerdict::Blocked(why) if why == "outside the map"));
        let mut queue = ConstructionPlanQueue::default();
        assert_eq!(commit_corridor_path(&c, kind, &mut queue), 1);
        let s = summarize_path(&c, kind);
        assert_eq!((s.stretches_works, s.stretches_blocked), (1, 1));
        assert!(s.notes[0].contains("blocked"));
    }
}
