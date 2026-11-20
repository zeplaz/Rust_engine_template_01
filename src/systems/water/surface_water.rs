//! DF-2 — `SurfaceWaterField` sole writer: level-pool + 2D virtual-pipes ROI.
//!
//! No GPU readback. Render/UI read snapshots only (DF-3).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

pub const G_MPS2: f32 = 9.81;
pub const SURFACE_WATER_SCHEMA_V1: u32 = 1;

/// Hybrid solve mode for any ROI (player dam, flood, benchmark).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Reflect, Serialize, Deserialize)]
pub enum SurfaceWaterMode {
    /// Flat reservoir surface; inundation = bed < stage (slow fill / intact dam).
    #[default]
    LevelPool,
    /// 2D virtual-pipes shallow water (river, breach wave, fast phases).
    VirtualPipes,
}

/// Level-pool storage while the dam is intact.
#[derive(Clone, Debug, Reflect, Serialize, Deserialize, PartialEq)]
pub struct ReservoirLevelPool {
    pub stage_m: f32,
    pub storage_m3: f64,
    pub area_m2: f64,
    pub bed_min_m: f32,
}

impl Default for ReservoirLevelPool {
    fn default() -> Self {
        Self {
            stage_m: 0.0,
            storage_m3: 0.0,
            area_m2: 0.0,
            bed_min_m: 0.0,
        }
    }
}

/// Spillway sill below the dam crest (m): the freeboard a full reservoir keeps (real dams
/// spill here; the embankment crest is never the spill crest).
pub const SPILLWAY_FREEBOARD_M: f32 = 3.0;
/// Emergency (ungated) spillway width as a share of the gated spillway length.
/// Sized so the design inflow passes with ~1.5 m head over the sill: a full pool keeps
/// ~1.5 m freeboard under the crest (0.8 m let the first 2D wave overtop).
pub const EMERGENCY_SPILLWAY_SHARE: f32 = 2.0;

/// Barrier / outlet / spillway params — player dam, hydro plant, or scenario-fed.
#[derive(Clone, Debug, Default, Reflect, Serialize, Deserialize, PartialEq)]
pub struct BarrierHydraulics {
    /// Crest elevation along barrier cells (world metres). 0 = no barrier.
    pub barrier_crest_m: Vec<f32>,
    pub spillway_length_m: f32,
    pub outlet_area_m2: f32,
    /// Mid-level outlet orifice area (augment / irrigation path).
    #[serde(default)]
    pub mid_outlet_area_m2: f32,
    pub turbine_design_m3s: f32,
    pub min_release_m3s: f32,
    pub spillway_c: f32,
    pub orifice_cd: f32,
    /// Installed footprint barrier: lowest bed under it and the dam height above that bed
    /// (metres). Per-column damage lowers a column's crest from `invert + height` (DW-3).
    #[serde(default)]
    pub barrier_invert_m: f32,
    #[serde(default)]
    pub barrier_height_m: f32,
    /// Some column stands below the full crest (damaged / breached notch): flow must route in
    /// 2D through the notch, never the level pool.
    #[serde(default)]
    pub barrier_damaged: bool,
}

impl BarrierHydraulics {
    #[must_use]
    pub fn default_earth_fill(outlet_area_m2: f32) -> Self {
        Self {
            barrier_crest_m: Vec::new(),
            spillway_length_m: 40.0,
            outlet_area_m2,
            mid_outlet_area_m2: (outlet_area_m2 * 0.55).max(1.0),
            turbine_design_m3s: 80.0,
            min_release_m3s: 15.0,
            spillway_c: 1.7,
            orifice_cd: 0.6,
            barrier_invert_m: 0.0,
            barrier_height_m: 0.0,
            barrier_damaged: false,
        }
    }

    /// Gated spillway weir: Q = C·L·gate·H^1.5 (`head_m` = stage − crest).
    #[must_use]
    /// Ungated emergency spillway over the sill ([`SPILLWAY_FREEBOARD_M`] below the crest): a
    /// full reservoir passes its inflow here even with every gate shut, so the embankment only
    /// overtops (and erodes) in a flood larger than the spillway.
    #[must_use]
    pub fn emergency_spillway_flow_m3s(&self, head_over_sill_m: f32) -> f32 {
        let h = head_over_sill_m.max(0.0);
        if h <= 0.0 {
            return 0.0;
        }
        self.spillway_c * self.spillway_length_m * EMERGENCY_SPILLWAY_SHARE * h.powf(1.5)
    }

    pub fn spillway_flow_m3s(&self, head_m: f32, gate_open01: f32) -> f32 {
        let h = head_m.max(0.0);
        if h <= 0.0 || gate_open01 <= 0.0 {
            return 0.0;
        }
        self.spillway_c * self.spillway_length_m * gate_open01.clamp(0.0, 1.0) * h.powf(1.5)
    }

    /// Uncontrolled crest overflow when stage exceeds the embankment / freeboard crest —
    /// fires even with gates shut (force-close / hack / experiment path).
    #[must_use]
    pub fn overtopping_flow_m3s(&self, head_above_crest_m: f32) -> f32 {
        let h = head_above_crest_m.max(0.0);
        if h <= 0.0 {
            return 0.0;
        }
        // Slightly higher C than gated bay: sheet flow over the embankment crest.
        self.spillway_c * 1.05 * self.spillway_length_m * h.powf(1.5)
    }

    /// Orifice / bottom outlet: Q = Cd·A·√(2gh)
    #[must_use]
    pub fn outlet_flow_m3s(&self, head_m: f32, outlet_open01: f32) -> f32 {
        Self::orifice_flow_m3s(self.orifice_cd, self.outlet_area_m2, head_m, outlet_open01)
    }

    /// Mid-level outlet orifice (same formula, separate area).
    #[must_use]
    pub fn mid_outlet_flow_m3s(&self, head_m: f32, mid_open01: f32) -> f32 {
        let area = if self.mid_outlet_area_m2 > 0.0 {
            self.mid_outlet_area_m2
        } else {
            self.outlet_area_m2 * 0.55
        };
        Self::orifice_flow_m3s(self.orifice_cd, area, head_m, mid_open01)
    }

    #[must_use]
    fn orifice_flow_m3s(cd: f32, area_m2: f32, head_m: f32, open01: f32) -> f32 {
        let h = head_m.max(0.0);
        if h <= 0.0 || open01 <= 0.0 || area_m2 <= 0.0 {
            return 0.0;
        }
        cd * area_m2 * open01.clamp(0.0, 1.0) * (2.0 * G_MPS2 * h).sqrt()
    }
}

/// Hypsometric curve of the cells a level pool can fill: stage <-> stored volume over the real bed.
struct PoolCurve {
    beds: Vec<f32>,
    /// `prefix[k]` = sum of the lowest `k` beds (f64).
    prefix: Vec<f64>,
    cell_a: f64,
}

impl PoolCurve {
    /// `beds` ascending.
    fn new(beds: Vec<f32>, cell_a: f64) -> Self {
        let mut prefix = Vec::with_capacity(beds.len() + 1);
        prefix.push(0.0);
        let mut acc = 0.0_f64;
        for b in &beds {
            acc += f64::from(*b);
            prefix.push(acc);
        }
        Self { beds, prefix, cell_a }
    }

    fn is_empty(&self) -> bool {
        self.beds.is_empty()
    }

    /// Volume held at `stage` (m3): every cell below it holds `stage - bed`.
    fn volume_at(&self, stage: f64) -> f64 {
        let k = self.beds.partition_point(|b| f64::from(*b) < stage);
        self.cell_a * (k as f64 * stage - self.prefix[k]).max(0.0)
    }

    /// Wet area at `stage` (m2).
    fn wet_area_m2(&self, stage: f32) -> f64 {
        self.beds.partition_point(|b| *b < stage) as f64 * self.cell_a
    }

    /// Stage whose volume equals `volume`; `None` for an empty curve.
    fn stage_for(&self, volume: f64) -> Option<f32> {
        let n = self.beds.len();
        if n == 0 {
            return None;
        }
        let v = volume.max(0.0);
        if v <= 0.0 {
            return Some(self.beds[0]);
        }
        // Smallest k >= 1 such that a lake exactly reaching bed k already holds >= `v`.
        let (mut lo, mut hi) = (1_usize, n);
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.volume_at(f64::from(self.beds[mid])) >= v {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        let k = lo.min(n);
        Some(((v / self.cell_a + self.prefix[k]) / k as f64) as f32)
    }
}

/// ROI surface-water field — sole sim water depth/flux authority (DF-2).
#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct SurfaceWaterField {
    pub schema_version: u32,
    pub wired: bool,
    pub mode: SurfaceWaterMode,
    /// World-tile origin (inclusive) of the ROI.
    pub origin_x: i32,
    pub origin_z: i32,
    pub width: u32,
    pub height: u32,
    pub dx_m: f32,
    pub depth_m: Vec<f32>,
    pub bed_m: Vec<f32>,
    /// DW-4: top (world metres) of a standing structure on the cell; 0 = none. Water passes a
    /// face only when its stage clears the higher of the two tops, like the dam crest — but kept
    /// apart from `barrier_crest_m` so a building never becomes the dam's spill crest.
    pub obstacle_top_m: Vec<f32>,
    pub qx_m3s: Vec<f32>,
    pub qy_m3s: Vec<f32>,
    pub wet_since_s: Vec<f32>,
    /// Baseline river depth per cell (the world's static river the ROI was seeded from). Beyond
    /// the ROI's south edge the world keeps this depth, so the free-drain boundary and the
    /// level-pool downstream reach hand over to it without a step (OP-10). Empty = none.
    pub ref_depth_m: Vec<f32>,
    pub pool: ReservoirLevelPool,
    pub hydraulics: BarrierHydraulics,
    pub inflow_m3s: f32,
    pub rain_runoff_m3s: f32,
    pub outlet_flow_m3s: f32,
    pub mid_outlet_flow_m3s: f32,
    pub spillway_flow_m3s: f32,
    /// Free overflow when stage > crest (gates closed still overtop).
    pub overtopping_flow_m3s: f32,
    /// Turbine penstock flow that actually left the pool (availability-capped).
    pub turbine_flow_m3s: f32,
    pub breach_flow_m3s: f32,
    /// Local ROI z of the installed barrier row (`None` until install). Upstream = lz > this.
    pub barrier_lz: Option<u32>,
    pub mass_m3: f64,
    pub mass_error_frac: f64,
    pub solver_ms: f32,
    pub dirty_chunks: Vec<IVec2>,
}

impl Default for SurfaceWaterField {
    fn default() -> Self {
        Self {
            schema_version: SURFACE_WATER_SCHEMA_V1,
            wired: false,
            mode: SurfaceWaterMode::VirtualPipes,
            origin_x: 0,
            origin_z: 0,
            width: 0,
            height: 0,
            dx_m: TILE_METRES_GAMEPLAY,
            depth_m: Vec::new(),
            bed_m: Vec::new(),
            obstacle_top_m: Vec::new(),
            qx_m3s: Vec::new(),
            qy_m3s: Vec::new(),
            wet_since_s: Vec::new(),
            ref_depth_m: Vec::new(),
            pool: ReservoirLevelPool::default(),
            hydraulics: BarrierHydraulics::default_earth_fill(8.0),
            inflow_m3s: 0.0,
            rain_runoff_m3s: 0.0,
            outlet_flow_m3s: 0.0,
            mid_outlet_flow_m3s: 0.0,
            spillway_flow_m3s: 0.0,
            overtopping_flow_m3s: 0.0,
            turbine_flow_m3s: 0.0,
            breach_flow_m3s: 0.0,
            barrier_lz: None,
            mass_m3: 0.0,
            mass_error_frac: 0.0,
            solver_ms: 0.0,
            dirty_chunks: Vec::new(),
        }
    }
}

impl SurfaceWaterField {
    #[inline]
    #[must_use]
    pub fn cell_count(&self) -> usize {
        (self.width as usize).saturating_mul(self.height as usize)
    }

    #[inline]
    #[must_use]
    pub fn idx(&self, lx: u32, lz: u32) -> usize {
        (lz as usize) * (self.width as usize) + (lx as usize)
    }

    #[inline]
    #[must_use]
    pub fn contains_world(&self, wx: i32, wz: i32) -> bool {
        wx >= self.origin_x
            && wz >= self.origin_z
            && wx < self.origin_x + self.width as i32
            && wz < self.origin_z + self.height as i32
    }

    /// DW-1: the ground under world tile `(wx, wz)` moved by `delta_m` (crater / rim / rubble).
    /// Water depth is kept (mass conserved), so the surface follows the bed. Returns false when
    /// the tile is outside the ROI. Barrier crests stay absolute.
    pub fn apply_bed_delta(&mut self, wx: i32, wz: i32, delta_m: f32) -> bool {
        if !self.wired || !self.contains_world(wx, wz) || self.bed_m.len() != self.cell_count() {
            return false;
        }
        let i = self.idx((wx - self.origin_x) as u32, (wz - self.origin_z) as u32);
        self.bed_m[i] += delta_m;
        self.pool.bed_min_m = self.pool.bed_min_m.min(self.bed_m[i]);
        true
    }

    #[inline]
    #[must_use]
    pub fn depth_at_world(&self, wx: i32, wz: i32) -> Option<f32> {
        if !self.contains_world(wx, wz) || !self.wired {
            return None;
        }
        let lx = (wx - self.origin_x) as u32;
        let lz = (wz - self.origin_z) as u32;
        self.depth_m.get(self.idx(lx, lz)).copied()
    }

    #[inline]
    #[must_use]
    pub fn surface_at(&self, i: usize) -> f32 {
        self.bed_m.get(i).copied().unwrap_or(0.0) + self.depth_m.get(i).copied().unwrap_or(0.0)
    }

    /// Allocate / reset ROI buffers. Topology from bed; depths cleared.
    pub fn allocate(
        &mut self,
        origin_x: i32,
        origin_z: i32,
        width: u32,
        height: u32,
        bed: Vec<f32>,
        inflow_m3s: f32,
    ) {
        let n = (width as usize).saturating_mul(height as usize);
        assert_eq!(bed.len(), n, "bed length must match ROI");
        self.schema_version = SURFACE_WATER_SCHEMA_V1;
        self.wired = true;
        self.origin_x = origin_x;
        self.origin_z = origin_z;
        self.width = width;
        self.height = height;
        self.dx_m = TILE_METRES_GAMEPLAY;
        self.bed_m = bed;
        self.depth_m = vec![0.0; n];
        self.qx_m3s = vec![0.0; n];
        self.qy_m3s = vec![0.0; n];
        self.wet_since_s = vec![0.0; n];
        self.ref_depth_m = vec![0.0; n];
        self.inflow_m3s = inflow_m3s;
        self.rain_runoff_m3s = 0.0;
        self.breach_flow_m3s = 0.0;
        self.overtopping_flow_m3s = 0.0;
        self.mid_outlet_flow_m3s = 0.0;
        self.turbine_flow_m3s = 0.0;
        self.barrier_lz = None;
        self.hydraulics.barrier_crest_m = vec![0.0; n];
        self.obstacle_top_m = vec![0.0; n];
        self.pool.bed_min_m = self
            .bed_m
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min);
        if !self.pool.bed_min_m.is_finite() {
            self.pool.bed_min_m = 0.0;
        }
        self.pool.stage_m = self.pool.bed_min_m;
        self.pool.storage_m3 = 0.0;
        self.recompute_mass();
        self.mass_error_frac = 0.0;
        self.dirty_chunks.clear();
        self.mark_all_chunks_dirty();
    }

    pub fn mark_all_chunks_dirty(&mut self) {
        self.dirty_chunks.clear();
        let chunk = crate::terrain::world_scale_contract::CHUNK_TILES_SIM as i32;
        let x0 = self.origin_x.div_euclid(chunk);
        let z0 = self.origin_z.div_euclid(chunk);
        let x1 = (self.origin_x + self.width as i32 - 1).div_euclid(chunk);
        let z1 = (self.origin_z + self.height as i32 - 1).div_euclid(chunk);
        for cz in z0..=z1 {
            for cx in x0..=x1 {
                self.dirty_chunks.push(IVec2::new(cx, cz));
            }
        }
    }

    pub fn recompute_mass(&mut self) {
        let cell_a = (self.dx_m as f64) * (self.dx_m as f64);
        self.mass_m3 = self
            .depth_m
            .iter()
            .map(|h| f64::from(*h) * cell_a)
            .sum();
        if self.mode == SurfaceWaterMode::LevelPool {
            self.mass_m3 = self.pool.storage_m3;
        }
    }

    /// Copy world static water depth into the ROI (one fluid, CLOSE-W3). Returns wet cells seeded.
    pub fn seed_from_static_depth(&mut self, st: &super::WorldStaticWaterDepth) -> u32 {
        if !self.wired || st.width == 0 {
            return 0;
        }
        let mut wet = 0;
        for lz in 0..self.height {
            for lx in 0..self.width {
                let wx = self.origin_x + lx as i32;
                let wz = self.origin_z + lz as i32;
                if wx < 0 || wz < 0 || wx as u32 >= st.width || wz as u32 >= st.height {
                    continue;
                }
                let d = st.depth_m.get(wz as usize * st.width as usize + wx as usize).copied().unwrap_or(0.0);
                if d > 0.05 {
                    let i = self.idx(lx, lz);
                    self.depth_m[i] = d.max(self.depth_m[i]);
                    if let Some(r) = self.ref_depth_m.get_mut(i) {
                        *r = d;
                    }
                    wet += 1;
                }
            }
        }
        self.recompute_mass();
        wet
    }

    /// Seed a north→south river corridor with baseflow depth.
    pub fn seed_river_corridor(&mut self, center_x_world: i32, half_width: u32, depth_m: f32) {
        if !self.wired {
            return;
        }
        let half = half_width.max(1) as i32;
        for lz in 0..self.height {
            for lx in 0..self.width {
                let wx = self.origin_x + lx as i32;
                if (wx - center_x_world).abs() <= half {
                    let i = self.idx(lx, lz);
                    self.depth_m[i] = depth_m.max(self.depth_m[i]);
                    if let Some(r) = self.ref_depth_m.get_mut(i) {
                        *r = depth_m;
                    }
                }
            }
        }
        self.recompute_mass();
    }

    /// Spill crest of the installed barrier: the LOWEST positive crest cell (water leaves over the
    /// lowest point — the channel). Cells under the dam's ends keep their bank ground as crest, so
    /// the max was bank height and the spillway / overtopping head was never reached. 0 = none.
    #[must_use]
    pub fn channel_crest_m(&self) -> f32 {
        let c = self
            .hydraulics
            .barrier_crest_m
            .iter()
            .copied()
            .filter(|c| *c > 0.0)
            .fold(f32::INFINITY, f32::min);
        if c.is_finite() { c } else { 0.0 }
    }

    /// Install a barrier across a world-z row (dam at narrows).
    /// `crest_height_m` is the dam's height above the lowest bed in the span; the stored crest
    /// is that absolute elevation (level crest), never below a cell's own bed. Absolute crest
    /// values broke once the bed became real terrain metres (CLOSE-W3): a "40 m" crest sat under
    /// a 300 m valley floor and blocked nothing.
    /// `center_world_x` defaults to ROI mid-x when `None`.
    pub fn install_barrier_row(
        &mut self,
        world_z: i32,
        crest_height_m: f32,
        half_span: u32,
        center_world_x: Option<i32>,
    ) {
        if !self.wired || !self.contains_world(self.origin_x, world_z) {
            return;
        }
        let lz = (world_z - self.origin_z) as u32;
        self.barrier_lz = Some(lz);
        let cx_world = center_world_x.unwrap_or(self.origin_x + (self.width as i32) / 2);
        let cx = cx_world - self.origin_x;
        let half = half_span.max(1) as i32;
        let span: Vec<usize> = (0..self.width as i32)
            .filter(|lx| (lx - cx).abs() <= half)
            .map(|lx| self.idx(lx as u32, lz))
            .collect();
        let invert = span.iter().map(|&i| self.bed_m[i]).fold(f32::INFINITY, f32::min);
        if !invert.is_finite() {
            return;
        }
        let crest_abs = invert + crest_height_m.max(0.0);
        for i in span {
            self.hydraulics.barrier_crest_m[i] = crest_abs.max(self.bed_m[i]);
        }
    }

    /// Install the barrier on explicit world tiles (a dam's footprint — DF-4.4 crest profile).
    /// Level crest at `crest_height_m` above the lowest footprint bed; never below a cell's bed.
    /// Upstream for the level pool starts above the footprint's highest row.
    pub fn install_barrier_tiles(&mut self, tiles: &[IVec2], crest_height_m: f32) {
        if !self.wired {
            return;
        }
        let cells: Vec<(usize, u32)> = tiles
            .iter()
            .filter(|t| self.contains_world(t.x, t.y))
            .map(|t| {
                let lz = (t.y - self.origin_z) as u32;
                (self.idx((t.x - self.origin_x) as u32, lz), lz)
            })
            .collect();
        let invert = cells.iter().map(|&(i, _)| self.bed_m[i]).fold(f32::INFINITY, f32::min);
        if !invert.is_finite() {
            return;
        }
        let crest_abs = invert + crest_height_m.max(0.0);
        for &(i, _) in &cells {
            self.hydraulics.barrier_crest_m[i] = crest_abs.max(self.bed_m[i]);
        }
        self.hydraulics.barrier_invert_m = invert;
        self.hydraulics.barrier_height_m = crest_height_m.max(0.0);
        self.hydraulics.barrier_damaged = false;
        // The dam body is solid: water standing in its cells moves to the first non-barrier cell
        // upstream (+z), volume kept. Otherwise the crest traps it and a water column pokes
        // through the dam forever.
        let w = self.width as usize;
        for &(i, lz) in &cells {
            let d = self.depth_m[i];
            if d <= 0.0 {
                continue;
            }
            let mut z = lz as usize + 1;
            while z < self.height as usize {
                let j = z * w + i % w;
                if self.hydraulics.barrier_crest_m[j] <= 0.0 {
                    self.depth_m[j] += d;
                    self.depth_m[i] = 0.0;
                    break;
                }
                z += 1;
            }
        }
        self.barrier_lz = cells.iter().map(|&(_, lz)| lz).max();
        self.recompute_mass();
    }

    /// Upstream of the barrier (north / higher z when valley runs +Y → -Y).
    #[inline]
    #[must_use]
    pub fn is_upstream_lz(&self, lz: u32) -> bool {
        match self.barrier_lz {
            Some(bz) => lz > bz,
            None => lz >= self.height / 2,
        }
    }

    /// Barrier cells' upstream (+z) and downstream (-z) open neighbours.
    fn dam_faces(&self) -> (Vec<usize>, Vec<usize>) {
        let (w, h) = (self.width as usize, self.height as usize);
        let (mut up, mut down) = (Vec::new(), Vec::new());
        for z in 0..h {
            for x in 0..w {
                let i = z * w + x;
                if self.hydraulics.barrier_crest_m.get(i).copied().unwrap_or(0.0) <= 0.0 {
                    continue;
                }
                if z + 1 < h && self.hydraulics.barrier_crest_m[i + w] <= 0.0 {
                    up.push(i + w);
                }
                if z > 0 && self.hydraulics.barrier_crest_m[i - w] <= 0.0 {
                    down.push(i - w);
                }
            }
        }
        up.sort_unstable();
        up.dedup();
        down.sort_unstable();
        down.dedup();
        (up, down)
    }

    /// Highest water surface right above the dam (m); bed min when dry.
    #[must_use]
    pub fn dam_upstream_surface_m(&self) -> f32 {
        let (up, _) = self.dam_faces();
        up.iter()
            .filter(|i| self.depth_m[**i] > 0.05)
            .map(|i| self.surface_at(*i))
            .fold(self.pool.bed_min_m, f32::max)
    }

    /// Move `q_m3s * dt_s` from the wet cells above the dam to the cells below it (outlet /
    /// spillway / turbine works through the body). Volume conserving; limited by what is there.
    pub fn transfer_through_dam(&mut self, q_m3s: f32, dt_s: f32) {
        if q_m3s <= 0.0 || dt_s <= 0.0 {
            return;
        }
        let (up, down) = self.dam_faces();
        if up.is_empty() || down.is_empty() {
            return;
        }
        let cell_a = self.dx_m * self.dx_m;
        let avail: f32 = up.iter().map(|i| self.depth_m[*i]).sum::<f32>() * cell_a;
        let take = (q_m3s * dt_s).min(avail * 0.5);
        if take <= 0.0 || avail <= 0.0 {
            return;
        }
        let frac = take / avail;
        for i in &up {
            self.depth_m[*i] *= 1.0 - frac;
        }
        // Released water leaves at the toe, deepest channel cells first (lowest bed).
        let bed_lo = down.iter().map(|i| self.bed_m[*i]).fold(f32::INFINITY, f32::min);
        let toe: Vec<usize> = down.iter().copied().filter(|i| self.bed_m[*i] <= bed_lo + 1.0).collect();
        let per = take / (toe.len().max(1) as f32 * cell_a);
        for i in &toe {
            self.depth_m[*i] += per;
        }
    }

    /// Wall top on cell `i` for the face flux: dam crest or standing structure, whichever is higher.
    #[inline]
    #[must_use]
    pub fn wall_top_m(&self, i: usize) -> f32 {
        let o = self.obstacle_top_m.get(i).copied().unwrap_or(0.0);
        self.hydraulics.barrier_crest_m[i].max(o)
    }

    /// DW-4: set / clear structure tops (`top_above_ground_m` <= 0 clears). Water standing in a
    /// newly walled cell moves to its lowest open neighbour (volume kept).
    pub fn set_obstacles(&mut self, cells: &[(IVec2, f32)]) {
        if !self.wired {
            return;
        }
        if self.obstacle_top_m.len() != self.bed_m.len() {
            self.obstacle_top_m = vec![0.0; self.bed_m.len()];
        }
        let (w, h) = (self.width as i32, self.height as i32);
        for &(t, top) in cells {
            if !self.contains_world(t.x, t.y) {
                continue;
            }
            let (lx, lz) = (t.x - self.origin_x, t.y - self.origin_z);
            let i = self.idx(lx as u32, lz as u32);
            if top <= 0.0 {
                self.obstacle_top_m[i] = 0.0;
                continue;
            }
            self.obstacle_top_m[i] = self.bed_m[i] + top;
            let d = self.depth_m[i];
            if d > 0.0 {
                let open = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .into_iter()
                    .map(|(dx, dz)| (lx + dx, lz + dz))
                    .filter(|&(x, z)| x >= 0 && z >= 0 && x < w && z < h)
                    .map(|(x, z)| self.idx(x as u32, z as u32))
                    .filter(|&j| self.obstacle_top_m[j] <= 0.0 && self.hydraulics.barrier_crest_m[j] <= 0.0)
                    .min_by(|&a, &b| self.surface_at(a).total_cmp(&self.surface_at(b)));
                if let Some(j) = open {
                    self.depth_m[j] += d;
                    self.depth_m[i] = 0.0;
                }
            }
        }
        self.recompute_mass();
    }

    /// DW-3: per-column crest from damage. `frac` = remaining height fraction of each dam
    /// column (1 = intact, 0 = gone). A column's crest is `invert + height * frac`, never below
    /// its own bed; a gone column holds nothing (crest 0), so water pours through the notch
    /// and the face velocity there erodes it further. Returns true if any crest changed.
    pub fn set_barrier_column_fractions(&mut self, cols: &[(IVec2, f32)]) -> bool {
        if !self.wired || self.hydraulics.barrier_height_m <= 0.0 {
            return false;
        }
        let (invert, height) = (self.hydraulics.barrier_invert_m, self.hydraulics.barrier_height_m);
        let mut changed = false;
        for &(t, frac) in cols {
            if !self.contains_world(t.x, t.y) {
                continue;
            }
            let i = self.idx((t.x - self.origin_x) as u32, (t.y - self.origin_z) as u32);
            let f = frac.clamp(0.0, 1.0);
            let crest = if f <= 0.0 { 0.0 } else { (invert + height * f).max(self.bed_m[i]) };
            if (self.hydraulics.barrier_crest_m[i] - crest).abs() > 1e-3 {
                self.hydraulics.barrier_crest_m[i] = crest;
                changed = true;
            }
        }
        // Damage only grows (no repair path): any column below full makes the barrier a notch.
        if changed && cols.iter().any(|(_, f)| *f < 0.999) {
            self.hydraulics.barrier_damaged = true;
        }
        changed
    }

    /// Clear crest and/or arm a breach outflow (destroy cascade → stage falls).
    /// Sole writer path for crest tear-down; coolers trip via stage, not scene hacks.
    pub fn apply_barrier_breach(&mut self, clear_crest: bool, breach_flow_m3s: f32) {
        if !self.wired {
            return;
        }
        if clear_crest {
            for c in &mut self.hydraulics.barrier_crest_m {
                *c = 0.0;
            }
            // Immediate storage dump so reservoir-pool intakes see stage collapse.
            let pulse = (breach_flow_m3s.max(0.0) as f64) * 120.0;
            let dump = self.pool.storage_m3 * 0.85 + pulse;
            self.pool.storage_m3 = (self.pool.storage_m3 - dump).max(0.0);
            let area = self.pool.area_m2.max(1.0);
            self.pool.stage_m =
                self.pool.bed_min_m + (self.pool.storage_m3 / area) as f32;
        }
        self.breach_flow_m3s = breach_flow_m3s.max(0.0);
        // Wave / drain needs 2D routing once the wall is gone.
        if clear_crest || breach_flow_m3s > 0.0 {
            self.mode = SurfaceWaterMode::VirtualPipes;
        }
    }

    /// Apply level-pool for `dt` seconds. Returns net volume change m³.
    ///
    /// Beds of the cells the level pool can fill (upstream of the barrier), ascending.
    fn pool_beds_sorted(&self) -> Vec<f32> {
        let w = self.width.max(1) as usize;
        let mut beds: Vec<f32> = self
            .bed_m
            .iter()
            .enumerate()
            .filter(|(i, _)| self.is_upstream_lz((*i / w) as u32))
            .map(|(_, b)| *b)
            .collect();
        beds.sort_by(f32::total_cmp);
        beds
    }

    /// Re-derive the level-pool state (storage, stage, wet area) from the painted depth so the
    /// pool starts from the water the virtual-pipes field really holds when the outlets close
    /// (OP-11: it used to restart at 0 and zeroed the whole reservoir).
    pub fn sync_pool_from_depth(&mut self) {
        if !self.wired || self.barrier_lz.is_none() {
            return;
        }
        let cell_a = f64::from(self.dx_m) * f64::from(self.dx_m);
        let w = self.width.max(1) as usize;
        let stored: f64 = self
            .depth_m
            .iter()
            .enumerate()
            .filter(|(i, _)| self.is_upstream_lz((*i / w) as u32))
            .map(|(_, d)| f64::from(*d) * cell_a)
            .sum();
        let curve = PoolCurve::new(self.pool_beds_sorted(), cell_a);
        self.pool.storage_m3 = stored;
        self.pool.stage_m = curve.stage_for(stored).unwrap_or(self.pool.bed_min_m);
        self.pool.area_m2 = curve.wet_area_m2(self.pool.stage_m).max(cell_a);
    }

    /// Level pool: the stage is the surface of a flat lake holding `storage_m3` over the real bed
    /// (hypsometric curve of the upstream cells), so it spreads over every cell below the stage,
    /// floodplain included, and volume is conserved. Release paths: low outlet + mid outlet +
    /// gated spill + free overtop + turbine penstock, sub-stepped so a long (time-skip) `dt_s`
    /// cannot overshoot the crest.
    pub fn step_level_pool(
        &mut self,
        dt_s: f32,
        gate_open01: f32,
        outlet_open01: f32,
        mid_outlet_open01: f32,
        turbine_setpoint_m3s: f32,
    ) -> f64 {
        let dt = dt_s.max(0.0) as f64;
        if dt <= 0.0 || !self.wired {
            return 0.0;
        }
        let cell_a = (self.dx_m as f64) * (self.dx_m as f64);
        let mut curve = PoolCurve::new(self.pool_beds_sorted(), cell_a);
        if curve.is_empty() {
            // No upstream cells (degenerate ROI): spread over the nominal upstream block.
            let up_rows = self
                .barrier_lz
                .map(|bz| self.height.saturating_sub(bz.saturating_add(1)))
                .unwrap_or_else(|| (self.height / 2).max(1))
                .max(1);
            curve = PoolCurve::new(
                vec![self.pool.bed_min_m; (self.width.max(1) * up_rows) as usize],
                cell_a,
            );
        }

        let crest = self.channel_crest_m();
        let bed_min = self.pool.bed_min_m;
        let q_in = self.inflow_m3s as f64 + self.rain_runoff_m3s as f64;
        let q_breach = self.breach_flow_m3s.max(0.0) as f64;
        // Shut = no controlled release. A full pool spills over the sill (crest minus freeboard)
        // through the gated spillway and the ungated emergency spillway; the embankment only
        // overtops — and erodes — when a flood beats the spillways. (Spilling at the crest
        // itself made every closed-gate fill overtop and wash the earth dam out.)
        let q_turb = turbine_setpoint_m3s.clamp(0.0, self.hydraulics.turbine_design_m3s.max(0.0));
        let shut = outlet_open01 < 0.05
            && mid_outlet_open01 < 0.05
            && gate_open01 < 0.05
            && q_turb < 0.5;
        // Stability: the weir response dQ/dh over the wet area bounds the explicit step.
        let n_sub = (dt / 30.0).ceil().clamp(1.0, 2048.0) as u32;
        let sub_dt = dt / f64::from(n_sub);

        let mut storage = self.pool.storage_m3.max(0.0);
        let start_storage = storage;
        let hyd = &self.hydraulics;
        let mut flows = (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
        for _ in 0..n_sub {
            let stage = curve.stage_for(storage).unwrap_or(bed_min);
            let head_pool = (stage - bed_min).max(0.0);
            let head_crest = if crest > 0.0 { (stage - crest).max(0.0) } else { 0.0 };
            let head_sill = if crest > 0.0 { (stage - (crest - SPILLWAY_FREEBOARD_M)).max(0.0) } else { 0.0 };
            let q_spill =
                hyd.spillway_flow_m3s(head_sill, gate_open01) + hyd.emergency_spillway_flow_m3s(head_sill);
            let q_overtop = hyd.overtopping_flow_m3s(head_crest);
            let q_low_raw = hyd.outlet_flow_m3s(head_pool, outlet_open01);
            let q_mid = hyd.mid_outlet_flow_m3s(head_pool, mid_outlet_open01);
            let q_out = if shut || outlet_open01 < 0.05 {
                0.0
            } else {
                q_low_raw.max(hyd.min_release_m3s * outlet_open01)
            };
            flows = (q_out, q_mid, q_spill, q_overtop);
            let q_release = f64::from(q_out + q_mid + q_spill + q_overtop + q_turb);
            storage = (storage + (q_in - q_release - q_breach) * sub_dt).max(0.0);
        }
        let stage = curve.stage_for(storage).unwrap_or(bed_min);
        self.outlet_flow_m3s = flows.0;
        self.mid_outlet_flow_m3s = flows.1;
        self.spillway_flow_m3s = flows.2;
        self.overtopping_flow_m3s = flows.3;
        self.turbine_flow_m3s = q_turb;
        let d_storage = storage - start_storage;
        self.pool.storage_m3 = storage;
        self.pool.stage_m = stage;
        self.pool.area_m2 = curve.wet_area_m2(stage).max(cell_a);

        // Paint inundation from stage onto depth (upstream + overtopping downstream shallow).
        let q_vis = (self.outlet_flow_m3s
            + self.mid_outlet_flow_m3s
            + self.spillway_flow_m3s
            + self.overtopping_flow_m3s
            + self.turbine_flow_m3s
            + self.breach_flow_m3s)
            .max(0.0);
        let w = self.width.max(1) as usize;
        // Downstream river: the baseline corridor (`ref_depth_m`, the world's static river)
        // scaled by a rating curve of the release, so a baseflow release reproduces the static
        // river exactly and nothing "appears" where the ROI hands over to world static water
        // (OP-10). Legacy centred-sheet painting only when no baseline was recorded.
        let ref_cols: Vec<u32> = if self.ref_depth_m.len() == self.depth_m.len() {
            (0..self.height as usize)
                .map(|z| {
                    self.ref_depth_m[z * w..(z + 1) * w]
                        .iter()
                        .filter(|d| **d > 0.05)
                        .count() as u32
                })
                .collect()
        } else {
            Vec::new()
        };
        let have_ref = ref_cols.iter().any(|c| *c > 0);
        let rating = (q_vis / self.inflow_m3s.max(1.0)).clamp(0.0, 2.0).powf(0.6);
        for i in 0..self.cell_count() {
            let bed = self.bed_m[i];
            let crest_i = self.hydraulics.barrier_crest_m.get(i).copied().unwrap_or(0.0);
            let lz = (i / w) as u32;
            if self.is_upstream_lz(lz) {
                self.depth_m[i] = (self.pool.stage_m - bed).max(0.0);
                self.qy_m3s[i] = 0.0;
            } else if crest_i > 0.0 {
                // Barrier cell: keep small depth if overtopping.
                self.depth_m[i] = (self.pool.stage_m - crest_i).max(0.0).min(2.0);
                self.qy_m3s[i] = 0.0;
            } else if have_ref {
                let d = self.ref_depth_m[i] * rating;
                if d > 0.05 && q_vis >= 1.0 {
                    self.depth_m[i] = d;
                    self.qy_m3s[i] = -(q_vis / ref_cols[lz as usize].max(1) as f32);
                } else {
                    self.depth_m[i] = 0.0;
                    self.qy_m3s[i] = 0.0;
                }
            } else {
                // Downstream sheet scales with total release (outlet + spill + breach) so
                // gate/outlet moves are readable in PixelWorld / GPU water.
                let sheet = (q_vis / (self.width.max(1) as f32 * self.dx_m * 0.35)).clamp(0.0, 4.5);
                let half = if q_vis < 1.0 {
                    0
                } else if q_vis < 5.0 {
                    2
                } else if q_vis < 80.0 {
                    4
                } else if q_vis < 250.0 {
                    6
                } else {
                    8
                };
                let in_channel = half > 0 && {
                    let lx = (i % w) as i32;
                    let cx = (self.width / 2) as i32;
                    (lx - cx).abs() <= half
                };
                if in_channel && sheet > 0.02 {
                    self.depth_m[i] = sheet;
                    // Mild downstream discharge so foam / GPU velocity encode release.
                    self.qy_m3s[i] = -(q_vis / (half.max(1) as f32 * 2.0 + 1.0));
                } else {
                    self.depth_m[i] = 0.0;
                    self.qy_m3s[i] = 0.0;
                }
            }
            if self.depth_m[i] > 0.05 {
                self.wet_since_s[i] += dt_s;
            } else {
                self.wet_since_s[i] = 0.0;
            }
        }
        self.recompute_mass();
        self.mark_all_chunks_dirty();
        d_storage
    }

    /// One virtual-pipes sub-step. Conservative volume update.
    /// Returns the boundary volume actually applied (inflow − south drain, m³).
    pub fn step_virtual_pipes_sub(&mut self, dt_s: f32) -> f64 {
        if !self.wired || self.width < 2 || self.height < 2 || dt_s <= 0.0 {
            return 0.0;
        }
        let w = self.width as usize;
        let h = self.height as usize;
        let dx = self.dx_m;
        let cell_a = dx * dx;
        let n = w * h;
        let mut flux_x = vec![0.0_f32; n]; // east face of cell
        let mut flux_y = vec![0.0_f32; n]; // north face of cell

        // Calibrated so dam-break front celerity matches Ritter 2√(gh) within ±10%
        // on the general kernel (see `measure_dam_break_front_distance`).
        const VP_CONDUCTANCE_K: f32 = 16.0;
        // Stability bound: flux = C·Δη is explicit diffusion, stable only while C·dt/A <= 1/4
        // per face. Above it (deep reservoir: limit ~0.007 s at 35 m; the caller's sub-step cap
        // gave ~0.016 s) cells overshoot their neighbours and blow up into an odd-even
        // checkerboard (0 m / 46 m / 0 m above a dam), which also "eroded" intact dams. Capping
        // each face at A/(8·dt) means a cell can never pass a neighbour in one sub-step; when the
        // sub-step is small enough the physical conductance is used unchanged.
        let c_stable = cell_a / (8.0 * dt_s);
        let conductance = |h_a: f32, h_b: f32| -> f32 {
            // Upwind depth so dry-bed wetting is not starved by a zero neighbor.
            let hp = h_a.max(h_b);
            if hp < 1e-4 {
                0.0
            } else {
                // Gravity-wave scale: C = k·Δx·√(g·h) → volume flux = C·Δη.
                (VP_CONDUCTANCE_K * dx * (G_MPS2 * hp).sqrt()).min(c_stable)
            }
        };

        for z in 0..h {
            for x in 0..w {
                let i = z * w + x;
                let eta_i = self.surface_at(i);
                let h_i = self.depth_m[i];
                let crest_i = self.wall_top_m(i);
                // East neighbor
                if x + 1 < w {
                    let j = i + 1;
                    let mut eta_j = self.surface_at(j);
                    let h_j = self.depth_m[j];
                    // Barrier blocks unless stage above crest.
                    let crest = crest_i.max(self.wall_top_m(j));
                    if crest > 0.0 {
                        let pass = eta_i.max(eta_j) > crest;
                        // CLOSE-W2: no `continue` here — it skipped the north (z) flux below,
                        // so a crest row never leaked or overtopped along the river.
                        if pass {
                            // Effective surfaces clipped to crest for orifice-like leak.
                            let eta_i2 = eta_i.max(crest);
                            eta_j = eta_j.max(crest);
                            let c = conductance(h_i, h_j);
                            flux_x[i] = c * (eta_i2 - eta_j);
                        } else {
                            flux_x[i] = 0.0;
                        }
                    } else {
                        let c = conductance(h_i, h_j);
                        flux_x[i] = c * (eta_i - eta_j);
                    }
                }
                // North neighbor (+z)
                if z + 1 < h {
                    let j = i + w;
                    let mut eta_j = self.surface_at(j);
                    let h_j = self.depth_m[j];
                    let crest = crest_i.max(self.wall_top_m(j));
                    if crest > 0.0 {
                        let pass = eta_i.max(eta_j) > crest;
                        if pass {
                            let eta_i2 = eta_i.max(crest);
                            eta_j = eta_j.max(crest);
                            let c = conductance(h_i, h_j);
                            flux_y[i] = c * (eta_i2 - eta_j);
                        }
                    } else {
                        let c = conductance(h_i, h_j);
                        flux_y[i] = c * (eta_i - eta_j);
                    }
                }
            }
        }

        // CLOSE-W4 conservation: (1) only the donor (higher surface) side can give water and the
        // driving head is capped at the donor's depth — a dry cell gives nothing, and a thin film
        // over a terrain step can't push the whole bed drop; (2) each cell's total outflow per
        // sub-step is scaled to the water it holds. Without these, real-terrain beds (CLOSE-W3)
        // created water on every steep face and the field went to NaN.
        {
            let depth = &self.depth_m;
            let cap = |f: f32, donor: usize, eta_drop: f32| -> f32 {
                let hd = depth[donor];
                if hd < 1e-4 || eta_drop <= 0.0 {
                    0.0
                } else {
                    f * (hd / eta_drop).min(1.0)
                }
            };
            for z in 0..h {
                for x in 0..w {
                    let i = z * w + x;
                    if x + 1 < w {
                        let j = i + 1;
                        let f = flux_x[i];
                        let drop = (self.surface_at(i) - self.surface_at(j)).abs();
                        flux_x[i] = if f >= 0.0 { cap(f, i, drop) } else { cap(f, j, drop) };
                    }
                    if z + 1 < h {
                        let j = i + w;
                        let f = flux_y[i];
                        let drop = (self.surface_at(i) - self.surface_at(j)).abs();
                        flux_y[i] = if f >= 0.0 { cap(f, i, drop) } else { cap(f, j, drop) };
                    }
                }
            }
            // Outflow limiter: scale every outgoing face of a cell so it can't give more than it has.
            let mut scale = vec![1.0_f32; n];
            for z in 0..h {
                for x in 0..w {
                    let i = z * w + x;
                    let mut out = 0.0_f32;
                    out += flux_x[i].max(0.0);
                    out += flux_y[i].max(0.0);
                    if x > 0 {
                        out += (-flux_x[i - 1]).max(0.0);
                    }
                    if z > 0 {
                        out += (-flux_y[i - w]).max(0.0);
                    }
                    let vol = depth[i] * cell_a;
                    let want = out * dt_s;
                    if want > vol && want > 0.0 {
                        scale[i] = vol / want;
                    }
                }
            }
            for z in 0..h {
                for x in 0..w {
                    let i = z * w + x;
                    if x + 1 < w {
                        let donor = if flux_x[i] >= 0.0 { i } else { i + 1 };
                        flux_x[i] *= scale[donor];
                    }
                    if z + 1 < h {
                        let donor = if flux_y[i] >= 0.0 { i } else { i + w };
                        flux_y[i] *= scale[donor];
                    }
                }
            }
        }

        // Inflow along north edge (highest z).
        let q_edge = self.inflow_m3s + self.rain_runoff_m3s;
        let north_z = h - 1;
        // CLOSE-W2: inflow enters the river where it already runs (wet north cells), not
        // smeared over the whole edge — that fed the dry banks and flooded the valley.
        let north_row = north_z * w;
        let wet_north = (0..w).filter(|&x| self.depth_m[north_row + x] > 0.05).count();
        let inflow_cols = if wet_north > 0 { wet_north } else { w };
        let per_cell = q_edge / inflow_cols.max(1) as f32;

        let mut applied = 0.0_f64;
        let mut next = self.depth_m.clone();
        for z in 0..h {
            for x in 0..w {
                let i = z * w + x;
                let mut div = 0.0_f32;
                // East face out
                div += flux_x[i];
                // West face in = east face of west neighbor
                if x > 0 {
                    div -= flux_x[i - 1];
                }
                // North face out
                div += flux_y[i];
                // South face in
                if z > 0 {
                    div -= flux_y[i - w];
                }
                let mut dvol = -div * dt_s;
                if z == north_z && (wet_north == 0 || self.depth_m[i] > 0.05) {
                    dvol += per_cell * dt_s;
                    applied += f64::from(per_cell * dt_s);
                }
                // CLOSE-W2: open downstream boundary. The south edge drains like a free
                // overfall into a dry ghost cell (head = local depth). The old sink was the
                // dam outlet flow (~40 m³/s) vs 220 m³/s inflow → water piled up forever.
                // Dam releases are internal to the ROI; they are not this boundary.
                if z == 0 {
                    let h0 = self.depth_m[i];
                    // Ghost cell below the edge holds the world's static river depth (same bed):
                    // water leaves only while the edge stands above it, so the ROI hands over to
                    // world static water without a drop (OP-10). Dry ghost = old free overfall.
                    let h_ghost = self.ref_depth_m.get(i).copied().unwrap_or(0.0);
                    let q_free = if h0 > h_ghost {
                        conductance(h0, h_ghost) * (h0 - h_ghost)
                    } else {
                        0.0
                    };
                    let avail = (h0 * cell_a + dvol).max(0.0);
                    let sink = (q_free * dt_s).clamp(0.0, avail);
                    dvol -= sink;
                    applied -= f64::from(sink);
                }
                next[i] = (self.depth_m[i] + dvol / cell_a).max(0.0);
                self.qx_m3s[i] = flux_x[i];
                self.qy_m3s[i] = flux_y[i];
            }
        }
        self.depth_m = next;
        for i in 0..n {
            if self.depth_m[i] > 0.05 {
                self.wet_since_s[i] += dt_s;
            } else {
                self.wet_since_s[i] = 0.0;
            }
        }
        applied
    }

    /// Advance `dt_s` game seconds (may sub-step for CFL).
    pub fn step(
        &mut self,
        dt_s: f32,
        gate_open01: f32,
        outlet_open01: f32,
        mid_outlet_open01: f32,
        turbine_setpoint_m3s: f32,
    ) {
        if !self.wired || dt_s <= 0.0 {
            return;
        }
        let t0 = std::time::Instant::now();
        let mass_before = self.mass_m3;
        // Expected net uses published releases (incl. mid / overtop / turbine / breach) —
        // do not invent a min_release floor here; that inflated mass_error when gates shut.
        let q_out_expected = self.outlet_flow_m3s as f64
            + self.mid_outlet_flow_m3s as f64
            + self.spillway_flow_m3s as f64
            + self.overtopping_flow_m3s as f64
            + self.turbine_flow_m3s as f64
            + self.breach_flow_m3s as f64;
        let mut expected_net = (self.inflow_m3s as f64 + self.rain_runoff_m3s as f64 - q_out_expected)
            * dt_s as f64;

        match self.mode {
            SurfaceWaterMode::LevelPool => {
                // Sub-stepped releases: the conservation check compares against the volume the
                // pool really integrated, not last frame's published flows.
                expected_net = self.step_level_pool(
                    dt_s,
                    gate_open01,
                    outlet_open01,
                    mid_outlet_open01,
                    turbine_setpoint_m3s,
                );
            }
            SurfaceWaterMode::VirtualPipes => {
                // Update release estimates from pool/channel head at dam row.
                let head = self
                    .depth_m
                    .iter()
                    .copied()
                    .fold(0.0_f32, f32::max)
                    .max(0.5);
                let crest = self.channel_crest_m();
                let head_crest = if crest > 0.0 {
                    (self.pool.stage_m.max(head) - crest).max(0.0)
                } else {
                    0.0
                };
                let shut = outlet_open01 < 0.05
                    && mid_outlet_open01 < 0.05
                    && gate_open01 < 0.05
                    && turbine_setpoint_m3s < 0.5;
                self.outlet_flow_m3s = if shut || outlet_open01 < 0.05 {
                    0.0
                } else {
                    self.hydraulics
                        .outlet_flow_m3s(head, outlet_open01)
                        .max(self.hydraulics.min_release_m3s * outlet_open01)
                };
                self.mid_outlet_flow_m3s = if mid_outlet_open01 < 0.05 {
                    0.0
                } else {
                    self.hydraulics.mid_outlet_flow_m3s(head, mid_outlet_open01)
                };
                // Spillways measure head over the sill (crest minus freeboard), like the pool.
                let up_eta = self.dam_upstream_surface_m();
                let head_sill = if crest > 0.0 {
                    (up_eta - (crest - SPILLWAY_FREEBOARD_M)).max(0.0)
                } else {
                    0.0
                };
                self.spillway_flow_m3s = self.hydraulics.spillway_flow_m3s(head_sill, gate_open01)
                    + self.hydraulics.emergency_spillway_flow_m3s(head_sill);
                // Overtopping in 2D is the face flux over the crest itself (reported only).
                self.overtopping_flow_m3s = self.hydraulics.overtopping_flow_m3s(head_crest);
                self.turbine_flow_m3s = turbine_setpoint_m3s
                    .clamp(0.0, self.hydraulics.turbine_design_m3s.max(0.0));
                // Controlled releases really pass the dam in 2D: outlets, spillways and turbines
                // move water from the cells above the dam to the cells below it (volume kept).
                // They were only reported numbers here, so once gates opened (S4) the only path
                // past the dam was over the crest: the pool rose, overtopped and eroded it.
                let q_release = self.outlet_flow_m3s
                    + self.mid_outlet_flow_m3s
                    + self.spillway_flow_m3s
                    + self.turbine_flow_m3s;
                self.transfer_through_dam(q_release, dt_s);

                let h_max = self.depth_m.iter().copied().fold(0.1_f32, f32::max).max(0.1);
                let wave = 0.5 * self.dx_m / (G_MPS2 * h_max).sqrt();
                // Match VP_CONDUCTANCE_K=16 used in step_virtual_pipes_sub.
                let cond = 0.2 * self.dx_m * self.dx_m
                    / (16.0 * self.dx_m * (G_MPS2 * h_max).sqrt() * h_max).max(1e-6);
                let cfl = wave.min(cond).max(1e-4);
                let sub = (dt_s / cfl).ceil().max(1.0).min(64.0) as u32;
                let sub_dt = dt_s / sub as f32;
                // CLOSE-W2: conservation vs what the boundaries really applied.
                expected_net = 0.0;
                for _ in 0..sub {
                    expected_net += self.step_virtual_pipes_sub(sub_dt);
                }
                self.sync_pool_from_depth();
                self.recompute_mass();
                self.mark_all_chunks_dirty();
            }
        }

        let mass_after = self.mass_m3;
        let denom = mass_before.abs().max(1.0);
        // Track residual vs expected net for conservation (inflow−outflow accounting).
        let actual_net = mass_after - mass_before;
        self.mass_error_frac = ((actual_net - expected_net).abs() / denom).min(10.0);
        self.solver_ms = t0.elapsed().as_secs_f32() * 1000.0;
    }

    /// Cross-section discharge (m³/s) along a world-z row near `wx`.
    #[must_use]
    pub fn gauge_discharge(&self, wx: i32, wz: i32, half: u32) -> (f32, f32, f32) {
        if !self.wired || !self.contains_world(wx, wz) {
            return (0.0, 0.0, 0.0);
        }
        let lz = (wz - self.origin_z) as u32;
        let cx = wx - self.origin_x;
        let half = half as i32;
        let mut q = 0.0_f32;
        let mut stage = 0.0_f32;
        let mut vel = 0.0_f32;
        let mut n = 0_u32;
        for lx in 0..self.width as i32 {
            if (lx - cx).abs() > half {
                continue;
            }
            let i = self.idx(lx as u32, lz);
            let h = self.depth_m[i];
            let face = self.qy_m3s[i].abs().max(self.qx_m3s[i].abs());
            q += face.max(h * self.dx_m * 0.5); // fallback conveyance when flux quiet
            stage += h;
            if h > 1e-3 {
                vel += face / (h * self.dx_m).max(1e-3);
            }
            n += 1;
        }
        if n == 0 {
            return (0.0, 0.0, 0.0);
        }
        (q, stage / n as f32, vel / n as f32)
    }
}

/// Analytic Ritter dam-break front speed on a flat dry channel: `2·√(g·h₀)`.
#[must_use]
pub fn ritter_front_speed_mps(h0_m: f32) -> f32 {
    2.0 * (G_MPS2 * h0_m.max(0.0)).sqrt()
}

/// Turbine electrical power [MW]: `P = η·ρ·g·Q·H / 1e6`.
#[must_use]
pub fn hydro_power_mw(eta: f32, q_m3s: f32, head_m: f32) -> f32 {
    const RHO: f32 = 1000.0;
    let p_w = eta.clamp(0.0, 1.0) * RHO * G_MPS2 * q_m3s.max(0.0) * head_m.max(0.0);
    (p_w / 1.0e6).max(0.0)
}

/// Build a flat dry channel, remove a mid barrier, and measure how far the wet front advances south.
/// Returns (distance_m, duration_s). Used to check dam-break propagation on the general solver.
/// Upstream reservoir depth is held (classic Ritter BC) so finite-tank depletion does not skew celerity.
#[must_use]
pub fn measure_dam_break_front_distance(h0_m: f32, duration_s: f32) -> (f32, f32) {
    // Fine dx so the front can resolve Ritter celerity on the general virtual-pipes kernel.
    let width = 3_u32;
    let height = 320_u32;
    let dx = 1.0_f32;
    let n = (width * height) as usize;
    let mut field = SurfaceWaterField::default();
    field.allocate(0, 0, width, height, vec![0.0; n], 0.0);
    field.dx_m = dx;
    field.mode = SurfaceWaterMode::VirtualPipes;
    field.hydraulics.min_release_m3s = 0.0;
    field.outlet_flow_m3s = 0.0;
    let mid = height / 2;
    for lz in 0..height {
        for lx in 0..width {
            let i = field.idx(lx, lz);
            if lz == mid {
                field.hydraulics.barrier_crest_m[i] = h0_m + 5.0;
            }
            if lz > mid {
                field.depth_m[i] = h0_m;
            }
        }
    }
    field.recompute_mass();
    for v in &mut field.hydraulics.barrier_crest_m {
        *v = 0.0;
    }
    // Conductance-limited CFL: dt ≤ 0.2·Δx² / (k·Δx·√(g·h)·h) with k=16.
    let k = 16.0_f32;
    let c_scale = k * dx * (G_MPS2 * h0_m.max(0.1)).sqrt() * h0_m.max(0.1);
    let cfl = (0.2 * dx * dx / c_scale.max(1e-6)).min(0.25 * dx / (G_MPS2 * h0_m.max(0.1)).sqrt());
    let steps = ((duration_s / cfl).ceil() as u32).max(64).min(20_000);
    let dt = duration_s / steps as f32;
    for _ in 0..steps {
        field.step_virtual_pipes_sub(dt);
        // Hold infinite upstream reservoir (Ritter idealization).
        for lz in (mid + 1)..height {
            for lx in 0..width {
                let i = field.idx(lx, lz);
                field.depth_m[i] = h0_m;
            }
        }
    }
    let mut front_z = mid;
    for lz in (0..mid).rev() {
        let wet = (0..width).any(|lx| field.depth_m[field.idx(lx, lz)] > 0.02 * h0_m);
        if wet {
            front_z = lz;
        } else if front_z < mid {
            break;
        }
    }
    let dist_m = (mid.saturating_sub(front_z)) as f32 * field.dx_m;
    (dist_m, duration_s.max(1e-3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_mass_conserved_steady_inflow() {
        // Steady Qin = Qout on level-pool: storage must hold within 0.5% over 24 game hours.
        let w = 20_u32;
        let h = 20_u32;
        let n = (w * h) as usize;
        let bed = vec![0.0_f32; n];
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, w, h, bed, 100.0);
        field.mode = SurfaceWaterMode::LevelPool;
        // Seed an initial pool, then run with outlet sized so release ≈ inflow at that head.
        field.pool.storage_m3 = 200_000.0;
        field.pool.stage_m = 5.0;
        field.hydraulics.outlet_area_m2 = 0.0;
        field.hydraulics.min_release_m3s = 100.0;
        field.outlet_flow_m3s = 100.0;
        let mass0 = field.pool.storage_m3;
        let mut t = 0.0_f32;
        while t < 86_400.0 {
            // Fully open outlet path that enforces min_release = inflow.
            field.step_level_pool(60.0, 0.0, 1.0, 0.0, 0.0);
            // Force release equal to inflow for the conservation check (steady ops).
            field.hydraulics.min_release_m3s = field.inflow_m3s;
            t += 60.0;
        }
        let err = ((field.pool.storage_m3 - mass0).abs() / mass0.max(1.0)) as f32;
        assert!(
            err < 0.005,
            "mass err frac={err} storage={} vs {}",
            field.pool.storage_m3,
            mass0
        );
    }

    #[test]
    fn level_pool_fill_matches_volume() {
        let w = 20_u32;
        let h = 20_u32;
        let n = (w * h) as usize;
        let bed = vec![0.0_f32; n];
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, w, h, bed, 100.0);
        field.mode = SurfaceWaterMode::LevelPool;
        field.hydraulics.min_release_m3s = 0.0;
        field.hydraulics.outlet_area_m2 = 0.0;
        // Close outlets fully — only inflow fills.
        let cell_a = (TILE_METRES_GAMEPLAY as f64).powi(2);
        let area = cell_a * (w as f64) * ((h / 2) as f64);
        let target_storage = 50_000.0_f64;
        let fill_s = (target_storage / 100.0) as f32;
        field.step_level_pool(fill_s, 0.0, 0.0, 0.0, 0.0);
        let ratio = field.pool.storage_m3 / target_storage;
        assert!(
            (ratio - 1.0).abs() < 0.02,
            "storage {} vs target {} (area used {})",
            field.pool.storage_m3,
            target_storage,
            area
        );
    }

    #[test]
    fn dam_break_wave_ritter_front() {
        let h0 = 10.0_f32;
        let analytic = ritter_front_speed_mps(h0);
        assert!((analytic - 2.0 * (G_MPS2 * h0).sqrt()).abs() < 1e-4);
        let (dist_m, dur) = measure_dam_break_front_distance(h0, 2.0);
        let measured = dist_m / dur;
        assert!(
            dist_m >= 10.0,
            "front must leave the dam row dist={dist_m}"
        );
        let rel = ((measured / analytic) - 1.0).abs();
        assert!(
            rel <= 0.10,
            "Ritter ±10% failed measured={measured} analytic={analytic} rel={rel} dist={dist_m}"
        );
    }

    #[test]
    fn barrier_closed_outlets_fill_level_pool() {
        let w = 24_u32;
        let h = 24_u32;
        let n = (w * h) as usize;
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, w, h, vec![0.0; n], 80.0);
        field.install_barrier_row(12, 25.0, 8, Some(12));
        field.mode = SurfaceWaterMode::LevelPool;
        field.hydraulics.min_release_m3s = 0.0;
        field.hydraulics.outlet_area_m2 = 0.0;
        let before = field.pool.storage_m3;
        field.step_level_pool(600.0, 0.0, 0.0, 0.0, 0.0);
        assert!(
            field.pool.storage_m3 > before + 10_000.0,
            "closed barrier must impound storage {} → {}",
            before,
            field.pool.storage_m3
        );
        assert!(field.pool.stage_m > field.pool.bed_min_m + 0.5);
    }

    #[test]
    fn shut_gates_overtop_when_stage_exceeds_crest() {
        let w = 24_u32;
        let h = 24_u32;
        let n = (w * h) as usize;
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, w, h, vec![0.0; n], 200.0);
        field.install_barrier_row(12, 10.0, 8, Some(12));
        field.mode = SurfaceWaterMode::LevelPool;
        field.hydraulics.min_release_m3s = 15.0;
        field.hydraulics.outlet_area_m2 = 4.0;
        // Force pool above crest with valves shut — free overflow must fire.
        // Stage derives from storage over the real bed (900k m3 over ~68k m2 = ~13 m > 10 m crest).
        field.pool.storage_m3 = 900_000.0;
        field.step_level_pool(30.0, 0.0, 0.0, 0.0, 0.0);
        assert!(
            field.overtopping_flow_m3s > 5.0,
            "stage above crest with shut gates must overtop (got {})",
            field.overtopping_flow_m3s
        );
        assert!(field.outlet_flow_m3s < 0.5, "controlled outlet stays shut during force-close overtop");
        assert!(field.spillway_flow_m3s > 5.0, "the ungated emergency spillway carries the flood too");
        let down: f32 = (0..n)
            .filter(|&i| {
                let lz = (i / w as usize) as u32;
                !field.is_upstream_lz(lz)
                    && field.hydraulics.barrier_crest_m.get(i).copied().unwrap_or(0.0) <= 0.0
            })
            .map(|i| field.depth_m[i])
            .fold(0.0, f32::max);
        assert!(down > 0.05, "overtopping must wet downstream sheet");
    }

    #[test]
    fn shut_gates_hold_and_dry_downstream_sheet() {
        let w = 24_u32;
        let h = 24_u32;
        let n = (w * h) as usize;
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, w, h, vec![0.0; n], 120.0);
        field.install_barrier_row(12, 30.0, 8, Some(12));
        field.mode = SurfaceWaterMode::LevelPool;
        field.hydraulics.min_release_m3s = 15.0;
        field.hydraulics.outlet_area_m2 = 4.0;
        // ~6 m of head over the bed (stage derives from storage, not from a stale field).
        field.pool.storage_m3 = 400_000.0;
        // Open flood dump first so a channel exists, then shut.
        field.step_level_pool(30.0, 0.85, 1.0, 0.0, 0.0);
        assert!(
            field.outlet_flow_m3s + field.spillway_flow_m3s > 20.0,
            "open release must be visible"
        );
        let down_idxs: Vec<usize> = (0..n)
            .filter(|&i| {
                let lz = (i / w as usize) as u32;
                !field.is_upstream_lz(lz)
                    && field
                        .hydraulics
                        .barrier_crest_m
                        .get(i)
                        .copied()
                        .unwrap_or(0.0)
                        <= 0.0
            })
            .collect();
        let open_down: f32 = down_idxs
            .iter()
            .map(|&i| field.depth_m[i])
            .fold(0.0, f32::max);
        assert!(open_down > 0.05, "open dam must wet downstream");
        let storage_open = field.pool.storage_m3;
        field.step_level_pool(120.0, 0.0, 0.0, 0.0, 0.0);
        assert!(
            field.outlet_flow_m3s + field.spillway_flow_m3s < 0.5,
            "shut must zero published release"
        );
        assert!(
            field.pool.storage_m3 > storage_open,
            "shut must hold back vs open release"
        );
        let shut_down: f32 = down_idxs
            .iter()
            .map(|&i| field.depth_m[i])
            .fold(0.0, f32::max);
        assert!(
            shut_down < 0.05,
            "shut must dry the downstream sheet (got {shut_down})"
        );
    }

    #[test]
    fn breach_drains_pool_and_clears_crest() {
        let w = 16_u32;
        let h = 16_u32;
        let n = (w * h) as usize;
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, w, h, vec![0.0; n], 0.0);
        field.install_barrier_row(8, 20.0, 6, None);
        assert!(field.hydraulics.barrier_crest_m.iter().any(|c| *c > 0.0));
        field.mode = SurfaceWaterMode::LevelPool;
        field.pool.area_m2 = 10_000.0;
        field.pool.storage_m3 = 100_000.0;
        field.pool.stage_m = 10.0;
        let before = field.pool.storage_m3;
        field.apply_barrier_breach(true, 500.0);
        assert!(field.hydraulics.barrier_crest_m.iter().all(|c| *c == 0.0));
        assert!((field.breach_flow_m3s - 500.0).abs() < 1e-3);
        assert_eq!(field.mode, SurfaceWaterMode::VirtualPipes);
        assert!(
            field.pool.storage_m3 < before * 0.3,
            "breach must dump most storage {} → {}",
            before,
            field.pool.storage_m3
        );
        assert!(field.pool.stage_m < 3.0);
    }

    #[test]
    fn hydro_power_scales_with_qh() {
        let p = hydro_power_mw(0.9, 40.0, 50.0);
        // 0.9*1000*9.81*40*50 / 1e6 ≈ 17.658 MW
        assert!((p - 17.658).abs() < 0.05);
        assert_eq!(hydro_power_mw(0.9, 0.0, 50.0), 0.0);
    }

    /// A valley: channel column bed 0, terrace 5 m, walls 100 m; dam across row `dam_z`.
    fn terrace_valley(width: u32, height: u32, dam_z: i32) -> SurfaceWaterField {
        let n = (width * height) as usize;
        let mut bed = vec![5.0_f32; n];
        for z in 0..height {
            for x in 0..width {
                let i = (z * width + x) as usize;
                bed[i] = if x == width / 2 {
                    0.0
                } else if x < 2 || x + 2 >= width {
                    100.0
                } else {
                    5.0
                };
            }
        }
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, width, height, bed, 100.0);
        f.hydraulics.min_release_m3s = 0.0;
        let tiles: Vec<IVec2> = (0..width as i32).map(|x| IVec2::new(x, dam_z)).collect();
        f.install_barrier_tiles(&tiles, 40.0);
        f
    }

    /// A full reservoir switched to operate (2D, gates + outlet + turbine open) keeps its water
    /// below the crest: releases pass through the dam, nothing flows over the intact crest, so
    /// nothing erodes it.
    #[test]
    fn operating_a_full_dam_does_not_flow_over_the_crest() {
        let (w, h) = (24_u32, 40_u32);
        let mut f = terrace_valley(w, h, 10);
        f.mode = SurfaceWaterMode::LevelPool;
        for _ in 0..40 {
            f.step_level_pool(24_000.0, 0.0, 0.0, 0.0, 0.0);
        }
        let crest = f.channel_crest_m();
        assert!(f.pool.stage_m < crest, "full pool {} under crest {crest}", f.pool.stage_m);
        let tiles: Vec<IVec2> = (0..w as i32).map(|x| IVec2::new(x, 10)).collect();
        let mut worst = 0.0_f32;
        let mut worst_step = 0;
        for k in 0..600 {
            f.mode = SurfaceWaterMode::VirtualPipes;
            f.step(1.0, 0.2, 0.35, 0.0, 40.0);
            let v = crate::construction::dam_face_velocities_m_s(&f, &tiles)
                .into_iter()
                .fold(0.0_f32, f32::max);
            if v > worst {
                worst = v;
                worst_step = k;
            }
        }
        assert!(worst < 0.5, "flow over the intact crest: {worst} m/s at step {worst_step}");
    }

    #[test]
    fn level_pool_covers_the_terrace_and_conserves_volume() {
        let (w, h) = (24_u32, 40_u32);
        let mut f = terrace_valley(w, h, 10);
        f.mode = SurfaceWaterMode::LevelPool;
        let cell_a = f64::from(f.dx_m) * f64::from(f.dx_m);
        let crest = f.channel_crest_m();
        assert!((crest - 40.0).abs() < 1e-3, "crest {crest}");
        // Fill 20 coarse steps (time skip): enough for the 40 m pool to reach the crest.
        let mut reached = false;
        for _ in 0..40 {
            f.step_level_pool(24_000.0, 0.0, 0.0, 0.0, 0.0);
            // Painted volume equals storage (no mass lost to a stale area).
            let painted: f64 = (0..f.cell_count())
                .filter(|i| f.is_upstream_lz((*i / w as usize) as u32))
                .map(|i| f64::from(f.depth_m[i]) * cell_a)
                .sum();
            assert!(
                (painted - f.pool.storage_m3).abs() <= 1e-4 * f.pool.storage_m3.max(1.0),
                "painted {painted} vs storage {}",
                f.pool.storage_m3
            );
            if f.pool.stage_m >= crest - SPILLWAY_FREEBOARD_M {
                reached = true;
                // The spillway holds a full pool over its sill, below the crest: a closed dam fills
                // and spills; the embankment is never overtopped by its own inflow.
                assert!(f.pool.stage_m < crest, "stage {} overtops the crest", f.pool.stage_m);
            }
        }
        assert!(reached, "pool never reached the spillway sill: stage {}", f.pool.stage_m);
        // Terrace (5 m bed) is under ~35 m of water; walls (100 m) are dry.
        let up = (h - 2) as usize * w as usize + 5;
        assert!(f.depth_m[up] > 20.0, "terrace depth {}", f.depth_m[up]);
        let wall = (h - 2) as usize * w as usize;
        assert_eq!(f.depth_m[wall], 0.0);
        // At equilibrium the spillway carries the inflow and nothing overtops the embankment.
        assert!(
            (f.spillway_flow_m3s - f.inflow_m3s).abs() < 0.05 * f.inflow_m3s,
            "spill {} vs inflow {}",
            f.spillway_flow_m3s,
            f.inflow_m3s
        );
        assert!(f.overtopping_flow_m3s < 1e-3, "overtop {}", f.overtopping_flow_m3s);
    }

    #[test]
    fn level_pool_entry_keeps_the_water_the_field_holds() {
        let (w, h) = (24_u32, 40_u32);
        let mut f = terrace_valley(w, h, 10);
        let cell_a = f64::from(f.dx_m) * f64::from(f.dx_m);
        // Virtual-pipes state: 3 m of water on every upstream cell.
        for i in 0..f.cell_count() {
            if f.is_upstream_lz((i / w as usize) as u32) {
                f.depth_m[i] = 3.0;
            }
        }
        f.sync_pool_from_depth();
        let held: f64 = (0..f.cell_count())
            .filter(|i| f.is_upstream_lz((*i / w as usize) as u32))
            .map(|i| f64::from(f.depth_m[i]) * cell_a)
            .sum();
        assert!((f.pool.storage_m3 - held).abs() < 1e-6 * held);
        // A zero-length-ish level-pool step must not erase the reservoir (it used to restart at 0).
        f.mode = SurfaceWaterMode::LevelPool;
        f.inflow_m3s = 0.0;
        f.step_level_pool(1.0, 0.0, 0.0, 0.0, 0.0);
        let after: f64 = (0..f.cell_count())
            .filter(|i| f.is_upstream_lz((*i / w as usize) as u32))
            .map(|i| f64::from(f.depth_m[i]) * cell_a)
            .sum();
        assert!((after - held).abs() < 1e-3 * held, "held {held} after {after}");
    }

    #[test]
    fn level_pool_downstream_reach_matches_the_static_river_at_baseflow() {
        let (w, h) = (24_u32, 40_u32);
        let mut f = terrace_valley(w, h, 10);
        let c = (w / 2) as usize;
        for z in 0..10_usize {
            f.ref_depth_m[z * w as usize + c] = 3.5;
        }
        f.mode = SurfaceWaterMode::LevelPool;
        for _ in 0..40 {
            f.step_level_pool(24_000.0, 0.0, 0.0, 0.0, 0.0);
        }
        // Overtopping at the baseflow: the downstream river is exactly the static depth.
        assert!((f.depth_m[3 * w as usize + c] - 3.5).abs() < 0.2, "{}", f.depth_m[3 * w as usize + c]);
        // Closed dam: downstream river dries rather than keeping a phantom film.
        let mut g = terrace_valley(w, h, 10);
        for z in 0..10_usize {
            g.ref_depth_m[z * w as usize + c] = 3.5;
        }
        g.mode = SurfaceWaterMode::LevelPool;
        g.step_level_pool(600.0, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(g.depth_m[3 * w as usize + c], 0.0);
    }

    #[test]
    fn south_boundary_holds_the_static_river_depth() {
        // River column already at the static depth on a flat bed: nothing may drain out of the
        // ROI edge (the old dry ghost cell pulled it to ~0.1 m, leaving a step at the seam).
        let (w, h) = (5_u32, 8_u32);
        let n = (w * h) as usize;
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, w, h, vec![0.0; n], 0.0);
        f.mode = SurfaceWaterMode::VirtualPipes;
        for i in 0..n {
            f.depth_m[i] = 2.0;
            f.ref_depth_m[i] = 2.0;
        }
        f.recompute_mass();
        for _ in 0..50 {
            f.step_virtual_pipes_sub(0.05);
        }
        assert!((f.depth_m[2] - 2.0).abs() < 1e-3, "south edge depth {}", f.depth_m[2]);
        // Without a baseline the edge is the old free overfall.
        let mut g = SurfaceWaterField::default();
        g.allocate(0, 0, w, h, vec![0.0; n], 0.0);
        g.mode = SurfaceWaterMode::VirtualPipes;
        g.depth_m.iter_mut().for_each(|d| *d = 2.0);
        g.step_virtual_pipes_sub(0.05);
        assert!(g.depth_m[2] < 2.0);
    }
}
