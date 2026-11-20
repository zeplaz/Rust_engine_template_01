//! Rain, infiltration and evaporation of the map-wide field (WM-2).
//!
//! Rain is per cell, from the weather field's rain rate over that cell (never a map-wide mean).
//! Water that falls on soil first fills the ground: each cell has a soil store (class from its
//! terrain family, resting level from its moisture) that takes water at the class's infiltration
//! rate, slower as it fills. What the ground cannot take stays on the surface as depth, where the
//! solver runs it downhill into low spots and rivers; where the ground is saturated, rock or paved
//! it runs off at once, where it is dry and porous it disappears. Standing water above a cell's
//! baseline evaporates, the soil drains back to its resting level.
//!
//! Sleep: this pass is the only thing that touches a sleeping tile. Rain wakes a tile only while it
//! delivers meaningful excess water ([`RAIN_WAKE_MM_H`], or [`DRIFT_REPORT_M`] of accumulated
//! water); a drizzle the ground absorbs, a puddle drying and a soil draining change depth in place
//! on a sleeping tile, on a slow cadence ([`SLOW_PASS_S`]), and only mark it dirty for the render.

use bevy::prelude::*;

use crate::systems::weather::WeatherField;

use super::sources::BASELINE_WATER_MIN_M;
use super::surface_water::SurfaceWaterField;

/// Weather cells raining less than this (mm/h) deliver nothing (below evaporation).
pub const RAIN_MIN_MM_H: f32 = 0.1;
/// Net surface gain (mm/h) at which a sleeping tile wakes to let the water run.
pub const RAIN_WAKE_MM_H: f32 = 0.5;
/// Accumulated change (m) on a tile that is reported to the render / wakes a slowly gaining tile.
pub const DRIFT_REPORT_M: f32 = 1.0e-3;
/// Cadence (sim s) of the pass over tiles that are not raining but still hold loose water.
pub const SLOW_PASS_S: f32 = 30.0;
/// Evaporation of standing water above a cell's baseline (mm/h).
pub const EVAP_MM_H: f32 = 0.25;
/// Soil store drain back to its resting level: evapotranspiration + percolation (mm/h).
pub const SOIL_DRAIN_MM_H: f32 = 0.08;
/// Resting soil store as a share of capacity at moisture 1.
const SOIL_REST_SHARE: f32 = 0.7;
/// Soil store above resting level by less than this (mm) counts as settled.
const SOIL_SETTLED_MM: f32 = 1.0e-2;
/// Standing water above baseline under this (m) counts as settled.
const SURFACE_SETTLED_M: f32 = 1.0e-6;

const MM_H_TO_M_S: f32 = 1.0 / 3.6e6;

/// What the ground under a cell is, for the water that falls on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SoilClass {
    /// Bare rock, cliffs, mountains.
    Rock,
    /// Concrete / roads: sheds nearly everything.
    Paved,
    /// Swamp, tundra, snow: slow, small store.
    Clay,
    #[default]
    Loam,
    /// Forest litter: large store, fast intake.
    Forest,
    /// Beach, desert: drinks fast, holds little.
    Sand,
}

impl SoilClass {
    /// Infiltration rate of dry ground (mm/h).
    #[must_use]
    pub const fn ksat_mm_h(self) -> f32 {
        match self {
            Self::Rock => 1.0,
            Self::Paved => 0.3,
            Self::Clay => 4.0,
            Self::Loam => 15.0,
            Self::Forest => 30.0,
            Self::Sand => 80.0,
        }
    }

    /// Water the ground can store (mm of depth).
    #[must_use]
    pub const fn capacity_mm(self) -> f32 {
        match self {
            Self::Rock => 4.0,
            Self::Paved => 1.0,
            Self::Clay => 40.0,
            Self::Loam => 60.0,
            Self::Forest => 100.0,
            Self::Sand => 50.0,
        }
    }

    /// Class of a terrain family (by its registry name).
    #[must_use]
    pub fn from_family_name(name: &str) -> Self {
        match name {
            "Mountain" | "SnowCappedMountain" | "Cliff" | "Stone" => Self::Rock,
            "Concrete" => Self::Paved,
            "Swamp" | "Tundra" | "Snow" | "ShallowWater" | "DeepWater" => Self::Clay,
            "Forest" | "DenseForest" => Self::Forest,
            "Beach" | "Desert" => Self::Sand,
            _ => Self::Loam,
        }
    }

    fn resting_mm(self, moisture01: f32) -> f32 {
        self.capacity_mm() * SOIL_REST_SHARE * moisture01.clamp(0.0, 1.0)
    }
}

/// Runtime rain / soil state of a field (not saved: rebuilt from the world on load).
#[derive(Clone, Debug, Default)]
pub struct RainSoil {
    coarse_dims: (u32, u32),
    /// Weather cells per field cell.
    coarse_per_cell: f32,
    coarse_mm_h: Vec<f32>,
    max_mm_h: f32,
    pub(super) weather_revision: Option<u64>,
    soil: Vec<SoilClass>,
    soil_mm: Vec<f32>,
    rest_mm: Vec<f32>,
    /// Per tile: holds water the slow pass still has to settle.
    tile_loose: Vec<bool>,
    /// Per tile: change since the render / the solver last heard of it (m).
    tile_drift_m: Vec<f32>,
    acc_s: f32,
}

impl RainSoil {
    /// Rain (mm/h) over the weather cell holding world cell `(wx, wz)`.
    fn coarse_at(&self, wx: f32, wz: f32) -> f32 {
        let (cw, ch) = self.coarse_dims;
        if cw == 0 || ch == 0 || self.coarse_mm_h.len() != (cw * ch) as usize {
            return 0.0;
        }
        let cx = (((wx + 0.5) * self.coarse_per_cell) as i64).clamp(0, i64::from(cw) - 1);
        let cz = (((wz + 0.5) * self.coarse_per_cell) as i64).clamp(0, i64::from(ch) - 1);
        self.coarse_mm_h[(cz as u32 * cw + cx as u32) as usize]
    }

    /// Highest rain (mm/h) over the weather cells a world-cell rectangle covers.
    fn rect_max(&self, wx0: f32, wz0: f32, wx1: f32, wz1: f32) -> f32 {
        let (cw, ch) = self.coarse_dims;
        if cw == 0 || ch == 0 || self.coarse_mm_h.len() != (cw * ch) as usize {
            return 0.0;
        }
        let idx = |v: f32, n: u32| (((v + 0.5) * self.coarse_per_cell) as i64).clamp(0, i64::from(n) - 1) as u32;
        let (x0, x1, z0, z1) = (idx(wx0, cw), idx(wx1, cw), idx(wz0, ch), idx(wz1, ch));
        let mut m = 0.0_f32;
        for z in z0..=z1 {
            for x in x0..=x1 {
                m = m.max(self.coarse_mm_h[(z * cw + x) as usize]);
            }
        }
        m
    }
}

impl SurfaceWaterField {
    /// Size the soil store for the field (default loam at 45 % moisture). Called by `allocate`.
    pub(super) fn reset_rain_soil(&mut self) {
        let n = self.cell_count();
        let nt = self.tiles.tile_count();
        let loam = SoilClass::Loam;
        self.rain = RainSoil {
            soil: vec![loam; n],
            soil_mm: vec![loam.resting_mm(0.45); n],
            rest_mm: vec![loam.resting_mm(0.45); n],
            tile_loose: vec![false; nt],
            tile_drift_m: vec![0.0; nt],
            ..RainSoil::default()
        };
    }

    /// Soil of the whole map: class and resting moisture (0..1) per field cell. The store starts
    /// at its resting level.
    pub fn set_soil(&mut self, classes: &[SoilClass], moisture01: &[f32]) {
        let n = self.cell_count();
        if classes.len() != n || moisture01.len() != n {
            return;
        }
        self.rain.soil = classes.to_vec();
        self.rain.rest_mm = classes.iter().zip(moisture01).map(|(c, m)| c.resting_mm(*m)).collect();
        self.rain.soil_mm = self.rain.rest_mm.clone();
    }

    /// Rain rate (mm/h) per weather cell, `dims` cells of `cell_size_m` metres from the map origin.
    pub fn set_rain_grid(&mut self, dims: (u32, u32), cell_size_m: f32, mm_h: Vec<f32>) {
        if dims.0 == 0 || dims.1 == 0 || mm_h.len() != (dims.0 * dims.1) as usize || cell_size_m <= 0.0 {
            self.rain.coarse_dims = (0, 0);
            self.rain.coarse_mm_h.clear();
            self.rain.max_mm_h = 0.0;
            return;
        }
        self.rain.coarse_dims = dims;
        self.rain.coarse_per_cell = self.dx_m / cell_size_m;
        self.rain.max_mm_h = mm_h.iter().copied().fold(0.0, f32::max);
        self.rain.coarse_mm_h = mm_h;
    }

    /// Highest rain over the map (mm/h).
    #[must_use]
    pub fn rain_max_mm_h(&self) -> f32 {
        self.rain.max_mm_h
    }

    /// Soil store (mm) of field cell `i`.
    #[must_use]
    pub fn soil_store_mm(&self, i: usize) -> f32 {
        self.rain.soil_mm.get(i).copied().unwrap_or(0.0)
    }

    /// Tiles whose loose water (puddles, wet soil) the slow pass is still settling.
    #[must_use]
    pub fn loose_tile_count(&self) -> usize {
        self.rain.tile_loose.iter().filter(|l| **l).count()
    }

    /// One rain / infiltration / evaporation pass over `dt_s` sim seconds. Returns the volume it
    /// added to (+) or took from (-) the field (m3), for the conservation books.
    ///
    /// Raining tiles run every call; tiles that only hold loose water run on the slow cadence with
    /// the accumulated time. Barrier crests and structures shed no rain onto themselves.
    pub(super) fn apply_surface_sources(&mut self, dt_s: f32) -> f64 {
        let n = self.cell_count();
        if !self.wired || dt_s <= 0.0 || self.rain.soil_mm.len() != n {
            return 0.0;
        }
        let nt = self.tiles.tile_count();
        if self.rain.tile_loose.len() != nt {
            self.rain.tile_loose = vec![false; nt];
            self.rain.tile_drift_m = vec![0.0; nt];
        }
        self.rain.acc_s += dt_s;
        let raining = self.rain.max_mm_h >= RAIN_MIN_MM_H;
        let slow_due = self.rain.acc_s >= SLOW_PASS_S;
        if !raining && !slow_due {
            return 0.0;
        }
        let slow_dt = self.rain.acc_s;
        if slow_due {
            self.rain.acc_s = 0.0;
        }

        let w = self.width as usize;
        let cell_a = self.dx_m * self.dx_m;
        let (ox, oz) = (self.origin_x as f32, self.origin_z as f32);
        let mut applied = 0.0_f64;
        let mut runoff_m3 = 0.0_f64;
        let mut rained = false;
        for t in 0..nt {
            let (x0, z0, x1, z1) = self.tiles.tile_rect(t);
            let tile_rain = if raining {
                self.rain.rect_max(ox + x0 as f32, oz + z0 as f32, ox + (x1 - 1) as f32, oz + (z1 - 1) as f32)
            } else {
                0.0
            };
            let tile_raining = tile_rain >= RAIN_MIN_MM_H;
            let dt = if tile_raining {
                dt_s
            } else if slow_due && self.rain.tile_loose[t] {
                slow_dt
            } else {
                continue;
            };
            rained |= tile_raining;
            let (mut gain_max, mut abs_max, mut loose) = (0.0_f32, 0.0_f32, false);
            let mut vol = 0.0_f64;
            for z in z0..z1 {
                for x in x0..x1 {
                    let i = z * w + x;
                    if self.hydraulics.barrier_crest_m[i] > 0.0 || self.obstacle_top_m[i] > 0.0 {
                        continue;
                    }
                    let rain_mm_h = if tile_raining {
                        let r = self.rain.coarse_at(ox + x as f32, oz + z as f32);
                        if r >= RAIN_MIN_MM_H {
                            r
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    let d0 = self.depth_m[i];
                    let base = self.ref_depth_m.get(i).copied().unwrap_or(0.0);
                    let rain_m = rain_mm_h * MM_H_TO_M_S * dt;
                    let mut surf = d0 + rain_m;
                    let mut infil_m = 0.0_f32;
                    if base <= BASELINE_WATER_MIN_M {
                        let class = self.rain.soil[i];
                        let cap = class.capacity_mm();
                        let mut store = self.rain.soil_mm[i];
                        let sat = (store / cap).clamp(0.0, 1.0);
                        let infil_mm = (class.ksat_mm_h() * (1.0 - sat) * dt / 3600.0)
                            .min(cap - store)
                            .min(surf * 1000.0)
                            .max(0.0);
                        store += infil_mm;
                        infil_m = infil_mm / 1000.0;
                        surf -= infil_m;
                        let rest = self.rain.rest_mm[i];
                        if store > rest {
                            store -= (SOIL_DRAIN_MM_H * dt / 3600.0).min(store - rest);
                        }
                        self.rain.soil_mm[i] = store;
                        loose |= store > rest + SOIL_SETTLED_MM;
                    }
                    let excess = (surf - base).max(0.0);
                    surf -= (EVAP_MM_H * MM_H_TO_M_S * dt).min(excess);
                    loose |= surf - base > SURFACE_SETTLED_M;
                    let surf = surf.max(0.0);
                    let delta = surf - d0;
                    if delta != 0.0 {
                        self.depth_m[i] = surf;
                        vol += f64::from(delta);
                        gain_max = gain_max.max(delta);
                        abs_max = abs_max.max(delta.abs());
                    }
                    runoff_m3 += f64::from((rain_m - infil_m) * cell_a);
                }
            }
            applied += vol * f64::from(cell_a);
            self.rain.tile_loose[t] = loose;
            let drift = &mut self.rain.tile_drift_m[t];
            *drift += abs_max;
            if gain_max / dt >= RAIN_WAKE_MM_H * MM_H_TO_M_S || (gain_max > 0.0 && *drift >= DRIFT_REPORT_M) {
                *drift = 0.0;
                self.tiles.touch_tile(t);
            } else if *drift >= DRIFT_REPORT_M {
                *drift = 0.0;
                self.tiles.mark_tile_dirty(t);
            }
        }
        self.rain_runoff_m3s = if rained { (runoff_m3 / f64::from(dt_s)) as f32 } else { 0.0 };
        applied
    }
}

/// Publish the weather field's rain to the water field (sole weather -> water hand-off): rain that
/// is not snow, per weather cell, whenever the weather solved again or the field was just seeded.
pub fn sample_weather_rain_system(weather: Option<Res<WeatherField>>, mut field: ResMut<SurfaceWaterField>) {
    let Some(weather) = weather.filter(|w| w.is_ready()) else {
        return;
    };
    if !field.wired || field.rain.weather_revision == Some(weather.revision) {
        return;
    }
    let field = field.bypass_change_detection();
    let next = &weather.next;
    let n = weather.cell_count();
    let mm_h: Vec<f32> = (0..n)
        .map(|i| {
            let snow = next.temp_c.get(i).is_some_and(|t| *t < 0.0);
            if snow {
                0.0
            } else {
                next.rain_mm_h.get(i).copied().unwrap_or(0.0).max(0.0)
            }
        })
        .collect();
    field.set_rain_grid((weather.dims.x, weather.dims.y), weather.cell_size_m, mm_h);
    field.rain.weather_revision = Some(weather.revision);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::water::WATER_SLEEP_STEPS;

    const S: u32 = 64;

    fn flat_map(bed: impl Fn(u32, u32) -> f32) -> SurfaceWaterField {
        let mut b = vec![0.0_f32; (S * S) as usize];
        for z in 0..S {
            for x in 0..S {
                b[(z * S + x) as usize] = bed(x, z);
            }
        }
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, S, S, b, 0.0);
        f
    }

    /// Uniform rain over the whole map (one weather cell of 2 km).
    fn rain(f: &mut SurfaceWaterField, mm_h: f32) {
        f.set_rain_grid((1, 1), 2000.0, vec![mm_h]);
    }

    fn set_class(f: &mut SurfaceWaterField, class: SoilClass, moisture: f32) {
        let n = f.cell_count();
        f.set_soil(&vec![class; n], &vec![moisture; n]);
    }

    fn total_m3(f: &SurfaceWaterField) -> f64 {
        f.depth_m.iter().map(|d| f64::from(*d)).sum::<f64>() * f64::from(f.dx_m * f.dx_m)
    }

    /// Run `secs` sim seconds in 1 s steps; returns the volume the pass added (+) / took (-).
    fn run(f: &mut SurfaceWaterField, secs: u32) -> f64 {
        let mut net = 0.0;
        for _ in 0..secs {
            let before = total_m3(f);
            f.advance(1.0);
            net += total_m3(f) - before;
        }
        net
    }

    /// Rain on dry porous ground is drunk by the soil; the same rain on saturated ground stays on
    /// the surface.
    #[test]
    fn soil_drinks_rain_until_it_is_full() {
        let mut dry = flat_map(|_, _| 0.0);
        set_class(&mut dry, SoilClass::Loam, 0.0);
        rain(&mut dry, 5.0);
        run(&mut dry, 600);
        let surface_dry = total_m3(&dry);
        assert!(surface_dry < 1e-3, "dry loam took 0.83 mm of rain into the ground: {surface_dry} m3 left on top");
        assert!(dry.soil_store_mm(100) > 0.5, "the soil store filled: {}", dry.soil_store_mm(100));

        // A rim of high ground keeps the rain from draining off the open map edge.
        let mut wet = flat_map(|x, z| if x < 3 || z < 3 || x >= S - 3 || z >= S - 3 { 50.0 } else { 0.0 });
        set_class(&mut wet, SoilClass::Loam, 1.0);
        wet.rain.soil_mm.fill(SoilClass::Loam.capacity_mm());
        rain(&mut wet, 5.0);
        run(&mut wet, 600);
        let want = 5.0 / 3.6e6 * 600.0 * f64::from(wet.dx_m * wet.dx_m) * f64::from(S * S);
        assert!(total_m3(&wet) > 0.8 * want, "saturated ground keeps the rain on top: {} of {want}", total_m3(&wet));
        assert!(wet.mass_error_frac < 1e-3, "books balance: {}", wet.mass_error_frac);
    }

    /// Rain on a basin ponds in the low spot; rain on a slope runs off downhill instead.
    #[test]
    fn rain_ponds_in_a_basin_and_runs_off_a_slope() {
        // Slope falls toward z = 0 (0.4 m per tile) into a closed bowl (x 28..36, z 26..34) whose
        // rim is the slope itself; the map edge drains the rest.
        let mut f = flat_map(|x, z| {
            let slope = 0.4 * z as f32 + 3.0;
            let bowl = (28..36).contains(&x) && (26..34).contains(&z);
            if bowl {
                slope - 4.0
            } else {
                slope
            }
        });
        set_class(&mut f, SoilClass::Clay, 1.0);
        f.rain.soil_mm.fill(SoilClass::Clay.capacity_mm());
        rain(&mut f, 40.0);
        run(&mut f, 1800);
        let bowl_mean: f32 = (26..34)
            .flat_map(|z| (28..36).map(move |x| (x, z)))
            .map(|(x, z)| f.depth_m[f.idx(x, z)])
            .sum::<f32>()
            / 64.0;
        assert!(bowl_mean > 0.05, "the rain gathered in the bowl: mean {bowl_mean} m");
        // A slope cell far from the bowl holds only the thin sheet in transit: the Manning normal
        // depth of the rain falling on the catchment above it (all of it runs off, clay is full).
        let slope_cell = f.depth_m[f.idx(10, 50)];
        let q_sheet = 40.0 / 3.6e6 * f.dx_m * (S - 50) as f32; // m2/s per metre of width
        let sheet_m = (q_sheet * crate::systems::water::step_kernel::MANNING_LAND / (0.4 / f.dx_m).sqrt()).powf(0.6);
        assert!(slope_cell < 1.25 * sheet_m, "the slope runs off: {slope_cell} m vs sheet {sheet_m} m");
        // More than the bowl's own catchment ended up in it: it drains the slope above it.
        let rain_on_bowl = 40.0 / 3.6e6 * 1800.0;
        assert!(bowl_mean > 3.0 * rain_on_bowl, "the bowl is fed by the slope above it, not only by its own rain");
    }

    /// Rain runs off a slope into a river and the river carries it to the open map edge.
    #[test]
    fn runoff_reaches_the_river_and_leaves_by_the_open_edge() {
        let mut f = flat_map(|x, z| {
            let valley = (x as f32 - 32.0).abs() * 0.5;
            valley + 0.2 * z as f32
        });
        set_class(&mut f, SoilClass::Rock, 0.5);
        rain(&mut f, 30.0);
        run(&mut f, 900);
        let channel = f.depth_m[f.idx(32, 30)];
        let bank = f.depth_m[f.idx(20, 30)];
        assert!(channel > 5.0 * bank.max(1e-4), "the valley floor collects it: {channel} vs bank {bank}");
        let river_flux: f32 = (31..34).map(|x| f.qy_m3s[f.idx(x, 20)].abs()).sum();
        assert!(river_flux > 0.2, "and carries it down the valley: {river_flux} m3/s");
    }

    /// A dry map goes to sleep after the rain stops: puddles dry on a sleeping tile, the soil drains
    /// back, and nothing runs.
    #[test]
    fn a_map_sleeps_after_the_rain_stops() {
        let mut f = flat_map(|x, z| {
            let slope = 0.3 * z as f32 + 2.0;
            if (20..28).contains(&x) && (20..28).contains(&z) {
                slope - 2.0
            } else {
                slope
            }
        });
        set_class(&mut f, SoilClass::Clay, 0.5);
        rain(&mut f, 15.0);
        run(&mut f, 600);
        assert!(f.tile_stats().2 > 0, "the storm keeps tiles awake");
        rain(&mut f, 0.0);
        run(&mut f, 600);
        f.refresh_active_tiles();
        assert_eq!(f.tile_stats().2, 0, "no tile is awake once the rain has stopped and the water settled");
        let loose_before = f.loose_tile_count();
        assert!(loose_before > 0, "the puddles still dry on the slow pass");
        let m0 = total_m3(&f);
        run(&mut f, 3600 * 6);
        assert!(total_m3(&f) < m0, "puddles dried while asleep: {m0} -> {}", total_m3(&f));
        assert_eq!(f.tile_stats().2, 0, "and nothing woke");
        assert!(WATER_SLEEP_STEPS > 0);
        assert!(f.mass_error_frac < 1e-3, "books balance: {}", f.mass_error_frac);
    }

    /// Rain the ground absorbs, or too light to matter, wakes nothing; heavy rain on ground that
    /// cannot take it wakes the tiles it falls on and only those.
    #[test]
    fn only_meaningful_rain_wakes_tiles() {
        let mut f = flat_map(|_, z| 0.3 * z as f32);
        set_class(&mut f, SoilClass::Sand, 0.0);
        rain(&mut f, 8.0);
        run(&mut f, 300);
        assert_eq!(f.tile_stats().2, 0, "sand drinks 8 mm/h: nothing ponds, nothing wakes");

        let mut g = flat_map(|_, z| 0.3 * z as f32);
        set_class(&mut g, SoilClass::Paved, 0.1);
        // Rain over the western half only (two 1 km weather cells).
        g.set_rain_grid((2, 1), 512.0, vec![12.0, 0.0]);
        run(&mut g, 30);
        let (total, _, awake) = g.tile_stats();
        assert!(awake > 0 && awake < total, "paved ground sheds the rain; only the wet half wakes: {awake}/{total}");
        let dry_side = g.depth_m[g.idx(60, 40)];
        assert_eq!(dry_side, 0.0, "no rain, no water on the dry side");
    }

    /// Rain falling on a dam crest or a building stays out of it (a wall is not a pond).
    #[test]
    fn rain_skips_crests_and_structures() {
        let mut f = flat_map(|_, _| 0.0);
        set_class(&mut f, SoilClass::Rock, 0.0);
        let (crest, roof) = (f.idx(10, 10), f.idx(12, 10));
        f.hydraulics.barrier_crest_m[crest] = 20.0;
        f.obstacle_top_m[roof] = 8.0;
        rain(&mut f, 20.0);
        run(&mut f, 120);
        assert_eq!(f.depth_m[crest], 0.0);
        assert_eq!(f.depth_m[roof], 0.0);
        assert!(f.depth_m[f.idx(11, 10)] >= 0.0);
    }
}
