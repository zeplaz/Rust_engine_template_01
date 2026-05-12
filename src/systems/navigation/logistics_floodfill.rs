//! Grid **logistics pathfinding** — Dijkstra on a rectangular cell field (flood-fill style expansion),
//! parallel to potential-field navigation in `potental_feild_nav`.
//!
//! Each step uses average `cost_mul` between cells and tracks **max environmental risk** along the path
//! (fire exposure, stuck risk, chunk-level risk proxies). Callers map terrain/mobility + ecology/fire
//! into [`LogisticsTile`] via [`logistics_tile_from_mobility_and_env`] or custom closures.

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

fn better_candidate(new_dist: f32, new_risk: f32, old_dist: f32, old_risk: f32) -> bool {
    const EPS: f32 = 1e-4;
    if new_dist + EPS < old_dist {
        return true;
    }
    (new_dist - old_dist).abs() <= EPS && new_risk + EPS < old_risk
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DijkstraState {
    cost: f32,
    risk: f32,
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
        self.cost
            .total_cmp(&other.cost)
            .then_with(|| self.risk.total_cmp(&other.risk))
            .then_with(|| self.x.cmp(&other.x))
            .then_with(|| self.y.cmp(&other.y))
    }
}

/// Dijkstra shortest path on a dense `width × height` grid, 4-neighbor edges.
///
/// Edge weight from `a` to `b`: `0.5 * (tile(a).cost_mul + tile(b).cost_mul)` (both ends must be non-blocked).
/// Path **max_env_risk** is the maximum of `env_risk` on every entered cell (including start and goal).
pub fn logistics_path_dijkstra(
    width: i32,
    height: i32,
    start: (i32, i32),
    goal: (i32, i32),
    mut tile_at: impl FnMut(i32, i32) -> LogisticsTile,
) -> Option<LogisticsPathResult> {
    if width <= 0 || height <= 0 {
        return None;
    }

    let start_t = tile_or_blocked(width, height, start.0, start.1, &mut tile_at);
    let goal_t = tile_or_blocked(width, height, goal.0, goal.1, &mut tile_at);
    if start_t.blocked || goal_t.blocked {
        return None;
    }

    let n = (width * height) as usize;
    let mut dist = vec![f32::INFINITY; n];
    let mut path_risk = vec![0.0_f32; n];
    let mut prev: Vec<Option<(i32, i32)>> = vec![None; n];

    let si = idx(start.0, start.1, width);
    dist[si] = 0.0;
    path_risk[si] = start_t.env_risk.clamp(0.0, 1.0);

    let mut heap: BinaryHeap<Reverse<DijkstraState>> = BinaryHeap::new();
    heap.push(Reverse(DijkstraState {
        cost: 0.0,
        risk: path_risk[si],
        x: start.0,
        y: start.1,
    }));

    static DX: [i32; 4] = [1, -1, 0, 0];
    static DY: [i32; 4] = [0, 0, 1, -1];

    const EPS: f32 = 1e-4;
    while let Some(Reverse(DijkstraState { cost: d, risk: r, x, y })) = heap.pop() {
        let i = idx(x, y, width);
        if d > dist[i] + EPS {
            continue;
        }
        if (d - dist[i]).abs() <= EPS && r > path_risk[i] + EPS {
            continue;
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
            let edge = 0.5 * (ta.cost_mul.max(1e-3) + tb.cost_mul.max(1e-3));
            let nd = d + edge;
            let nr = r.max(ta.env_risk).max(tb.env_risk).clamp(0.0, 1.0);
            let ni = idx(nx, ny, width);
            if better_candidate(nd, nr, dist[ni], path_risk[ni]) {
                dist[ni] = nd;
                path_risk[ni] = nr;
                prev[ni] = Some((x, y));
                heap.push(Reverse(DijkstraState {
                    cost: nd,
                    risk: nr,
                    x: nx,
                    y: ny,
                }));
            }
        }
    }

    let gi = idx(goal.0, goal.1, width);
    if !dist[gi].is_finite() {
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
        cur = prev[pi];
    }
    if path.last().map(|p| (p.0, p.1)) != Some(start) {
        return None;
    }
    path.reverse();

    Some(LogisticsPathResult {
        path,
        total_cost: dist[gi],
        max_env_risk: path_risk[gi],
    })
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

    #[test]
    fn straight_path_open_grid() {
        let mut g = constant_grid(5, 5, LogisticsTile::default());
        let r = logistics_path_dijkstra(5, 5, (0, 0), (4, 0), |x, y| g(x, y)).unwrap();
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
        let r = logistics_path_dijkstra(5, 5, (0, 0), (4, 4), |x, y| tiles[y as usize][x as usize]).unwrap();
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
        let r = logistics_path_dijkstra(3, 3, (0, 1), (2, 1), |x, y| tiles[y as usize][x as usize]).unwrap();
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
