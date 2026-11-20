//! Activity bookkeeping for the map-sized surface-water solver (WM-1).
//!
//! The field is cut into square tiles of [`WATER_TILE_CELLS`] cells. Only *active* tiles are
//! stepped: a tile is awake while it changed recently (or was touched by a barrier, obstacle, bed
//! delta, boundary inflow...), and it wakes a neighbour only when water can actually cross the
//! shared edge (a wet cell on either side of it). A tile that stayed calm for
//! [`WATER_SLEEP_STEPS`] steps falls asleep and costs nothing until something wakes it.

use bevy::prelude::IVec2;

/// Tile edge in cells.
pub const WATER_TILE_CELLS: u32 = 16;
/// Calm steps before a tile sleeps.
pub const WATER_SLEEP_STEPS: u16 = 6;
/// A tile is calm while no cell's depth moves faster than this (m/s) ...
pub const CALM_DEPTH_RATE_M_S: f32 = 2.0e-5;
/// ... and no face carries more than this (m3/s). A flowing river is never calm: its tiles carry
/// the flow, so skipping them would stop the river.
pub const CALM_FLUX_M3S: f32 = 0.02;
/// Below this per-substep depth rate (m/s) a tile counts as unchanged (no dirty mark).
pub const CHANGED_DEPTH_RATE_M_S: f32 = 1.0e-6;
/// Cell depth under which an edge cell cannot pass water to its neighbour tile.
pub const EDGE_DRY_M: f32 = 1.0e-4;

/// Tile activity of one [`super::SurfaceWaterField`]. Runtime only (rebuilt from depth on load).
#[derive(Clone, Debug, Default)]
pub struct WaterTileGrid {
    pub edge: u32,
    pub nx: u32,
    pub nz: u32,
    width: u32,
    height: u32,
    origin_x: i32,
    origin_z: i32,
    /// Steps of life left; > 0 = awake.
    awake: Vec<u16>,
    is_active: Vec<bool>,
    /// Tiles stepped this step (refreshed by the field before each step).
    pub active: Vec<u32>,
    base: Vec<u32>,
    /// Per-tile stats of the current step.
    pub max_rate: Vec<f32>,
    pub max_flux: Vec<f32>,
    /// Tiles holding a boundary inflow cell (always awake while the inflow runs).
    pub inflow_tiles: Vec<u32>,
    /// Number of boundary inflow cells (the inflow is split over them).
    pub inflow_cells: usize,
    dirty_tile: Vec<bool>,
    dirty_list: Vec<u32>,
    chunk_x0: i32,
    chunk_z0: i32,
    chunk_nx: i32,
    chunk_mask: Vec<bool>,
}

impl WaterTileGrid {
    #[must_use]
    pub fn new(width: u32, height: u32, origin_x: i32, origin_z: i32) -> Self {
        let edge = WATER_TILE_CELLS;
        let nx = width.div_ceil(edge).max(1);
        let nz = height.div_ceil(edge).max(1);
        let nt = (nx * nz) as usize;
        let chunk = crate::terrain::world_scale_contract::CHUNK_TILES_SIM as i32;
        let chunk_x0 = origin_x.div_euclid(chunk);
        let chunk_z0 = origin_z.div_euclid(chunk);
        let chunk_x1 = (origin_x + width as i32 - 1).div_euclid(chunk);
        let chunk_z1 = (origin_z + height as i32 - 1).div_euclid(chunk);
        let chunk_nx = chunk_x1 - chunk_x0 + 1;
        let chunk_nz = chunk_z1 - chunk_z0 + 1;
        Self {
            edge,
            nx,
            nz,
            width,
            height,
            origin_x,
            origin_z,
            awake: vec![0; nt],
            is_active: vec![false; nt],
            active: Vec::new(),
            base: Vec::new(),
            max_rate: vec![0.0; nt],
            max_flux: vec![0.0; nt],
            inflow_tiles: Vec::new(),
            inflow_cells: 0,
            dirty_tile: vec![false; nt],
            dirty_list: Vec::new(),
            chunk_x0,
            chunk_z0,
            chunk_nx,
            chunk_mask: vec![false; (chunk_nx * chunk_nz).max(0) as usize],
        }
    }

    #[inline]
    #[must_use]
    pub fn tile_count(&self) -> usize {
        (self.nx * self.nz) as usize
    }

    #[inline]
    #[must_use]
    pub fn tile_of(&self, x: u32, z: u32) -> usize {
        ((z / self.edge) * self.nx + x / self.edge) as usize
    }

    /// Cell rectangle `[x0, x1) x [z0, z1)` of tile `t`.
    #[inline]
    #[must_use]
    pub fn tile_rect(&self, t: usize) -> (usize, usize, usize, usize) {
        let (tx, tz) = (t as u32 % self.nx, t as u32 / self.nx);
        let (x0, z0) = (tx * self.edge, tz * self.edge);
        (
            x0 as usize,
            z0 as usize,
            (x0 + self.edge).min(self.width) as usize,
            (z0 + self.edge).min(self.height) as usize,
        )
    }

    #[inline]
    #[must_use]
    pub fn is_active(&self, t: usize) -> bool {
        self.is_active.get(t).copied().unwrap_or(false)
    }

    #[inline]
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Tiles still awake (will be stepped next time at least).
    #[must_use]
    pub fn awake_count(&self) -> usize {
        self.awake.iter().filter(|a| **a > 0).count()
    }

    fn touch_tile(&mut self, t: usize) {
        if let Some(a) = self.awake.get_mut(t) {
            *a = WATER_SLEEP_STEPS;
        }
        if let Some(d) = self.dirty_tile.get_mut(t) {
            if !*d {
                *d = true;
                self.dirty_list.push(t as u32);
            }
        }
    }

    /// Something changed at local cell `(x, z)`: wake its tile.
    pub fn touch_cell(&mut self, x: u32, z: u32) {
        if x < self.width && z < self.height {
            let t = self.tile_of(x, z);
            self.touch_tile(t);
        }
    }

    /// Wake every tile overlapping the local cell rectangle `[x0, x1) x [z0, z1)`.
    pub fn touch_rect(&mut self, x0: u32, z0: u32, x1: u32, z1: u32) {
        let (x1, z1) = (x1.min(self.width), z1.min(self.height));
        if x0 >= x1 || z0 >= z1 {
            return;
        }
        for tz in (z0 / self.edge)..=((z1 - 1) / self.edge) {
            for tx in (x0 / self.edge)..=((x1 - 1) / self.edge) {
                self.touch_tile((tz * self.nx + tx) as usize);
            }
        }
    }

    /// Queue every tile for dirty-chunk marking without waking it.
    pub fn mark_all_dirty(&mut self) {
        for t in 0..self.tile_count() {
            if !self.dirty_tile[t] {
                self.dirty_tile[t] = true;
                self.dirty_list.push(t as u32);
            }
        }
    }

    pub fn touch_all(&mut self) {
        for t in 0..self.tile_count() {
            self.touch_tile(t);
        }
    }

    /// Wake the tiles that hold any water (after a seed / restore).
    pub fn touch_wet(&mut self, depth: &[f32]) {
        let w = self.width as usize;
        for t in 0..self.tile_count() {
            let (x0, z0, x1, z1) = self.tile_rect(t);
            let wet = (z0..z1).any(|z| depth[z * w + x0..z * w + x1].iter().any(|d| *d > EDGE_DRY_M));
            if wet {
                self.touch_tile(t);
            }
        }
    }

    /// Reset the per-step state before the field picks this step's active set.
    pub(super) fn begin_refresh(&mut self) {
        for t in &self.active {
            self.is_active[*t as usize] = false;
        }
        self.active.clear();
        self.base.clear();
        for t in 0..self.awake.len() {
            if self.awake[t] > 0 {
                self.is_active[t] = true;
                self.base.push(t as u32);
            }
        }
    }

    /// Add a tile to the base set (boundary activity); no-op when already active.
    pub(super) fn add_base(&mut self, t: usize) {
        if !self.is_active[t] {
            self.is_active[t] = true;
            self.base.push(t as u32);
        }
    }

    /// Finish picking: halo tiles were added with [`Self::add_halo`]; publish the active list and
    /// clear the per-step stats.
    pub(super) fn base_tiles(&self) -> Vec<u32> {
        self.base.clone()
    }

    pub(super) fn add_halo(&mut self, t: usize) {
        if !self.is_active[t] {
            self.is_active[t] = true;
            self.active.push(t as u32);
        }
    }

    pub(super) fn seal_active(&mut self) {
        let mut all = std::mem::take(&mut self.base);
        all.extend_from_slice(&self.active);
        self.active = all;
        for t in &self.active {
            self.max_rate[*t as usize] = 0.0;
            self.max_flux[*t as usize] = 0.0;
        }
    }

    /// Fold this step's stats: calm tiles count down to sleep, busy ones stay awake; tiles whose
    /// depth moved are queued for dirty-chunk marking.
    pub(super) fn settle(&mut self) {
        for k in 0..self.active.len() {
            let t = self.active[k] as usize;
            let calm = self.max_rate[t] < CALM_DEPTH_RATE_M_S && self.max_flux[t] < CALM_FLUX_M3S;
            if calm {
                self.awake[t] = self.awake[t].saturating_sub(1);
            } else {
                self.awake[t] = WATER_SLEEP_STEPS;
            }
            if self.max_rate[t] > CHANGED_DEPTH_RATE_M_S && !self.dirty_tile[t] {
                self.dirty_tile[t] = true;
                self.dirty_list.push(t as u32);
            }
        }
    }

    /// True when tiles are waiting to be turned into dirty chunks.
    #[must_use]
    pub fn has_pending_dirty(&self) -> bool {
        !self.dirty_list.is_empty()
    }

    /// Turn the queued dirty tiles into chunk coordinates appended to `out` (deduplicated against
    /// the chunks already pending in `out`'s owner via the mask).
    pub(super) fn flush_dirty_into(&mut self, out: &mut Vec<IVec2>) {
        let chunk = crate::terrain::world_scale_contract::CHUNK_TILES_SIM as i32;
        let list = std::mem::take(&mut self.dirty_list);
        for t in &list {
            let t = *t as usize;
            self.dirty_tile[t] = false;
            let (x0, z0, x1, z1) = self.tile_rect(t);
            for (x, z) in [(x0, z0), (x1 - 1, z0), (x0, z1 - 1), (x1 - 1, z1 - 1)] {
                let cx = (self.origin_x + x as i32).div_euclid(chunk);
                let cz = (self.origin_z + z as i32).div_euclid(chunk);
                let m = (cz - self.chunk_z0) * self.chunk_nx + (cx - self.chunk_x0);
                if let Some(slot) = usize::try_from(m).ok().and_then(|m| self.chunk_mask.get_mut(m)) {
                    if !*slot {
                        *slot = true;
                        out.push(IVec2::new(cx, cz));
                    }
                }
            }
        }
        self.dirty_list = list;
        self.dirty_list.clear();
    }

    /// Forget the pending-chunk mask for chunks the owner just consumed.
    pub(super) fn clear_chunk_mask(&mut self, taken: &[IVec2]) {
        for c in taken {
            let m = (c.y - self.chunk_z0) * self.chunk_nx + (c.x - self.chunk_x0);
            if let Some(slot) = usize::try_from(m).ok().and_then(|m| self.chunk_mask.get_mut(m)) {
                *slot = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_wakes_only_the_tile_and_settle_puts_it_to_sleep() {
        let mut g = WaterTileGrid::new(64, 64, 0, 0);
        assert_eq!(g.tile_count(), 16);
        g.touch_cell(20, 3);
        assert_eq!(g.awake_count(), 1);
        g.begin_refresh();
        g.seal_active();
        assert_eq!(g.active_count(), 1);
        for _ in 0..WATER_SLEEP_STEPS {
            g.settle();
        }
        assert_eq!(g.awake_count(), 0);
    }
}
