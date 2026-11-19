//! **Control relaxation** (PLAN-FRONT-COMBAT-v1 FC-2): per-cell faction control evolves over sim time
//! so fronts move.
//!
//! Per cell and slot, with `presence` = `Zone{Control}` falloff, `sf = floor + (1-floor)·supply`:
//!
//! ```text
//! dc/dt =  gain · presence · sf                       (held ground is re-asserted)
//!        + spread · sf · max(0, max_nb(c) − c) · (1 − hostile_here)   (an advance creeps forward)
//!        − decay · c · (1 − presence)                 (abandoned ground fades)
//!        − pressure · c · F_hostile                  (hostile force here or next door erodes it)
//!        + transfer · (that loss, credited to the strongest hostile)   (conquest: ground changes hands)
//! ```
//!
//! Pure and deterministic ([`relax_step`], `BTreeMap` order, double-buffered). The Bevy wrapper
//! [`control_relaxation_system`] is the only writer of `faction_control` from `Zone{Control}` once
//! [`ControlRelaxation`] exists (the zone pass skips Control zones then; without the resource the
//! legacy additive latch is unchanged).

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;

use super::balance::FrontBalance;
use super::front::{Hostility, SlotControl};
use super::front::hostility_from_stances;
use crate::strategic::settlement::FactionStanceBook;
use crate::strategic::{ChunkStrategicOverlay, Zone, ZoneKind, MAX_STRATEGIC_FACTION_SLOTS};
use crate::systems::sim_control::SimTimeMicros;

const SLOTS: usize = MAX_STRATEGIC_FACTION_SLOTS;
const NEIGHBORS4: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
/// Share of fighting force a slot keeps with no troops present (its control alone still resists).
const FORCE_BASE: f32 = 0.25;
const SNAP_ZERO: f32 = 1e-3;
/// Most sub-steps one relaxation pass may run (bounds cost after a big sim-time jump).
const MAX_SUBSTEPS: usize = 16;

/// Tunables of one relaxation run (from the balance table).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RelaxParams {
    pub decay_rate: f32,
    pub gain_per_s: f32,
    pub pressure_per_s: f32,
    pub spread_per_s: f32,
    pub supply_floor: f32,
    pub conquest_transfer: f32,
    pub max_substep_s: f32,
}

impl RelaxParams {
    #[must_use]
    pub fn from_balance(b: &FrontBalance) -> Self {
        Self {
            decay_rate: b.control_decay_rate(),
            gain_per_s: b.control.stamp_gain_per_s,
            pressure_per_s: b.control.pressure_per_s,
            spread_per_s: b.control.spread_per_s,
            supply_floor: b.control.supply_floor,
            conquest_transfer: b.control.conquest_transfer,
            max_substep_s: b.control.relax_max_substep_s.max(1e-3),
        }
    }
}

/// One tile's inputs and state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RelaxCell {
    pub control: SlotControl,
    /// `Zone{Control}` presence per slot (0..1).
    pub presence: SlotControl,
    /// `logistics_strength` per slot (0..1).
    pub supply: SlotControl,
}

pub type RelaxGrid = BTreeMap<(i32, i32), RelaxCell>;

/// Fighting force of one slot on one tile: control backed by present, supplied troops.
fn force(c: &RelaxCell, slot: usize, supply_floor: f32) -> f32 {
    let sf = supply_floor + (1.0 - supply_floor) * c.supply[slot].clamp(0.0, 1.0);
    c.control[slot] * (FORCE_BASE + (1.0 - FORCE_BASE) * c.presence[slot].clamp(0.0, 1.0)) * sf
}

/// Live-lane mask of one slot on one tile (CONC-08 · gea4v1 4.10.7 vector predication; LEGACY
/// `simd` gea_v1_e4 p324). A dead lane (no control, presence or friendly neighbour) must come out
/// of the step bit-identical and credit no conquest: the update is zero there, so the scalar
/// early-out and the fully predicated (compute-then-select) form agree. A lane batch may drop the
/// branch; `conc08_predicated_step_matches_early_out` locks the contract.
#[inline]
fn slot_live(c: f32, presence: f32, nb_c: f32) -> bool {
    !(c <= 0.0 && presence <= 0.0 && nb_c <= 0.0)
}

#[cfg(test)]
fn step_once(grid: &mut RelaxGrid, hostility: &Hostility, p: &RelaxParams, dt: f32) {
    step_once_lanes::<true>(grid, hostility, p, dt);
}

/// `SKIP_DEAD = true` is the production early-out; `false` evaluates every lane and selects by
/// [`slot_live`] (the SIMD-shaped form, used by the equivalence tripwire).
/// Dense, double-buffered view of a [`RelaxGrid`] over its bounding box (`None` = no tile).
/// The relaxation runs here: neighbours are index arithmetic and a sub-step copies one flat
/// buffer, instead of cloning a `BTreeMap` and doing four tree lookups per cell per sub-step
/// (that cost 60-190 ms per pass on a 100k-cell valley at high sim speed).
struct DenseRelax {
    x0: i32,
    y0: i32,
    w: usize,
    h: usize,
    cells: Vec<Option<RelaxCell>>,
}

impl DenseRelax {
    fn from_grid(grid: &RelaxGrid) -> Self {
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for &(x, y) in grid.keys() {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        let w = (x1 - x0 + 1).max(0) as usize;
        let h = (y1 - y0 + 1).max(0) as usize;
        let mut cells = vec![None; w * h];
        for (&(x, y), c) in grid {
            cells[(y - y0) as usize * w + (x - x0) as usize] = Some(*c);
        }
        Self { x0, y0, w, h, cells }
    }

    fn write_back(&self, grid: &mut RelaxGrid) {
        for (&(x, y), c) in grid.iter_mut() {
            if let Some(d) = self.cells[(y - self.y0) as usize * self.w + (x - self.x0) as usize] {
                *c = d;
            }
        }
    }

    fn step<const SKIP_DEAD: bool>(&mut self, hostility: &Hostility, p: &RelaxParams, dt: f32) {
        let old = self.cells.clone();
        let (w, h) = (self.w as i32, self.h as i32);
        for yi in 0..h {
            for xi in 0..w {
                let idx = (yi * w + xi) as usize;
                let Some(me) = old[idx] else { continue };
                let mut nb = [None; 4];
                for (k, (dx, dy)) in NEIGHBORS4.iter().enumerate() {
                    let (nx, ny) = (xi + dx, yi + dy);
                    if nx >= 0 && ny >= 0 && nx < w && ny < h {
                        nb[k] = old[(ny * w + nx) as usize];
                    }
                }
                let out = relax_cell::<SKIP_DEAD>(&me, nb.iter().flatten(), hostility, p, dt);
                if let Some(c) = self.cells[idx].as_mut() {
                    c.control = out;
                }
            }
        }
    }
}

/// One tile's next control from its own state and its present 4-neighbours (unchanged maths).
fn relax_cell<'a, const SKIP_DEAD: bool>(
    me: &RelaxCell,
    near: impl Iterator<Item = &'a RelaxCell> + Clone,
    hostility: &Hostility,
    p: &RelaxParams,
    dt: f32,
) -> SlotControl {
    // Strongest friendly control next door (what an advance spills from) and fighting force
    // here or next door (what a hostile pushes with).
    let mut nb_c = [0.0f32; SLOTS];
    let mut push = [0.0f32; SLOTS];
    for s in 0..SLOTS {
        push[s] = force(me, s, p.supply_floor);
    }
    for n in near {
        for s in 0..SLOTS {
            nb_c[s] = nb_c[s].max(n.control[s]);
            push[s] = push[s].max(force(n, s, p.supply_floor));
        }
    }
    let mut conquest = [0.0f32; SLOTS];
    let mut next = me.control;
    for s in 0..SLOTS {
        let c = me.control[s];
        let presence = me.presence[s].clamp(0.0, 1.0);
        let live = slot_live(c, presence, nb_c[s]);
        if SKIP_DEAD && !live {
            continue;
        }
        // Strongest hostile pushing on this slot, and whether it is also in the cell itself.
        let mut top: Option<(usize, f32)> = None;
        let mut hostile_here = 0.0f32;
        for hh in 0..SLOTS {
            if hh != s && hostility.hostile(s as u8, hh as u8) {
                hostile_here = hostile_here.max(me.control[hh]);
                if push[hh] > 0.0 && top.is_none_or(|(_, best)| push[hh] > best) {
                    top = Some((hh, push[hh]));
                }
            }
        }
        // Dead lane: drop the pressure term so nothing is credited (select, not branch).
        if !live {
            top = None;
        }
        let sf = p.supply_floor + (1.0 - p.supply_floor) * me.supply[s].clamp(0.0, 1.0);
        let gain = p.gain_per_s * presence * sf;
        let spill = p.spread_per_s * sf * (nb_c[s] - c).max(0.0) * (1.0 - hostile_here.min(1.0));
        let decay = p.decay_rate * c * (1.0 - presence);
        let loss = top.map_or(0.0, |(_, f)| p.pressure_per_s * c * f);
        if let Some((hh, _)) = top {
            conquest[hh] += p.conquest_transfer * loss;
        }
        let updated = c + dt * (gain + spill - decay - loss);
        next[s] = if live { updated } else { c };
    }
    let mut out = [0.0f32; SLOTS];
    for s in 0..SLOTS {
        let v = (next[s] + dt * conquest[s]).clamp(0.0, 1.0);
        out[s] = if v < SNAP_ZERO { 0.0 } else { v };
    }
    out
}

#[cfg(test)]
fn step_once_lanes<const SKIP_DEAD: bool>(grid: &mut RelaxGrid, hostility: &Hostility, p: &RelaxParams, dt: f32) {
    if grid.is_empty() {
        return;
    }
    let mut dense = DenseRelax::from_grid(grid);
    dense.step::<SKIP_DEAD>(hostility, p, dt);
    dense.write_back(grid);
}

/// Integrate `dt_s` sim seconds (split into sub-steps ≤ `max_substep_s`, at most [`MAX_SUBSTEPS`]).
pub fn relax_step(grid: &mut RelaxGrid, hostility: &Hostility, params: &RelaxParams, dt_s: f32) {
    if dt_s <= 0.0 || grid.is_empty() {
        return;
    }
    let n = ((dt_s / params.max_substep_s).ceil() as usize).clamp(1, MAX_SUBSTEPS);
    let sub = dt_s / n as f32;
    // Densify once per pass; sub-steps double-buffer the flat grid.
    let mut dense = DenseRelax::from_grid(grid);
    for _ in 0..n {
        dense.step::<true>(hostility, params, sub);
    }
    dense.write_back(grid);
}

/// Cadence state of the relaxation pass. Its presence switches the zone pass off for Control zones.
#[derive(Resource, Clone, Debug, Default)]
pub struct ControlRelaxation {
    last_micros: Option<u64>,
    accumulated_s: f32,
    /// Relaxation passes run so far.
    pub steps: u64,
    /// Sim seconds integrated by the last pass.
    pub last_dt_s: f32,
}

/// Projected presence of one `Zone{Control}` into `out` (same falloff as the zone pass).
fn add_zone_presence(z: &Zone, out: &mut BTreeMap<(i32, i32), SlotControl>) {
    let r = z.radius_tiles.max(0.0);
    if r <= 0.0 || z.intensity <= 0.0 {
        return;
    }
    let slot = (z.faction_slot as usize).min(SLOTS - 1);
    let (cx, cy) = (z.center_tile.x as f32, z.center_tile.y as f32);
    for ty in (cy - r).floor() as i32..=(cy + r).ceil() as i32 {
        for tx in (cx - r).floor() as i32..=(cx + r).ceil() as i32 {
            let d = ((tx as f32 + 0.5 - cx).powi(2) + (ty as f32 + 0.5 - cy).powi(2)).sqrt();
            if d >= r {
                continue;
            }
            let w = z.intensity * (1.0 - d / r);
            let e = out.entry((tx, ty)).or_insert([0.0; SLOTS]);
            e[slot] = (e[slot] + w).min(1.0);
        }
    }
}

/// Advance faction control by elapsed sim time (cadence: `control.relax_step_s`).
pub fn control_relaxation_system(
    balance: Res<FrontBalance>,
    stances: Option<Res<FactionStanceBook>>,
    time: Option<Res<SimTimeMicros>>,
    mut state: ResMut<ControlRelaxation>,
    zones: Query<&Zone>,
    mut overlays: Query<&mut ChunkStrategicOverlay>,
) {
    let _perf = crate::render::PerfScope::new("upd_control_relax");
    let Some(time) = time else { return };
    let now = time.0;
    let dt = match state.last_micros {
        Some(prev) if now >= prev => (now - prev) as f32 / 1_000_000.0,
        _ => 0.0,
    };
    state.last_micros = Some(now);
    state.accumulated_s += dt;
    if state.accumulated_s < balance.control.relax_step_s {
        return;
    }
    let dt_s = std::mem::take(&mut state.accumulated_s);

    let mut presence: BTreeMap<(i32, i32), SlotControl> = BTreeMap::new();
    for z in zones.iter().filter(|z| z.kind == ZoneKind::Control) {
        add_zone_presence(z, &mut presence);
    }

    // Active chunks: any control, or presence inside, dilated by one chunk (so spread can cross borders).
    let mut live: BTreeSet<(i32, i32)> = BTreeSet::new();
    let origin_of = |ov: &ChunkStrategicOverlay| {
        (
            ov.chunk_coord.x.saturating_mul(ov.size.x as i32),
            ov.chunk_coord.y.saturating_mul(ov.size.y as i32),
        )
    };
    for ov in overlays.iter() {
        let (ox, oy) = origin_of(&ov);
        let has_control = ov.faction_control.iter().any(|c| c.iter().any(|v| *v > 0.0));
        let has_presence = (0..ov.size.y as i32).any(|ly| {
            (0..ov.size.x as i32).any(|lx| presence.contains_key(&(ox + lx, oy + ly)))
        });
        if has_control || has_presence {
            live.insert((ov.chunk_coord.x, ov.chunk_coord.y));
        }
    }
    let mut active: BTreeSet<(i32, i32)> = BTreeSet::new();
    for (x, y) in &live {
        for dy in -1..=1 {
            for dx in -1..=1 {
                active.insert((x + dx, y + dy));
            }
        }
    }

    let mut grid = RelaxGrid::new();
    for ov in overlays.iter() {
        if !active.contains(&(ov.chunk_coord.x, ov.chunk_coord.y)) {
            continue;
        }
        let (ox, oy) = origin_of(&ov);
        let w = ov.size.x.max(1) as usize;
        for ci in 0..ov.len_cells() {
            let t = (ox + (ci % w) as i32, oy + (ci / w) as i32);
            grid.insert(
                t,
                RelaxCell {
                    control: ov.faction_control[ci],
                    presence: presence.get(&t).copied().unwrap_or([0.0; SLOTS]),
                    supply: ov.logistics_strength[ci],
                },
            );
        }
    }
    if grid.is_empty() {
        return;
    }
    let hostility = hostility_from_stances(stances.as_deref());
    relax_step(&mut grid, &hostility, &RelaxParams::from_balance(&balance), dt_s);

    for mut ov in overlays.iter_mut() {
        if !active.contains(&(ov.chunk_coord.x, ov.chunk_coord.y)) {
            continue;
        }
        let (ox, oy) = origin_of(&ov);
        let w = ov.size.x.max(1) as usize;
        for ci in 0..ov.len_cells() {
            if let Some(c) = grid.get(&(ox + (ci % w) as i32, oy + (ci / w) as i32)) {
                if ov.faction_control[ci] != c.control {
                    ov.faction_control[ci] = c.control;
                }
            }
        }
    }
    state.steps += 1;
    state.last_dt_s = dt_s;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> RelaxParams {
        RelaxParams::from_balance(&FrontBalance::default())
    }

    /// 1-D strip of `n` tiles: slot 0 on x < split (presence `p0`), slot 1 on x >= split (presence `p1`).
    fn strip(n: i32, split: i32, p0: f32, p1: f32, supply1: f32) -> RelaxGrid {
        let mut g = RelaxGrid::new();
        for x in 0..n {
            let mut cell = RelaxCell::default();
            let s = usize::from(x >= split);
            cell.control[s] = 0.8;
            cell.presence[s] = if s == 0 { p0 } else { p1 };
            cell.supply[0] = 1.0;
            cell.supply[1] = supply1;
            g.insert((x, 0), cell);
        }
        g
    }

    fn boundary(g: &RelaxGrid) -> i32 {
        g.iter().filter(|(_, c)| c.control[0] > c.control[1] && c.control[0] >= 0.25).count() as i32
    }

    #[test]
    fn front_advances_under_one_sided_pressure() {
        let h = Hostility::legacy_pair();
        let mut g = strip(12, 6, 1.0, 0.25, 1.0);
        let start = boundary(&g);
        for _ in 0..600 {
            relax_step(&mut g, &h, &params(), 1.0);
        }
        assert!(boundary(&g) > start, "slot 0 cells {} -> {}", start, boundary(&g));
    }

    #[test]
    fn balanced_pressure_holds_the_line() {
        let h = Hostility::legacy_pair();
        let mut g = strip(12, 6, 1.0, 1.0, 1.0);
        let start = boundary(&g);
        for _ in 0..600 {
            relax_step(&mut g, &h, &params(), 1.0);
        }
        assert_eq!(boundary(&g), start);
    }

    #[test]
    fn starved_side_loses_ground() {
        let h = Hostility::legacy_pair();
        let mut g = strip(12, 6, 1.0, 1.0, 0.0);
        let start = boundary(&g);
        for _ in 0..900 {
            relax_step(&mut g, &h, &params(), 1.0);
        }
        assert!(boundary(&g) > start);
    }

    #[test]
    fn abandoned_ground_decays_with_half_life() {
        let h = Hostility::legacy_pair();
        let mut g = RelaxGrid::new();
        let mut c = RelaxCell::default();
        c.control[0] = 0.8;
        g.insert((0, 0), c);
        let b = FrontBalance::default();
        let mut t = 0.0;
        while t < b.control.half_life_s {
            relax_step(&mut g, &h, &params(), 1.0);
            t += 1.0;
        }
        assert!((g[&(0, 0)].control[0] - 0.4).abs() < 0.02, "{}", g[&(0, 0)].control[0]);
    }

    #[test]
    fn relaxation_is_deterministic_and_substep_stable() {
        let h = Hostility::legacy_pair();
        let mut a = strip(12, 6, 1.0, 0.4, 0.7);
        let mut b = a.clone();
        relax_step(&mut a, &h, &params(), 8.0);
        relax_step(&mut b, &h, &params(), 8.0);
        assert_eq!(a, b);
        assert!(a.values().all(|c| c.control.iter().all(|v| (0.0..=1.0).contains(v))));
    }

    /// CONC-08 tripwire: the fully predicated step (every lane computed, selected by `slot_live`)
    /// is bit-identical to the production early-out over a sparse hashed grid, multi-step.
    #[test]
    fn conc08_predicated_step_matches_early_out() {
        let h = Hostility::legacy_pair();
        let p = params();
        let mut g = RelaxGrid::new();
        let mut dead_slots = 0usize;
        for y in 0..9 {
            for x in 0..9 {
                let mut cell = RelaxCell::default();
                for s in 0..SLOTS {
                    let r = |salt: u32| {
                        let mut n = (x as u32).wrapping_mul(374_761_393)
                            ^ (y as u32).wrapping_mul(668_265_263)
                            ^ (s as u32 + 1).wrapping_mul(0x9E37_79B9)
                            ^ salt;
                        n = (n ^ (n >> 15)).wrapping_mul(0x85EB_CA6B);
                        n ^= n >> 13;
                        (n >> 8) as f32 / (1u32 << 24) as f32
                    };
                    // ~70% of lanes empty so dead lanes sit next to live ones.
                    if r(1) < 0.3 {
                        cell.control[s] = r(2);
                        cell.presence[s] = r(3);
                    }
                    cell.supply[s] = r(4);
                    if cell.control[s] <= 0.0 && cell.presence[s] <= 0.0 {
                        dead_slots += 1;
                    }
                }
                g.insert((x, y), cell);
            }
        }
        assert!(dead_slots > 0, "fixture must contain dead lanes");
        let (mut a, mut b) = (g.clone(), g);
        for _ in 0..24 {
            step_once_lanes::<true>(&mut a, &h, &p, 0.5);
            step_once_lanes::<false>(&mut b, &h, &p, 0.5);
            assert_eq!(a, b, "predicated lanes diverged from early-out");
        }
        // Mask contract: a lane with no control, presence or friendly neighbour stays exactly 0.
        let mut lone = RelaxGrid::new();
        lone.insert((0, 0), RelaxCell::default());
        step_once_lanes::<false>(&mut lone, &h, &p, 1.0);
        assert_eq!(lone[&(0, 0)], RelaxCell::default());
        assert!(!slot_live(0.0, 0.0, 0.0) && slot_live(0.0, 0.0, 0.1));
    }

    #[test]
    fn system_advances_overlay_control_over_sim_time() {
        let mut app = App::new();
        app.insert_resource(FrontBalance::default())
            .insert_resource(SimTimeMicros(0))
            .init_resource::<ControlRelaxation>()
            .add_systems(Update, control_relaxation_system);
        let mut ov = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(8, 1));
        ov.faction_control[0][0] = 0.8;
        let e = app.world_mut().spawn(ov).id();
        app.update();
        app.world_mut().resource_mut::<SimTimeMicros>().0 = 60_000_000;
        app.update();
        let ov = app.world().entity(e).get::<ChunkStrategicOverlay>().unwrap();
        assert!(ov.faction_control[0][0] < 0.8, "unheld control fades");
        assert!(ov.faction_control[1][0] > 0.0, "control spills into the neighbour");
        assert_eq!(app.world().resource::<ControlRelaxation>().steps, 1);
    }
}
