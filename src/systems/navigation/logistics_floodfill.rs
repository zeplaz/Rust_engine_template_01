//! Grid **logistics pathfinding** — Dijkstra on a rectangular cell field (flood-fill style expansion),
//! parallel to potential-field navigation in `potental_feild_nav`.
//!
//! Each step uses average `cost_mul` between cells and tracks **max environmental risk** along the path
//! (fire exposure, stuck risk, chunk-level risk proxies). Callers map terrain/mobility + ecology/fire
//! into [`LogisticsTile`] via [`logistics_tile_from_mobility_and_env`] or custom closures.
//!
//! **NAV-04** (aig3 4.3.7 · LEGACY `node_array_astar` aig_e3 p249): dense per-cell
//! [`LogisticsNodeRecord`] array with Unvisited/Open/Closed category + generation reuse —
//! no `HashMap` per search. Priority is concern-blended cost; heuristic is 0 when
//! `risk_weight > 0` (no safe Manhattan lower bound under max-risk blend).
//!
//! **NAV-06** (aig3 4.4.6–4.4.7 · LEGACY `pathfinding` aig_e3): concern blend = cost
//! function; [`smooth_logistics_tile_path`] string-pulls grid paths for vehicle motion
//! (keeps search path intact — callers opt in after search).

use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

use crate::systems::ecology::{ChunkEcology, VegetationField};
use crate::systems::fire::ChunkFuelProfile;
use crate::systems::fire::combustion::profile_weighted_smoke_toxic_explosion;
use crate::systems::fire::ChunkSmokeField;
use crate::terrain::mobility::MovementHint;

/// One cell in the logistics grid — **not** ECS; built from mobility hints + environment samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogisticsTile {
    pub blocked: bool,
    /// Multiplicative factor folded into edge weight (≥ small epsilon when traversable).
    pub cost_mul: f32,
    /// Local risk in `[0, 1]` — combined with path metric as the max over visited cells.
    pub env_risk: f32,
}

impl Default for LogisticsTile {
    fn default() -> Self {
        Self {
            blocked: false,
            cost_mul: 1.0,
            env_risk: 0.0,
        }
    }
}

/// Optional scalar bundle from [`crate::systems::ecology::ChunkEcology`] / fire (caller supplies per-cell or chunk mean).
#[derive(Clone, Copy, Debug, Default)]
pub struct LogisticsEnvironmentSample {
    /// Active heat `[0, 1]` (e.g. [`crate::systems::fire::ChunkSurfaceFire::heat`] or per-cell overlay).
    pub fire_heat: f32,
    /// Ignition / spread pressure `[0, 1]`.
    pub fire_risk: f32,
    /// Biomass / canopy proxy `[0, 1]` — slows movement through dense growth.
    pub biomass: f32,
    /// Meso concealment `[0, 1]` (`VegetationField::concealment`) — slows convoys / raises ambush risk proxy.
    pub concealment: f32,
    /// Combined fuel `[0, 1]` for logistics exposure (surface + canopy stress).
    pub fuel_load: f32,
    /// Canopy / terrain smoke absorption from meso vegetation `[0, 1]`.
    pub smoke_absorption: f32,
    /// Smoke column density `[0, 1]` — AI / LOS / helo stubs (`base_fire_sim.md` §6).
    pub smoke_density: f32,
    /// Inhalation / chemical hazard proxy `[0, 1]`.
    pub toxicity: f32,
    /// Normalized explosion / pressure risk from volatile fuels under current heat.
    pub explosion_risk: f32,
}

impl LogisticsEnvironmentSample {
    /// Chunk-uniform sample from macro + meso ecology (`VegetationField`) and surface heat.
    pub fn from_chunk_ecology_vegetation(eco: &ChunkEcology, veg: &VegetationField, fire_heat: f32) -> Self {
        Self {
            fire_heat,
            fire_risk: eco.fire_risk,
            biomass: eco.biomass,
            concealment: veg.concealment,
            fuel_load: veg.fuel_load,
            smoke_absorption: veg.smoke_absorption,
            smoke_density: veg.smoke_absorption * 0.65,
            toxicity: 0.0,
            explosion_risk: 0.0,
        }
    }

    /// Chunk sim sample: ecology + vegetation + surface heat + optional smoke and layered fuel profile.
    pub fn from_chunk_with_fire_smoke(
        eco: &ChunkEcology,
        veg: &VegetationField,
        fire_heat: f32,
        smoke: Option<&ChunkSmokeField>,
        fuel_profile: Option<&ChunkFuelProfile>,
    ) -> Self {
        let (_, _, ex) = fuel_profile
            .map(profile_weighted_smoke_toxic_explosion)
            .unwrap_or((0.0, 0.0, 0.0));
        let heat = fire_heat.clamp(0.0, 1.0);
        Self {
            fire_heat: heat,
            fire_risk: eco.fire_risk,
            biomass: eco.biomass,
            concealment: veg.concealment,
            fuel_load: veg.fuel_load,
            smoke_absorption: veg.smoke_absorption,
            smoke_density: smoke.map(|s| s.density).unwrap_or(veg.smoke_absorption * 0.5),
            toxicity: smoke.map(|s| s.toxicity).unwrap_or(0.0),
            explosion_risk: ex * heat,
        }
    }
}

/// Shortest path under summed edge costs, with conservative **max** env risk on the route.
#[derive(Clone, Debug, PartialEq)]
pub struct LogisticsPathResult {
    pub path: Vec<(i32, i32)>,
    pub total_cost: f32,
    pub max_env_risk: f32,
}

#[inline]
fn idx(x: i32, y: i32, width: i32) -> usize {
    (y * width + x) as usize
}

#[inline]
fn tile_or_blocked(width: i32, height: i32, x: i32, y: i32, tile_at: &mut impl FnMut(i32, i32) -> LogisticsTile) -> LogisticsTile {
    if x < 0 || y < 0 || x >= width || y >= height {
        return LogisticsTile {
            blocked: true,
            cost_mul: 1.0,
            env_risk: 0.0,
        };
    }
    tile_at(x, y)
}

fn better_candidate(
    new_dist: f32,
    new_risk: f32,
    old_dist: f32,
    old_risk: f32,
    concern: super::path_concerns::LogisticsPathConcern,
) -> bool {
    const EPS: f32 = 1e-4;
    let nb = concern.blend_cost(new_dist, new_risk);
    let ob = concern.blend_cost(old_dist, old_risk);
    if nb + EPS < ob {
        return true;
    }
    (nb - ob).abs() <= EPS && new_risk + EPS < old_risk
}

/// Book open/closed membership on the record itself (aig3 4.3.7) — closed list is the category bit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LogisticsNodeCategory {
    Unvisited = 0,
    Open = 1,
    Closed = 2,
}

/// Per-cell A\* / Dijkstra record (aig3 4.3.7 node array — indexed, not hashed).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogisticsNodeRecord {
    pub cost: f32,
    pub risk: f32,
    pub parent: Option<(i32, i32)>,
    pub category: LogisticsNodeCategory,
    /// Matches [`LogisticsNodeArray::generation`] when this slot belongs to the active search.
    pub generation: u32,
}

impl LogisticsNodeRecord {
    const fn empty() -> Self {
        Self {
            cost: f32::INFINITY,
            risk: 0.0,
            parent: None,
            category: LogisticsNodeCategory::Unvisited,
            generation: 0,
        }
    }
}

/// Dense node-array scratch for logistics searches (NAV-04).
///
/// Call [`LogisticsNodeArray::begin_search`] before each run — bumps generation so stale
/// slots are ignored without zero-filling the whole buffer (book clear trick).
#[derive(Clone, Debug)]
pub struct LogisticsNodeArray {
    nodes: Vec<LogisticsNodeRecord>,
    width: i32,
    height: i32,
    generation: u32,
}

impl Default for LogisticsNodeArray {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            width: 0,
            height: 0,
            generation: 0,
        }
    }
}

impl LogisticsNodeArray {
    #[must_use]
    pub fn with_dimensions(width: i32, height: i32) -> Self {
        let mut s = Self::default();
        s.begin_search(width, height);
        s
    }

    /// Capacity in cells (may exceed `width * height` after growth).
    #[must_use]
    pub fn capacity_cells(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn generation(&self) -> u32 {
        self.generation
    }

    #[must_use]
    pub fn dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// Prepare for a search of `width × height` without a full `HashMap` clear.
    pub fn begin_search(&mut self, width: i32, height: i32) {
        debug_assert!(width > 0 && height > 0);
        let n = (width as usize).saturating_mul(height as usize);
        if self.nodes.len() < n {
            self.nodes.resize(n, LogisticsNodeRecord::empty());
        }
        self.width = width;
        self.height = height;
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            for slot in &mut self.nodes {
                *slot = LogisticsNodeRecord::empty();
            }
            self.generation = 1;
        }
    }

    #[inline]
    fn is_current(&self, rec: &LogisticsNodeRecord) -> bool {
        rec.generation == self.generation
    }

    #[inline]
    fn cost_of(&self, i: usize) -> f32 {
        let rec = &self.nodes[i];
        if self.is_current(rec) {
            rec.cost
        } else {
            f32::INFINITY
        }
    }

    #[inline]
    fn risk_of(&self, i: usize) -> f32 {
        let rec = &self.nodes[i];
        if self.is_current(rec) {
            rec.risk
        } else {
            0.0
        }
    }

    #[inline]
    fn write(
        &mut self,
        i: usize,
        cost: f32,
        risk: f32,
        parent: Option<(i32, i32)>,
        category: LogisticsNodeCategory,
    ) {
        let gen = self.generation;
        self.nodes[i] = LogisticsNodeRecord {
            cost,
            risk,
            parent,
            category,
            generation: gen,
        };
    }

    #[inline]
    fn category_of(&self, i: usize) -> LogisticsNodeCategory {
        let rec = &self.nodes[i];
        if self.is_current(rec) {
            rec.category
        } else {
            LogisticsNodeCategory::Unvisited
        }
    }

    #[inline]
    fn set_category(&mut self, i: usize, category: LogisticsNodeCategory) {
        let gen = self.generation;
        let rec = &mut self.nodes[i];
        if rec.generation != gen {
            *rec = LogisticsNodeRecord {
                generation: gen,
                category,
                ..LogisticsNodeRecord::empty()
            };
        } else {
            rec.category = category;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DijkstraState {
    cost: f32,
    risk: f32,
    blend: f32,
    x: i32,
    y: i32,
}

impl Eq for DijkstraState {}

impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.blend
            .total_cmp(&other.blend)
            .then_with(|| self.risk.total_cmp(&other.risk))
            .then_with(|| self.x.cmp(&other.x))
            .then_with(|| self.y.cmp(&other.y))
    }
}

/// NAV-04 / NAV-13 / NAV-14: Dijkstra on a dense `width × height` grid, 4-neighbor edges.
///
/// Reuse `nodes` across queries via [`LogisticsNodeArray::begin_search`] each call (no per-search HashMap).
/// Edge weight from `a` to `b`: `0.5 * (tile(a).cost_mul + tile(b).cost_mul)` when both are traversable.
/// Optional NAV-13 forecast scales edge cost from `depart_tick`; concern blends time vs max env risk.
pub fn logistics_grid_path(
    width: i32,
    height: i32,
    start: (i32, i32),
    goal: (i32, i32),
    mut tile_at: impl FnMut(i32, i32) -> LogisticsTile,
    concern: super::path_concerns::LogisticsPathConcern,
    forecast: Option<&super::cost_forecast::LogisticsCostForecastBook>,
    depart_tick: u64,
    nodes: &mut LogisticsNodeArray,
) -> Option<LogisticsPathResult> {
    if width <= 0 || height <= 0 {
        return None;
    }

    let start_t = tile_or_blocked(width, height, start.0, start.1, &mut tile_at);
    let goal_t = tile_or_blocked(width, height, goal.0, goal.1, &mut tile_at);
    if start_t.blocked || goal_t.blocked {
        return None;
    }

    nodes.begin_search(width, height);

    let si = idx(start.0, start.1, width);
    let start_risk = start_t.env_risk.clamp(0.0, 1.0);
    nodes.write(si, 0.0, start_risk, None, LogisticsNodeCategory::Open);

    let mut heap: BinaryHeap<Reverse<DijkstraState>> = BinaryHeap::new();
    let start_blend = concern.blend_cost(0.0, start_risk);
    heap.push(Reverse(DijkstraState {
        cost: 0.0,
        risk: start_risk,
        blend: start_blend,
        x: start.0,
        y: start.1,
    }));

    static DX: [i32; 4] = [1, -1, 0, 0];
    static DY: [i32; 4] = [0, 0, 1, -1];

    const EPS: f32 = 1e-4;
    // Pure-time concerns settle the goal on first pop (Dijkstra / consistent h=0).
    let early_goal = concern.risk_weight <= 1e-6;
    while let Some(Reverse(DijkstraState {
        cost: d,
        risk: r,
        blend: _,
        x,
        y,
    })) = heap.pop()
    {
        let i = idx(x, y, width);
        let cur_cost = nodes.cost_of(i);
        let cur_risk = nodes.risk_of(i);
        if d > cur_cost + EPS {
            continue;
        }
        if (d - cur_cost).abs() <= EPS && r > cur_risk + EPS {
            continue;
        }
        if nodes.category_of(i) == LogisticsNodeCategory::Closed {
            continue;
        }
        nodes.set_category(i, LogisticsNodeCategory::Closed);

        if early_goal && x == goal.0 && y == goal.1 {
            break;
        }

        let ta = tile_or_blocked(width, height, x, y, &mut tile_at);
        if ta.blocked {
            continue;
        }

        for k in 0..4 {
            let nx = x + DX[k];
            let ny = y + DY[k];
            let tb = tile_or_blocked(width, height, nx, ny, &mut tile_at);
            if tb.blocked {
                continue;
            }
            let base = 0.5 * (ta.cost_mul.max(1e-3) + tb.cost_mul.max(1e-3));
            let traverse = base.ceil().max(1.0) as u64;
            let edge = match forecast {
                Some(book) if book.ready => book.edge_cost(base, depart_tick, traverse),
                _ => base,
            };
            let nd = d + edge;
            let nr = r.max(ta.env_risk).max(tb.env_risk).clamp(0.0, 1.0);
            let ni = idx(nx, ny, width);
            let old_dist = nodes.cost_of(ni);
            let old_risk = nodes.risk_of(ni);
            let cat = nodes.category_of(ni);
            // Risk-blended searches may reopen Closed; pure-time never needs reopen after settle.
            let may_touch = cat != LogisticsNodeCategory::Closed || !early_goal;
            if may_touch && better_candidate(nd, nr, old_dist, old_risk, concern) {
                nodes.write(ni, nd, nr, Some((x, y)), LogisticsNodeCategory::Open);
                heap.push(Reverse(DijkstraState {
                    cost: nd,
                    risk: nr,
                    blend: concern.blend_cost(nd, nr),
                    x: nx,
                    y: ny,
                }));
            }
        }
    }

    let gi = idx(goal.0, goal.1, width);
    let goal_cost = nodes.cost_of(gi);
    if !goal_cost.is_finite() {
        return None;
    }

    let mut path = Vec::new();
    let mut cur = Some(goal);
    while let Some((cx, cy)) = cur {
        path.push((cx, cy));
        if cx == start.0 && cy == start.1 {
            break;
        }
        let pi = idx(cx, cy, width);
        let rec = &nodes.nodes[pi];
        cur = if nodes.is_current(rec) {
            rec.parent
        } else {
            None
        };
    }
    if path.last().map(|p| (p.0, p.1)) != Some(start) {
        return None;
    }
    path.reverse();

    Some(LogisticsPathResult {
        path,
        total_cost: goal_cost,
        max_env_risk: nodes.risk_of(gi),
    })
}

/// True when every grid cell on the Bresenham segment `a → b` is traversable.
#[must_use]
pub fn logistics_tile_line_clear(
    a: (i32, i32),
    b: (i32, i32),
    width: i32,
    height: i32,
    tile_at: &mut impl FnMut(i32, i32) -> LogisticsTile,
) -> bool {
    let (mut x0, mut y0) = a;
    let (x1, y1) = b;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if tile_or_blocked(width, height, x0, y0, tile_at).blocked {
            return false;
        }
        if x0 == x1 && y0 == y1 {
            return true;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// NAV-06 path smoothing (aig3 4.4.7): axis-aligned string-pull for tile grids.
///
/// Collapses runs that share a row or column when the segment is clear. Diagonal
/// shortcuts are intentionally **not** taken (prevents corner-cutting through
/// blocked cells). Search cost / risk stay on the raw Dijkstra result.
#[must_use]
pub fn smooth_logistics_tile_path(
    path: &[(i32, i32)],
    width: i32,
    height: i32,
    tile_at: &mut impl FnMut(i32, i32) -> LogisticsTile,
) -> Vec<(i32, i32)> {
    if path.len() <= 2 {
        return path.to_vec();
    }
    let mut out = Vec::with_capacity(path.len());
    let mut i = 0;
    out.push(path[0]);
    while i < path.len() - 1 {
        let mut best = i + 1;
        let mut j = i + 2;
        while j < path.len() {
            let (ax, ay) = path[i];
            let (bx, by) = path[j];
            let axis_aligned = ax == bx || ay == by;
            if axis_aligned && logistics_tile_line_clear(path[i], path[j], width, height, tile_at)
            {
                best = j;
                j += 1;
            } else {
                break;
            }
        }
        out.push(path[best]);
        i = best;
    }
    out
}

impl LogisticsPathResult {
    /// Opt-in NAV-06 smooth for vehicle / motion consumers (does not mutate search metrics).
    #[must_use]
    pub fn smoothed_for_motion(
        &self,
        width: i32,
        height: i32,
        tile_at: &mut impl FnMut(i32, i32) -> LogisticsTile,
    ) -> Vec<(i32, i32)> {
        smooth_logistics_tile_path(&self.path, width, height, tile_at)
    }
}

/// Map mobility evaluation + ecology/fire scalars into a [`LogisticsTile`] for the floodfill grid.
pub fn logistics_tile_from_mobility_and_env(hint: &MovementHint, env: &LogisticsEnvironmentSample) -> LogisticsTile {
    let biomass = env.biomass.clamp(0.0, 1.0);
    let heat = env.fire_heat.clamp(0.0, 1.0);
    let risk = env.fire_risk.clamp(0.0, 1.0);

    let mut cost_mul = hint.cost_mul.max(1e-3);
    cost_mul *= 1.0 + 0.18 * biomass;
    cost_mul *= 1.0 + 0.55 * heat;
    let concealment = env.concealment.clamp(0.0, 1.0);
    cost_mul *= 1.0 + 0.12 * concealment;
    let fuel_load = env.fuel_load.clamp(0.0, 1.0);
    cost_mul *= 1.0 + 0.14 * fuel_load;

    let smoke_col = env
        .smoke_density
        .max(env.smoke_absorption * 0.5)
        .clamp(0.0, 1.0);
    cost_mul *= 1.0 + smoke_col * 0.4;
    cost_mul *= 1.0 + env.explosion_risk.clamp(0.0, 1.0) * 2.0;

    let env_risk = hint
        .stuck_risk
        .clamp(0.0, 1.0)
        .max(risk)
        .max(heat)
        .max(smoke_col * 0.38)
        .max(env.toxicity.clamp(0.0, 1.0) * 0.35);

    LogisticsTile {
        blocked: hint.blocked,
        cost_mul,
        env_risk,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn constant_grid(w: i32, h: i32, t: LogisticsTile) -> impl FnMut(i32, i32) -> LogisticsTile {
        move |x, y| {
            if x < 0 || y < 0 || x >= w || y >= h {
                LogisticsTile {
                    blocked: true,
                    ..t
                }
            } else {
                t
            }
        }
    }

    fn grid_path(
        width: i32,
        height: i32,
        start: (i32, i32),
        goal: (i32, i32),
        tile_at: impl FnMut(i32, i32) -> LogisticsTile,
        concern: super::path_concerns::LogisticsPathConcern,
        forecast: Option<&super::cost_forecast::LogisticsCostForecastBook>,
        depart_tick: u64,
    ) -> Option<LogisticsPathResult> {
        let mut nodes = LogisticsNodeArray::default();
        logistics_grid_path(
            width,
            height,
            start,
            goal,
            tile_at,
            concern,
            forecast,
            depart_tick,
            &mut nodes,
        )
    }

    #[test]
    fn straight_path_open_grid() {
        let mut g = constant_grid(5, 5, LogisticsTile::default());
        let r = grid_path(5, 5, (0, 0), (4, 0), |x, y| g(x, y), super::path_concerns::LogisticsPathConcern::DEFAULT, None, 0).unwrap();
        assert_eq!(r.path.len(), 5);
        assert!((r.total_cost - 4.0).abs() < 0.01, "cost {}", r.total_cost);
        assert!(r.max_env_risk < 0.01);
    }

    #[test]
    fn wall_forces_detour() {
        let mut tiles = [[LogisticsTile::default(); 5]; 5];
        for x in 0..5 {
            tiles[2][x] = LogisticsTile {
                blocked: x != 4,
                ..LogisticsTile::default()
            };
        }
        let r = grid_path(5, 5, (0, 0), (4, 4), |x, y| tiles[y as usize][x as usize], super::path_concerns::LogisticsPathConcern::DEFAULT, None, 0).unwrap();
        assert!(r.path.iter().all(|p| p.1 != 2 || p.0 == 4));
    }

    #[test]
    fn picks_lower_cost_branch() {
        // 4-neighbor only: route must step around a high-cost corridor cell (1,1), not cut diagonally.
        let mut tiles = [[LogisticsTile::default(); 3]; 3];
        tiles[1][1] = LogisticsTile {
            blocked: false,
            cost_mul: 12.0,
            env_risk: 0.0,
        };
        let r = grid_path(3, 3, (0, 1), (2, 1), |x, y| tiles[y as usize][x as usize], super::path_concerns::LogisticsPathConcern::DEFAULT, None, 0).unwrap();
        assert!(
            !r.path.contains(&(1, 1)),
            "should avoid expensive center: {:?}",
            r.path
        );
        let via_center = 0.5 * (1.0 + 12.0) + 0.5 * (12.0 + 1.0);
        assert!(
            r.total_cost + 0.05 < via_center,
            "expected cheaper than through (1,1); cost {}",
            r.total_cost
        );
    }

    #[test]
    fn risk_averse_order_takes_longer_safe_route() {
        let w = 7_i32;
        let h = 3_i32;
        let mut tiles = [[LogisticsTile::default(); 7]; 3];
        for x in 1..6 {
            tiles[1][x as usize] = LogisticsTile {
                blocked: false,
                cost_mul: 1.0,
                env_risk: 0.95,
            };
        }
        let tile_at = |x: i32, y: i32| {
            if x < 0 || y < 0 || x >= w || y >= h {
                LogisticsTile {
                    blocked: true,
                    ..LogisticsTile::default()
                }
            } else {
                tiles[y as usize][x as usize]
            }
        };
        let direct = grid_path(
            w,
            h,
            (0, 1),
            (6, 1),
            tile_at,
            super::path_concerns::LogisticsPathConcern::DEFAULT,
            None,
            0,
        )
        .expect("direct route");
        assert!(
            direct.max_env_risk > 0.9,
            "baseline should traverse the hot corridor"
        );
        let cautious = grid_path(
            w,
            h,
            (0, 1),
            (6, 1),
            tile_at,
            crate::systems::navigation::LogisticsPathConcern {
                time_weight: 1.0,
                risk_weight: 12.0,
            },
            None,
            0,
        )
        .expect("detour route");
        assert!(
            cautious.total_cost > direct.total_cost,
            "safe route is longer in time"
        );
        assert!(
            cautious.max_env_risk < 0.2,
            "risk-averse path should avoid the hot corridor"
        );
        assert!(
            !cautious.path.iter().any(|&(x, y)| y == 1 && (1..6).contains(&x)),
            "path {:?} should detour around y=1 risk strip",
            cautious.path
        );
    }

    #[test]
    fn time_aware_forecast_raises_path_cost() {
        let tile = |_x: i32, _y: i32| LogisticsTile {
            blocked: false,
            cost_mul: 1.0,
            env_risk: 0.0,
        };
        let concern = crate::systems::navigation::LogisticsPathConcern::DEFAULT;
        let static_path = grid_path(3, 1, (0, 0), (2, 0), tile, concern, None, 0).expect("static");
        let mut book = crate::systems::navigation::LogisticsCostForecastBook::default();
        book.ready = true;
        book.buckets = [
            crate::systems::navigation::WeatherCostBucket {
                start_tick: 0,
                mul: 1.0,
            },
            crate::systems::navigation::WeatherCostBucket {
                start_tick: 1,
                mul: 0.25,
            },
            crate::systems::navigation::WeatherCostBucket {
                start_tick: 100,
                mul: 0.25,
            },
            crate::systems::navigation::WeatherCostBucket {
                start_tick: 200,
                mul: 0.25,
            },
        ];
        let late = grid_path(3, 1, (0, 0), (2, 0), tile, concern, Some(&book), 40).expect("late");
        assert!(
            late.total_cost > static_path.total_cost,
            "late={} static={}",
            late.total_cost,
            static_path.total_cost
        );
    }

    #[test]
    fn env_sample_raises_risk_and_cost() {
        let hint = MovementHint {
            cost_mul: 1.0,
            blocked: false,
            stuck_risk: 0.1,
        };
        let t0 = logistics_tile_from_mobility_and_env(
            &hint,
            &LogisticsEnvironmentSample {
                fire_heat: 0.0,
                fire_risk: 0.0,
                biomass: 0.0,
                ..Default::default()
            },
        );
        let t1 = logistics_tile_from_mobility_and_env(
            &hint,
            &LogisticsEnvironmentSample {
                fire_heat: 0.8,
                fire_risk: 0.5,
                biomass: 0.5,
                ..Default::default()
            },
        );
        assert!(t1.cost_mul > t0.cost_mul);
        assert!(t1.env_risk >= t0.env_risk);
        assert!(t1.env_risk >= 0.5);
    }
}

#[cfg(test)]
mod nav04_tests {
    use super::*;

    fn grid_path(
        width: i32,
        height: i32,
        start: (i32, i32),
        goal: (i32, i32),
        tile_at: impl FnMut(i32, i32) -> LogisticsTile,
        concern: super::path_concerns::LogisticsPathConcern,
        forecast: Option<&super::cost_forecast::LogisticsCostForecastBook>,
        depart_tick: u64,
    ) -> Option<LogisticsPathResult> {
        let mut nodes = LogisticsNodeArray::default();
        logistics_grid_path(
            width,
            height,
            start,
            goal,
            tile_at,
            concern,
            forecast,
            depart_tick,
            &mut nodes,
        )
    }

    /// NAV-04 tripwire: scratch capacity is dense width×height (no HashMap growth).
    #[test]
    fn nav04_node_array_is_dense_width_times_height() {
        let mut nodes = LogisticsNodeArray::default();
        nodes.begin_search(8, 5);
        assert_eq!(nodes.capacity_cells(), 40);
        assert_eq!(nodes.dimensions(), (8, 5));
        assert_eq!(nodes.generation(), 1);
    }

    /// NAV-04 tripwire: generation bump reuses buffer; second search stays correct.
    #[test]
    fn nav04_generation_reuse_keeps_paths_correct() {
        let tile = |_x: i32, _y: i32| LogisticsTile::default();
        let concern = crate::systems::navigation::LogisticsPathConcern::DEFAULT;
        let mut nodes = LogisticsNodeArray::default();
        let a = logistics_grid_path(
            4, 4, (0, 0), (3, 0), tile, concern, None, 0, &mut nodes,
        )
        .expect("first");
        let gen1 = nodes.generation();
        let cap1 = nodes.capacity_cells();
        let b = logistics_grid_path(
            4, 4, (0, 3), (3, 3), tile, concern, None, 0, &mut nodes,
        )
        .expect("second");
        assert!(nodes.generation() > gen1);
        assert_eq!(nodes.capacity_cells(), cap1);
        assert_eq!(a.path.len(), 4);
        assert_eq!(b.path.len(), 4);
        assert!((a.total_cost - 3.0).abs() < 0.01);
        assert!((b.total_cost - 3.0).abs() < 0.01);
    }

    /// NAV-04 tripwire: scratch path matches fresh alloc path (node array ≠ algorithm change).
    #[test]
    fn nav04_scratch_matches_fresh_search() {
        let mut tiles = [[LogisticsTile::default(); 5]; 5];
        for x in 0..5 {
            tiles[2][x] = LogisticsTile {
                blocked: x != 4,
                ..LogisticsTile::default()
            };
        }
        let tile_at = |x: i32, y: i32| tiles[y as usize][x as usize];
        let fresh = grid_path(
            5,
            5,
            (0, 0),
            (4, 4),
            tile_at,
            crate::systems::navigation::LogisticsPathConcern::DEFAULT,
            None,
            0,
        )
        .unwrap();
        let mut nodes = LogisticsNodeArray::default();
        let reused = logistics_grid_path(
            5,
            5,
            (0, 0),
            (4, 4),
            tile_at,
            crate::systems::navigation::LogisticsPathConcern::DEFAULT,
            None,
            0,
            &mut nodes,
        )
        .unwrap();
        assert_eq!(fresh.path, reused.path);
        assert!((fresh.total_cost - reused.total_cost).abs() < 1e-4);
    }

    /// NAV-04: goal settles Closed under pure-time concern (early exit path).
    #[test]
    fn nav04_pure_time_marks_goal_closed() {
        let mut nodes = LogisticsNodeArray::default();
        let tile = |_x: i32, _y: i32| LogisticsTile::default();
        let _ = logistics_grid_path(
            3,
            1,
            (0, 0),
            (2, 0),
            tile,
            crate::systems::navigation::LogisticsPathConcern::DEFAULT,
            None,
            0,
            &mut nodes,
        )
        .expect("path");
        let gi = 2usize;
        assert_eq!(nodes.category_of(gi), LogisticsNodeCategory::Closed);
    }

    #[test]
    fn nav06_smooth_pulls_open_corridor_to_endpoints() {
        let open = |_x: i32, _y: i32| LogisticsTile::default();
        let raw = [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)];
        let mut tile = open;
        let smooth = smooth_logistics_tile_path(&raw, 5, 1, &mut tile);
        assert_eq!(smooth, vec![(0, 0), (4, 0)]);
    }

    #[test]
    fn nav06_smooth_keeps_bend_around_block() {
        // L-path: cannot axis-pull (0,0)→(1,2); vertical run (1,0)→(1,2) collapses.
        let mut tile = |x: i32, y: i32| {
            if x == 0 && y == 1 {
                LogisticsTile {
                    blocked: true,
                    ..LogisticsTile::default()
                }
            } else {
                LogisticsTile::default()
            }
        };
        let raw = [(0, 0), (1, 0), (1, 1), (1, 2)];
        let smooth = smooth_logistics_tile_path(&raw, 3, 3, &mut tile);
        assert_eq!(smooth, vec![(0, 0), (1, 0), (1, 2)]);
    }

    #[test]
    fn nav06_smoothed_for_motion_preserves_search_metrics() {
        let r = grid_path(
            5,
            1,
            (0, 0),
            (4, 0),
            |_x, _y| LogisticsTile::default(),
            super::path_concerns::LogisticsPathConcern::DEFAULT,
            None,
            0,
        )
        .unwrap();
        assert_eq!(r.path.len(), 5);
        let mut tile = |_x: i32, _y: i32| LogisticsTile::default();
        let motion = r.smoothed_for_motion(5, 1, &mut tile);
        assert_eq!(motion, vec![(0, 0), (4, 0)]);
        assert!(r.total_cost.is_finite());
    }
}
