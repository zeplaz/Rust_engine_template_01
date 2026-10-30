//! Per-cell **standing vegetation** — the vegetation side of the fire loop.
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

use std::sync::Arc;

use bevy::prelude::*;

use super::vegetation_field::{CanopyDrivers, VegetationField};
use super::vegetation_suitability::{max_stage_for_canopy_cap, ChunkVegetationSuitability};
use super::landscape_grammar_lg2::SuccessionTopologyStage;
use crate::systems::fire::combustion::{profile_cell_fuel_seed, MIN_CELL_FUEL_FOR_SPREAD};
use crate::systems::fire::{chunk_fuel_profile_from_vegetation, ChunkFireOverlay, ChunkSurfaceFire};
use crate::systems::sim_control::{SimControlState, SimTick, SimTimeMicros};
use crate::terrain::fire::{fuel_material_def, FuelMaterialKind};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Per-cell render codes (see [`ChunkCellStand::render`]).
pub const CELL_NONE: u8 = 0;
pub const CELL_GRASS: u8 = 1;
pub const CELL_SHRUB: u8 = 2;
pub const CELL_YOUNG: u8 = 3;
pub const CELL_OLD: u8 = 4;
pub const CELL_BURNING: u8 = 5;
pub const CELL_BURNT: u8 = 6;
pub const CELL_REGROWTH: u8 = 7;
/// Open water (river, lake, ocean): nothing grows or burns here.
pub const CELL_WATER: u8 = 8;

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

/// Burn and regrowth timings per fuel kind (grass, brush, timber). Defaults come from the fuel material
/// table (`burn_duration`); the rest is tuning.
#[derive(Resource, Clone, Debug)]
pub struct StandBurnConfig {
    /// Multiplier on the fuel consumption rate: grass burns out fast, timber slow.
    pub burn_rate: [f32; 3],
    /// Seconds a burnt stand stays charred standing dead (snags) before anything comes back.
    pub snag_secs: [f32; 3],
    /// Seconds of regrowth after the snag phase until the stand is live again.
    pub regrow_secs: [f32; 3],
    /// Share of a cell's fuel that is canopy (spared by a ground fire).
    pub crown_share: [f32; 3],
    /// Seconds for surface fuel (litter) to come back after a ground fire.
    pub litter_secs: [f32; 3],
    /// Cell heat at or above which the canopy share burns too.
    pub crown_heat: f32,
    /// Sim ticks between render snapshot refreshes of an active chunk.
    pub snapshot_period_ticks: u64,
}

impl Default for StandBurnConfig {
    fn default() -> Self {
        let rate = |k| (35.0 / fuel_material_def(k).burn_duration.max(1.0)).clamp(0.2, 4.0);
        Self {
            burn_rate: [
                rate(FuelMaterialKind::Grass),
                rate(FuelMaterialKind::Brush),
                rate(FuelMaterialKind::Timber),
            ],
            snag_secs: [6.0, 25.0, 90.0],
            regrow_secs: [25.0, 120.0, 600.0],
            crown_share: [0.0, 0.2, 0.5],
            litter_secs: [8.0, 30.0, 60.0],
            crown_heat: 0.55,
            snapshot_period_ticks: 20,
        }
    }
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
    /// Changes whenever `render` does.
    pub render_rev: u32,
    seeded: bool,
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
            render_rev: 0,
            seeded: false,
            last_refresh: 0,
            last_snapshot: 0,
            last_pass: 0,
        }
    }

    /// Phase of cell `i` at sim time `now` (seconds).
    #[must_use]
    pub fn phase(&self, i: usize, now: f32, cfg: &StandBurnConfig) -> StandPhase {
        let t = self.burnt_at.get(i).copied().unwrap_or(-1.0);
        if t < 0.0 {
            return StandPhase::Live;
        }
        let k = self.kind[i];
        let age = now - t;
        let snag = cfg.snag_secs[k.slot()];
        let regrow = cfg.regrow_secs[k.slot()].max(0.001);
        if age < snag {
            StandPhase::Burnt
        } else if age < snag + regrow {
            StandPhase::Regrowth(((age - snag) / regrow).clamp(0.0, 1.0))
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

        // Burn-out: fuel exhausted while the cell burns.
        for i in 0..n {
            if stand.is_burnt(i) || stand.kind[i] == StandKind::Bare {
                continue;
            }
            let hot = ovl.heat[i] >= CELL_BURN_HEAT || ovl.burn_hold[i] > 0;
            if hot
                && ovl.fuel[i] < MIN_CELL_FUEL_FOR_SPREAD
                && stand.full_cap[i] >= MIN_CELL_FUEL_FOR_SPREAD
            {
                stand.burnt_at[i] = now;
                stand.burnt_count += 1;
            }
        }

        // Phases: current carrying capacity per cell.
        let mut sum_w = 0.0f32;
        let mut lost_w = 0.0f32;
        let mut surface_w = 0.0f32;
        let mut scar_cells = 0u32;
        let mut land_cells = 0u32;
        for i in 0..n {
            let full = stand.full_cap[i];
            let (level, lost) = match stand.phase(i, now, cfg) {
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
                StandPhase::Regrowth(p) => {
                    scar_cells += 1;
                    (p, 1.0 - p)
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

        if reseed || tick.0.wrapping_sub(stand.last_snapshot) >= cfg.snapshot_period_ticks {
            stand.last_snapshot = tick.0;
            snapshot(&mut stand, &matrix, veg, suit, &ovl, now, cfg);
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
        if matrix.water[i].is_open_water() || stand.is_burnt(i) {
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
        if stand.is_burnt(i) && !matrix.water[i].is_open_water() {
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
) {
    let n = matrix.water.len();
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
        let burning = ovl.heat[i] >= CELL_BURN_HEAT && ovl.fuel[i] >= MIN_CELL_FUEL_FOR_SPREAD;
        codes[i] = if burning {
            CELL_BURNING
        } else {
            match stand.phase(i, now, cfg) {
                StandPhase::Burnt => CELL_BURNT,
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
        chunk_fire_overlay_tick, chunk_fuel_profile_tick, ChunkFuelProfile, FireEcologyWitness,
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
            ..d
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
            .init_resource::<FireEcologyWitness>()
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
            assert!(took >= min_secs - 0.1, "{name}: regrown after {took} s, configured {min_secs} s");
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
}
