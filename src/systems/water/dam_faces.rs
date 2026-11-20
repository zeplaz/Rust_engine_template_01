//! WM-3 — each dam's own control and its faces in the solver.
//!
//! A [`DamSite`] carries its own [`DamOps`] (gates, outlets, turbines). Its outlets, spillways and
//! turbines are not numbers that something teleports water for: they are pipes between the cell
//! above the dam and the cell below it, driven by the local head across the face with the orifice /
//! weir laws of [`BarrierHydraulics::face_flows`], and applied inside the 2D kernel
//! ([`super::step_kernel`]) with the same donor limiter as every other face. What reaches the river
//! below is exactly what left the reservoir.

use bevy::prelude::*;

use super::surface_water::{
    BarrierHydraulics, SurfaceWaterField, SPILLWAY_FREEBOARD_M,
};
use super::tiles::{WaterTileGrid, EDGE_DRY_M};
use super::BarrierControlIntent;

/// Dam cells whose bed is within this of the dam's invert are the channel (the river's own bed):
/// the outlet works discharge there.
pub const CHANNEL_BAND_M: f32 = 1.0;
/// Mid-level outlet centreline as a share of the dam height above the invert.
pub const MID_OUTLET_HEIGHT_FRAC: f32 = 0.4;
/// Turbines draw only while the head over the powerhouse intake is at least this (m).
pub const TURBINE_MIN_HEAD_M: f32 = 0.5;
/// Rows searched up / down from a dam cell for the open cell on each side of a thick dam.
const FACE_SEARCH_CELLS: usize = 8;
/// A source pipe (far-field release) has no donor cell in the field.
pub(super) const NO_DONOR: usize = usize::MAX;

/// Operating state of one dam's outlet works.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DamOps {
    /// Gated spillway bays 0..1.
    pub gate_open01: f32,
    /// Low-level / bottom outlet 0..1.
    pub outlet_open01: f32,
    /// Mid-level outlet 0..1.
    pub mid_outlet_open01: f32,
    /// Effective turbine discharge (m3/s), availability already applied.
    pub turbine_m3s: f32,
}

impl DamOps {
    /// Every opening shut, no turbine.
    #[must_use]
    pub fn shut(&self) -> bool {
        self.outlet_open01 < 0.05
            && self.mid_outlet_open01 < 0.05
            && self.gate_open01 < 0.05
            && self.turbine_m3s < 0.5
    }

    /// Controlled releases at most a trickle (the reservoir is filling).
    #[must_use]
    pub fn mostly_shut(&self) -> bool {
        self.outlet_open01.max(self.mid_outlet_open01) <= 0.2
            && self.gate_open01 <= 0.2
            && self.turbine_m3s < 1.0
    }
}

impl From<&BarrierControlIntent> for DamOps {
    fn from(c: &BarrierControlIntent) -> Self {
        Self {
            gate_open01: c.gate_open01.clamp(0.0, 1.0),
            outlet_open01: c.outlet_open01.clamp(0.0, 1.0),
            mid_outlet_open01: c.mid_outlet_open01.clamp(0.0, 1.0),
            turbine_m3s: c.effective_turbine_m3s().max(0.0),
        }
    }
}

/// Discharge through one face (or a whole dam), by path.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FaceFlows {
    pub outlet: f32,
    pub mid: f32,
    pub spill: f32,
    pub turbine: f32,
    /// Over the crest (embankment overtopping); only the far-field release carries it here, the
    /// 2D solver moves crest flow through the wall faces themselves.
    pub over: f32,
}

impl FaceFlows {
    #[must_use]
    pub fn total(&self) -> f32 {
        self.outlet + self.mid + self.spill + self.turbine + self.over
    }

    fn scaled(&self, s: f32) -> Self {
        Self {
            outlet: self.outlet * s,
            mid: self.mid * s,
            spill: self.spill * s,
            turbine: self.turbine * s,
            over: self.over * s,
        }
    }
}

/// Released discharge of one dam over the last step, and its head (what hydro and the dump read).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DamFlows {
    pub outlet_m3s: f32,
    pub mid_outlet_m3s: f32,
    /// Gated + emergency spillway over the sill.
    pub spillway_m3s: f32,
    /// Over the embankment crest (the 2D wall flux).
    pub overtop_m3s: f32,
    pub turbine_m3s: f32,
    /// Highest water surface right above the dam (m); the invert when dry.
    pub up_surface_m: f32,
    /// Gross head over the powerhouse intake: `up_surface - invert` (m).
    pub head_m: f32,
}

impl DamFlows {
    #[must_use]
    pub fn released_m3s(&self) -> f32 {
        self.outlet_m3s + self.mid_outlet_m3s + self.spillway_m3s + self.overtop_m3s + self.turbine_m3s
    }
}

/// Elevations a dam's openings sit at.
#[derive(Clone, Copy, Debug)]
pub struct DamGeometry {
    pub invert_m: f32,
    pub mid_sill_m: f32,
    /// Spillway sill (crest minus the freeboard); infinite without a crest.
    pub sill_m: f32,
}

impl DamGeometry {
    /// Lowest opening in use: the surface a pipe can drain its upstream cell down to.
    fn floor_for(&self, f: &FaceFlows) -> f32 {
        if f.outlet > 0.0 || f.turbine > 0.0 {
            self.invert_m
        } else if f.mid > 0.0 {
            self.mid_sill_m
        } else {
            self.sill_m
        }
    }
}

/// A channel column of a dam: the open cells just above and below its crest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DamFace {
    pub up: usize,
    pub down: usize,
}

/// One solver pipe for the current sub-step.
#[derive(Clone, Copy, Debug)]
pub(super) struct Pipe {
    pub from: usize,
    pub to: usize,
    pub dam: usize,
    pub req: FaceFlows,
    /// Discharge actually moved (after the donor limiter).
    pub q: f32,
}

/// One installed dam: crest cells, per-dam numbers for the column-damage model, its own control
/// and the discharge it released. Runtime only (rebuilt from the crest cells on load).
#[derive(Clone, Debug, Default)]
pub struct DamSite {
    pub cells: Vec<usize>,
    pub invert_m: f32,
    /// Dam height above its invert; 0 = installed as a bare row (no column damage model).
    pub height_m: f32,
    pub damaged: bool,
    pub lz: u32,
    pub owner: Option<Entity>,
    /// This dam's gates / outlets / turbines.
    pub ops: DamOps,
    /// Released discharge of the last step.
    pub flows: DamFlows,
    pub(super) faces: Vec<DamFace>,
    /// Volumes (m3) released since the step began, by path.
    pub(super) acc: DamFlows,
}

impl DamSite {
    #[must_use]
    pub fn new(cells: Vec<usize>, invert_m: f32, height_m: f32, lz: u32, owner: Option<Entity>) -> Self {
        Self { cells, invert_m, height_m, lz, owner, ..Self::default() }
    }

    /// Spill crest of this dam: the lowest standing crest cell (0 when none stands).
    #[must_use]
    pub fn crest_m(&self, barrier_crest_m: &[f32]) -> f32 {
        let c = self
            .cells
            .iter()
            .filter_map(|&i| barrier_crest_m.get(i).copied())
            .filter(|c| *c > 0.0)
            .fold(f32::INFINITY, f32::min);
        if c.is_finite() { c } else { 0.0 }
    }

    #[must_use]
    pub fn geometry(&self, crest_m: f32) -> DamGeometry {
        DamGeometry {
            invert_m: self.invert_m,
            mid_sill_m: self.invert_m + MID_OUTLET_HEIGHT_FRAC * self.height_m.max(0.0),
            sill_m: if crest_m > 0.0 {
                (crest_m - SPILLWAY_FREEBOARD_M).max(self.invert_m)
            } else {
                f32::INFINITY
            },
        }
    }
}

impl BarrierHydraulics {
    /// Discharge through one dam face that carries `share` of the dam's openings, from the local
    /// water surfaces above (`eta_up`) and below (`eta_down`, `-inf` = free discharge).
    ///
    /// Orifices (low + mid outlet): `Cd·A·open·sqrt(2g·h)` with `h` over the higher of the tailwater
    /// and the opening's centreline; a minimum release floors the low outlet. Spillways (gated bay
    /// + the ungated emergency bay) are weirs over the sill, throttled by submergence. A turbine
    /// draws its setpoint while the intake head holds.
    #[must_use]
    pub fn face_flows(
        &self,
        ops: &DamOps,
        geo: &DamGeometry,
        eta_up: f32,
        eta_down: f32,
        share: f32,
    ) -> FaceFlows {
        let tail = |sill: f32| eta_down.max(sill);
        let mut f = FaceFlows::default();
        if ops.outlet_open01 >= 0.05 {
            let head = eta_up - tail(geo.invert_m);
            if head > 0.0 {
                f.outlet = self
                    .outlet_flow_m3s(head, ops.outlet_open01)
                    .max(self.min_release_m3s * ops.outlet_open01.clamp(0.0, 1.0))
                    * share;
            }
        }
        if ops.mid_outlet_open01 >= 0.05 {
            let head = eta_up - tail(geo.mid_sill_m);
            if head > 0.0 {
                f.mid = self.mid_outlet_flow_m3s(head, ops.mid_outlet_open01) * share;
            }
        }
        let h_sill = eta_up - geo.sill_m;
        if h_sill > 0.0 {
            // Villemonte submergence: a drowned weir passes less than a free one.
            let s = ((eta_down - geo.sill_m).max(0.0) / h_sill).min(1.0);
            let free = (1.0 - s.powf(1.5)).max(0.0).powf(0.385);
            f.spill = (self.spillway_flow_m3s(h_sill, ops.gate_open01)
                + self.emergency_spillway_flow_m3s(h_sill))
                * share
                * free;
        }
        if ops.turbine_m3s > 0.0 && eta_up - geo.invert_m > TURBINE_MIN_HEAD_M {
            f.turbine = ops.turbine_m3s.clamp(0.0, self.turbine_design_m3s.max(0.0)) * share;
        }
        f
    }
}

impl SurfaceWaterField {
    /// Set every dam's control: `f` is asked once per dam with its owner entity.
    pub fn set_dam_ops(&mut self, mut f: impl FnMut(Option<Entity>) -> DamOps) {
        for d in &mut self.dams {
            d.ops = f(d.owner);
        }
    }

    /// Control of the dam at `index` (0 = primary).
    #[must_use]
    pub fn dam_ops(&self, index: usize) -> Option<DamOps> {
        self.dams.get(index).map(|d| d.ops)
    }

    /// Released discharge of every installed dam over the last step, with its owner.
    pub fn dam_flows(&self) -> impl Iterator<Item = (Option<Entity>, DamFlows)> + '_ {
        self.dams.iter().map(|d| (d.owner, d.flows))
    }

    /// Discharge of the dam owned by `owner` over the last step.
    #[must_use]
    pub fn dam_flows_of(&self, owner: Entity) -> Option<DamFlows> {
        self.dams.iter().find(|d| d.owner == Some(owner)).map(|d| d.flows)
    }

    /// Rebuild each dam's channel faces from its crest cells (a dam may lose columns).
    pub(super) fn refresh_dam_faces(&mut self) {
        let (w, h) = (self.width.max(1) as usize, self.height as usize);
        for k in 0..self.dams.len() {
            let mut faces: Vec<DamFace> = Vec::new();
            let site = &self.dams[k];
            for &i in &site.cells {
                let standing = self.hydraulics.barrier_crest_m.get(i).is_some_and(|c| *c > 0.0);
                if !standing || self.bed_m[i] > site.invert_m + CHANNEL_BAND_M {
                    continue;
                }
                let (x, z) = (i % w, i / w);
                let open = |j: usize| self.hydraulics.barrier_crest_m[j] <= 0.0;
                let up = (1..=FACE_SEARCH_CELLS)
                    .map(|s| z + s)
                    .take_while(|zz| *zz < h)
                    .map(|zz| zz * w + x)
                    .find(|j| open(*j));
                let down = (1..=FACE_SEARCH_CELLS)
                    .take_while(|s| *s <= z)
                    .map(|s| (z - s) * w + x)
                    .find(|j| open(*j));
                if let (Some(up), Some(down)) = (up, down) {
                    let face = DamFace { up, down };
                    if !faces.contains(&face) {
                        faces.push(face);
                    }
                }
            }
            self.dams[k].faces = faces;
        }
    }

    /// Pipes of the current sub-step. With `far` set, the primary dam's release comes from the
    /// far-field reservoir (no donor cell); every other face is driven by its own local head.
    pub(super) fn collect_dam_pipes(
        &self,
        dt_s: f32,
        tg: &WaterTileGrid,
        far: Option<&FaceFlows>,
        out: &mut Vec<Pipe>,
    ) {
        let w = self.width.max(1) as usize;
        let cell_a = self.dx_m * self.dx_m;
        for (k, d) in self.dams.iter().enumerate() {
            let n = d.faces.len();
            let crest = d.crest_m(&self.hydraulics.barrier_crest_m);
            if n == 0 || crest <= 0.0 {
                continue;
            }
            let geo = d.geometry(crest);
            let share = 1.0 / n as f32;
            let far_release = if k == 0 { far } else { None };
            for f in &d.faces {
                let (tu, td) = (tg.tile_of((f.up % w) as u32, (f.up / w) as u32), tg.tile_of((f.down % w) as u32, (f.down / w) as u32));
                if !tg.is_active(tu) || !tg.is_active(td) {
                    continue;
                }
                if let Some(rel) = far_release {
                    let req = rel.scaled(share);
                    if req.total() > 0.0 {
                        out.push(Pipe { from: NO_DONOR, to: f.down, dam: k, q: req.total(), req });
                    }
                    continue;
                }
                if self.depth_m[f.up] < EDGE_DRY_M {
                    continue;
                }
                let (eta_up, eta_down) = (self.surface_at(f.up), self.surface_at(f.down));
                let mut fl = self.hydraulics.face_flows(&d.ops, &geo, eta_up, eta_down, share);
                let total = fl.total();
                if total <= 0.0 {
                    continue;
                }
                // Explicit-step bound: one sub-step may move at most a quarter of the head across
                // the face, so the two cells never swap sides.
                let cap = cell_a * (eta_up - eta_down.max(geo.floor_for(&fl))).max(0.0) / (4.0 * dt_s);
                if total > cap {
                    fl = fl.scaled(cap / total);
                }
                out.push(Pipe { from: f.up, to: f.down, dam: k, q: fl.total(), req: fl });
            }
        }
    }

    /// Wake the tiles of every face that is passing (or being fed) water this step, so a sleeping
    /// reservoir with an opened gate starts to flow.
    pub(super) fn wake_dam_tiles(&self, tg: &mut WaterTileGrid) {
        let w = self.width.max(1) as usize;
        for (k, d) in self.dams.iter().enumerate() {
            let crest = d.crest_m(&self.hydraulics.barrier_crest_m);
            if d.faces.is_empty() || crest <= 0.0 {
                continue;
            }
            let geo = d.geometry(crest);
            let share = 1.0 / d.faces.len() as f32;
            for f in &d.faces {
                let flowing = if k == 0 && self.far_release.is_some() {
                    true
                } else {
                    self.depth_m[f.up] >= EDGE_DRY_M
                        && self
                            .hydraulics
                            .face_flows(&d.ops, &geo, self.surface_at(f.up), self.surface_at(f.down), share)
                            .total()
                            > 0.0
                };
                if flowing {
                    for c in [f.up, f.down] {
                        let t = tg.tile_of((c % w) as u32, (c / w) as u32);
                        tg.add_base(t);
                    }
                }
            }
        }
    }

    /// Fold the volumes released during this step into per-dam discharge and the field totals.
    /// `dt_basis_s` is the time the accumulated volumes cover.
    pub(super) fn publish_dam_flows(&mut self, dt_basis_s: f32) {
        if self.dams.is_empty() || dt_basis_s <= 0.0 {
            return;
        }
        let inv = 1.0 / dt_basis_s;
        let ups: Vec<f32> = self
            .dams
            .iter()
            .map(|d| {
                d.faces
                    .iter()
                    .filter(|f| self.depth_m[f.up] > 0.05)
                    .map(|f| self.surface_at(f.up))
                    .fold(d.invert_m, f32::max)
            })
            .collect();
        let mut total = DamFlows::default();
        for (d, up) in self.dams.iter_mut().zip(ups) {
            d.flows = DamFlows {
                outlet_m3s: d.acc.outlet_m3s * inv,
                mid_outlet_m3s: d.acc.mid_outlet_m3s * inv,
                spillway_m3s: d.acc.spillway_m3s * inv,
                overtop_m3s: d.acc.overtop_m3s * inv,
                turbine_m3s: d.acc.turbine_m3s * inv,
                up_surface_m: up,
                head_m: (up - d.invert_m).max(0.0),
            };
            total.outlet_m3s += d.flows.outlet_m3s;
            total.mid_outlet_m3s += d.flows.mid_outlet_m3s;
            total.spillway_m3s += d.flows.spillway_m3s;
            total.overtop_m3s += d.flows.overtop_m3s;
            total.turbine_m3s += d.flows.turbine_m3s;
        }
        self.outlet_flow_m3s = total.outlet_m3s;
        self.mid_outlet_flow_m3s = total.mid_outlet_m3s;
        self.spillway_flow_m3s = total.spillway_m3s;
        self.overtopping_flow_m3s = total.overtop_m3s;
        self.turbine_flow_m3s = total.turbine_m3s;
    }

    /// Add the discharge a pipe moved this sub-step (`dt_s` long) to its dam's tally.
    pub(super) fn tally_pipe(&mut self, p: &Pipe, dt_s: f32) {
        let req = p.req.total();
        if req <= 0.0 {
            return;
        }
        let r = (p.q / req).clamp(0.0, 1.0) * dt_s;
        if let Some(d) = self.dams.get_mut(p.dam) {
            d.acc.outlet_m3s += p.req.outlet * r;
            d.acc.mid_outlet_m3s += p.req.mid * r;
            d.acc.spillway_m3s += p.req.spill * r;
            d.acc.turbine_m3s += p.req.turbine * r;
            d.acc.overtop_m3s += p.req.over * r;
        }
    }

    /// Flow over each dam's standing crest this sub-step, from the final face fluxes: water that
    /// leaves a crest cell downstream (`flux_y` below the cell is negative when it goes -z).
    pub(super) fn tally_overtop(&mut self, tg: &WaterTileGrid, flux_y: &[f32], dt_s: f32) {
        let w = self.width.max(1) as usize;
        for k in 0..self.dams.len() {
            let mut over = 0.0_f32;
            for &i in &self.dams[k].cells {
                if i < w || self.hydraulics.barrier_crest_m.get(i).is_none_or(|c| *c <= 0.0) {
                    continue;
                }
                let (x, z) = (i % w, i / w);
                if !tg.is_active(tg.tile_of(x as u32, z as u32))
                    || !tg.is_active(tg.tile_of(x as u32, (z - 1) as u32))
                {
                    continue;
                }
                over += (-flux_y[i - w]).max(0.0);
            }
            self.dams[k].acc.overtop_m3s += over * dt_s;
        }
    }

    /// Zero the per-dam tallies at the start of a step.
    pub(super) fn begin_dam_tally(&mut self) {
        for d in &mut self.dams {
            d.acc = DamFlows::default();
        }
    }

    /// Reservoir: the primary dam is shut in (nothing to release but a full pool's spill) and
    /// intact, so a fast far-field fill may stand in for the 2D solve of its reservoir window.
    #[must_use]
    pub fn far_field_eligible(&self) -> bool {
        self.dams.first().is_some_and(|d| d.ops.mostly_shut())
            && self.has_barrier()
            && self.breach_flow_m3s <= 0.0
            && !self.hydraulics.barrier_damaged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::water::surface_water::{SurfaceWaterMode, FAR_FIELD_2D_DT_CAP_S};

    fn hyd() -> BarrierHydraulics {
        BarrierHydraulics { turbine_design_m3s: 80.0, ..BarrierHydraulics::default_earth_fill(8.0) }
    }

    fn geo() -> DamGeometry {
        DamGeometry { invert_m: 0.0, mid_sill_m: 16.0, sill_m: 37.0 }
    }

    #[test]
    fn faces_split_the_openings_and_obey_the_laws() {
        let h = hyd();
        let ops = DamOps { gate_open01: 0.5, outlet_open01: 0.35, mid_outlet_open01: 0.0, turbine_m3s: 40.0 };
        let whole = h.face_flows(&ops, &geo(), 38.0, f32::NEG_INFINITY, 1.0);
        let half = h.face_flows(&ops, &geo(), 38.0, f32::NEG_INFINITY, 0.5);
        assert!((whole.total() - 2.0 * half.total()).abs() < 1e-3, "flow scales with the share of the openings");
        let want = 0.6 * 8.0 * 0.35 * (2.0 * 9.81 * 38.0_f32).sqrt();
        assert!((whole.outlet - want).abs() / want < 1e-3, "orifice law: {} vs {want}", whole.outlet);
        assert!(whole.spill > 0.0 && whole.turbine == 40.0);
        // Below the sill nothing spills; below the turbine head nothing is drawn.
        let low = h.face_flows(&ops, &geo(), 30.0, f32::NEG_INFINITY, 1.0);
        assert_eq!(low.spill, 0.0);
        assert_eq!(h.face_flows(&ops, &geo(), 0.3, f32::NEG_INFINITY, 1.0).turbine, 0.0);
        // A drowned tailwater throttles the spill.
        let drowned = h.face_flows(&ops, &geo(), 38.0, 37.9, 1.0);
        assert!(drowned.spill < 0.5 * whole.spill);
    }

    #[test]
    fn a_shut_dam_passes_only_the_ungated_spill_over_the_sill() {
        let h = hyd();
        let shut = DamOps::default();
        assert!(shut.shut());
        let below = h.face_flows(&shut, &geo(), 36.0, f32::NEG_INFINITY, 1.0);
        assert_eq!(below.total(), 0.0);
        let above = h.face_flows(&shut, &geo(), 38.0, f32::NEG_INFINITY, 1.0);
        assert!(above.spill > 0.0 && above.outlet == 0.0 && above.mid == 0.0 && above.turbine == 0.0);
    }

    // ---- behaviour on the 2D solver --------------------------------------------------------

    const VW: u32 = 24;
    const H: u32 = 40;
    const DAM_Z: i32 = 10;

    fn owner(k: u64) -> Entity {
        Entity::from_bits(1 << 32 | (k + 7))
    }

    /// `n` side-by-side valleys (+ `extra_w` flat columns): channel column bed 0, terrace 5 m, side
    /// and back walls 100 m (the reservoir never touches the open north edge). Valley `k` is closed
    /// by a dam `crest_h` high across row `DAM_Z`, owned by `owner(k)`.
    fn valleys(n: u32, crest_h: f32, extra_w: u32) -> SurfaceWaterField {
        let w = n * VW + extra_w;
        let mut bed = vec![5.0_f32; (w * H) as usize];
        for z in 0..H {
            for x in 0..n * VW {
                let lx = x % VW;
                bed[(z * w + x) as usize] = if z >= H - 6 || lx < 2 || lx + 2 >= VW {
                    100.0
                } else if lx == VW / 2 {
                    0.0
                } else {
                    5.0
                };
            }
        }
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, w, H, bed, 0.0);
        f.hydraulics.min_release_m3s = 0.0;
        for k in 0..n {
            let tiles: Vec<IVec2> = (0..VW as i32).map(|x| IVec2::new((k * VW) as i32 + x, DAM_Z)).collect();
            f.install_barrier_tiles_for(&tiles, crest_h, Some(owner(u64::from(k))));
        }
        f
    }

    /// Fill valley `k` above its dam up to the water `surface_m` (still water).
    fn fill_reservoir(f: &mut SurfaceWaterField, k: u32, surface_m: f32) {
        for z in (DAM_Z as u32 + 1)..(H - 6) {
            for lx in 2..VW - 2 {
                let i = f.idx(k * VW + lx, z);
                f.depth_m[i] = (surface_m - f.bed_m[i]).max(0.0);
            }
        }
        f.recompute_mass();
        f.wake_all_tiles();
    }

    fn reservoir_m3(f: &SurfaceWaterField, k: u32) -> f64 {
        let a = f64::from(f.dx_m * f.dx_m);
        let mut v = 0.0;
        for z in (DAM_Z as u32 + 1)..H {
            for lx in 0..VW {
                v += f64::from(f.depth_m[f.idx(k * VW + lx, z)]) * a;
            }
        }
        v
    }

    fn below_dam_m3(f: &SurfaceWaterField, k: u32) -> f64 {
        let a = f64::from(f.dx_m * f.dx_m);
        let mut v = 0.0;
        for z in 0..DAM_Z as u32 {
            for lx in 0..VW {
                v += f64::from(f.depth_m[f.idx(k * VW + lx, z)]) * a;
            }
        }
        v
    }

    /// Two dams on one map, same reservoir, different gate settings: each releases by its own
    /// control, so the more open dam drains more and delivers more to its river.
    #[test]
    fn two_dams_with_different_gates_release_different_flows() {
        let mut f = valleys(2, 40.0, 0);
        fill_reservoir(&mut f, 0, 30.0);
        fill_reservoir(&mut f, 1, 30.0);
        let (v0, v1) = (reservoir_m3(&f, 0), reservoir_m3(&f, 1));
        assert!((v0 - v1).abs() < 1e-6 * v0, "same start");
        let open = DamOps { outlet_open01: 0.8, ..DamOps::default() };
        let barely = DamOps { outlet_open01: 0.2, ..DamOps::default() };
        f.set_dam_ops(|o| if o == Some(owner(0)) { open } else { barely });
        for _ in 0..20 {
            f.advance(1.0);
        }
        let (a, b) = (f.dam_flows_of(owner(0)).unwrap(), f.dam_flows_of(owner(1)).unwrap());
        assert!(b.outlet_m3s > 1.0, "the nearly shut dam still passes a little: {}", b.outlet_m3s);
        assert!(a.outlet_m3s > 2.5 * b.outlet_m3s, "outlet flows follow each dam's own gate: {} vs {}", a.outlet_m3s, b.outlet_m3s);
        assert_eq!(a.spillway_m3s + b.spillway_m3s, 0.0, "pool under both sills");
        let (d0, d1) = (v0 - reservoir_m3(&f, 0), v1 - reservoir_m3(&f, 1));
        assert!(d0 > 2.0 * d1 && d1 > 0.0, "the open dam drained more: {d0} vs {d1}");
        assert!(below_dam_m3(&f, 0) > below_dam_m3(&f, 1), "and fed its river more");
        assert!(f.mass_error_frac < 1e-3, "mass kept: {}", f.mass_error_frac);
    }

    /// Gates shut, bottom outlet open: the water goes through the outlet face only; the crest and
    /// the spillway see none of it.
    #[test]
    fn an_open_outlet_passes_water_through_the_outlet_face_only() {
        let mut f = valleys(1, 40.0, 0);
        fill_reservoir(&mut f, 0, 30.0);
        let v0 = reservoir_m3(&f, 0);
        f.set_dam_ops(|_| DamOps { outlet_open01: 0.4, ..DamOps::default() });
        let crest_cells: Vec<usize> = f.dams[0].cells.clone();
        for _ in 0..20 {
            f.advance(1.0);
            assert_eq!(f.dam_flows_of(owner(0)).unwrap().overtop_m3s, 0.0, "nothing over the crest");
            assert!(crest_cells.iter().all(|&i| f.depth_m[i] == 0.0), "no water stands on the dam body");
        }
        let fl = f.dam_flows_of(owner(0)).unwrap();
        let want = 0.6 * 8.0 * 0.4 * (2.0 * 9.81 * 30.0_f32).sqrt();
        assert!(fl.outlet_m3s > 0.5 * want && fl.outlet_m3s < 1.2 * want, "orifice flow {} ~ {want}", fl.outlet_m3s);
        assert_eq!(fl.spillway_m3s + fl.mid_outlet_m3s + fl.turbine_m3s, 0.0);
        assert!(reservoir_m3(&f, 0) < v0, "the reservoir drained");
        assert!(below_dam_m3(&f, 0) > 100.0, "the river below got what left the dam");
        let tiles: Vec<IVec2> = (0..VW as i32).map(|x| IVec2::new(x, DAM_Z)).collect();
        assert!(crate::construction::dam_face_velocities_m_s(&f, &tiles).iter().all(|v| *v == 0.0));
    }

    /// A full reservoir behind shut gates spills at the sill (crest minus freeboard) through the
    /// ungated bay; the embankment crest is never overtopped.
    #[test]
    fn a_full_dam_spills_at_the_sill_not_over_the_crest() {
        let crest_h = 10.0;
        let mut f = valleys(1, crest_h, 0);
        let sill = crest_h - SPILLWAY_FREEBOARD_M;
        fill_reservoir(&mut f, 0, 9.0); // between the sill (7 m) and the crest (10 m)
        f.set_dam_ops(|_| DamOps::default());
        let up = f.idx(VW / 2, DAM_Z as u32 + 1);
        let mut peak = 0.0_f32;
        for k in 0..40 {
            f.advance(1.0);
            let fl = f.dam_flows_of(owner(0)).unwrap();
            peak = peak.max(f.surface_at(up));
            assert_eq!(fl.overtop_m3s, 0.0, "step {k}: crest overtopped");
            if k == 0 {
                assert!(fl.spillway_m3s > 10.0, "spills over the sill at once: {}", fl.spillway_m3s);
                assert_eq!(fl.outlet_m3s + fl.mid_outlet_m3s + fl.turbine_m3s, 0.0, "controlled outlets stay shut");
            }
        }
        assert!(peak < crest_h, "the surface stayed under the crest: {peak}");
        let fl = f.dam_flows_of(owner(0)).unwrap();
        // The spill is tens of m3/s off a pool of ~10^5 m2, so in 40 s it takes centimetres off the
        // pool, not metres: the surface stays between the sill and where it started.
        assert!(f.surface_at(up) < 9.0 && f.surface_at(up) > sill, "the spill drew the pool down toward the sill: {}", f.surface_at(up));
        assert!(fl.up_surface_m < crest_h);
        assert!(below_dam_m3(&f, 0) > 100.0, "the spill reached the river below");
    }

    /// Time skip: the reservoir window is filled by the level pool, but the rest of the map keeps
    /// stepping in 2D — a river elsewhere still gets its inflow and carries it, and what the dam
    /// releases is delivered at its toe, not painted.
    #[test]
    fn under_time_skip_the_rest_of_the_map_still_flows() {
        let mut f = valleys(1, 40.0, 64);
        // Beyond the dam's reservoir window (48 cells either side of its footprint).
        let (river_x0, river_x1) = (VW + 56, VW + 60);
        for z in 0..H {
            for x in river_x0..river_x1 {
                let i = f.idx(x, z);
                f.bed_m[i] = 0.0;
                f.depth_m[i] = 1.0;
                f.ref_depth_m[i] = 1.0;
            }
        }
        // 40 m3/s enters the strip at its upstream edge.
        let head: Vec<(u32, f32)> =
            (river_x0..river_x1).map(|x| (f.idx(x, H - 1) as u32, 40.0 / (river_x1 - river_x0) as f32)).collect();
        f.set_river_sources(&head);
        f.recompute_mass();
        f.wake_all_tiles();
        f.set_dam_ops(|_| DamOps::default());
        let strip = |f: &SurfaceWaterField| -> f64 {
            let mut v = 0.0;
            for z in 0..H {
                for x in river_x0..river_x1 {
                    v += f64::from(f.depth_m[f.idx(x, z)]);
                }
            }
            v * f64::from(f.dx_m * f.dx_m)
        };
        let before = strip(&f);
        f.mode = SurfaceWaterMode::LevelPool;
        assert!(f.far_field_eligible());
        f.advance(600.0);
        assert!(f.pool.storage_m3 > 5.0e3, "the reservoir window was filled by the pool: {}", f.pool.storage_m3);
        let gained = strip(&f) - before;
        assert!(
            gained > 0.5 * 40.0 * f64::from(FAR_FIELD_2D_DT_CAP_S),
            "the river kept its inflow under time skip: +{gained} m3"
        );
        let qy: f32 = (0..H).map(|z| f.qy_m3s[f.idx(river_x0 + 1, z)].abs()).fold(0.0, f32::max);
        assert!(qy > 0.05, "the river carries flow: {qy}");
        assert!(f.tile_stats().1 > 0, "tiles outside the window ran");
        assert!(f.mass_error_frac < 1e-3, "mass kept: {}", f.mass_error_frac);
    }

    /// A full dam in far-field mode releases over its sill: that water enters the river at the toe
    /// as a source (nothing painted), and the books still balance.
    #[test]
    fn far_field_release_arrives_below_the_dam_as_real_water() {
        let mut f = valleys(1, 40.0, 0);
        f.inflow_m3s = 100.0;
        f.set_dam_ops(|_| DamOps::default());
        f.mode = SurfaceWaterMode::LevelPool;
        for _ in 0..40 {
            f.advance(24_000.0);
            assert!(f.mass_error_frac < 1e-3, "mass kept: {}", f.mass_error_frac);
        }
        let fl = f.dam_flows_of(owner(0)).unwrap();
        assert!((fl.spillway_m3s - 100.0).abs() < 8.0, "the full pool passes its inflow over the sill: {}", fl.spillway_m3s);
        assert_eq!(fl.overtop_m3s, 0.0);
        assert!(below_dam_m3(&f, 0) > 10.0, "the toe holds the released water");
    }
}
