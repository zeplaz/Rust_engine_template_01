//! DISASTER-PROXY — cheap hydro/fire → integrity / demolish (construction funnel).

use bevy::prelude::*;

use super::demolish::{
    destructible_stage_from_integrity, execute_demolish_at_tile, DestructibleDamage,
    DestructibleDamageStage, DESTRUCTIBLE_COLLAPSE_INTEGRITY,
};
use super::structural_role::{SiteStructuralRoles, StructuralRole};
use crate::economy::activation::BuildingDefinitionRef;
use crate::render::pixel_world::{DestructionMaskBook, MASK_DESTROYED};
use crate::strategic::{
    BuildSiteTile, ConstructionSite, PlannedSite, SiteFootprint, TileOccupationBook,
    VoxelOccupationBook, VoxelOccupationConfig,
};
use crate::systems::water::WaterTileProbe;
#[cfg(test)]
use crate::systems::water::SurfaceWaterField;

/// Depth (m) over a Foundation tile that starts flood integrity loss.
pub const FLOOD_FOUNDATION_DEPTH_M: f32 = 1.0;

/// Queue of site origins to demolish after disaster proxies trip Collapsed.
#[derive(Resource, Debug, Default)]
pub struct StructuralCollapseQueue {
    pub tiles: Vec<BuildSiteTile>,
    pub collapsed: u64,
}

/// Flood over Foundation tiles → escalate [`DestructibleDamage`] + mask; terminal → queue.
pub fn flood_foundation_integrity_proxy_system(
    water: WaterTileProbe,
    mut mask: Option<ResMut<DestructionMaskBook>>,
    mut queue: ResMut<StructuralCollapseQueue>,
    clock: Res<crate::systems::sim_control::SimStepClock>,
    mut sites: Query<
        (
            &SiteFootprint,
            &PlannedSite,
            &SiteStructuralRoles,
            &mut DestructibleDamage,
        ),
        // DF-8 / §8: a dam's damage is its `DamBody` (voxel integrity, leak, erosion) — never
        // whole-site demolish from a wet foundation.
        Without<super::dam_barrier::DamBody>,
    >,
) {
    if !water.seeded() {
        return;
    }
    let dt_micros = clock.step_micros.saturating_mul(u64::from(clock.due));
    for (fp, planned, roles, mut dmg) in &mut sites {
        // Same rule as placement (`preview_commit_guard`): a building allowed to stand in water
        // (port, intake, rig, weir…) has a wet foundation by design, so wetness is not damage.
        if super::preview_commit_guard::building_may_occupy_water(planned.catalog_id.as_deref()) {
            continue;
        }
        let n = fp.tiles.len().min(roles.roles.len());
        if n == 0 {
            continue;
        }
        let mut max_depth = 0.0f32;
        let mut wet_tiles: Vec<IVec2> = Vec::new();
        for i in 0..n {
            if roles.roles[i] != StructuralRole::Foundation {
                continue;
            }
            let t = fp.tiles[i];
            let d = water.depth_m(t);
            if d >= FLOOD_FOUNDATION_DEPTH_M {
                max_depth = max_depth.max(d);
                wet_tiles.push(t);
            }
        }
        if wet_tiles.is_empty() {
            continue;
        }
        // Integrity falls as depth exceeds gate (2m ≈ total loss); the stage then walks the
        // ladder one dwell at a time (D-8) instead of vanishing in one frame.
        let integrity = (1.0 - ((max_depth - FLOOD_FOUNDATION_DEPTH_M) / 2.0).clamp(0.0, 1.0))
            .clamp(0.0, 1.0);
        let target = destructible_stage_from_integrity(integrity, DESTRUCTIBLE_COLLAPSE_INTEGRITY);
        dmg.advance_toward(target, dt_micros);
        // A failing foundation (Compromised or worse) drops its ground column; the cascade
        // then takes the load-bearing walls.
        if dmg.stage.rank() >= DestructibleDamageStage::Compromised.rank() {
            if let Some(mask) = mask.as_mut() {
                for t in &wet_tiles {
                    mask.mark_tile(t.x, t.y, MASK_DESTROYED);
                }
            }
        }
        if dmg.stage.is_terminal() {
            queue.tiles.push(planned.origin);
        }
    }
}

/// Drain [`StructuralCollapseQueue`] through [`execute_demolish_at_tile`].
pub fn apply_structural_collapse_demolish(
    mut commands: Commands,
    mut queue: ResMut<StructuralCollapseQueue>,
    mut occupation: Option<ResMut<TileOccupationBook>>,
    mut voxel_occupation: Option<ResMut<VoxelOccupationBook>>,
    voxel_cfg: Option<Res<VoxelOccupationConfig>>,
    mut destruction_mask: Option<ResMut<DestructionMaskBook>>,
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
    let mut seen = std::collections::HashSet::new();
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disaster_proxy_fire_path_uses_demolish_funnel() {
        let fire = include_str!("../systems/fire/entity_exposure.rs");
        let prod = fire.split_once("#[cfg(test)]").map_or(fire, |(p, _)| p);
        assert!(prod.contains("execute_demolish_at_tile"));
        assert!(prod.contains("FireCollapseQueue"));
    }

    /// Each `app.update()` is one whole sim second (`due = 1`).
    fn one_second_clock() -> crate::systems::sim_control::SimStepClock {
        let mut c = crate::systems::sim_control::SimStepClock::default();
        c.step_micros = 1_000_000;
        c.due = 1;
        c
    }

    fn flooded_app(catalog: Option<&str>, dam: bool) -> (App, Entity) {
        use crate::strategic::{FootprintTiles, LayerType, SiteArchetype, SiteId};
        let mut app = App::new();
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, 4, 4, vec![0.0; 16], 0.0);
        field.depth_m.fill(3.0); // well over FLOOD_FOUNDATION_DEPTH_M + 2 m gate
        app.insert_resource(field);
        app.init_resource::<StructuralCollapseQueue>();
        app.insert_resource(one_second_clock());
        let tile = IVec2::new(1, 1);
        let origin = BuildSiteTile { x: 1, z: 1 };
        let mut e = app.world_mut().spawn((
            PlannedSite {
                site_id: SiteId(7),
                origin,
                footprint: FootprintTiles { width: 1, depth: 1 },
                archetype: SiteArchetype::Factory,
                layer: LayerType::Surface,
                catalog_id: catalog.map(str::to_string),
                placement: None,
            },
            SiteFootprint { tiles: vec![tile], layer: LayerType::Surface },
            SiteStructuralRoles { roles: vec![StructuralRole::Foundation] },
            DestructibleDamage::default(),
        ));
        if dam {
            e.insert(super::super::dam_barrier::DamBody::default());
        }
        let id = e.id();
        app.add_systems(Update, flood_foundation_integrity_proxy_system);
        (app, id)
    }

    /// CLOSE-D1: ordinary buildings still fail on a deep wet foundation.
    #[test]
    fn close_d1_flood_collapses_ordinary_site() {
        let (mut app, _) = flooded_app(Some("factory_light_v1"), false);
        app.update();
        // D-8: first frame only scars; collapse is staged, not a one-frame vanish.
        assert!(app.world().resource::<StructuralCollapseQueue>().tiles.is_empty());
        for _ in 0..6 {
            app.update();
        }
        assert!(!app.world().resource::<StructuralCollapseQueue>().tiles.is_empty());
    }

    /// D-2 + D-8 end to end: an ordinary building gets default roles, deep water walks it
    /// Scarred -> Compromised -> Collapsed, the cascade marks its walls, and the collapse goes
    /// through `StructuralCollapseQueue` -> `execute_demolish_at_tile` (entity despawned).
    #[test]
    fn d2_d8_ordinary_building_flood_stages_cascade_and_funnel_collapse() {
        use crate::construction::structural_cascade::cascade_mask_from_roles_system;
        use crate::construction::structural_role::ensure_default_structural_roles_system;
        use crate::strategic::{
            ConstructionSite, FootprintTiles, LayerType, SiteArchetype, SiteConstructionPhase,
            SiteId,
        };
        let mut app = App::new();
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, 8, 8, vec![0.0; 64], 0.0);
        field.depth_m.fill(3.0);
        app.insert_resource(field);
        app.init_resource::<StructuralCollapseQueue>();
        app.insert_resource(one_second_clock());
        let mut mask = DestructionMaskBook::default();
        mask.ensure_size(16, 16);
        app.insert_resource(mask);
        // 3x3 house: corners Foundation, edges Column, centre Facade.
        let tiles: Vec<IVec2> = (1..4)
            .flat_map(|y| (1..4).map(move |x| IVec2::new(x, y)))
            .collect();
        let site = app
            .world_mut()
            .spawn((
                ConstructionSite {
                    site_id: 9,
                    owner: Entity::PLACEHOLDER,
                    archetype: SiteArchetype::Factory,
                    phase: SiteConstructionPhase::default(),
                    operational_readiness: 1.0,
                },
                PlannedSite {
                    site_id: SiteId(9),
                    origin: BuildSiteTile { x: 1, z: 1 },
                    footprint: FootprintTiles { width: 3, depth: 3 },
                    archetype: SiteArchetype::Factory,
                    layer: LayerType::Surface,
                    catalog_id: Some("factory_light_v1".to_string()),
                    placement: None,
                },
                SiteFootprint { tiles, layer: LayerType::Surface },
            ))
            .id();
        app.add_systems(
            Update,
            (
                ensure_default_structural_roles_system,
                flood_foundation_integrity_proxy_system,
                cascade_mask_from_roles_system,
                apply_structural_collapse_demolish,
            )
                .chain(),
        );
        app.update();
        let roles = app.world().get::<SiteStructuralRoles>(site).expect("roles");
        assert!(roles.roles.contains(&StructuralRole::Foundation));
        assert!(roles.roles.contains(&StructuralRole::Column));
        // Frame 1 sees no damage component yet (inserted via commands) -> no stage change.
        let mut seen = Vec::new();
        for _ in 0..8 {
            app.update();
            if let Some(d) = app.world().get::<DestructibleDamage>(site) {
                seen.push(d.stage);
                if d.stage == DestructibleDamageStage::Compromised {
                    // Cascade marked the load-bearing edge (2,1) while the house still stands.
                    assert_eq!(
                        app.world().resource::<DestructionMaskBook>().cell(2, 1),
                        MASK_DESTROYED
                    );
                }
            }
            if app.world().get_entity(site).is_err() {
                break;
            }
        }
        assert!(seen.contains(&DestructibleDamageStage::Scarred), "{seen:?}");
        assert!(seen.contains(&DestructibleDamageStage::Compromised), "{seen:?}");
        assert!(app.world().get_entity(site).is_err(), "collapse must despawn via the funnel");
        assert!(app.world().resource::<StructuralCollapseQueue>().collapsed >= 1);
    }

    /// CLOSE-D1: water-legal buildings (placement rule) and dams (DamBody owns damage) are not
    /// demolished for standing in water.
    #[test]
    fn close_d1_flood_skips_water_structures_and_dams() {
        for (catalog, dam) in [(Some("harbor_v1"), false), (Some("dam_embankment_v1"), true)] {
            let (mut app, _) = flooded_app(catalog, dam);
            app.update();
            assert!(
                app.world().resource::<StructuralCollapseQueue>().tiles.is_empty(),
                "{catalog:?} must not collapse from a wet foundation"
            );
        }
    }

    #[test]
    fn disaster_proxy_flood_gate_constant_positive() {
        assert!(FLOOD_FOUNDATION_DEPTH_M > 0.0);
    }
}
