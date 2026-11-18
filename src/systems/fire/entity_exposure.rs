//! FIRE-ENTITY-1 — site fire stages from [`ChunkFireOverlay`] heat.
//!
//! **LGT-09** (gea4v2 12.2.4.5): entity radiant exposure uses inverse-square falloff from
//! burning cells (not tile-local heat alone). Prior path sampled overlay heat at the receiver
//! tile only — no distance attenuation.
//!
//! Sole writer of [`SiteFireState`] / [`FormationFireExposure`] (stamp). Collapse enqueues
//! [`FireCollapseQueue`] for the construction demolish funnel (no parallel despawn).

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::combat::formation::Formation;
use crate::construction::execute_demolish_at_tile;
use crate::economy::activation::BuildingDefinitionRef;
use crate::entities::production::core::ManufacturingNode;
use crate::strategic::{
    BuildSiteTile, ConstructionSite, PlannedSite, SiteFootprint, TileOccupationBook,
};
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::fire::{
    profile_for_site_archetype, unit_fire_susceptibility, StructureFireProfile,
};
use crate::terrain::generation::{tile_to_chunk_coord, Chunk, ChunkCellMatrix};

use super::cell_spread::RAIN_BLOCKS_SPREAD_MM_H;
use super::types::ChunkFireOverlay;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Irradiance at **1 m** from a cell with `heat == 1` (kW/m²).
pub const RADIANT_IRRADIANCE_AT_1M_KW_M2: f32 = 80.0;
/// Minimum standoff (m) — clamps singularity at the source (`gea4v2` 12.2.4.5).
pub const RADIANT_NEAR_CLAMP_M: f32 = 1.0;
/// Radiant flux that maps to normalized `heat == 1` for gameplay gates (kW/m²).
pub const RADIANT_HEAT_NORM_KW_M2: f32 = 20.0;
/// Ignore contributors beyond this radius (m).
const RADIANT_CONTRIBUTOR_MAX_M: f32 = 160.0;
/// Overlay heat below this is not a radiant source.
const RADIANT_SOURCE_HEAT_MIN: f32 = 0.02;

/// Inverse-square radiant flux density (irradiance) at `distance_m` from a cell at `source_heat`.
#[must_use]
pub fn radiant_irradiance_kw_m2(source_heat: f32, distance_m: f32) -> f32 {
    if source_heat <= 0.0 || !distance_m.is_finite() {
        return 0.0;
    }
    let r = distance_m.max(RADIANT_NEAR_CLAMP_M);
    let i_ref = source_heat * RADIANT_IRRADIANCE_AT_1M_KW_M2;
    i_ref / (r * r)
}

#[must_use]
pub fn tile_center_world_m(tile: IVec2) -> Vec2 {
    let m = TILE_METRES_GAMEPLAY;
    Vec2::new((tile.x as f32 + 0.5) * m, (tile.y as f32 + 0.5) * m)
}

/// Peak radiant flux (kW/m²) at `receiver_tile` from all overlay heat in `index`.
#[must_use]
pub fn sample_radiant_flux_kw_m2(
    index: &HashMap<IVec2, (UVec2, &[f32], f32)>,
    receiver_tile: IVec2,
) -> f32 {
    if receiver_tile.x < 0 || receiver_tile.y < 0 {
        return 0.0;
    }
    let recv = tile_center_world_m(receiver_tile);
    let mut peak = 0.0f32;
    for (&chunk_coord, &(size, heat_slice, _rain)) in index {
        let cw = size.x.max(1) as usize;
        let ch = size.y.max(1) as usize;
        if heat_slice.len() < cw * ch {
            continue;
        }
        let base_tx = chunk_coord.x * size.x as i32;
        let base_tz = chunk_coord.y * size.y as i32;
        for lz in 0..ch {
            for lx in 0..cw {
                let h = heat_slice[lz * cw + lx];
                if h < RADIANT_SOURCE_HEAT_MIN {
                    continue;
                }
                let tx = base_tx + lx as i32;
                let tz = base_tz + lz as i32;
                let d = recv.distance(tile_center_world_m(IVec2::new(tx, tz)));
                if d > RADIANT_CONTRIBUTOR_MAX_M {
                    continue;
                }
                peak = peak.max(radiant_irradiance_kw_m2(h, d));
            }
        }
    }
    peak
}

#[must_use]
pub fn normalized_heat_from_flux_kw_m2(flux_kw_m2: f32) -> f32 {
    (flux_kw_m2 / RADIANT_HEAT_NORM_KW_M2).clamp(0.0, 1.0)
}

/// Burn / exposure stage for one construction site.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SiteFireStage {
    #[default]
    Clear,
    Threatened,
    Disrupted,
    Deteriorating,
    Collapsing,
}

impl SiteFireStage {
    #[must_use]
    pub const fn ops_mul(self) -> f32 {
        match self {
            Self::Clear | Self::Threatened => 1.0,
            Self::Disrupted => 0.65,
            Self::Deteriorating => 0.35,
            Self::Collapsing => 0.0,
        }
    }

    /// 0..1 wash strength for presentation burn overlay.
    #[must_use]
    pub const fn burn_wash01(self) -> f32 {
        match self {
            Self::Clear => 0.0,
            Self::Threatened => 0.18,
            Self::Disrupted => 0.4,
            Self::Deteriorating => 0.72,
            Self::Collapsing => 1.0,
        }
    }
}

/// Per-site fire exposure (component on the construction site entity).
#[derive(Component, Clone, Debug)]
pub struct SiteFireState {
    pub stage: SiteFireStage,
    /// 1 = intact, 0 = ready to collapse.
    pub integrity: f32,
    pub heat_mean: f32,
    /// LGT-09 mean radiant flux across the footprint (kW/m²).
    pub flux_kw_m2: f32,
    pub suppression: bool,
    pub profile: StructureFireProfile,
}

impl Default for SiteFireState {
    fn default() -> Self {
        Self {
            stage: SiteFireStage::Clear,
            integrity: 1.0,
            heat_mean: 0.0,
            flux_kw_m2: 0.0,
            suppression: false,
            profile: profile_for_site_archetype(crate::strategic::SiteArchetype::CivilHousing),
        }
    }
}

/// Heat stamp on a formation — resolve is the sole strength/morale writer.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct FormationFireExposure {
    /// Normalized 0..1 heat for legacy gates (derived from [`Self::flux_kw_m2`] when stamped).
    pub heat: f32,
    pub suscept: f32,
    /// LGT-09 radiant flux at the formation tile (kW/m²).
    pub flux_kw_m2: f32,
}

impl FormationFireExposure {
    #[must_use]
    pub fn exposure(self) -> f32 {
        let h = self
            .heat
            .max(normalized_heat_from_flux_kw_m2(self.flux_kw_m2));
        (h * self.suscept).clamp(0.0, 1.0)
    }
}

/// Sites that reached collapse this tick — processed by [`apply_fire_collapse_demolish`].
#[derive(Resource, Default, Debug)]
pub struct FireCollapseQueue {
    pub tiles: Vec<BuildSiteTile>,
    pub collapsed: u64,
}

/// Dump / matrix counters.
#[derive(Resource, Default, Debug, Clone)]
pub struct FireEntityExposureStats {
    pub sites_threatened: u32,
    pub sites_disrupted: u32,
    pub sites_deteriorating: u32,
    pub sites_collapsed_to_fire: u64,
    pub formations_exposed: u32,
    pub formation_heat_sum: f32,
    pub vehicles_delayed: u64,
    pub vehicles_lost_to_fire: u64,
    pub cargo_lost_to_fire: u64,
}

/// Presentation-only footprint wash (tile → 0..1). Written from [`SiteFireState`]; never writes sim.
#[derive(Resource, Default, Debug, Clone)]
pub struct SiteBurnWashFrame {
    pub tiles: HashMap<IVec2, f32>,
}

/// Sample overlay heat (+ rain mm/h) at a world tile.
#[must_use]
pub fn sample_heat_at_tile(
    by_chunk: &HashMap<IVec2, (UVec2, &[f32], f32)>,
    tile: IVec2,
) -> (f32, f32) {
    if tile.x < 0 || tile.y < 0 {
        return (0.0, 0.0);
    }
    let tx = tile.x as u32;
    let tz = tile.y as u32;
    for (&coord, &(size, heat, rain)) in by_chunk {
        let cw = size.x.max(1);
        let ch = size.y.max(1);
        let c = tile_to_chunk_coord(tx, tz, size);
        if c != coord {
            continue;
        }
        let lx = tx % cw;
        let lz = tz % ch;
        let i = (lz * cw + lx) as usize;
        let h = heat.get(i).copied().unwrap_or(0.0);
        return (h, rain);
    }
    (0.0, 0.0)
}

/// Build heat index from chunk overlays (borrowed heat slices).
pub fn build_heat_index<'a>(
    q: impl Iterator<
        Item = (
            &'a Chunk,
            &'a ChunkCellMatrix,
            &'a ChunkFireOverlay,
            Option<&'a ChunkWeather>,
        ),
    >,
) -> HashMap<IVec2, (UVec2, &'a [f32], f32)> {
    let mut map = HashMap::new();
    for (chunk, matrix, ovl, wx) in q {
        let n = (matrix.size.x * matrix.size.y) as usize;
        if ovl.heat.len() != n {
            continue;
        }
        let rain = wx.map(|w| w.rain_mm_h).unwrap_or(0.0);
        map.insert(chunk.coord, (matrix.size, ovl.heat.as_slice(), rain));
    }
    map
}

/// World xz (gameplay metres) → approximate world tile for heat sample.
#[must_use]
pub fn xz_to_world_tile(xz: Vec2) -> IVec2 {
    let m = crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;
    IVec2::new((xz.x / m).floor() as i32, (xz.y / m).floor() as i32)
}

pub fn ensure_site_fire_state(
    mut commands: Commands,
    q: Query<(Entity, &PlannedSite), (With<ConstructionSite>, Without<SiteFireState>)>,
) {
    for (e, planned) in &q {
        commands.entity(e).insert((
            SiteFireState {
                profile: profile_for_site_archetype(planned.archetype),
                ..Default::default()
            },
            crate::construction::DestructibleDamage::default(),
        ));
    }
}

/// Sites that already have fire state but predate PHYS-08 damage components.
pub fn ensure_site_destructible_damage(
    mut commands: Commands,
    q: Query<
        Entity,
        (
            With<ConstructionSite>,
            With<SiteFireState>,
            Without<crate::construction::DestructibleDamage>,
        ),
    >,
) {
    for e in &q {
        commands
            .entity(e)
            .insert(crate::construction::DestructibleDamage::default());
    }
}

pub fn ensure_formation_fire_exposure(
    mut commands: Commands,
    q: Query<(Entity, &Formation), Without<FormationFireExposure>>,
) {
    for (e, f) in &q {
        commands.entity(e).insert(FormationFireExposure {
            heat: 0.0,
            suscept: unit_fire_susceptibility(f.kind),
            flux_kw_m2: 0.0,
        });
    }
}

/// Sole writer of [`SiteFireState`] (except despawn on collapse).
pub fn site_fire_exposure_tick(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    mut stats: ResMut<FireEntityExposureStats>,
    mut collapse: ResMut<FireCollapseQueue>,
    chunks: Query<(
        &Chunk,
        &ChunkCellMatrix,
        &ChunkFireOverlay,
        Option<&ChunkWeather>,
    )>,
    mut sites: Query<(
        &SiteFootprint,
        &PlannedSite,
        &mut SiteFireState,
        Option<&mut crate::construction::DestructibleDamage>,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }
    let index = build_heat_index(chunks.iter());
    stats.sites_threatened = 0;
    stats.sites_disrupted = 0;
    stats.sites_deteriorating = 0;

    for (footprint, planned, mut state, destructible) in &mut sites {
        state.profile = profile_for_site_archetype(planned.archetype);
        let mut flux_sum = 0.0f32;
        let mut rain_max = 0.0f32;
        let mut n = 0u32;
        for &tile in &footprint.tiles {
            let (_h, rain) = sample_heat_at_tile(&index, tile);
            flux_sum += sample_radiant_flux_kw_m2(&index, tile);
            rain_max = rain_max.max(rain);
            n += 1;
        }
        let flux = if n > 0 {
            flux_sum / n as f32
        } else {
            0.0
        };
        state.flux_kw_m2 = flux;
        let heat = normalized_heat_from_flux_kw_m2(flux);
        state.heat_mean = heat;
        state.suppression = rain_max >= RAIN_BLOCKS_SPREAD_MM_H;

        let ignite = (1.0 - state.profile.ignition_resistance).clamp(0.05, 1.0);
        let threat_gate = 0.06 * (0.4 + ignite);
        let disrupt_gate = 0.14 * (0.5 + ignite * 0.5);

        if heat >= disrupt_gate && !state.suppression {
            let rate = state.profile.fuel_load.max(0.2)
                * ignite
                * (heat - disrupt_gate * 0.5).max(0.0)
                * 0.08;
            state.integrity = (state.integrity - rate * dt).clamp(0.0, 1.0);
        } else if heat < threat_gate && state.suppression {
            state.integrity = (state.integrity + 0.02 * dt).min(1.0);
        } else if heat < threat_gate * 0.5 {
            state.integrity = (state.integrity + 0.01 * dt).min(1.0);
        }

        let collapse_bar = state.profile.collapse_threshold.clamp(0.05, 0.5);
        state.stage = if state.integrity <= collapse_bar && heat >= disrupt_gate && !state.suppression
        {
            SiteFireStage::Collapsing
        } else if state.integrity < 0.85 && heat >= disrupt_gate {
            SiteFireStage::Deteriorating
        } else if heat >= disrupt_gate {
            SiteFireStage::Disrupted
        } else if heat >= threat_gate {
            SiteFireStage::Threatened
        } else if state.integrity >= 0.98 {
            SiteFireStage::Clear
        } else {
            SiteFireStage::Disrupted
        };

        match state.stage {
            SiteFireStage::Threatened => {
                stats.sites_threatened = stats.sites_threatened.saturating_add(1);
            }
            SiteFireStage::Disrupted => {
                stats.sites_disrupted = stats.sites_disrupted.saturating_add(1);
            }
            SiteFireStage::Deteriorating | SiteFireStage::Collapsing => {
                stats.sites_deteriorating = stats.sites_deteriorating.saturating_add(1);
            }
            _ => {}
        }

        // PHYS-08: map remaining integrity onto the construction damage ladder.
        let ladder = crate::construction::destructible_stage_from_integrity(
            state.integrity,
            collapse_bar,
        );
        if let Some(mut dmg) = destructible {
            dmg.stage = dmg.stage.escalate(ladder);
        }

        if state.stage == SiteFireStage::Collapsing {
            if let Some(&tile) = footprint.tiles.first() {
                collapse.tiles.push(BuildSiteTile {
                    x: tile.x.max(0) as u32,
                    z: tile.y.max(0) as u32,
                });
            }
        }
    }
}

/// Stamp [`FormationFireExposure`] from overlay heat (resolve reads; does not write strength).
pub fn stamp_formation_fire_exposure(
    ctrl: Res<SimControlState>,
    mut stats: ResMut<FireEntityExposureStats>,
    chunks: Query<(
        &Chunk,
        &ChunkCellMatrix,
        &ChunkFireOverlay,
        Option<&ChunkWeather>,
    )>,
    mut q: Query<(&Formation, &mut FormationFireExposure)>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let index = build_heat_index(chunks.iter());
    stats.formations_exposed = 0;
    stats.formation_heat_sum = 0.0;
    for (f, mut exp) in &mut q {
        let flux = sample_radiant_flux_kw_m2(&index, f.tile);
        let heat = normalized_heat_from_flux_kw_m2(flux);
        exp.flux_kw_m2 = flux;
        exp.heat = heat;
        exp.suscept = unit_fire_susceptibility(f.kind);
        if heat >= 0.08 {
            stats.formations_exposed = stats.formations_exposed.saturating_add(1);
            stats.formation_heat_sum += heat;
        }
    }
}

/// Drain [`FireCollapseQueue`] through the existing demolish execute funnel.
pub fn apply_fire_collapse_demolish(
    mut commands: Commands,
    mut queue: ResMut<FireCollapseQueue>,
    mut stats: ResMut<FireEntityExposureStats>,
    mut occupation: Option<ResMut<TileOccupationBook>>,
    mut voxel_occupation: Option<ResMut<crate::strategic::VoxelOccupationBook>>,
    voxel_cfg: Option<Res<crate::strategic::VoxelOccupationConfig>>,
    mut destruction_mask: Option<ResMut<crate::render::pixel_world::DestructionMaskBook>>,
    mut debris_frame: Option<ResMut<crate::render::WorldFireParticleFrame>>,
    sites: Query<(
        Entity,
        &ConstructionSite,
        &PlannedSite,
        &SiteFootprint,
        Option<&BuildingDefinitionRef>,
    )>,
) {
    if queue.tiles.is_empty() {
        return;
    }
    let commit_auth = voxel_cfg
        .as_ref()
        .is_some_and(|c| c.commit_authority_enabled);
    let tiles = std::mem::take(&mut queue.tiles);
    let mut seen = HashSet::new();
    for tile in tiles {
        let key = (tile.x, tile.z);
        if !seen.insert(key) {
            continue;
        }
        let (n, _) = execute_demolish_at_tile(
            &mut commands,
            tile,
            &sites,
            occupation.as_deref_mut(),
            voxel_occupation.as_deref_mut(),
            commit_auth,
            destruction_mask.as_deref_mut(),
            debris_frame.as_deref_mut(),
        );
        if n > 0 {
            queue.collapsed = queue.collapsed.saturating_add(u64::from(n));
            stats.sites_collapsed_to_fire =
                stats.sites_collapsed_to_fire.saturating_add(u64::from(n));
        }
    }
}

/// Derate plant throughput from [`SiteFireState`] (after MFG tick).
pub fn apply_site_fire_ops_derate(
    mut q: Query<(&SiteFireState, &mut ManufacturingNode)>,
) {
    for (fire, mut node) in &mut q {
        let mul = fire.stage.ops_mul();
        if mul < 1.0 {
            node.current_throughput *= mul;
        }
    }
}

/// Presentation: copy footprint wash strengths for raster / stamps.
pub fn sync_site_burn_wash_frame(
    mut frame: ResMut<SiteBurnWashFrame>,
    sites: Query<(
        &SiteFootprint,
        &SiteFireState,
        Option<&crate::construction::DestructibleDamage>,
    )>,
) {
    frame.tiles.clear();
    for (fp, state, destructible) in &sites {
        let mut wash = state.stage.burn_wash01();
        if let Some(dmg) = destructible {
            wash = wash.max(dmg.stage.visual_wash01());
        }
        if wash <= 0.0 {
            continue;
        }
        for &tile in &fp.tiles {
            let e = frame.tiles.entry(tile).or_insert(0.0);
            *e = (*e).max(wash);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::balance::UnitKind;
    use crate::strategic::SiteArchetype;
    use crate::terrain::fire::profile_for_site_archetype;

    #[test]
    fn wood_loses_integrity_faster_than_concrete_under_same_heat() {
        let wood = profile_for_site_archetype(SiteArchetype::CivilHousing);
        let concrete = profile_for_site_archetype(SiteArchetype::MilitaryBase);
        let heat = 0.5f32;
        let wood_ign = 1.0 - wood.ignition_resistance;
        let conc_ign = 1.0 - concrete.ignition_resistance;
        let wood_rate = wood.fuel_load * wood_ign * heat * 0.08;
        let conc_rate = concrete.fuel_load * conc_ign * heat * 0.08;
        assert!(
            wood_rate > conc_rate * 1.5,
            "wood {wood_rate} concrete {conc_rate}"
        );
    }

    #[test]
    fn stage_ops_mul_monotone() {
        assert!(SiteFireStage::Clear.ops_mul() > SiteFireStage::Disrupted.ops_mul());
        assert!(SiteFireStage::Disrupted.ops_mul() > SiteFireStage::Deteriorating.ops_mul());
        assert_eq!(SiteFireStage::Collapsing.ops_mul(), 0.0);
    }

    #[test]
    fn infantry_exposure_higher_than_armor() {
        let inf = FormationFireExposure {
            heat: 0.5,
            suscept: unit_fire_susceptibility(UnitKind::Infantry),
            flux_kw_m2: 0.0,
        };
        let armor = FormationFireExposure {
            heat: 0.5,
            suscept: unit_fire_susceptibility(UnitKind::Armor),
            flux_kw_m2: 0.0,
        };
        assert!(inf.exposure() > armor.exposure());
    }

    #[test]
    fn radiant_flux_quarter_at_double_distance_outside_near_clamp() {
        let source_heat = 1.0;
        let d = 12.0_f32;
        let i_near = radiant_irradiance_kw_m2(source_heat, d);
        let i_far = radiant_irradiance_kw_m2(source_heat, 2.0 * d);
        let ratio = i_far / i_near;
        assert!(
            (ratio - 0.25).abs() < 1e-5,
            "expected 1/4 at 2d, got ratio {ratio} (near={i_near} far={i_far})"
        );
    }

    #[test]
    fn radiant_flux_clamped_inside_near_radius() {
        let source_heat = 0.8;
        let at_clamp = radiant_irradiance_kw_m2(source_heat, RADIANT_NEAR_CLAMP_M);
        let inside = radiant_irradiance_kw_m2(source_heat, RADIANT_NEAR_CLAMP_M * 0.25);
        assert!((at_clamp - inside).abs() < 1e-6, "inside clamp must match clamp distance");
    }
}
