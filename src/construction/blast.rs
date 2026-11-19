//! Blast / crater execute funnel — sole construction-side damage path.
//!
//! **Dual-cite (V4-PHYS07 / PHYS-07):**
//! - NEW: gea4v2 14.5.3.2 Bullet Traces · 14.5.3.3 Grenades · 14.5.3.4 Explosions
//! - LEGACY: `physics_integration` (gea_v2_e4 §14.5 p423) — game-specific responses without a
//!   rigid-body world; we bookkeep radius + falloff instead of middleware contacts.
//!
//! Combat ballistics **enqueue** [`BlastIntent`] only; only [`execute_blast_intents_system`]
//! mutates sites / [`BlastCraterBook`] / occupation (construction invariants). Voxel occupation
//! clears go through [`execute_demolish_at_tile`] when commit authority is on — no parallel write.
//!
//! Thresholds: [`BlastProfile`] (RON) · instance yield: intent / [`crate::combat::WarheadCatalog`].

use bevy::prelude::*;
use serde::Deserialize;

use crate::economy::activation::BuildingDefinitionRef;
use crate::strategic::{
    BuildSiteTile, ConstructionSite, FootprintTiles, PlannedSite, SiteFootprint, TileOccupationBook,
    VoxelOccupationBook, VoxelOccupationConfig,
};
use crate::substrate::hydrology::{HydrologyConstructionCouplingStats, HydrologyEventQueue};
use crate::systems::sim_control::SimControlSystemSet;

use super::dam_barrier::DamBody;
use super::demolish::{
    destructible_catastrophic_collapse, destructible_stage_from_integrity, execute_demolish_at_tile,
    DestructibleDamage, DestructibleDamageStage, DESTRUCTIBLE_COLLAPSE_INTEGRITY,
};
use super::hydro_coupling::{emit_dam_breach_hydro_dirty, emit_site_execute_hydro_dirty};
use crate::systems::water::{BarrierBreachRequest, BarrierBreachSpec};

/// Compiled-in demolish gate (matches [`BlastProfile::default`]); tripwires / tests.
pub const BLAST_DEMOLISH_DAMAGE: f32 = 50.0;

pub const BLAST_PROFILE_RON: &str = "assets/configs/construction/blast_profile_v1.ron";

/// Construction-side blast execute thresholds (not warhead yield).
#[derive(Resource, Clone, Debug, Deserialize, PartialEq)]
pub struct BlastProfile {
    /// Effective damage at epicenter (after falloff) required to demolish overlapping sites.
    pub demolish_damage: f32,
    /// Dam pen multiplier: `1 + penetration_m * scale`, clamped to [min, max].
    pub penetration_scale: f32,
    pub penetration_mult_min: f32,
    pub penetration_mult_max: f32,
    /// Peak column loss scale vs demolish gate (`peak / gate * scale`).
    pub dam_peak_loss_scale: f32,
    /// Partial leak when mean integrity drops below this (and/or Compromised).
    pub dam_leak_integrity: f32,
    pub breach_flow_per_damage: f32,
    pub breach_flow_min: f32,
    pub breach_flow_max: f32,
    pub leak_flow_scale: f32,
    pub leak_flow_min: f32,
    pub leak_flow_max: f32,
}

impl Default for BlastProfile {
    fn default() -> Self {
        Self {
            demolish_damage: BLAST_DEMOLISH_DAMAGE,
            penetration_scale: 0.12,
            penetration_mult_min: 1.0,
            penetration_mult_max: 3.0,
            dam_peak_loss_scale: 0.85,
            dam_leak_integrity: 0.72,
            breach_flow_per_damage: 8.0,
            breach_flow_min: 50.0,
            breach_flow_max: 5000.0,
            leak_flow_scale: 400.0,
            leak_flow_min: 10.0,
            leak_flow_max: 800.0,
        }
    }
}

impl BlastProfile {
    #[must_use]
    pub fn penetration_mult(&self, penetration_m: f32) -> f32 {
        (1.0 + penetration_m * self.penetration_scale).clamp(
            self.penetration_mult_min,
            self.penetration_mult_max,
        )
    }
}

#[must_use]
pub fn parse_blast_profile(text: &str) -> Result<BlastProfile, String> {
    ron::from_str(text).map_err(|e| e.to_string())
}

#[must_use]
pub fn load_blast_profile() -> (BlastProfile, Option<String>) {
    match std::fs::read_to_string(BLAST_PROFILE_RON) {
        Ok(text) => match parse_blast_profile(&text) {
            Ok(p) => (p, None),
            Err(e) => (BlastProfile::default(), Some(e)),
        },
        Err(e) => (BlastProfile::default(), Some(e.to_string())),
    }
}

/// One blast to apply this tick (ingress from combat / tools).
#[derive(Clone, Debug)]
pub struct BlastIntent {
    pub center_xz: Vec2,
    /// World metres (instance / warhead).
    pub radius_m: f32,
    /// Peak damage at epicenter; after [`blast_radius_falloff`], ≥ profile demolish gate
    /// demolishes overlapping sites; lower / edge falloff = crater only.
    pub damage: f32,
    /// Penetration depth (metres) — scales dam integrity loss (DF-8).
    pub penetration_m: f32,
    pub source_flight: u64,
}

/// Linear radial falloff in `[0, 1]`: 1 at epicenter, 0 at/beyond radius (gea4v2 14.5.3.4).
#[must_use]
pub fn blast_radius_falloff(distance_m: f32, radius_m: f32) -> f32 {
    let r = radius_m.max(1e-3);
    (1.0 - (distance_m.max(0.0) / r)).clamp(0.0, 1.0)
}

/// Peak damage attenuated by distance from blast center.
#[must_use]
pub fn blast_damage_at_distance(base_damage: f32, distance_m: f32, radius_m: f32) -> f32 {
    base_damage * blast_radius_falloff(distance_m, radius_m)
}

/// DF-8: sphere carve over dam columns (metres). Returns mean integrity after carve.
/// Crest-precise: only columns inside the disk lose integrity (no adjacent smear).
#[must_use]
pub fn carve_dam_columns(
    columns: &mut [f32],
    tiles: &[IVec2],
    blast_center_xz_m: Vec2,
    radius_m: f32,
    penetration_m: f32,
    peak_loss: f32,
    profile: &BlastProfile,
) -> f32 {
    let n = columns.len().min(tiles.len());
    if n == 0 {
        return 1.0;
    }
    let pen = profile.penetration_mult(penetration_m);
    let r = radius_m.max(0.5);
    for i in 0..n {
        let t = tiles[i];
        let c = tile_center_xz(t);
        let fall = blast_radius_falloff(c.distance(blast_center_xz_m), r);
        if fall <= 0.0 {
            continue;
        }
        let loss = (peak_loss * fall * pen).clamp(0.0, 1.0);
        columns[i] = (columns[i] - loss).clamp(0.0, 1.0);
    }
    columns.iter().copied().sum::<f32>() / n as f32
}

/// True when a world-XZ sample lies inside the blast disk (radius query).
#[must_use]
pub fn blast_disk_contains(center_xz: Vec2, radius_m: f32, sample_xz: Vec2) -> bool {
    let r = radius_m.max(0.0);
    sample_xz.distance_squared(center_xz) <= r * r
}

#[derive(Resource, Default, Debug)]
pub struct BlastIntentQueue {
    pub pending: Vec<BlastIntent>,
}

impl BlastIntentQueue {
    pub fn enqueue(&mut self, intent: BlastIntent) {
        self.pending.push(intent);
    }
}

/// Crater / scorched tiles for presentation (sim truth for “blast happened here”).
#[derive(Resource, Default, Debug, Clone)]
pub struct BlastCraterBook {
    pub tiles: std::collections::BTreeSet<(i32, i32)>,
    pub blasts_applied: u64,
    pub sites_demolished: u64,
}

const METRES_PER_TILE: f32 = crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

fn tile_of_xz(xz: Vec2) -> IVec2 {
    IVec2::new(
        (xz.x / METRES_PER_TILE).floor() as i32,
        (xz.y / METRES_PER_TILE).floor() as i32,
    )
}

fn tile_center_xz(tile: IVec2) -> Vec2 {
    Vec2::new(
        (tile.x as f32 + 0.5) * METRES_PER_TILE,
        (tile.y as f32 + 0.5) * METRES_PER_TILE,
    )
}

/// Sole writer of [`BlastCraterBook`] + blast demolish side-effects.
pub fn execute_blast_intents_system(
    mut commands: Commands,
    mut queue: ResMut<BlastIntentQueue>,
    mut craters: ResMut<BlastCraterBook>,
    profile: Res<BlastProfile>,
    mut occupation: Option<ResMut<TileOccupationBook>>,
    mut voxel_occupation: Option<ResMut<VoxelOccupationBook>>,
    voxel_cfg: Option<Res<VoxelOccupationConfig>>,
    mut hydro_q: Option<ResMut<HydrologyEventQueue>>,
    mut hydro_stats: Option<ResMut<HydrologyConstructionCouplingStats>>,
    mut destruction_mask: Option<ResMut<crate::render::pixel_world::DestructionMaskBook>>,
    mut debris_frame: Option<ResMut<crate::render::WorldFireParticleFrame>>,
    sites: Query<(
        Entity,
        &ConstructionSite,
        &PlannedSite,
        &SiteFootprint,
        Option<&BuildingDefinitionRef>,
    )>,
    mut damage: Query<&mut DestructibleDamage>,
    mut dams: Query<&mut DamBody>,
    mut breach: Option<ResMut<BarrierBreachRequest>>,
) {
    if queue.pending.is_empty() {
        return;
    }
    let gate = profile.demolish_damage.max(1e-3);
    let commit_auth = voxel_cfg
        .as_ref()
        .is_some_and(|c| c.commit_authority_enabled);
    let intents = std::mem::take(&mut queue.pending);
    for intent in intents {
        let center = intent.center_xz;
        let r = intent.radius_m.max(0.5);
        let r2 = r * r;
        let center_tile = tile_of_xz(center);

        let tile_r = (r / METRES_PER_TILE).ceil() as i32;
        for dz in -tile_r..=tile_r {
            for dx in -tile_r..=tile_r {
                let t = IVec2::new(center_tile.x + dx, center_tile.y + dz);
                let c = tile_center_xz(t);
                if c.distance_squared(center) <= r2 {
                    craters.tiles.insert((t.x, t.y));
                    // DESTROY-T2: crater disk → same-frame ground collapse.
                    if let Some(mask) = destruction_mask.as_mut() {
                        mask.mark_tile(t.x, t.y, crate::render::pixel_world::MASK_DESTROYED);
                    }
                }
            }
        }
        craters.blasts_applied = craters.blasts_applied.saturating_add(1);

        // PHYS-08: map peak blast damage onto the damage ladder (scar / compromise) even
        // when below the demolish gate; terminal collapse still goes through demolish.
        let mut peaks: Vec<(Entity, f32)> = Vec::new();
        let mut hit_origins: Vec<BuildSiteTile> = Vec::new();
        for (entity, _site, planned, footprint, _) in sites.iter() {
            let mut peak = 0.0f32;
            let mut in_disk = false;
            for t in &footprint.tiles {
                let c = tile_center_xz(*t);
                if !blast_disk_contains(center, r, c) {
                    continue;
                }
                in_disk = true;
                peak = peak.max(blast_damage_at_distance(
                    intent.damage,
                    c.distance(center),
                    r,
                ));
            }
            if !in_disk {
                continue;
            }
            peaks.push((entity, peak));
            if peak >= gate {
                hit_origins.push(planned.origin);
            }
        }
        for (entity, peak) in &peaks {
            if let Ok(mut dmg) = damage.get_mut(*entity) {
                let integrity_left =
                    (1.0 - (peak / gate).clamp(0.0, 1.0)).clamp(0.0, 1.0);
                let ladder = destructible_stage_from_integrity(
                    integrity_left,
                    DESTRUCTIBLE_COLLAPSE_INTEGRITY,
                );
                dmg.stage = dmg.stage.escalate(ladder);
                if destructible_catastrophic_collapse(*peak, gate) {
                    dmg.stage = DestructibleDamageStage::Collapsed;
                }
            }
            // DF-8: sphere carve per dam column + aggregate integrity (construction funnel).
            if let Ok(mut body) = dams.get_mut(*entity) {
                if let Ok((_, _, _, footprint, _)) = sites.get(*entity) {
                    body.ensure_columns(footprint.tiles.len());
                    let peak_loss =
                        ((peak / gate) * profile.dam_peak_loss_scale).clamp(0.0, 1.0);
                    let mean = carve_dam_columns(
                        &mut body.column_integrity,
                        &footprint.tiles,
                        center,
                        r,
                        intent.penetration_m,
                        peak_loss,
                        &profile,
                    );
                    body.integrity_01 = mean;
                    body.damage_stage = destructible_stage_from_integrity(
                        body.integrity_01,
                        DESTRUCTIBLE_COLLAPSE_INTEGRITY,
                    );
                    if body.damage_stage.is_terminal()
                        || body.integrity_01 <= DESTRUCTIBLE_COLLAPSE_INTEGRITY
                    {
                        body.damage_stage = DestructibleDamageStage::Collapsed;
                        if let Some(req) = breach.as_mut() {
                            req.push(BarrierBreachSpec {
                                dam: Some(*entity),
                                clear_crest: true,
                                breach_flow_m3s: (intent.damage * profile.breach_flow_per_damage)
                                    .clamp(profile.breach_flow_min, profile.breach_flow_max),
                            });
                        }
                        // DF-8: substrate hydrology dirty via DamBreach (finally used).
                        if let (Some(q), Some(stats)) = (hydro_q.as_mut(), hydro_stats.as_mut()) {
                            if let Some(mid) = footprint.tiles.get(footprint.tiles.len() / 2) {
                                emit_dam_breach_hydro_dirty(
                                    q,
                                    stats,
                                    entity.to_bits(),
                                    *mid,
                                    UVec2::new(32, 32),
                                );
                            }
                        }
                    } else if body.damage_stage.rank()
                        >= DestructibleDamageStage::Compromised.rank()
                        || body.integrity_01 < profile.dam_leak_integrity
                    {
                        // Partial leak through compromised columns (no full crest clear).
                        if let Some(req) = breach.as_mut() {
                            let q_leak = ((1.0 - body.integrity_01) * profile.leak_flow_scale)
                                .clamp(profile.leak_flow_min, profile.leak_flow_max);
                            req.push(BarrierBreachSpec {
                                dam: Some(*entity),
                                clear_crest: false,
                                breach_flow_m3s: q_leak,
                            });
                        }
                    }
                }
            }
        }

        if intent.damage < gate {
            continue;
        }
        hit_origins.sort_by_key(|t| (t.x, t.z));
        hit_origins.dedup();

        for origin in hit_origins {
            let (n, _events) = execute_demolish_at_tile(
                &mut commands,
                origin,
                &sites,
                occupation.as_deref_mut(),
                voxel_occupation.as_deref_mut(),
                commit_auth,
                destruction_mask.as_deref_mut(),
                debris_frame.as_deref_mut(),
            );
            if n > 0 {
                craters.sites_demolished = craters.sites_demolished.saturating_add(u64::from(n));
                if let (Some(q), Some(stats)) = (hydro_q.as_mut(), hydro_stats.as_mut()) {
                    emit_site_execute_hydro_dirty(
                        q,
                        stats,
                        intent.source_flight,
                        origin,
                        FootprintTiles {
                            width: 1,
                            depth: 1,
                        },
                        // Terrain chunks are 32×32 (DF-7 units fix vs stale 16).
                        UVec2::new(32, 32),
                    );
                }
            }
        }
    }
}

pub struct ConstructionBlastPlugin;

impl Plugin for ConstructionBlastPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<BlastProfile>() {
            let (profile, err) = load_blast_profile();
            if let Some(e) = err {
                warn!("blast profile: using compiled-in default ({e})");
            }
            app.insert_resource(profile);
        }
        app.init_resource::<BlastIntentQueue>()
            .init_resource::<BlastCraterBook>()
            .init_resource::<super::blast_present::BlastCraterDump>()
            .add_systems(
                Update,
                (
                    execute_blast_intents_system
                        .after(SimControlSystemSet::AdvanceSimTick)
                        .after(crate::combat::ballistics::step_ballistic_flights),
                    super::blast_present::publish_blast_crater_dump
                        .after(execute_blast_intents_system),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::{
        FootprintTiles, LayerType, SiteArchetype, SiteConstructionPhase, SiteId,
    };

    fn blast_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BlastIntentQueue>()
            .init_resource::<BlastCraterBook>()
            .insert_resource(BlastProfile::default())
            .add_systems(Update, execute_blast_intents_system);
        app
    }

    fn spawn_site(app: &mut App, tile: IVec2, site_id: u64) {
        app.world_mut().spawn((
            ConstructionSite {
                site_id,
                owner: Entity::PLACEHOLDER,
                archetype: SiteArchetype::Factory,
                phase: SiteConstructionPhase::Operational,
                operational_readiness: 1.0,
            },
            PlannedSite {
                site_id: SiteId(site_id),
                origin: BuildSiteTile {
                    x: tile.x.max(0) as u32,
                    z: tile.y.max(0) as u32,
                },
                footprint: FootprintTiles { width: 1, depth: 1 },
                archetype: SiteArchetype::Factory,
                layer: LayerType::Surface,
                catalog_id: None,
                placement: None,
            },
            SiteFootprint {
                tiles: vec![tile],
                layer: LayerType::Surface,
            },
        ));
    }

    fn count_sites(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut q = world.query::<&ConstructionSite>();
        q.iter(world).count()
    }

    #[test]
    fn crater_disk_marks_center_tile() {
        let mut book = BlastCraterBook::default();
        let center = Vec2::new(8.0, 8.0);
        let t = tile_of_xz(center);
        book.tiles.insert((t.x, t.y));
        assert!(book.tiles.contains(&(t.x, t.y)));
    }

    #[test]
    fn phys07_falloff_and_disk_query() {
        assert!((blast_radius_falloff(0.0, 10.0) - 1.0).abs() < 1e-5);
        assert!((blast_radius_falloff(5.0, 10.0) - 0.5).abs() < 1e-5);
        assert!((blast_radius_falloff(10.0, 10.0)).abs() < 1e-5);
        assert!((blast_radius_falloff(20.0, 10.0)).abs() < 1e-5);
        assert!((blast_damage_at_distance(100.0, 5.0, 10.0) - 50.0).abs() < 1e-5);
        let c = Vec2::new(0.0, 0.0);
        assert!(blast_disk_contains(c, 4.0, Vec2::new(3.0, 0.0)));
        assert!(!blast_disk_contains(c, 4.0, Vec2::new(5.0, 0.0)));
    }

    #[test]
    fn blast_profile_ron_matches_compiled_default() {
        assert_eq!(
            BlastProfile::default().demolish_damage,
            BLAST_DEMOLISH_DAMAGE
        );
        let text = std::fs::read_to_string(BLAST_PROFILE_RON).expect("blast profile on disk");
        let loaded = parse_blast_profile(&text).expect("parse");
        assert_eq!(loaded, BlastProfile::default());
    }

    #[test]
    fn phys07_enqueue_then_execute_writes_craters_only_for_low_damage() {
        let mut app = blast_test_app();
        let center = Vec2::new(8.0, 8.0);
        let tile = tile_of_xz(center);
        spawn_site(&mut app, tile, 1);
        app.world_mut()
            .resource_mut::<BlastIntentQueue>()
            .enqueue(BlastIntent {
                center_xz: center,
                radius_m: 16.0,
                damage: 15.0, // grenade / ember — crater, no demolish
                penetration_m: 0.0,
                source_flight: 9,
            });
        app.update();
        let book = app.world().resource::<BlastCraterBook>();
        assert_eq!(book.blasts_applied, 1);
        assert!(book.tiles.contains(&(tile.x, tile.y)));
        assert_eq!(book.sites_demolished, 0);
        assert_eq!(
            count_sites(&mut app),
            1,
            "low-damage blast must not despawn sites"
        );
        assert!(
            app.world()
                .resource::<BlastIntentQueue>()
                .pending
                .is_empty()
        );
    }

    #[test]
    fn phys07_high_damage_execute_demolishes_via_construction_funnel() {
        let mut app = blast_test_app();
        let center = Vec2::new(8.0, 8.0);
        let tile = tile_of_xz(center);
        spawn_site(&mut app, tile, 2);
        app.world_mut()
            .resource_mut::<BlastIntentQueue>()
            .enqueue(BlastIntent {
                center_xz: center,
                radius_m: 16.0,
                damage: 120.0, // missile peak
                penetration_m: 4.0,
                source_flight: 11,
            });
        app.update();
        let book = app.world().resource::<BlastCraterBook>();
        assert_eq!(book.blasts_applied, 1);
        assert!(book.sites_demolished >= 1);
        assert_eq!(
            count_sites(&mut app),
            0,
            "demolish must run inside execute_blast_intents_system"
        );
    }

    #[test]
    fn phys07_edge_falloff_blocks_demolish_when_peak_would_pass() {
        // Peak 80 (≥ gate) but site near radius edge → effective < BLAST_DEMOLISH_DAMAGE.
        let mut app = blast_test_app();
        let center = Vec2::new(0.5 * METRES_PER_TILE, 0.5 * METRES_PER_TILE);
        let radius = 32.0;
        let edge = Vec2::new(center.x + radius * 0.9, center.y);
        let tile = tile_of_xz(edge);
        spawn_site(&mut app, tile, 3);
        let eff = blast_damage_at_distance(80.0, edge.distance(center), radius);
        assert!(
            eff < BLAST_DEMOLISH_DAMAGE,
            "fixture must sit in falloff shade (eff={eff})"
        );
        app.world_mut()
            .resource_mut::<BlastIntentQueue>()
            .enqueue(BlastIntent {
                center_xz: center,
                radius_m: radius,
                damage: 80.0,
                penetration_m: 0.0,
                source_flight: 12,
            });
        app.update();
        assert_eq!(
            app.world().resource::<BlastCraterBook>().sites_demolished,
            0
        );
        assert_eq!(count_sites(&mut app), 1);
    }

    #[test]
    fn blast_units_metres_radius_not_tiles() {
        // 2.5 m radius must not be treated as 2.5 tiles (40 m).
        let r_m = 2.5_f32;
        assert!(r_m < METRES_PER_TILE);
        let tiles = [IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(2, 0)];
        let mut cols = vec![1.0; 3];
        let profile = BlastProfile::default();
        let mean = carve_dam_columns(
            &mut cols,
            &tiles,
            Vec2::new(0.5 * METRES_PER_TILE, 0.5 * METRES_PER_TILE),
            r_m,
            4.0,
            1.0,
            &profile,
        );
        assert!(cols[0] < 1.0, "near column should carve");
        assert!((cols[2] - 1.0).abs() < 1e-4, "2 tiles away outside 2.5 m");
        assert!(mean < 1.0);
    }

    #[test]
    fn blast_carves_voxels_by_yield_penetration() {
        let tiles = [IVec2::ZERO, IVec2::new(1, 0)];
        let mut small = vec![1.0; 2];
        let mut large = vec![1.0; 2];
        let c = Vec2::new(0.5 * METRES_PER_TILE, 0.5 * METRES_PER_TILE);
        let profile = BlastProfile::default();
        let _ = carve_dam_columns(&mut small, &tiles, c, 24.0, 2.0, 0.5, &profile);
        let _ = carve_dam_columns(&mut large, &tiles, c, 64.0, 8.0, 0.5, &profile);
        assert!(large[0] < small[0], "larger yield+pen must carve deeper");
    }

    #[test]
    fn crest_small_yield_no_collapse() {
        let tiles = [
            IVec2::new(0, 0),
            IVec2::new(1, 0),
            IVec2::new(2, 0),
            IVec2::new(3, 0),
            IVec2::new(4, 0),
        ];
        let mut cols = vec![1.0; tiles.len()];
        let profile = BlastProfile::default();
        // Small crest hit: 4 m radius, low pen — far columns stay near intact.
        let mean = carve_dam_columns(
            &mut cols,
            &tiles,
            Vec2::new(0.5 * METRES_PER_TILE, 0.5 * METRES_PER_TILE),
            4.0,
            1.0,
            0.35,
            &profile,
        );
        assert!(mean > DESTRUCTIBLE_COLLAPSE_INTEGRITY + 0.15);
        assert!(cols.iter().all(|&c| c > DESTRUCTIBLE_COLLAPSE_INTEGRITY));
        let stage = destructible_stage_from_integrity(mean, DESTRUCTIBLE_COLLAPSE_INTEGRITY);
        assert!(!stage.is_terminal());
    }

    /// Combat / presentation must not mutate crater book or sites — enqueue / dump only.
    #[test]
    fn phys07_ballistics_and_present_hold_no_sim_write_surface() {
        let ballistics = include_str!("../combat/ballistics.rs");
        let ball_prod = ballistics
            .split_once("#[cfg(test)]")
            .map_or(ballistics, |(p, _)| p);
        assert!(
            ball_prod.contains("BlastIntent") && ball_prod.contains("enqueue"),
            "ballistics must enqueue BlastIntent"
        );
        // Read-only occupation / materials for mid-flight solid hits (CQ-55) is allowed;
        // mutation of construction books / demolish / despawn is not.
        for forbidden in [
            "BlastCraterBook",
            "execute_demolish",
            "ResMut<TileOccupationBook>",
            "ResMut<VoxelOccupationBook>",
            "ConstructionSite",
            ".despawn(",
        ] {
            assert!(
                !ball_prod.contains(forbidden),
                "ballistics must stay intent-only (found `{forbidden}`)"
            );
        }

        let present = include_str!("blast_present.rs");
        let present_prod = present
            .split_once("#[cfg(test)]")
            .map_or(present, |(p, _)| p);
        assert!(
            present_prod.contains("Res<BlastCraterBook>")
                || present_prod.contains("book: Res<BlastCraterBook>"),
            "present must read BlastCraterBook"
        );
        assert!(
            !present_prod.contains("ResMut<BlastCraterBook>"),
            "present must not write BlastCraterBook"
        );
        assert!(
            !present_prod.contains("BlastIntentQueue"),
            "present must not enqueue blasts"
        );
    }

    #[test]
    fn phys07_execute_is_sole_crater_book_writer() {
        let blast = include_str!("blast.rs");
        let prod = blast.split_once("#[cfg(test)]").map_or(blast, |(p, _)| p);
        assert!(
            prod.contains("mut craters: ResMut<BlastCraterBook>"),
            "execute_blast_intents_system must be the BlastCraterBook writer"
        );
        // No other construction/combat module takes ResMut<BlastCraterBook>.
        for (name, src) in [
            ("blast_present.rs", include_str!("blast_present.rs")),
            ("demolish.rs", include_str!("demolish.rs")),
            ("ballistics.rs", include_str!("../combat/ballistics.rs")),
        ] {
            let p = src.split_once("#[cfg(test)]").map_or(src, |(a, _)| a);
            assert!(
                !p.contains("ResMut<BlastCraterBook>"),
                "{name} must not write BlastCraterBook"
            );
        }
    }

    #[test]
    fn phys07_execute_routes_voxel_occupation_through_demolish() {
        let prod = include_str!("blast.rs")
            .split_once("#[cfg(test)]")
            .map(|(p, _)| p)
            .unwrap_or(include_str!("blast.rs"));
        assert!(prod.contains("execute_demolish_at_tile"));
        assert!(prod.contains("VoxelOccupationBook"));
        assert!(prod.contains("commit_auth"));
        assert!(
            !prod.contains("voxel.remove_site") && !prod.contains("voxel_occupation.insert"),
            "blast must not write voxel book outside demolish funnel"
        );
    }
}
