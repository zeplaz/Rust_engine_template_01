//! River sources of the map-wide field (WM-2): where rivers get their water.
//!
//! The world draws each river as a channel (`River` class cells held at a baseline depth in
//! `ref_depth_m`, width grown with the flow). The river is seeded at that depth, so its baseflow is
//! what keeps it there: the uniform-flow discharge of the channel at its baseline depth and its own
//! surface slope, in the solver's own flux law. Fed less it would drain to a film, fed more it would
//! pile up and flood its banks. Each headwater cell injects its column's share, bounded by what the
//! cross-section can carry at [`RIVER_MAX_SPEED_MPS`].
//!
//! A headwater is a river cell that nothing upstream feeds (no river neighbour with a higher water
//! surface) and that drains into a lower river cell: the top of a stream, a lake outlet, or the cell
//! where a river enters the map edge (its upstream lies off the map). Flat standing channels have
//! none. Every other map edge stays an open outflow.
//!
//! A scene can pin the total (`SurfaceWaterBaseflow`): the pinned rate is shared over the derived
//! sources in proportion to their columns. Flow accumulation from world generation is not kept at
//! runtime; the channel mask and baseline depths it produced are, and they are all this needs.

use super::step_kernel::cfl_limited_conductance;
use super::surface_water::SurfaceWaterField;

/// Fastest cross-section mean speed (m/s) a source may be given (steep mountain streams).
pub const RIVER_MAX_SPEED_MPS: f32 = 1.5;
/// A cell holds baseline water (river / lake / sea) above this depth.
pub const BASELINE_WATER_MIN_M: f32 = 0.05;
/// Water surfaces closer than this are level (no drop between the two cells).
const LEVEL_EPS_M: f32 = 1.0e-3;

/// One headwater cell and the discharge its channel carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiverSource {
    /// Field-local cell index.
    pub cell: u32,
    /// Discharge (m3/s) derived from the cell's baseline cross-section.
    pub derived_m3s: f32,
}

/// Runtime source set of a field (not saved: re-derived from the world on load).
#[derive(Clone, Debug, Default)]
pub struct RiverSources {
    list: Vec<RiverSource>,
    /// Effective injection per cell (m3/s); empty = none.
    rate: Vec<f32>,
    pinned_m3s: Option<f32>,
    /// Arrays were replaced (load): sources must be derived again from the world.
    pub(super) stale: bool,
}

impl RiverSources {
    #[must_use]
    pub fn sources(&self) -> &[RiverSource] {
        &self.list
    }

    /// Injection rate (m3/s) at field cell `i`.
    #[inline]
    #[must_use]
    pub fn rate_at(&self, i: usize) -> f32 {
        self.rate.get(i).copied().unwrap_or(0.0)
    }
}

impl SurfaceWaterField {
    /// Injection rate (m3/s) of the river source at cell `i` (0 = not a source).
    #[inline]
    #[must_use]
    pub fn river_rate_at(&self, i: usize) -> f32 {
        self.river.rate_at(i)
    }

    /// The derived river sources.
    #[must_use]
    pub fn river_sources(&self) -> &[RiverSource] {
        self.river.sources()
    }

    /// Sources must be derived from the world again (arrays were replaced by a load).
    #[must_use]
    pub fn river_sources_stale(&self) -> bool {
        self.river.stale
    }

    /// Derive the headwater sources from the channel mask `river` (one flag per field cell) over
    /// the field's own bed and baseline depths, then publish their rates.
    pub fn derive_river_sources(&mut self, river: &[bool]) {
        let (w, h) = (self.width as usize, self.height as usize);
        let n = w * h;
        self.river.stale = false;
        self.river.list.clear();
        if n == 0 || river.len() != n || self.bed_m.len() != n || self.ref_depth_m.len() != n {
            self.apply_river_rates();
            return;
        }
        let surface = |i: usize| self.bed_m[i] + self.ref_depth_m[i];
        let mut list = Vec::new();
        for i in 0..n {
            if !river[i] || self.ref_depth_m[i] <= BASELINE_WATER_MIN_M {
                continue;
            }
            let (x, z) = (i % w, i / w);
            let s = surface(i);
            let mut neighbours = [usize::MAX; 4];
            if x > 0 {
                neighbours[0] = i - 1;
            }
            if x + 1 < w {
                neighbours[1] = i + 1;
            }
            if z > 0 {
                neighbours[2] = i - w;
            }
            if z + 1 < h {
                neighbours[3] = i + w;
            }
            let mut fed = false;
            let mut drop_m = 0.0_f32;
            for j in neighbours.into_iter().filter(|j| *j != usize::MAX) {
                if !river[j] {
                    continue;
                }
                let sj = surface(j);
                fed |= sj > s + LEVEL_EPS_M;
                if sj < s - LEVEL_EPS_M {
                    drop_m = drop_m.max(s - sj);
                }
            }
            if !fed && drop_m > 0.0 {
                let h = self.ref_depth_m[i];
                let uniform = cfl_limited_conductance(self.dx_m, h) * drop_m;
                let section = h * self.dx_m * RIVER_MAX_SPEED_MPS;
                list.push(RiverSource { cell: i as u32, derived_m3s: uniform.min(section) });
            }
        }
        self.river.list = list;
        self.apply_river_rates();
    }

    /// Replace the sources with explicit `(cell, m3/s)` injections (tests, tools).
    pub fn set_river_sources(&mut self, sources: &[(u32, f32)]) {
        self.river.stale = false;
        self.river.pinned_m3s = None;
        self.river.list = sources
            .iter()
            .filter(|(c, q)| (*c as usize) < self.cell_count() && *q > 0.0)
            .map(|(c, q)| RiverSource { cell: *c, derived_m3s: *q })
            .collect();
        self.apply_river_rates();
    }

    /// Pin the total river inflow (m3/s) shared over the sources by cross-section; `None` / 0
    /// releases the pin and every source carries its own channel's discharge.
    pub fn pin_river_inflow(&mut self, pinned_m3s: Option<f32>) {
        let pin = pinned_m3s.filter(|q| *q > 0.0);
        if pin != self.river.pinned_m3s {
            self.river.pinned_m3s = pin;
            self.apply_river_rates();
        }
    }

    /// Publish the per-cell injection, the total (`inflow_m3s`) and the tiles that must run.
    fn apply_river_rates(&mut self) {
        let n = self.cell_count();
        let derived_total: f64 = self.river.list.iter().map(|s| f64::from(s.derived_m3s)).sum();
        let count = self.river.list.len().max(1) as f32;
        self.river.rate.clear();
        let mut total = 0.0_f64;
        if !self.river.list.is_empty() {
            self.river.rate.resize(n, 0.0);
            for k in 0..self.river.list.len() {
                let s = self.river.list[k];
                let q = match self.river.pinned_m3s {
                    Some(p) if derived_total > 0.0 => p * (f64::from(s.derived_m3s) / derived_total) as f32,
                    Some(p) => p / count,
                    None => s.derived_m3s,
                };
                self.river.rate[s.cell as usize] += q;
                total += f64::from(q);
            }
        }
        self.inflow_m3s = total as f32;
        let w = self.width.max(1);
        let mut tiles: Vec<u32> = Vec::new();
        for s in &self.river.list {
            let t = self.tiles.tile_of(s.cell % w, s.cell / w) as u32;
            if !tiles.contains(&t) {
                tiles.push(t);
            }
        }
        for &t in &tiles {
            self.tiles.touch_tile(t as usize);
        }
        self.tiles.inflow_tiles = tiles;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `w x h` map, bed falling toward z = 0 by `fall` m per row; a river `x0..x1` wide at `depth`.
    fn stream(w: u32, h: u32, x0: u32, x1: u32, depth: f32, fall: f32) -> (SurfaceWaterField, Vec<bool>) {
        let n = (w * h) as usize;
        let mut bed = vec![0.0_f32; n];
        for z in 0..h {
            for x in 0..w {
                bed[(z * w + x) as usize] = fall * z as f32 + 5.0;
            }
        }
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, w, h, bed, 0.0);
        let mut river = vec![false; n];
        for z in 0..h {
            for x in x0..x1 {
                let i = f.idx(x, z);
                f.depth_m[i] = depth;
                f.ref_depth_m[i] = depth;
                river[i] = true;
            }
        }
        f.recompute_mass();
        (f, river)
    }

    /// The river that enters at the upstream map edge gets one source per channel cell there; a
    /// channel's discharge is its cross-section times the reference speed.
    #[test]
    fn a_river_entering_the_map_is_sourced_across_its_channel() {
        let (mut f, river) = stream(64, 64, 30, 34, 2.0, 0.05);
        f.derive_river_sources(&river);
        let cells: Vec<u32> = f.river_sources().iter().map(|s| s.cell).collect();
        assert_eq!(cells.len(), 4, "one source per channel cell at the entry: {cells:?}");
        assert!(cells.iter().all(|c| *c / 64 == 63), "all at the upstream edge");
        let want: f32 = f.river_sources().iter().map(|s| s.derived_m3s).sum();
        assert!(want > 0.0 && want <= 4.0 * 2.0 * f.dx_m * RIVER_MAX_SPEED_MPS + 1e-3, "rate {want} within the cross-section cap");
        assert!((f.inflow_m3s - want).abs() < 1e-3, "inflow {} vs derived {want}", f.inflow_m3s);
        assert!(f.tile_stats().2 > 0, "the source tiles are awake");
    }

    /// A flat channel (standing water) is nobody's headwater, and a pin is shared by cross-section.
    #[test]
    fn flat_channels_have_no_source_and_a_pin_is_shared_over_the_sources() {
        let (mut flat, river) = stream(32, 32, 10, 14, 2.0, 0.0);
        flat.derive_river_sources(&river);
        assert!(flat.river_sources().is_empty() && flat.inflow_m3s == 0.0);

        let (mut f, river) = stream(64, 64, 30, 34, 2.0, 0.05);
        f.derive_river_sources(&river);
        f.pin_river_inflow(Some(220.0));
        assert!((f.inflow_m3s - 220.0).abs() < 1e-2, "pinned total {}", f.inflow_m3s);
        let first = f.river_sources()[0].cell as usize;
        assert!((f.river_rate_at(first) - 55.0).abs() < 1e-2, "even cross-sections share evenly");
        f.pin_river_inflow(None);
        let derived: f32 = f.river_sources().iter().map(|s| s.derived_m3s).sum();
        assert!((f.inflow_m3s - derived).abs() < 1e-3, "released pin returns to the derived rate");
    }

    /// A river fed at its source reaches steady flow down its channel: the same discharge passes
    /// every cross-section below the source, and the river keeps what it is given (open map edges
    /// carry it away; nothing is lost on the way).
    #[test]
    #[ignore = "WM-2 in progress: behaviour not reached yet (see plan WM status)"]
    fn a_river_fed_at_its_source_reaches_steady_flow_down_its_channel() {
        let (mut f, river) = stream(48, 48, 20, 24, 2.0, 0.05);
        f.derive_river_sources(&river);
        let q_in = f.inflow_m3s;
        assert!(q_in > 10.0);
        for _ in 0..1500 {
            f.advance(1.0);
        }
        let section = |f: &SurfaceWaterField, z: u32| -> f32 {
            (20..24).map(|x| f.qy_m3s[f.idx(x, z)].abs()).sum()
        };
        let (upper, mid, lower) = (section(&f, 40), section(&f, 24), section(&f, 8));
        for (name, q) in [("upper", upper), ("mid", mid), ("lower", lower)] {
            assert!((q - q_in).abs() < 0.15 * q_in, "{name} reach carries {q} of {q_in} m3/s");
        }
        assert!(f.mass_error_frac < 1e-3, "books balance: {}", f.mass_error_frac);
    }
}
