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
        }
    }

    /// Gated spillway weir: Q = C·L·gate·H^1.5 (`head_m` = stage − crest).
    #[must_use]
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
    pub qx_m3s: Vec<f32>,
    pub qy_m3s: Vec<f32>,
    pub wet_since_s: Vec<f32>,
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
            qx_m3s: Vec::new(),
            qy_m3s: Vec::new(),
            wet_since_s: Vec::new(),
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
        self.inflow_m3s = inflow_m3s;
        self.rain_runoff_m3s = 0.0;
        self.breach_flow_m3s = 0.0;
        self.overtopping_flow_m3s = 0.0;
        self.mid_outlet_flow_m3s = 0.0;
        self.turbine_flow_m3s = 0.0;
        self.barrier_lz = None;
        self.hydraulics.barrier_crest_m = vec![0.0; n];
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
                }
            }
        }
        self.recompute_mass();
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

    /// Upstream of the barrier (north / higher z when valley runs +Y → -Y).
    #[inline]
    #[must_use]
    pub fn is_upstream_lz(&self, lz: u32) -> bool {
        match self.barrier_lz {
            Some(bz) => lz > bz,
            None => lz >= self.height / 2,
        }
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
    /// Release paths: low outlet + mid outlet + gated spill + free overtop + turbine penstock.
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
        // Storage area = cells with bed below current stage (or potential).
        let stage = self.pool.stage_m;
        let mut area = 0.0_f64;
        for (i, &bed) in self.bed_m.iter().enumerate() {
            let crest = self.hydraulics.barrier_crest_m.get(i).copied().unwrap_or(0.0);
            // Upstream of barrier counts toward pool (fixture: north / higher z).
            let lz = (i / self.width.max(1) as usize) as u32;
            if self.is_upstream_lz(lz) && bed < stage.max(crest.max(bed + 0.01)) {
                area += cell_a;
            }
        }
        if area < cell_a {
            let up_rows = self
                .barrier_lz
                .map(|bz| self.height.saturating_sub(bz.saturating_add(1)))
                .unwrap_or_else(|| (self.height / 2).max(1))
                .max(1);
            area = cell_a * (self.width as f64) * (up_rows as f64);
        }
        self.pool.area_m2 = area;

        let crest = self
            .hydraulics
            .barrier_crest_m
            .iter()
            .copied()
            .fold(0.0_f32, f32::max);
        let head_pool = (self.pool.stage_m - self.pool.bed_min_m).max(0.0);
        let head_crest = if crest > 0.0 {
            (self.pool.stage_m - crest).max(0.0)
        } else {
            0.0
        };
        // Gated bay uses head on crest; free overtopping is independent of gate position.
        let q_spill = self.hydraulics.spillway_flow_m3s(head_crest, gate_open01);
        let q_overtop = self.hydraulics.overtopping_flow_m3s(head_crest);
        let q_low_raw = self.hydraulics.outlet_flow_m3s(head_pool, outlet_open01);
        let q_mid = self
            .hydraulics
            .mid_outlet_flow_m3s(head_pool, mid_outlet_open01);
        // Turbine is its own penstock path (not double-counted as outlet).
        let q_turb = turbine_setpoint_m3s
            .clamp(0.0, self.hydraulics.turbine_design_m3s.max(0.0));
        // Shut = hold below crest. No forced min when all controlled paths closed — rain /
        // inflow can fill to freeboard, then overtopping weir takes over (and erodes).
        let shut = outlet_open01 < 0.05
            && mid_outlet_open01 < 0.05
            && gate_open01 < 0.05
            && q_turb < 0.5;
        let q_out = if shut || outlet_open01 < 0.05 {
            0.0
        } else {
            q_low_raw.max(self.hydraulics.min_release_m3s * outlet_open01)
        };
        let q_release = q_out + q_mid + q_spill + q_overtop + q_turb;
        self.outlet_flow_m3s = q_out;
        self.mid_outlet_flow_m3s = q_mid;
        self.spillway_flow_m3s = q_spill;
        self.overtopping_flow_m3s = q_overtop;
        self.turbine_flow_m3s = q_turb;

        let q_breach = self.breach_flow_m3s.max(0.0) as f64;
        let q_in = self.inflow_m3s as f64 + self.rain_runoff_m3s as f64;
        let q_net = q_in - q_release as f64 - q_breach;
        let d_storage = q_net * dt;
        self.pool.storage_m3 = (self.pool.storage_m3 + d_storage).max(0.0);
        self.pool.stage_m = self.pool.bed_min_m + (self.pool.storage_m3 / area) as f32;

        // Paint inundation from stage onto depth (upstream + overtopping downstream shallow).
        let q_vis = (self.outlet_flow_m3s
            + self.mid_outlet_flow_m3s
            + self.spillway_flow_m3s
            + self.overtopping_flow_m3s
            + self.turbine_flow_m3s
            + self.breach_flow_m3s)
            .max(0.0);
        for i in 0..self.cell_count() {
            let bed = self.bed_m[i];
            let crest = self.hydraulics.barrier_crest_m.get(i).copied().unwrap_or(0.0);
            let lz = (i / self.width.max(1) as usize) as u32;
            if self.is_upstream_lz(lz) {
                self.depth_m[i] = (self.pool.stage_m - bed).max(0.0);
                self.qy_m3s[i] = 0.0;
            } else if crest > 0.0 {
                // Barrier cell: keep small depth if overtopping.
                self.depth_m[i] = (self.pool.stage_m - crest).max(0.0).min(2.0);
                self.qy_m3s[i] = 0.0;
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
                    let lx = (i % self.width.max(1) as usize) as i32;
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
        let conductance = |h_a: f32, h_b: f32| -> f32 {
            // Upwind depth so dry-bed wetting is not starved by a zero neighbor.
            let hp = h_a.max(h_b);
            if hp < 1e-4 {
                0.0
            } else {
                // Gravity-wave scale: C = k·Δx·√(g·h) → volume flux = C·Δη.
                VP_CONDUCTANCE_K * dx * (G_MPS2 * hp).sqrt()
            }
        };

        for z in 0..h {
            for x in 0..w {
                let i = z * w + x;
                let eta_i = self.surface_at(i);
                let h_i = self.depth_m[i];
                let crest_i = self.hydraulics.barrier_crest_m[i];
                // East neighbor
                if x + 1 < w {
                    let j = i + 1;
                    let mut eta_j = self.surface_at(j);
                    let h_j = self.depth_m[j];
                    // Barrier blocks unless stage above crest.
                    let crest = crest_i.max(self.hydraulics.barrier_crest_m[j]);
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
                    let crest = crest_i.max(self.hydraulics.barrier_crest_m[j]);
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
                    let q_free = conductance(h0, h0) * h0;
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
                let _ = self.step_level_pool(
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
                let crest = self
                    .hydraulics
                    .barrier_crest_m
                    .iter()
                    .copied()
                    .fold(0.0_f32, f32::max);
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
                self.spillway_flow_m3s =
                    self.hydraulics.spillway_flow_m3s(head_crest, gate_open01);
                self.overtopping_flow_m3s = self.hydraulics.overtopping_flow_m3s(head_crest);
                self.turbine_flow_m3s = turbine_setpoint_m3s
                    .clamp(0.0, self.hydraulics.turbine_design_m3s.max(0.0));

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
        field.pool.storage_m3 = 500_000.0;
        field.pool.stage_m = 12.0;
        field.step_level_pool(30.0, 0.0, 0.0, 0.0, 0.0);
        assert!(
            field.overtopping_flow_m3s > 5.0,
            "stage above crest with shut gates must overtop (got {})",
            field.overtopping_flow_m3s
        );
        assert!(
            field.outlet_flow_m3s < 0.5 && field.spillway_flow_m3s < 0.5,
            "controlled releases stay shut during force-close overtop"
        );
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
        field.pool.storage_m3 = 80_000.0;
        field.pool.stage_m = 8.0;
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
}
