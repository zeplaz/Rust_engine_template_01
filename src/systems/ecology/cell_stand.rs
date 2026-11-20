//! Per-cell **standing vegetation** — the vegetation side of the fire loop.
//!
//! **DES-06** (aig3 15.2.4 *Ecosystem Design*): [`ecosystem_logistic_step`] is the
//! stable carrying-capacity feedback used by fuel/regrowth ramps.
//!
//! ```text
//! vegetation (canopy / understory / old growth / litter, per cell through the suitability ceiling)
//!   -> fuel capacity per cell                          (this module: `fuel_cap`, `kind`)
//!   -> fire ignition / spread / burn-down on that fuel (fire overlay tick reads `fuel_cap`)
//!   -> burning consumes the cell's fuel at a rate from its fuel kind (fire overlay tick, sole fuel writer)
//!   -> fuel exhausted while burning: the stand is Burnt   (this module: `burnt_at`)
//!        standing dead snags for `snag_secs`, then regrowth for `regrow_secs`, then live again
//!   -> losses reach the canopy only through the drivers  (`CanopyDrivers::burn_*_loss`;
//!        `vegetation_field_tick` stays the one writer of `canopy_density`)
//! ```
//!
//! A ground fire (heat under [`StandBurnConfig::crown_heat`]) only burns the surface share of a cell's fuel
//! and leaves the canopy share standing: understory and litter drop, canopy stays. A crown fire burns it all
//! and leaves the cell Burnt.
//!
//! The tactical tree layer reads [`ChunkCellStand::render`] (live stage / burning / burnt / regrowth per
//! cell) through the vegetation extract frame; it keeps no flag of its own.
//!
//! W2: snag / regrow / litter clocks are authored in **game days** (`assets/configs/ecology/stand_burn_v1.ron`)
//! and converted to sim seconds via [`GAME_DAY_SECS`]. Per-cell **value noise** (chunk + cell + burn salt)
//! staggers snag hold and regrowth progress so green does not advance as a front locked to `burnt_at`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::prelude::*;
use serde::Deserialize;

use super::vegetation_field::{CanopyDrivers, VegetationField};
use super::vegetation_suitability::{max_stage_for_canopy_cap, ChunkVegetationSuitability};
use super::landscape_grammar_lg2::SuccessionTopologyStage;
use crate::systems::fire::combustion::{profile_cell_fuel_seed, MIN_CELL_FUEL_FOR_SPREAD};
use crate::systems::fire::{chunk_fuel_profile_from_vegetation, ChunkFireOverlay, ChunkSurfaceFire};
use crate::systems::sim_control::{SimControlState, SimTick, SimTimeMicros};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// One game day in sim seconds (authoring unit for [`StandBurnConfig`] RON).
pub const GAME_DAY_SECS: f32 = 86_400.0;

pub const STAND_BURN_RON: &str = "assets/configs/ecology/stand_burn_v1.ron";
const EMBEDDED_STAND_BURN: &str = include_str!("../../../assets/configs/ecology/stand_burn_v1.ron");

/// Per-cell render codes (see [`ChunkCellStand::render`]).
pub const CELL_NONE: u8 = 0;
pub const CELL_GRASS: u8 = 1;
pub const CELL_SHRUB: u8 = 2;
pub const CELL_YOUNG: u8 = 3;
pub const CELL_OLD: u8 = 4;
pub const CELL_BURNING: u8 = 5;
pub const CELL_BURNT: u8 = 6;

/// DES-06: logistic growth toward carrying capacity `capacity` (stable negative feedback).
/// `rate` is per-step; result stays in `[0, capacity]`.
#[inline]
#[must_use]
pub fn ecosystem_logistic_step(biomass: f32, capacity: f32, rate: f32) -> f32 {
    let k = capacity.max(0.0);
    if k <= 1e-6 {
        return 0.0;
    }
    let x = biomass.clamp(0.0, k);
    let r = rate.max(0.0);
    (x + r * x * (1.0 - x / k)).clamp(0.0, k)
}

pub const CELL_REGROWTH: u8 = 7;
/// Open water (river, lake, ocean): nothing grows or burns here.
pub const CELL_WATER: u8 = 8;
/// Standing dead wood after flood drown (distinct from charred [`CELL_BURNT`]) — DF-5.
pub const CELL_DEAD: u8 = 9;

/// Cell heat at which the stand counts as burning (the neighbour-spread source threshold).
pub const CELL_BURN_HEAT: f32 = 0.08;

/// Fraction of land cells that are Burnt or regrowing at which the chunk's succession stage becomes BurnScar.
pub const SCAR_FRACTION: f32 = 0.5;

/// Sim ticks between capacity refreshes of an idle chunk (vegetation changes slowly).
const CAPACITY_REFRESH_TICKS: u64 = 120;

/// What a cell's standing vegetation burns like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum StandKind {
    /// Water, bare ground, rock: nothing to burn.
    Bare = 0,
    Grass = 1,
    Brush = 2,
    Timber = 3,
}

impl StandKind {
    #[inline]
    fn slot(self) -> usize {
        (self as usize).saturating_sub(1).min(2)
    }

    /// Kind from the cell's cover density (same bands as the succession stages).
    #[must_use]
    pub fn from_density(density: f32) -> Self {
        if density < 0.02 {
            Self::Bare
        } else {
            match max_stage_for_canopy_cap(density) {
                SuccessionTopologyStage::Grass => Self::Grass,
                SuccessionTopologyStage::Shrub => Self::Brush,
                _ => Self::Timber,
            }
        }
    }
}

/// RON authoring table (game days). Converted to [`StandBurnConfig`] seconds at load.
#[derive(Clone, Debug, Deserialize)]
struct StandBurnRon {
    burn_rate: (f32, f32, f32),
    snag_days: (f32, f32, f32),
    regrow_days: (f32, f32, f32),
    crown_share: (f32, f32, f32),
    litter_days: (f32, f32, f32),
    crown_heat: f32,
    snapshot_period_ticks: u64,
}

/// Burn and regrowth timings per fuel kind (grass, brush, timber).
/// Authored in game days via [`STAND_BURN_RON`]; runtime fields are sim seconds.
#[derive(Resource, Clone, Debug)]
pub struct StandBurnConfig {
    /// Multiplier on the fuel consumption rate: grass burns out fast, timber slow.
    pub burn_rate: [f32; 3],
    /// Sim seconds a burnt stand stays charred standing dead (snags) before anything comes back.
    pub snag_secs: [f32; 3],
    /// Sim seconds of regrowth after the snag phase until the stand is live again.
    pub regrow_secs: [f32; 3],
    /// Share of a cell's fuel that is canopy (spared by a ground fire).
    pub crown_share: [f32; 3],
    /// Sim seconds for surface fuel (litter) to come back while Live / Regrowth.
    pub litter_secs: [f32; 3],
    /// Cell heat at or above which the canopy share burns too.
    pub crown_heat: f32,
    /// Sim ticks between render snapshot refreshes of an active chunk.
    pub snapshot_period_ticks: u64,
}

fn days_to_secs(days: (f32, f32, f32)) -> [f32; 3] {
    [
        days.0.max(0.0) * GAME_DAY_SECS,
        days.1.max(0.0) * GAME_DAY_SECS,
        days.2.max(0.0) * GAME_DAY_SECS,
    ]
}

fn triple(t: (f32, f32, f32)) -> [f32; 3] {
    [t.0, t.1, t.2]
}

impl StandBurnRon {
    fn into_config(self) -> StandBurnConfig {
        StandBurnConfig {
            burn_rate: triple(self.burn_rate),
            snag_secs: days_to_secs(self.snag_days),
            regrow_secs: days_to_secs(self.regrow_days),
            crown_share: triple(self.crown_share),
            litter_secs: days_to_secs(self.litter_days),
            crown_heat: self.crown_heat,
            snapshot_period_ticks: self.snapshot_period_ticks.max(1),
        }
    }
}

impl Default for StandBurnConfig {
    fn default() -> Self {
        parse_stand_burn_ron(EMBEDDED_STAND_BURN).expect("embedded stand_burn_v1.ron must parse")
    }
}

pub fn parse_stand_burn_ron(text: &str) -> Result<StandBurnConfig, String> {
    let ron: StandBurnRon =
        ron::from_str(text).map_err(|e| format!("stand burn RON parse: {e}"))?;
    Ok(ron.into_config())
}

fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

pub fn load_stand_burn_from_path(path: &Path) -> Result<StandBurnConfig, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_stand_burn_ron(&text)
}

/// Disk table when present, else the compiled-in default.
#[must_use]
pub fn load_stand_burn_config() -> StandBurnConfig {
    load_stand_burn_from_path(&repo_asset_path(STAND_BURN_RON)).unwrap_or_else(|_| StandBurnConfig::default())
}

impl StandBurnConfig {
    #[inline]
    #[must_use]
    pub fn rate(&self, kind: StandKind) -> f32 {
        self.burn_rate[kind.slot()]
    }

    #[inline]
    #[must_use]
    pub fn crown(&self, kind: StandKind) -> f32 {
        self.crown_share[kind.slot()]
    }

    #[inline]
    #[must_use]
    pub fn litter(&self, kind: StandKind) -> f32 {
        self.litter_secs[kind.slot()]
    }
}

/// Where a cell is in the burn cycle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StandPhase {
    Live,
    /// Standing dead snags.
    Burnt,
    /// Coming back; `0..1` progress.
    Regrowth(f32),
    /// Still submerged past species flood tolerance (DF-5).
    Drowned,
    /// Exposed standing dead after drown (woody fuel kept).
    Dead,
}

#[inline]
fn hash_u32(x: u32) -> u32 {
    let mut n = x.wrapping_mul(0x9E37_79B9);
    n ^= n >> 16;
    n = n.wrapping_mul(0x85EB_CA6B);
    n ^= n >> 13;
    n
}

/// Deterministic white sample in `[0, 1)` at integer lattice point.
#[inline]
fn lattice_noise01(ix: i32, iy: i32, salt: u32) -> f32 {
    let h = hash_u32(
        (ix as u32)
            .wrapping_mul(374_761_393)
            .wrapping_add((iy as u32).wrapping_mul(668_265_263))
            .wrapping_add(salt),
    );
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// Bilinear value noise in `[0, 1)` — low-freq patches for pioneer islands.
#[inline]
fn value_noise01(x: f32, y: f32, salt: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);
    let n00 = lattice_noise01(x0, y0, salt);
    let n10 = lattice_noise01(x0 + 1, y0, salt);
    let n01 = lattice_noise01(x0, y0 + 1, salt);
    let n11 = lattice_noise01(x0 + 1, y0 + 1, salt);
    let a = n00 + (n10 - n00) * sx;
    let b = n01 + (n11 - n01) * sx;
    a + (b - a) * sy
}

/// Per-cell snag hold, regrowth duration, and progress exponent from spatial noise + moisture.
///
/// Returns `(snag_secs, regrow_secs, progress_pow)` — `progress_pow` > 1 delays mid-fill.
#[must_use]
pub fn regrowth_noise_scales(
    chunk_bits: u64,
    grid_w: u32,
    cell: usize,
    burnt_at: f32,
    moisture: f32,
    cfg: &StandBurnConfig,
    kind: StandKind,
) -> (f32, f32, f32) {
    let w = grid_w.max(1);
    let cx = (cell as u32 % w) as f32;
    let cy = (cell as u32 / w) as f32;
    let salt = (chunk_bits as u32)
        .wrapping_mul(0xA24B_AED5)
        .wrapping_add(burnt_at.to_bits().wrapping_mul(0x9E37_79B9));
    // Coarse patches (~6–8 cells) + finer breakup so islands appear inside scars.
    let patch = value_noise01(cx * 0.16 + 0.37, cy * 0.16 + 0.11, salt);
    let fine = value_noise01(cx * 0.55 + 1.7, cy * 0.55 + 2.3, salt ^ 0x51DE_ADE1);
    let n = (patch * 0.65 + fine * 0.35).clamp(0.0, 1.0);
    // Wetter cells pioneer a bit earlier; dry ash holds longer.
    let wet = (1.0 - moisture.clamp(0.0, 1.0) * 0.22).clamp(0.78, 1.0);
    let snag_base = cfg.snag_secs[kind.slot()];
    let regrow_base = cfg.regrow_secs[kind.slot()].max(0.001);
    // Snag: 0.55× … 1.70× authored — early nuclei vs late holdouts.
    let snag = snag_base * (0.55 + n * 1.15) * wet;
    // Regrowth length: independent octave so fill rate ≠ snag order.
    let n2 = value_noise01(cx * 0.22 + 4.1, cy * 0.22 + 0.9, salt ^ 0xC0FF_EE00);
    let regrow = regrow_base * (0.65 + n2 * 1.05);
    // Exponent 0.75..1.85 — high values linger in early regrowth visually.
    let progress_pow = 0.75 + (1.0 - n2) * 1.1;
    (snag.max(0.05), regrow, progress_pow)
}

/// Per-cell standing vegetation state for one chunk, aligned with [`ChunkCellMatrix`].
///
/// Written only by [`cell_stand_tick`]. The fire overlay tick reads `fuel_cap` / `kind`; the tactical tree
/// layer reads `render`.
#[derive(Component, Debug, Clone)]
pub struct ChunkCellStand {
    pub kind: Vec<StandKind>,
    /// Fuel the unburnt stand carries here (the ceiling of the fire reservoir when the cell is live).
    pub full_cap: Vec<f32>,
    /// Fuel the stand carries now: `full_cap` while live, zero while snags stand, ramping back in regrowth.
    pub fuel_cap: Vec<f32>,
    /// Sim seconds at which the stand burnt out; negative while it has not.
    pub burnt_at: Vec<f32>,
    pub burnt_count: u32,
    /// Share of land fuel that is Burnt or regrowing (drives the chunk scar decision).
    pub burnt_fraction: f32,
    /// One code per cell: [`CELL_NONE`]..[`CELL_WATER`].
    pub render: Arc<[u8]>,
    /// Cells drawn as burning. Lit at [`CELL_BURN_HEAT`], held until heat falls under half of it (hysteresis), so
    /// a cell on the threshold does not blink between burning and standing.
    lit: Vec<bool>,
    /// Changes whenever `render` does.
    pub render_rev: u32,
    /// Whether capacities have been rated at least once (persisted with the stand).
    pub seeded: bool,
    /// Sim seconds when flood submergence began; negative while dry (DF-5).
    pub submerged_since: Vec<f32>,
    /// Flood-killed standing dead (keeps woody `fuel_cap`).
    pub flood_dead: Vec<bool>,
    /// Ground destroyed here (crater / rubble, DW-1): no stand, and capacity refresh must not re-rate it.
    pub cleared: Vec<bool>,
    last_refresh: u64,
    last_snapshot: u64,
    last_pass: u64,
}

impl ChunkCellStand {
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            kind: vec![StandKind::Bare; n],
            full_cap: vec![0.0; n],
            fuel_cap: vec![0.0; n],
            burnt_at: vec![-1.0; n],
            burnt_count: 0,
            burnt_fraction: 0.0,
            render: vec![CELL_NONE; n].into(),
            lit: vec![false; n],
            render_rev: 0,
            seeded: false,
            submerged_since: vec![-1.0; n],
            flood_dead: vec![false; n],
            cleared: vec![false; n],
            last_refresh: 0,
            last_snapshot: 0,
            last_pass: 0,
        }
    }

    /// Destroy the stand at cell `i` for good (ground deformed beneath it): bare, no fuel, no regrowth.
    pub fn clear_cell(&mut self, i: usize) {
        if i >= self.kind.len() {
            return;
        }
        if self.cleared.len() != self.kind.len() {
            self.cleared.resize(self.kind.len(), false);
        }
        if self.cleared[i] {
            return;
        }
        self.cleared[i] = true;
        if self.burnt_at[i] >= 0.0 {
            self.burnt_count = self.burnt_count.saturating_sub(1);
        }
        self.kind[i] = StandKind::Bare;
        self.full_cap[i] = 0.0;
        self.fuel_cap[i] = 0.0;
        self.burnt_at[i] = -1.0;
        self.submerged_since[i] = -1.0;
        self.flood_dead[i] = false;
        // Force the next snapshot so the drawn stand disappears promptly.
        self.last_snapshot = 0;
    }

    /// Stand at cell `i` was destroyed by ground deformation.
    #[must_use]
    pub fn is_cleared(&self, i: usize) -> bool {
        self.cleared.get(i).copied().unwrap_or(false)
    }

    /// Default upland flood tolerance (~14 days). Species RON can override later.
    pub const FLOOD_TOLERANCE_S: f32 = 14.0 * GAME_DAY_SECS;

    /// DF-5: update submergence clocks from a per-cell wet mask (chunk-local).
    pub fn apply_flood_submergence(&mut self, wet: &[bool], now: f32) {
        let n = self.kind.len().min(wet.len()).min(self.submerged_since.len());
        for i in 0..n {
            if self.kind[i] == StandKind::Bare {
                continue;
            }
            if wet[i] {
                if self.submerged_since[i] < 0.0 {
                    self.submerged_since[i] = now;
                } else if now - self.submerged_since[i] >= Self::FLOOD_TOLERANCE_S {
                    self.flood_dead[i] = true;
                }
            } else if self.submerged_since[i] >= 0.0 {
                if now - self.submerged_since[i] >= Self::FLOOD_TOLERANCE_S {
                    self.flood_dead[i] = true;
                }
                self.submerged_since[i] = -1.0;
            }
        }
    }

    /// Phase of cell `i` at sim time `now` (seconds).
    ///
    /// `chunk_bits` + `grid_w` feed spatial value noise so neighbours that burned together do not
    /// leave snag / enter regrowth on the same clock. `moisture` (0..1) slightly favours wetter cells.
    #[must_use]
    pub fn phase(
        &self,
        i: usize,
        now: f32,
        cfg: &StandBurnConfig,
        chunk_bits: u64,
        grid_w: u32,
        moisture: f32,
    ) -> StandPhase {
        if self.flood_dead.get(i).copied().unwrap_or(false) {
            let sub = self.submerged_since.get(i).copied().unwrap_or(-1.0);
            return if sub >= 0.0 {
                StandPhase::Drowned
            } else {
                StandPhase::Dead
            };
        }
        let t = self.burnt_at.get(i).copied().unwrap_or(-1.0);
        if t < 0.0 {
            return StandPhase::Live;
        }
        let k = self.kind[i];
        let age = now - t;
        let (snag, regrow, progress_warp) =
            regrowth_noise_scales(chunk_bits, grid_w, i, t, moisture, cfg, k);
        if age < snag {
            StandPhase::Burnt
        } else if age < snag + regrow {
            let raw = ((age - snag) / regrow).clamp(0.0, 1.0);
            // Warp progress so patch interiors fill unevenly (not a linear ring).
            let p = (raw.powf(progress_warp)).clamp(0.0, 1.0);
            StandPhase::Regrowth(p)
        } else {
            StandPhase::Live
        }
    }

    /// The stand has been rated from the vegetation at least once (its capacities are meaningful).
    #[must_use]
    pub fn is_seeded(&self) -> bool {
        self.seeded
    }

    /// The stand at cell `i` burnt out and has not grown back.
    #[must_use]
    pub fn is_burnt(&self, i: usize) -> bool {
        self.burnt_at.get(i).is_some_and(|t| *t >= 0.0)
    }
}

pub(crate) fn spawn_chunk_cell_stand(
    mut commands: Commands,
    q: Query<(Entity, &ChunkCellMatrix), (With<Chunk>, With<ChunkFireOverlay>, Without<ChunkCellStand>)>,
) {
    for (e, matrix) in &q {
        let n = (matrix.size.x * matrix.size.y) as usize;
        if n > 0 {
            commands.entity(e).insert(ChunkCellStand::new(n));
        }
    }
}

/// Undo the burn losses on a copy of the field, so capacity is rated on the stand that would be there
/// unburnt (a burn must not shrink the fuel of the cells that did not burn).
fn unburnt_equivalent(veg: &VegetationField, drivers: &CanopyDrivers) -> VegetationField {
    let canopy_room = (1.0 - drivers.burn_canopy_loss).clamp(0.25, 1.0);
    let floor_room = (1.0 - drivers.burn_canopy_loss.max(drivers.burn_ground_loss)).clamp(0.25, 1.0);
    VegetationField {
        canopy_density: (veg.canopy_density / canopy_room).clamp(0.0, 1.0),
        understory_density: (veg.understory_density / floor_room).clamp(0.0, 1.0),
        ..*veg
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn cell_stand_tick(
    ctrl: Res<SimControlState>,
    tick: Res<SimTick>,
    sim_time: Res<SimTimeMicros>,
    cfg: Option<Res<StandBurnConfig>>,
    mut q: Query<(
        Entity,
        Ref<ChunkCellMatrix>,
        &VegetationField,
        Option<&ChunkVegetationSuitability>,
        &ChunkSurfaceFire,
        &mut ChunkFireOverlay,
        &mut ChunkCellStand,
        &mut CanopyDrivers,
    )>,
) {
    let _perf = crate::render::PerfScope::new("upd_eco_cell_stand_tick");
    if !ctrl.should_tick() {
        return;
    }
    let default_cfg = StandBurnConfig::default();
    let cfg = cfg.as_deref().unwrap_or(&default_cfg);
    let now = (sim_time.0 as f64 / 1_000_000.0) as f32;

    for (entity, matrix, veg, suit, fire, mut ovl, mut stand, mut drivers) in &mut q {
        let n = matrix.water.len();
        if n == 0 || ovl.fuel.len() != n || ovl.heat.len() != n || ovl.burn_hold.len() != n {
            continue;
        }
        if stand.kind.len() != n {
            *stand = ChunkCellStand::new(n);
        }
        let reseed = !stand.seeded || matrix.is_changed();
        // Spread idle-chunk refreshes over time instead of all on the same tick.
        let stagger = u64::from(entity.index_u32()) % CAPACITY_REFRESH_TICKS;
        let periodic = tick.0.wrapping_add(stagger).wrapping_sub(stand.last_refresh) >= CAPACITY_REFRESH_TICKS;
        let active = reseed
            || stand.burnt_count > 0
            || fire.heat > 0.0005
            || drivers.burn_ground_loss > 0.003;
        if !(active || periodic) {
            continue;
        }
        // Without live fire only the slow clocks (snags, regrowth, litter) move: a pass per snapshot period.
        let hot = fire.heat > 0.0005;
        if !hot && !reseed && !periodic && tick.0.wrapping_sub(stand.last_pass) < cfg.snapshot_period_ticks {
            continue;
        }
        stand.last_pass = tick.0;

        // While a burn is working through the chunk the canopy is the burn ceiling, not the stand: keep
        // the ratings the cells had and re-rate once the stand is back.
        if (reseed || periodic) && (!stand.seeded || drivers.burn_canopy_loss <= 0.02) {
            refresh_capacity(&mut stand, &matrix, veg, suit, &drivers);
            stand.last_refresh = tick.0.wrapping_add(stagger);
        }

        // Burn-out: surface (or full) fuel exhausted while the cell burns.
        // Ground fires stop at the crown floor and never cross MIN_CELL_FUEL — still a scar.
        for i in 0..n {
            if stand.is_burnt(i) || stand.kind[i] == StandKind::Bare {
                continue;
            }
            let hot = ovl.heat[i] >= CELL_BURN_HEAT || ovl.burn_hold[i] > 0;
            if !hot || stand.full_cap[i] < MIN_CELL_FUEL_FOR_SPREAD {
                continue;
            }
            let floor = cfg.crown(stand.kind[i]) * stand.full_cap[i];
            let surface_gone =
                ovl.fuel[i] < MIN_CELL_FUEL_FOR_SPREAD || ovl.fuel[i] <= floor + 0.01;
            if surface_gone {
                stand.burnt_at[i] = now;
                stand.burnt_count += 1;
            }
        }

        // Phases: current carrying capacity per cell (noise-staggered clocks).
        let mut sum_w = 0.0f32;
        let mut lost_w = 0.0f32;
        let mut surface_w = 0.0f32;
        let mut scar_cells = 0u32;
        let mut land_cells = 0u32;
        let chunk_bits = entity.to_bits();
        let grid_w = matrix.size.x;
        for i in 0..n {
            let full = stand.full_cap[i];
            let m = matrix.moisture.get(i).copied().unwrap_or(0.4);
            let (level, lost) = match stand.phase(i, now, cfg, chunk_bits, grid_w, m) {
                StandPhase::Live => {
                    if stand.burnt_at[i] >= 0.0 {
                        stand.burnt_at[i] = -1.0;
                        stand.burnt_count = stand.burnt_count.saturating_sub(1);
                    }
                    (1.0, 0.0)
                }
                StandPhase::Burnt => {
                    scar_cells += 1;
                    (0.0, 1.0)
                }
                StandPhase::Drowned => {
                    // Crown fines gone; woody fuel remains while submerged.
                    (0.85, 0.15)
                }
                StandPhase::Dead => {
                    // Exposed standing dead — woody fuel kept (DF-5).
                    (0.7, 0.3)
                }
                StandPhase::Regrowth(p) => {
                    scar_cells += 1;
                    // DES-06: logistic approach to capacity so late regrowth slows near full.
                    let level = ecosystem_logistic_step(p, 1.0, 0.35).clamp(0.0, 1.0);
                    (level, 1.0 - level)
                }
            };
            stand.fuel_cap[i] = full * level;
            if stand.kind[i] == StandKind::Bare {
                continue;
            }
            land_cells += 1;
            sum_w += full;
            lost_w += full * lost;
            // Share of the surface fuel this live cell has already burned off (ground fire).
            let crown = cfg.crown(stand.kind[i]);
            let surface = if lost > 0.0 {
                1.0
            } else if stand.fuel_cap[i] > 0.0 {
                // Before the first fill the reservoir is the vegetation's own fuel (nothing burnt yet).
                let fuel = if stand.seeded { ovl.fuel[i] } else { stand.fuel_cap[i] };
                let want = stand.fuel_cap[i] * (1.0 - crown).max(0.05);
                ((stand.fuel_cap[i] - fuel) / want).clamp(0.0, 1.0)
            } else {
                0.0
            };
            surface_w += full * surface;
        }
        let (canopy_loss, ground_loss, scar) = if sum_w > 1e-6 {
            (
                lost_w / sum_w,
                surface_w / sum_w,
                if land_cells > 0 { scar_cells as f32 / land_cells as f32 } else { 0.0 },
            )
        } else {
            (0.0, 0.0, 0.0)
        };
        stand.burnt_fraction = scar;

        if !stand.seeded {
            // First fill of the reservoir from the vegetation (no fire has consumed anything yet). Later
            // capacity changes reach the reservoir through the fire tick (clamp down, refill up).
            for i in 0..n {
                if !stand.is_burnt(i) {
                    ovl.fuel[i] = stand.fuel_cap[i];
                }
            }
            stand.seeded = true;
        }

        if (drivers.burn_canopy_loss - canopy_loss).abs() > 1e-4
            || (drivers.burn_ground_loss - ground_loss).abs() > 1e-4
        {
            drivers.burn_canopy_loss = canopy_loss;
            drivers.burn_ground_loss = ground_loss;
        }

        // A chunk with live fire is re-drawn often enough that the front advances steadily.
        let period = if hot { (cfg.snapshot_period_ticks / 4).max(3) } else { cfg.snapshot_period_ticks };
        if reseed || tick.0.wrapping_sub(stand.last_snapshot) >= period {
            stand.last_snapshot = tick.0;
            snapshot(&mut stand, &matrix, veg, suit, &ovl, now, cfg, chunk_bits);
        }
    }
}

/// Re-rate the fuel each cell can carry from the (unburnt-equivalent) vegetation.
fn refresh_capacity(
    stand: &mut ChunkCellStand,
    matrix: &ChunkCellMatrix,
    veg: &VegetationField,
    suit: Option<&ChunkVegetationSuitability>,
    drivers: &CanopyDrivers,
) {
    let n = matrix.water.len();
    let eq = unburnt_equivalent(veg, drivers);
    let seed = profile_cell_fuel_seed(&chunk_fuel_profile_from_vegetation(&eq));
    let mut dens = vec![0.0f32; n];
    let mut sum = 0.0f32;
    let mut land = 0u32;
    for i in 0..n {
        // A burnt cell keeps the stand it had: it regrows toward that, whatever the canopy shows now.
        if matrix.water[i].is_open_water() || stand.is_burnt(i) || stand.is_cleared(i) {
            continue;
        }
        let d = suit
            .map(|s| s.cell_density(eq.canopy_density, i))
            .unwrap_or(eq.canopy_density);
        dens[i] = d;
        if d >= 0.02 {
            sum += d;
            land += 1;
        }
    }
    let mean = if land > 0 { (sum / land as f32).max(1e-3) } else { 1.0 };
    for i in 0..n {
        if stand.is_cleared(i) || (stand.is_burnt(i) && !matrix.water[i].is_open_water()) {
            continue;
        }
        let kind = StandKind::from_density(dens[i]);
        stand.kind[i] = kind;
        stand.full_cap[i] = if kind == StandKind::Bare {
            0.0
        } else {
            (seed * dens[i] / mean).clamp(0.0, 0.95)
        };
    }
}

fn snapshot(
    stand: &mut ChunkCellStand,
    matrix: &ChunkCellMatrix,
    veg: &VegetationField,
    suit: Option<&ChunkVegetationSuitability>,
    ovl: &ChunkFireOverlay,
    now: f32,
    cfg: &StandBurnConfig,
    chunk_bits: u64,
) {
    let n = matrix.water.len();
    let grid_w = matrix.size.x;
    let mut codes = vec![CELL_NONE; n];
    for i in 0..n {
        if matrix.water[i].is_open_water() {
            codes[i] = CELL_WATER;
            continue;
        }
        if stand.kind[i] == StandKind::Bare {
            continue;
        }
        // Burning means heat with fuel left to burn. An exhausted cell is charred even while its heat cools.
        let threshold = if stand.lit[i] { CELL_BURN_HEAT * 0.5 } else { CELL_BURN_HEAT };
        let burning = ovl.heat[i] >= threshold && ovl.fuel[i] >= MIN_CELL_FUEL_FOR_SPREAD;
        stand.lit[i] = burning;
        codes[i] = if burning {
            CELL_BURNING
        } else {
            let m = matrix.moisture.get(i).copied().unwrap_or(0.4);
            match stand.phase(i, now, cfg, chunk_bits, grid_w, m) {
                StandPhase::Burnt => CELL_BURNT,
                StandPhase::Drowned | StandPhase::Dead => CELL_DEAD,
                StandPhase::Regrowth(_) => CELL_REGROWTH,
                StandPhase::Live => {
                    let d = suit
                        .map(|s| s.cell_density(veg.canopy_density, i))
                        .unwrap_or(veg.canopy_density);
                    match max_stage_for_canopy_cap(d) {
                        SuccessionTopologyStage::Grass => CELL_GRASS,
                        SuccessionTopologyStage::Shrub => CELL_SHRUB,
                        SuccessionTopologyStage::YoungForest => CELL_YOUNG,
                        _ => {
                            if veg.old_growth >= 0.55 {
                                CELL_OLD
                            } else {
                                CELL_YOUNG
                            }
                        }
                    }
                }
            }
        };
    }
    if stand.render.as_ref() != codes.as_slice() {
        stand.render = codes.into();
        stand.render_rev = stand.render_rev.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::systems::chunk_environment_persist::{ChunkEnvironmentDirty, ChunkEnvironmentPersistHooks};
    use crate::systems::chunk_sim_lod::ChunkSimLod;
    use crate::systems::ecology::vegetation_field::vegetation_field_tick;
    use crate::systems::ecology::vegetation_suitability::update_chunk_vegetation_suitability;
    use crate::systems::ecology::ChunkEcology;
    use crate::systems::fire::{
        chunk_fire_overlay_tick, chunk_fuel_profile_tick, ChunkFuelProfile, FireEcologyStats,
        SurfaceWaterFireGate,
    };
    use crate::systems::weather::ChunkWeather;
    use crate::terrain::default_terrain_families;

    const SIDE: u32 = 6;
    const CENTRE: usize = (3 * SIDE + 3) as usize;

    fn advance_clock(mut tick: ResMut<SimTick>, mut t: ResMut<SimTimeMicros>) {
        tick.0 += 1;
        t.0 += 16_000;
    }

    /// Short burn / regrowth timings and fast burn rates so a test runs in seconds of sim time.
    fn test_cfg() -> StandBurnConfig {
        let d = StandBurnConfig::default();
        StandBurnConfig {
            burn_rate: [d.burn_rate[0] * 10.0, d.burn_rate[1] * 10.0, d.burn_rate[2] * 10.0],
            snag_secs: [1.0, 2.0, 3.0],
            regrow_secs: [3.0, 6.0, 12.0],
            litter_secs: [2.0, 4.0, 8.0],
            ..d
        }
    }

    #[test]
    fn w2_default_timber_scar_spans_game_years() {
        let cfg = StandBurnConfig::default();
        let timber = 2usize;
        let cycle = cfg.snag_secs[timber] + cfg.regrow_secs[timber];
        assert!(
            cycle >= GAME_DAY_SECS * 365.0,
            "timber scar must last ≥1 game year, got {} days",
            cycle / GAME_DAY_SECS
        );
        assert!(cfg.snag_secs[0] + cfg.regrow_secs[0] < cfg.snag_secs[timber]);
    }

    #[test]
    fn regrowth_noise_staggers_neighbours_with_same_burnt_at() {
        let cfg = test_cfg();
        let mut stand = ChunkCellStand::new(64);
        let grid_w = 8u32;
        for i in 0..64 {
            stand.kind[i] = StandKind::Grass;
            stand.full_cap[i] = 0.5;
            stand.burnt_at[i] = 0.0;
        }
        stand.burnt_count = 64;
        // Just past authored snag (1s): some cells still Burnt, some already Regrowth.
        let now = cfg.snag_secs[0] * 1.05;
        let mut burnt = 0u32;
        let mut regrowing = 0u32;
        let mut progresses = Vec::new();
        for i in 0..64 {
            match stand.phase(i, now, &cfg, 0xDEAD_BEEF, grid_w, 0.4) {
                StandPhase::Burnt => burnt += 1,
                StandPhase::Regrowth(p) => {
                    regrowing += 1;
                    progresses.push(p);
                }
                StandPhase::Live | StandPhase::Drowned | StandPhase::Dead => {}
            }
        }
        assert!(burnt > 0, "noise should leave some cells in snag");
        assert!(regrowing > 0, "noise should let pioneer cells leave snag");
        if progresses.len() >= 2 {
            let min_p = progresses.iter().copied().fold(f32::INFINITY, f32::min);
            let max_p = progresses.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            assert!(
                max_p - min_p > 1e-4,
                "regrowth progress should differ across cells ({min_p}..{max_p})"
            );
        }
    }

    /// Fire + ecology slice of the sim: vegetation -> fuel -> fire -> burnt stand -> canopy drivers.
    fn fire_app(families: &[&str], moisture: f32) -> (App, Vec<Entity>) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(16)))
            .insert_resource(SimControlState::default())
            .init_resource::<SimTick>()
            .init_resource::<SimTimeMicros>()
            .insert_resource(test_cfg())
            .init_resource::<SurfaceWaterFireGate>()
            .init_resource::<ChunkEnvironmentPersistHooks>()
            .init_resource::<FireEcologyStats>()
            .add_systems(
                Update,
                (
                    advance_clock,
                    update_chunk_vegetation_suitability,
                    spawn_chunk_cell_stand,
                    chunk_fuel_profile_tick,
                    cell_stand_tick,
                    vegetation_field_tick,
                    chunk_fire_overlay_tick,
                )
                    .chain(),
            );
        let families_reg = default_terrain_families();
        let mut out = Vec::new();
        for (k, name) in families.iter().enumerate() {
            let mut matrix = ChunkCellMatrix::new(UVec2::splat(SIDE));
            for i in 0..matrix.elevation.len() {
                matrix.elevation[i] = 0.6;
                matrix.moisture[i] = moisture;
                matrix.temperature[i] = 0.3;
                matrix.family[i] = families_reg.id(name).unwrap();
            }
            let n = matrix.elevation.len();
            let e = app
                .world_mut()
                .spawn((
                    Chunk { coord: IVec2::new(k as i32, 0) },
                    matrix,
                    ChunkEcology {
                        biomass: 0.9,
                        regrowth_rate: 0.3,
                        shade_factor: 0.6,
                        ..ChunkEcology::default()
                    },
                    ChunkWeather::default(),
                    ChunkSimLod::default(),
                    VegetationField {
                        canopy_density: 0.85,
                        understory_density: 0.5,
                        old_growth: 0.7,
                        dryness: 0.4,
                        ..VegetationField::default()
                    },
                    ChunkFuelProfile::default(),
                    ChunkFireOverlay {
                        heat: vec![0.0; n],
                        fuel: vec![0.06; n],
                        smoke: vec![0.0; n],
                        toxic: vec![0.0; n],
                        burn_hold: vec![0; n],
                    },
                    ChunkSurfaceFire::default(),
                    ChunkEnvironmentDirty::default(),
                ))
                .id();
            out.push(e);
        }
        (app, out)
    }

    fn overlay_of(app: &App, e: Entity) -> ChunkFireOverlay {
        app.world().get::<ChunkFireOverlay>(e).unwrap().clone()
    }

    #[derive(Clone, Copy)]
    struct Track {
        cell: Option<usize>,
        burnt_tick: Option<usize>,
        seen_burnt_code: bool,
        seen_regrowth_code: bool,
        live_again_tick: Option<usize>,
        min_fuel_when_burnt: f32,
        canopy_min: f32,
        out_tick: Option<usize>,
        fuel_dropped: bool,
    }

    #[test]
    fn fire_burns_the_stand_down_leaves_snags_goes_out_and_the_stand_regrows_forest_slow_grass_fast() {
        // Forest chunk and a grass chunk (a family that only carries grass-density cover).
        let (mut app, chunks) = fire_app(&["DenseForest", "Dirt"], 0.5);
        for _ in 0..30 {
            app.update();
        }
        let cfg = test_cfg();
        for &e in &chunks {
            let stand = app.world().get::<ChunkCellStand>(e).expect("stand spawned");
            assert!(stand.is_seeded());
            assert!(stand.full_cap[CENTRE] > MIN_CELL_FUEL_FOR_SPREAD);
            // The fire reservoir is the vegetation's fuel, not the empty overlay seed.
            let fuel = overlay_of(&app, e).fuel[CENTRE];
            assert!((fuel - stand.fuel_cap[CENTRE]).abs() < 1e-4, "fuel {fuel} cap {}", stand.fuel_cap[CENTRE]);
        }
        assert_eq!(app.world().get::<ChunkCellStand>(chunks[0]).unwrap().kind[CENTRE], StandKind::Timber);
        assert_eq!(app.world().get::<ChunkCellStand>(chunks[1]).unwrap().kind[CENTRE], StandKind::Grass);
        let forest_fuel_before = overlay_of(&app, chunks[0]).fuel[CENTRE];
        let canopy_before = app.world().get::<VegetationField>(chunks[0]).unwrap().canopy_density;
        assert!(canopy_before > 0.5, "{canopy_before}");

        // A fully involved fire: every cell of both chunks alight.
        for &e in &chunks {
            for h in &mut app.world_mut().get_mut::<ChunkFireOverlay>(e).unwrap().heat {
                *h = 1.0;
            }
        }

        let fresh = Track {
            cell: None,
            burnt_tick: None,
            seen_burnt_code: false,
            seen_regrowth_code: false,
            live_again_tick: None,
            min_fuel_when_burnt: 9.0,
            canopy_min: 9.0,
            out_tick: None,
            fuel_dropped: false,
        };
        let mut tr = [fresh; 2];
        for t in 0..9000usize {
            app.update();

            for (k, &e) in chunks.iter().enumerate() {
                let stand = app.world().get::<ChunkCellStand>(e).unwrap();
                let ovl = app.world().get::<ChunkFireOverlay>(e).unwrap();
                let veg = app.world().get::<VegetationField>(e).unwrap();
                let tr = &mut tr[k];
                if ovl.fuel[CENTRE] < stand.full_cap[CENTRE] * 0.7 {
                    tr.fuel_dropped = true;
                }
                // Follow the first cell that burns out.
                if tr.cell.is_none() {
                    tr.cell = (0..stand.burnt_at.len()).find(|i| stand.is_burnt(*i));
                    if let Some(c) = tr.cell {
                        tr.burnt_tick = Some(t);
                        tr.min_fuel_when_burnt = ovl.fuel[c];
                    }
                }
                let c = tr.cell.unwrap_or(CENTRE);
                if stand.render[c] == CELL_BURNT {
                    tr.seen_burnt_code = true;
                }
                if stand.render[c] == CELL_REGROWTH {
                    tr.seen_regrowth_code = true;
                }
                if tr.burnt_tick.is_some() && tr.live_again_tick.is_none() && !stand.is_burnt(c) {
                    tr.live_again_tick = Some(t);
                }
                if tr.burnt_tick.is_some() {
                    tr.canopy_min = tr.canopy_min.min(veg.canopy_density);
                }
                if tr.burnt_tick.is_some() && tr.out_tick.is_none() && ovl.heat.iter().all(|h| *h == 0.0) {
                    tr.out_tick = Some(t);
                }
            }
            if tr.iter().all(|t| t.live_again_tick.is_some() && t.out_tick.is_some()) && t > 400 {
                break;
            }
        }
        let _ = forest_fuel_before;

        for (k, name) in ["forest", "grass"].iter().enumerate() {
            let t = tr[k];
            assert!(t.fuel_dropped, "{name}: burning never consumed the cell fuel");
            let burnt = t.burnt_tick.unwrap_or_else(|| panic!("{name}: stand never burnt out"));
            assert!(t.min_fuel_when_burnt < MIN_CELL_FUEL_FOR_SPREAD, "{name}: burnt with fuel left");
            assert!(t.seen_burnt_code, "{name}: no snag state rendered");
            assert!(t.seen_regrowth_code, "{name}: no regrowth state rendered");
            let out = t.out_tick.unwrap_or_else(|| panic!("{name}: fire never went out"));
            assert!(out >= burnt);
            let live = t.live_again_tick.unwrap_or_else(|| panic!("{name}: stand never regrew"));
            // Regrowth starts only after the snag time and finishes after the regrow time.
            let slot = if k == 0 { 2 } else { 0 };
            let min_secs = cfg.snag_secs[slot] + cfg.regrow_secs[slot];
            let took = (live - burnt) as f32 * 0.016;
            // Noise can shorten snag to ~0.55×; still must not instant-green.
            assert!(
                took >= min_secs * 0.5 - 0.1,
                "{name}: regrown after {took} s, configured {min_secs} s"
            );
        }
        // The forest canopy dropped with the burn (through the drivers) and came back.
        assert!(tr[0].canopy_min < canopy_before * 0.5, "canopy never fell: {}", tr[0].canopy_min);
        let canopy_end = app.world().get::<VegetationField>(chunks[0]).unwrap().canopy_density;
        assert!(canopy_end > tr[0].canopy_min + 0.1, "canopy did not regrow: {canopy_end}");
        // Grass burns out and regrows faster than forest.
        let (forest, grass) = (tr[0], tr[1]);
        assert!(grass.burnt_tick.unwrap() < forest.burnt_tick.unwrap(), "grass must burn out first");
        let grass_cycle = grass.live_again_tick.unwrap() - grass.burnt_tick.unwrap();
        let forest_cycle = forest.live_again_tick.unwrap() - forest.burnt_tick.unwrap();
        assert!(grass_cycle < forest_cycle, "grass {grass_cycle} forest {forest_cycle}");
    }

    #[test]
    fn a_ground_fire_burns_litter_and_understory_but_leaves_the_canopy() {
        let (mut app, chunks) = fire_app(&["DenseForest"], 0.5);
        for _ in 0..30 {
            app.update();
        }
        let e = chunks[0];
        let veg0 = *app.world().get::<VegetationField>(e).unwrap();
        // A ground fire has burned the surface share of every cell fuel and stopped there.
        {
            let stand = app.world().get::<ChunkCellStand>(e).unwrap().clone();
            let mut ovl = app.world_mut().get_mut::<ChunkFireOverlay>(e).unwrap();
            for i in 0..ovl.fuel.len() {
                ovl.fuel[i] = stand.fuel_cap[i] * 0.5;
            }
            // Warm, not burning: under the spread threshold, so the fire stays put.
            ovl.heat[0] = 0.05;
        }
        for _ in 0..60 {
            app.update();
        }
        let drivers = *app.world().get::<CanopyDrivers>(e).unwrap();
        let veg = *app.world().get::<VegetationField>(e).unwrap();
        assert!(drivers.burn_ground_loss > 0.3, "ground loss {}", drivers.burn_ground_loss);
        assert_eq!(drivers.burn_canopy_loss, 0.0, "no cell burnt out");
        assert!(
            veg.canopy_density > veg0.canopy_density * 0.9,
            "canopy {} -> {}",
            veg0.canopy_density,
            veg.canopy_density
        );
        assert!(
            veg.understory_density < veg0.understory_density * 0.8,
            "understory {} -> {}",
            veg0.understory_density,
            veg.understory_density
        );
        assert!(veg.ground_fuel < veg0.ground_fuel * 0.8, "litter {} -> {}", veg0.ground_fuel, veg.ground_fuel);
        assert_eq!(app.world().get::<ChunkCellStand>(e).unwrap().burnt_count, 0);
    }
    #[test]
    fn des06_logistic_feedback_approaches_capacity_without_overshoot() {
        let k = 1.0_f32;
        let mut x = 0.05_f32;
        for _ in 0..40 {
            let next = ecosystem_logistic_step(x, k, 0.35);
            assert!(next >= x - 1e-6, "monotonic rise while below K");
            assert!(next <= k + 1e-5);
            x = next;
        }
        assert!(x > 0.9, "should approach capacity, got {x}");
        assert!((ecosystem_logistic_step(k, k, 0.5) - k).abs() < 1e-5);
        assert_eq!(ecosystem_logistic_step(0.5, 0.0, 1.0), 0.0);
    }

}
