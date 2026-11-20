//! Active-tile virtual-pipes kernel (WM-1): the one explicit step of [`SurfaceWaterField`].
//!
//! Only tiles in [`WaterTileGrid::active`] are touched. A face is solved only when both of its
//! cells sit in active tiles, so a sleeping tile is never written and volume is conserved exactly.
//! The flux / limiter buffers are reused across sub-steps (no allocation per sub-step).

use super::dam_faces::{Pipe, NO_DONOR};
use super::surface_water::{SurfaceWaterField, G_MPS2};
use super::tiles::{WaterTileGrid, EDGE_DRY_M};

/// Reused per-sub-step buffers.
#[derive(Clone, Debug, Default)]
pub struct StepScratch {
    flux_x: Vec<f32>,
    flux_y: Vec<f32>,
    scale: Vec<f32>,
    /// Dam-face pipes of the sub-step ([`super::dam_faces`]).
    pipes: Vec<Pipe>,
    /// Pipe discharge leaving each donor cell (zero outside a sub-step).
    pipe_out: Vec<f32>,
}

/// Calibrated so dam-break front celerity matches Ritter 2*sqrt(g*h) within +-10%
/// (see `measure_dam_break_front_distance`).
pub const VP_CONDUCTANCE_K: f32 = 16.0;

/// The sim's base solver step (s): one sub-step of a 1x sim second.
pub const NOMINAL_SUBSTEP_S: f32 = 1.0;

/// The face conductance (m2/s) the kernel gives water of depth `h_m` at the nominal sub-step: the
/// physical `K dx sqrt(g h)` capped by the stability bound `A / (8 dt)`, with `dt` the nominal step
/// or the gravity-wave CFL (`dx / (2 sqrt(g h))`) when that is shorter. What a river's channel
/// conveys per metre of surface fall, so its baseflow keeps it at its baseline depth.
#[must_use]
pub fn nominal_conductance(dx_m: f32, h_m: f32) -> f32 {
    let c = (G_MPS2 * h_m.max(0.0)).sqrt();
    if c <= 0.0 {
        return 0.0;
    }
    let dt = NOMINAL_SUBSTEP_S.min(0.5 * dx_m / c);
    (VP_CONDUCTANCE_K * dx_m * c).min(dx_m * dx_m / (8.0 * dt))
}

/// Does the edge of tile `t` facing `dir` (0=E, 1=W, 2=N(+z), 3=S) hold any water?
fn edge_wet(tg: &WaterTileGrid, depth: &[f32], w: usize, t: usize, dir: u8) -> bool {
    let (x0, z0, x1, z1) = tg.tile_rect(t);
    match dir {
        0 => (z0..z1).any(|z| depth[z * w + x1 - 1] > EDGE_DRY_M),
        1 => (z0..z1).any(|z| depth[z * w + x0] > EDGE_DRY_M),
        2 => depth[(z1 - 1) * w + x0..(z1 - 1) * w + x1].iter().any(|d| *d > EDGE_DRY_M),
        _ => depth[z0 * w + x0..z0 * w + x1].iter().any(|d| *d > EDGE_DRY_M),
    }
}

impl SurfaceWaterField {
    /// Pick this step's active tiles: awake tiles, tiles with boundary activity, and the neighbours
    /// water can actually cross into (a wet cell on either side of the shared edge).
    pub fn refresh_active_tiles(&mut self) {
        if !self.wired || self.width == 0 || self.height == 0 {
            return;
        }
        let (w, h) = (self.width as usize, self.height as usize);
        if self.depth_m.len() != w * h {
            return;
        }
        let mut tg = std::mem::take(&mut self.tiles);
        if tg.tile_count() == 0 {
            tg = WaterTileGrid::new(self.width, self.height, self.origin_x, self.origin_z);
        }
        tg.begin_refresh();
        if self.inflow_m3s > 0.0 {
            for k in 0..tg.inflow_tiles.len() {
                let t = tg.inflow_tiles[k] as usize;
                tg.add_base(t);
            }
        }
        // Open boundary: an edge cell standing above its baseline drains, so its tile must run.
        let draining = |i: usize| {
            self.depth_m[i] > self.ref_depth_m.get(i).copied().unwrap_or(0.0) + 1e-3
                && self.river.rate_at(i) <= 0.0
        };
        for x in 0..w {
            for z in [0, h - 1] {
                if draining(z * w + x) {
                    let t = tg.tile_of(x as u32, z as u32);
                    tg.add_base(t);
                }
            }
        }
        for z in 0..h {
            for x in [0, w - 1] {
                if draining(z * w + x) {
                    let t = tg.tile_of(x as u32, z as u32);
                    tg.add_base(t);
                }
            }
        }
        // A dam passing water wakes the tiles on both sides of its faces.
        self.wake_dam_tiles(&mut tg);
        let base = tg.base_tiles();
        let (nx, nz) = (tg.nx as usize, tg.nz as usize);
        for &t in &base {
            let t = t as usize;
            let (tx, tz) = (t % nx, t / nx);
            let consider = |tg: &mut WaterTileGrid, n: usize, dir: u8, back: u8| {
                if !tg.is_active(n)
                    && (edge_wet(tg, &self.depth_m, w, t, dir) || edge_wet(tg, &self.depth_m, w, n, back))
                {
                    tg.add_halo(n);
                }
            };
            if tx + 1 < nx {
                consider(&mut tg, t + 1, 0, 1);
            }
            if tx > 0 {
                consider(&mut tg, t - 1, 1, 0);
            }
            if tz + 1 < nz {
                consider(&mut tg, t + nx, 2, 3);
            }
            if tz > 0 {
                consider(&mut tg, t - nx, 3, 2);
            }
        }
        tg.seal_active();
        self.tiles = tg;
    }

    /// Deepest water over the cells of the active tiles (CFL input; 0 when nothing is active).
    #[must_use]
    pub fn active_depth_max(&self) -> f32 {
        let w = self.width as usize;
        let mut m = 0.0_f32;
        for &t in &self.tiles.active {
            let (x0, z0, x1, z1) = self.tiles.tile_rect(t as usize);
            for z in z0..z1 {
                for d in &self.depth_m[z * w + x0..z * w + x1] {
                    m = m.max(*d);
                }
            }
        }
        m
    }

    /// One virtual-pipes sub-step over the active tiles. Conservative volume update; returns the
    /// boundary volume actually applied (inflow - open-edge drain, m3).
    pub fn step_active_substep(&mut self, dt_s: f32) -> f64 {
        if !self.wired || self.width < 2 || self.height < 2 || dt_s <= 0.0 {
            return 0.0;
        }
        let (w, h) = (self.width as usize, self.height as usize);
        let n = w * h;
        if self.depth_m.len() != n || self.tiles.active.is_empty() {
            return 0.0;
        }
        let dx = self.dx_m;
        let cell_a = dx * dx;
        let mut sc = std::mem::take(&mut self.scratch);
        if sc.flux_x.len() != n {
            sc.flux_x = vec![0.0; n];
            sc.flux_y = vec![0.0; n];
            sc.scale = vec![1.0; n];
        }
        if sc.pipe_out.len() != n {
            sc.pipe_out = vec![0.0; n];
        }
        let mut tg = std::mem::take(&mut self.tiles);
        let (nx, nz) = (tg.nx as usize, tg.nz as usize);

        // Stability bound: flux = C*d(eta) is explicit diffusion, stable only while C*dt/A <= 1/4
        // per face. Above it (deep reservoir) cells overshoot their neighbours and blow up into an
        // odd-even checkerboard. Capping each face at A/(8*dt) means a cell can never pass a
        // neighbour in one sub-step; when the sub-step is small enough the physical conductance is
        // used unchanged.
        let c_stable = cell_a / (8.0 * dt_s);
        let conductance = |h_a: f32, h_b: f32| -> f32 {
            // Upwind depth so dry-bed wetting is not starved by a zero neighbour.
            let hp = h_a.max(h_b);
            if hp < 1e-4 {
                0.0
            } else {
                // Gravity-wave scale: C = k*dx*sqrt(g*h) -> volume flux = C*d(eta).
                (VP_CONDUCTANCE_K * dx * (G_MPS2 * hp).sqrt()).min(c_stable)
            }
        };

        // Pass 1: face fluxes. CLOSE-W4: only the donor (higher surface) side gives water and the
        // driving head is capped at the donor's depth, so a dry cell gives nothing and a thin film
        // over a terrain step can't push the whole bed drop.
        {
            let face = |i: usize, j: usize| -> f32 {
                let (h_i, h_j) = (self.depth_m[i], self.depth_m[j]);
                let raw_i = self.bed_m[i] + h_i;
                let raw_j = self.bed_m[j] + h_j;
                let crest = self.wall_top_m(i).max(self.wall_top_m(j));
                let (eta_i, eta_j) = if crest > 0.0 {
                    // Barrier blocks unless stage above crest; effective surfaces clip to the crest.
                    if raw_i.max(raw_j) <= crest {
                        return 0.0;
                    }
                    (raw_i.max(crest), raw_j.max(crest))
                } else {
                    (raw_i, raw_j)
                };
                let f = conductance(h_i, h_j) * (eta_i - eta_j);
                let drop = (raw_i - raw_j).abs();
                let donor_h = if f >= 0.0 { h_i } else { h_j };
                if donor_h < 1e-4 || drop <= 0.0 {
                    0.0
                } else {
                    f * (donor_h / drop).min(1.0)
                }
            };
            for k in 0..tg.active.len() {
                let t = tg.active[k] as usize;
                let (x0, z0, x1, z1) = tg.tile_rect(t);
                let (tx, tz) = (t % nx, t / nx);
                let east_ok = tx + 1 < nx && tg.is_active(t + 1);
                let north_ok = tz + 1 < nz && tg.is_active(t + nx);
                let west_on = tx > 0 && tg.is_active(t - 1);
                let south_on = tz > 0 && tg.is_active(t - nx);
                for z in z0..z1 {
                    for x in x0..x1 {
                        let i = z * w + x;
                        if x + 1 < w {
                            sc.flux_x[i] = if x + 1 < x1 || east_ok { face(i, i + 1) } else { 0.0 };
                        }
                        if z + 1 < h {
                            sc.flux_y[i] = if z + 1 < z1 || north_ok { face(i, i + w) } else { 0.0 };
                        }
                        // Faces owned by a sleeping neighbour tile carry nothing.
                        if x == x0 && x > 0 && !west_on {
                            sc.flux_x[i - 1] = 0.0;
                        }
                        if z == z0 && z > 0 && !south_on {
                            sc.flux_y[i - w] = 0.0;
                        }
                    }
                }
            }
        }

        // Dam faces: outlets, spillways and turbines pass water from the cell above the dam to the
        // cell below it by the local head. They draw on the same donor budget as the face fluxes.
        sc.pipes.clear();
        self.collect_dam_pipes(dt_s, &tg, self.far_release.as_ref(), &mut sc.pipes);
        for p in &sc.pipes {
            if p.from != NO_DONOR {
                sc.pipe_out[p.from] += p.q;
            }
        }

        // Pass 2: outflow limiter — each cell can't give more than it holds.
        for k in 0..tg.active.len() {
            let (x0, z0, x1, z1) = tg.tile_rect(tg.active[k] as usize);
            for z in z0..z1 {
                for x in x0..x1 {
                    let i = z * w + x;
                    let mut out = sc.flux_x[i].max(0.0) + sc.flux_y[i].max(0.0) + sc.pipe_out[i];
                    if x > 0 {
                        out += (-sc.flux_x[i - 1]).max(0.0);
                    }
                    if z > 0 {
                        out += (-sc.flux_y[i - w]).max(0.0);
                    }
                    let vol = self.depth_m[i] * cell_a;
                    let want = out * dt_s;
                    sc.scale[i] = if want > vol && want > 0.0 { vol / want } else { 1.0 };
                }
            }
        }

        // Pass 3: apply the limiter to the faces (donor's scale), track per-tile peak flux.
        for k in 0..tg.active.len() {
            let t = tg.active[k] as usize;
            let (x0, z0, x1, z1) = tg.tile_rect(t);
            let mut peak = 0.0_f32;
            for z in z0..z1 {
                for x in x0..x1 {
                    let i = z * w + x;
                    if x + 1 < w {
                        let f = sc.flux_x[i];
                        if f != 0.0 {
                            let donor = if f >= 0.0 { i } else { i + 1 };
                            let f = f * sc.scale[donor];
                            sc.flux_x[i] = f;
                            peak = peak.max(f.abs());
                        }
                    }
                    if z + 1 < h {
                        let f = sc.flux_y[i];
                        if f != 0.0 {
                            let donor = if f >= 0.0 { i } else { i + w };
                            let f = f * sc.scale[donor];
                            sc.flux_y[i] = f;
                            peak = peak.max(f.abs());
                        }
                    }
                }
            }
            tg.max_flux[t] = tg.max_flux[t].max(peak);
        }

        // The pipes take the donor's limiter too; flow over each crest is read off the final fluxes.
        for p in &mut sc.pipes {
            if p.from != NO_DONOR {
                p.q *= sc.scale[p.from];
                sc.pipe_out[p.from] = 0.0;
            }
        }
        self.tally_overtop(&tg, &sc.flux_y, dt_s);

        // Pass 4: divergence + boundaries, depth updated in place (a cell reads only its own old
        // depth and the already-final face fluxes).
        let mut applied = 0.0_f64;
        for k in 0..tg.active.len() {
            let t = tg.active[k] as usize;
            let (x0, z0, x1, z1) = tg.tile_rect(t);
            let mut peak_rate = 0.0_f32;
            for z in z0..z1 {
                for x in x0..x1 {
                    let i = z * w + x;
                    let mut div = sc.flux_x[i] + sc.flux_y[i];
                    if x > 0 {
                        div -= sc.flux_x[i - 1];
                    }
                    if z > 0 {
                        div -= sc.flux_y[i - w];
                    }
                    let mut dvol = -div * dt_s;
                    // River source: the headwater cell's baseflow (WM-2, `sources.rs`).
                    let q_src = self.river.rate_at(i);
                    if q_src > 0.0 {
                        dvol += q_src * dt_s;
                        applied += f64::from(q_src * dt_s);
                    }
                    // Open map edge: an edge cell drains like a free overfall into a ghost cell that
                    // holds the baseline (static) depth, so water leaves only while the edge stands
                    // above the world's own water and hands over without a step.
                    if q_src <= 0.0 && (x == 0 || z == 0 || x == w - 1 || z == h - 1) {
                        let h0 = self.depth_m[i];
                        let h_ghost = self.ref_depth_m.get(i).copied().unwrap_or(0.0);
                        if h0 > h_ghost {
                            let q_free = conductance(h0, h_ghost) * (h0 - h_ghost);
                            let avail = (h0 * cell_a + dvol).max(0.0);
                            let sink = (q_free * dt_s).clamp(0.0, avail);
                            dvol -= sink;
                            applied -= f64::from(sink);
                        }
                    }
                    let dd = dvol / cell_a;
                    peak_rate = peak_rate.max(dd.abs() / dt_s);
                    let d = (self.depth_m[i] + dd).max(0.0);
                    self.depth_m[i] = d;
                    self.qx_m3s[i] = sc.flux_x[i];
                    self.qy_m3s[i] = sc.flux_y[i];
                    if d > 0.05 {
                        self.wet_since_s[i] += dt_s;
                    } else {
                        self.wet_since_s[i] = 0.0;
                    }
                }
            }
            tg.max_rate[t] = tg.max_rate[t].max(peak_rate);
        }

        // Apply the dam pipes (volume leaves the donor, arrives below the dam). A far-field source
        // has no donor: its volume is new to the field, so it counts as applied.
        for k in 0..sc.pipes.len() {
            let p = sc.pipes[k];
            let dvol = p.q * dt_s;
            if p.from != NO_DONOR {
                self.depth_m[p.from] = (self.depth_m[p.from] - dvol / cell_a).max(0.0);
            } else {
                applied += f64::from(dvol);
            }
            self.depth_m[p.to] += dvol / cell_a;
            for c in [p.from, p.to] {
                if c == NO_DONOR {
                    continue;
                }
                let t = tg.tile_of((c % w) as u32, (c / w) as u32);
                tg.max_rate[t] = tg.max_rate[t].max(p.q / cell_a);
                tg.max_flux[t] = tg.max_flux[t].max(p.q);
            }
            self.tally_pipe(&p, dt_s);
        }

        self.scratch = sc;
        self.tiles = tg;
        applied
    }

    /// One virtual-pipes sub-step over the whole field (wakes every tile first). For callers that
    /// drive the kernel directly (benchmarks, unit tests); the simulation uses [`Self::step`].
    pub fn step_virtual_pipes_sub(&mut self, dt_s: f32) -> f64 {
        if !self.wired {
            return 0.0;
        }
        self.tiles.touch_all();
        self.refresh_active_tiles();
        let applied = self.step_active_substep(dt_s);
        self.settle_active_tiles();
        applied
    }

    /// Fold the step's tile stats (calm tiles sleep) and queue the changed chunks as dirty.
    pub fn settle_active_tiles(&mut self) {
        self.tiles.settle();
        let mut tg = std::mem::take(&mut self.tiles);
        tg.flush_dirty_into(&mut self.dirty_chunks);
        self.tiles = tg;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::water::{SurfaceWaterMode, WATER_SLEEP_STEPS};
    use bevy::prelude::{Entity, IVec2};

    /// 128x128 map: dry ground at 30 m with a stone basin (x 30..80, z 30..70) whose bed is 10 m,
    /// holding a still 5 m lake (surface 15 m, well under the rim).
    fn lake_map() -> SurfaceWaterField {
        let (w, h) = (128_u32, 128_u32);
        let mut bed = vec![30.0_f32; (w * h) as usize];
        for z in 30..70 {
            for x in 30..80 {
                bed[(z * w + x) as usize] = 10.0;
            }
        }
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, w, h, bed, 0.0);
        f.mode = SurfaceWaterMode::VirtualPipes;
        for z in 30..70 {
            for x in 30..80 {
                let i = f.idx(x, z);
                f.depth_m[i] = 5.0;
            }
        }
        f.recompute_mass();
        f.tiles.touch_wet(&f.depth_m);
        f
    }

    #[test]
    fn a_still_lake_on_the_map_goes_to_sleep() {
        let mut f = lake_map();
        let mass0 = f.mass_m3;
        f.refresh_active_tiles();
        let (total, active, _) = f.tile_stats();
        assert!(active > 0 && active < total, "only the lake's tiles run: {active}/{total}");
        for _ in 0..(WATER_SLEEP_STEPS + 6) {
            f.step(1.0, 0.0, 1.0, 0.0, 0.0);
        }
        f.refresh_active_tiles();
        assert_eq!(f.tile_stats().1, 0, "a settled lake costs nothing");
        assert!((f.mass_m3 - mass0).abs() < 1e-6 * mass0, "mass kept: {} vs {mass0}", f.mass_m3);
        assert!(f.depth_m.iter().all(|d| *d <= 5.0 + 1e-3), "a lake at rest does not slosh");
    }

    #[test]
    fn a_crater_beside_a_sleeping_lake_wakes_it_and_the_water_flows_in() {
        let mut f = lake_map();
        for _ in 0..(WATER_SLEEP_STEPS + 6) {
            f.step(1.0, 0.0, 1.0, 0.0, 0.0);
        }
        f.refresh_active_tiles();
        assert_eq!(f.tile_stats().1, 0, "asleep before the strike");
        // The strike digs the dike just east of the lake (next tile over: x 80 is tile 5).
        assert!(f.apply_bed_delta(80, 50, -25.0));
        let pit = f.idx(80, 50);
        assert_eq!(f.depth_m[pit], 0.0);
        let mass0 = f.mass_m3;
        f.step(1.0, 0.0, 1.0, 0.0, 0.0);
        assert!(f.tile_stats().1 > 0, "the touch woke the tile and its wet neighbour");
        for _ in 0..20 {
            f.step(1.0, 0.0, 1.0, 0.0, 0.0);
        }
        assert!(f.depth_m[pit] > 1.0, "water ran into the crater: {}", f.depth_m[pit]);
        assert!((f.mass_m3 - mass0).abs() < 1e-6 * mass0, "volume conserved");
        assert!(!f.take_dirty_chunks().is_empty(), "the moved water queued its chunks");
    }

    #[test]
    fn active_tile_stepping_equals_the_full_field_step() {
        let make = || {
            let n = 96 * 96;
            let mut f = SurfaceWaterField::default();
            f.allocate(0, 0, 96, 96, vec![0.0; n], 0.0);
            for z in 40..56 {
                for x in 40..56 {
                    let i = f.idx(x, z);
                    f.depth_m[i] = 3.0;
                }
            }
            f.recompute_mass();
            f.tiles.touch_wet(&f.depth_m);
            f
        };
        let (mut full, mut active) = (make(), make());
        for _ in 0..30 {
            full.step_virtual_pipes_sub(0.05);
            active.refresh_active_tiles();
            active.step_active_substep(0.05);
            active.settle_active_tiles();
        }
        assert!(active.tile_stats().1 < active.tile_stats().0, "the far tiles never ran");
        let worst = full
            .depth_m
            .iter()
            .zip(&active.depth_m)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(worst < 1e-4, "active stepping drifted from the full solve by {worst} m");
        assert!(full.depth_m.iter().any(|d| *d > 0.5 && *d < 2.9), "the block really spread");
    }

    #[test]
    fn a_second_dam_far_from_the_first_gets_its_own_barrier() {
        let n = 256 * 256;
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, 256, 256, vec![0.0; n], 0.0);
        let row = |x0: i32, x1: i32, z: i32| -> Vec<IVec2> { (x0..x1).map(|x| IVec2::new(x, z)).collect() };
        let (a, b) = (Entity::from_bits(1 << 32 | 7), Entity::from_bits(1 << 32 | 9));
        let (ta, tb) = (row(20, 40, 40), row(200, 220, 210));
        f.install_barrier_tiles_for(&ta, 12.0, Some(a));
        f.install_barrier_tiles_for(&tb, 20.0, Some(b));
        let crest = |f: &SurfaceWaterField, t: &IVec2| f.hydraulics.barrier_crest_m[f.idx(t.x as u32, t.y as u32)];
        assert!(ta.iter().all(|t| crest(&f, t) > 11.9), "first dam stands");
        assert!(tb.iter().all(|t| crest(&f, t) > 19.9), "the far dam stands too");
        // Each dam keeps its own damage state: a broken column of B leaves A intact.
        assert!(f.set_barrier_column_fractions(&[(IVec2::new(210, 210), 0.0)]));
        assert_eq!(f.hydraulics.barrier_crest_m[f.idx(210, 210)], 0.0);
        assert!(ta.iter().all(|t| crest(&f, t) > 11.9));
        // A breach of B clears B only; A keeps holding.
        f.apply_barrier_breach_for(Some(b), true, 0.0);
        assert!(tb.iter().all(|t| crest(&f, t) == 0.0));
        assert!(ta.iter().all(|t| crest(&f, t) > 11.9));
        assert!(f.has_barrier());
    }
}
