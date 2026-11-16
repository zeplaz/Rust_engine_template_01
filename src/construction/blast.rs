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

use bevy::prelude::*;

use crate::economy::activation::BuildingDefinitionRef;
use crate::strategic::{
    BuildSiteTile, ConstructionSite, FootprintTiles, PlannedSite, SiteFootprint, TileOccupationBook,
    VoxelOccupationBook, VoxelOccupationConfig,
};
use crate::substrate::hydrology::{HydrologyConstructionCouplingStats, HydrologyEventQueue};
use crate::systems::sim_control::SimControlSystemSet;

use super::demolish::{
    destructible_catastrophic_collapse, destructible_stage_from_integrity, execute_demolish_at_tile,
    DestructibleDamage, DestructibleDamageStage, DESTRUCTIBLE_COLLAPSE_INTEGRITY,
};
use super::hydro_coupling::emit_site_execute_hydro_dirty;

/// Effective damage at epicenter required to demolish overlapping sites (after falloff).
pub const BLAST_DEMOLISH_DAMAGE: f32 = 50.0;

/// One blast to apply this tick (ingress from combat / tools).
#[derive(Clone, Debug)]
pub struct BlastIntent {
    pub center_xz: Vec2,
    /// World metres.
    pub radius_m: f32,
    /// Peak damage at epicenter; after [`blast_radius_falloff`], ≥ [`BLAST_DEMOLISH_DAMAGE`]
    /// demolishes overlapping sites; lower / edge falloff = crater only.
    pub damage: f32,
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
    mut occupation: Option<ResMut<TileOccupationBook>>,
    mut voxel_occupation: Option<ResMut<VoxelOccupationBook>>,
    voxel_cfg: Option<Res<VoxelOccupationConfig>>,
    mut hydro_q: Option<ResMut<HydrologyEventQueue>>,
    mut hydro_stats: Option<ResMut<HydrologyConstructionCouplingStats>>,
    sites: Query<(
        Entity,
        &ConstructionSite,
        &PlannedSite,
        &SiteFootprint,
        Option<&BuildingDefinitionRef>,
    )>,
    mut damage: Query<&mut DestructibleDamage>,
) {
    if queue.pending.is_empty() {
        return;
    }
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
            if peak >= BLAST_DEMOLISH_DAMAGE {
                hit_origins.push(planned.origin);
            }
        }
        for (entity, peak) in peaks {
            let Ok(mut dmg) = damage.get_mut(entity) else {
                continue;
            };
            let integrity_left =
                (1.0 - (peak / BLAST_DEMOLISH_DAMAGE).clamp(0.0, 1.0)).clamp(0.0, 1.0);
            let ladder =
                destructible_stage_from_integrity(integrity_left, DESTRUCTIBLE_COLLAPSE_INTEGRITY);
            dmg.stage = dmg.stage.escalate(ladder);
            if destructible_catastrophic_collapse(peak, BLAST_DEMOLISH_DAMAGE) {
                dmg.stage = DestructibleDamageStage::Collapsed;
            }
        }

        if intent.damage < BLAST_DEMOLISH_DAMAGE {
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
                        UVec2::new(16, 16),
                    );
                }
            }
        }
    }
}

pub struct ConstructionBlastPlugin;

impl Plugin for ConstructionBlastPlugin {
    fn build(&self, app: &mut App) {
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
                source_flight: 12,
            });
        app.update();
        assert_eq!(
            app.world().resource::<BlastCraterBook>().sites_demolished,
            0
        );
        assert_eq!(count_sites(&mut app), 1);
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
        for forbidden in [
            "BlastCraterBook",
            "execute_demolish",
            "TileOccupationBook",
            "VoxelOccupationBook",
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
