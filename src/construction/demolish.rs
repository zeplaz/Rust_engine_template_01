//! Demolish tool: pick target → pending → confirm despawns overlapping sites.
//!
//! **Dual-cite (V4-PHYS08 / PHYS-08):**
//! - NEW: gea4v2 14.5.3.5 Destructible Objects — undamaged cohesive → progressive
//!   damage stages → terminal collapse (no rigid-body middleware / multi-piece island).
//! - LEGACY: `physics_integration` (gea_v2_e4 §14.5 p423) — game-driven destruction
//!   responses bookkept as stages + execute funnel, not Havok Destruction.
//!
//! Progressive damage (fire integrity, blast gate) may *stage* a site; **only**
//! [`execute_demolish_at_tile`] performs terminal collapse (despawn + occupation clear).
//! Voxel occupation clears solely here when `commit_authority_enabled` (via
//! [`VoxelOccupationBook::remove_site_commit`] + tile `replace_from_voxel`).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::economy::activation::BuildingDefinitionRef;
use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, SimulationMapPointerGate, SimulationToolPointer,
};
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::strategic::{
    BuildSiteTile, CommitConstructionSiteEvent, ConstructionSite, FootprintTiles, LayerType,
    PlannedSite, SiteArchetype, SiteFootprint, SiteId,
};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::pending_construction::{
    PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
};
use super::roads::cursor_world_on_map;

/// Construction-owned destructible ladder (gea4v2 14.5.3.5).
///
/// Writers of progressive damage (fire integrity, blast threshold) map into this
/// vocabulary; terminal [`Self::Collapsed`] must route through
/// [`execute_demolish_at_tile`] — never a parallel despawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DestructibleDamageStage {
    /// Cohesive undamaged object.
    Intact,
    /// Light scarring / threatened — still structural.
    Scarred,
    /// Compromised / deteriorating — non-structural pieces failing.
    Compromised,
    /// Catastrophic collapse — execute demolish funnel only.
    Collapsed,
}

impl DestructibleDamageStage {
    /// Monotone rank for tripwires / sorting (Intact=0 … Collapsed=3).
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Intact => 0,
            Self::Scarred => 1,
            Self::Compromised => 2,
            Self::Collapsed => 3,
        }
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Collapsed)
    }

    /// 0..1 presentation wash (scar / compromise / collapse) for burn/damage overlays.
    #[must_use]
    pub const fn visual_wash01(self) -> f32 {
        match self {
            Self::Intact => 0.0,
            Self::Scarred => 0.22,
            Self::Compromised => 0.55,
            Self::Collapsed => 1.0,
        }
    }

    /// Keep the worse of two stages (monotone escalate only).
    #[must_use]
    pub const fn escalate(self, other: Self) -> Self {
        if other.rank() > self.rank() {
            other
        } else {
            self
        }
    }
}

/// Per-site PHYS-08 damage ladder (component on the construction site entity).
///
/// Sole writers: fire integrity sync + blast scar/collapse paths. Terminal
/// [`DestructibleDamageStage::Collapsed`] must still despawn via [`execute_demolish_at_tile`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DestructibleDamage {
    pub stage: DestructibleDamageStage,
}

impl Default for DestructibleDamage {
    fn default() -> Self {
        Self {
            stage: DestructibleDamageStage::Intact,
        }
    }
}

/// Default integrity floor at which a site is treated as collapsed (matches fire collapse bar band).
pub const DESTRUCTIBLE_COLLAPSE_INTEGRITY: f32 = 0.25;

/// Map remaining integrity ∈ `[0, 1]` onto the destructible stage ladder.
///
/// `collapse_threshold` is the integrity at/below which the object catastrophically
/// collapses (clamped to `(0, 0.5]`). Intermediate bands split the remaining range.
#[must_use]
pub fn destructible_stage_from_integrity(
    integrity: f32,
    collapse_threshold: f32,
) -> DestructibleDamageStage {
    let i = integrity.clamp(0.0, 1.0);
    let bar = collapse_threshold.clamp(0.05, 0.5);
    if i <= bar {
        DestructibleDamageStage::Collapsed
    } else if i < 0.85 {
        DestructibleDamageStage::Compromised
    } else if i < 0.98 {
        DestructibleDamageStage::Scarred
    } else {
        DestructibleDamageStage::Intact
    }
}

/// True when accumulated / peak damage should advance straight to terminal collapse
/// (catastrophic path — blast peak ≥ gate, fire collapse enqueue, player demolish confirm).
#[must_use]
pub fn destructible_catastrophic_collapse(damage: f32, demolish_gate: f32) -> bool {
    damage.is_finite() && demolish_gate.is_finite() && damage >= demolish_gate
}

pub fn demolish_pick_queue_system(
    buttons: Res<ButtonInput<MouseButton>>,
    tool: Res<ActiveBuildTool>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    mut pending: ResMut<PendingConstructionQueue>,
    pointer_gate: Option<Res<SimulationMapPointerGate>>,
) {
    if tool.tool != BuildTool::Demolish {
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    if simulation_tool_pointer(pointer_gate.as_deref()) != SimulationToolPointer::Free {
        return;
    }
    let Ok(window) = win.single() else {
        return;
    };
    let Some(world) = cursor_world_on_map(
        &window,
        pointer_gate.as_deref(),
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    ) else {
        return;
    };
    let origin = BuildSiteTile {
        x: world.x.floor().max(0.0) as u32,
        z: world.z.floor().max(0.0) as u32,
    };
    pending.push(PendingBuildBlueprint {
        kind: PendingEntryKind::Demolish,
        label: format!("demolish:{},{}", origin.x, origin.z),
        archetype: SiteArchetype::MilitaryBase,
        origin,
        footprint: FootprintTiles { width: 1, depth: 1 },
        layer: LayerType::Surface,
        rotation_quarter_turns: 0,
        mirror_x: false,
        approved: false,
        catalog_id: None,
    });
}

/// Despawn construction sites whose footprint covers `tile`. Returns commit events for undo restore.
///
/// X2-P2: when `commit_authority_enabled`, voxel book is sole occupation writer; tile is
/// refreshed as a derived cache. When false, preserves tile-only `remove_site` behavior.
///
/// PHYS-08: this is the **sole** terminal collapse for destructible stages — blast, fire
/// collapse queue, player confirm, and undo-redo all call here; no parallel despawn path.
pub fn execute_demolish_at_tile(
    commands: &mut Commands,
    tile: BuildSiteTile,
    sites: &Query<(
        Entity,
        &ConstructionSite,
        &PlannedSite,
        &SiteFootprint,
        Option<&BuildingDefinitionRef>,
    )>,
    mut occupation: Option<&mut crate::strategic::TileOccupationBook>,
    mut voxel_occupation: Option<&mut crate::strategic::VoxelOccupationBook>,
    commit_authority_enabled: bool,
) -> (u32, Vec<CommitConstructionSiteEvent>) {
    let target = IVec2::new(tile.x as i32, tile.z as i32);
    let mut n = 0u32;
    let mut restored = Vec::new();
    for (entity, site, planned, footprint, catalog) in sites.iter() {
        if !footprint.tiles.iter().any(|t| *t == target) {
            continue;
        }
        restored.push(CommitConstructionSiteEvent {
            site_id: SiteId(site.site_id),
            owner: site.owner,
            archetype: planned.archetype,
            origin: planned.origin,
            footprint: planned.footprint,
            layer: planned.layer,
            catalog_id: catalog.map(|c| c.catalog_id.clone()),
            placement: planned.placement.clone(),
        });
        if planned.placement.is_some() {
            let sid = SiteId(site.site_id);
            if commit_authority_enabled {
                if let Some(voxel) = voxel_occupation.as_mut() {
                    voxel.remove_site_commit(sid);
                    if let Some(book) = occupation.as_mut() {
                        book.replace_from_voxel(voxel);
                    }
                }
            } else if let Some(book) = occupation.as_mut() {
                book.remove_site(sid);
            }
        }
        commands.entity(entity).despawn();
        n += 1;
    }
    (n, restored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::{
        CommittedPlacementSnapshot, SiteConstructionPhase, SiteId, TileOccupationBook,
        VoxelOccupationBook,
    };

    fn spawn_site_with_placement(
        world: &mut World,
        tile: IVec2,
        site_id: u64,
        with_placement: bool,
    ) -> Entity {
        let origin = BuildSiteTile {
            x: tile.x.max(0) as u32,
            z: tile.y.max(0) as u32,
        };
        let placement = with_placement.then(|| CommittedPlacementSnapshot {
            origin,
            scale_factor: 1.0,
            effective_scale: 1.0,
            rotation_quarter_turns: 0,
            mirror_x: false,
            weights: vec![(tile, 1.0)],
        });
        world.spawn((
            ConstructionSite {
                site_id,
                owner: Entity::PLACEHOLDER,
                archetype: SiteArchetype::Factory,
                phase: SiteConstructionPhase::Operational,
                operational_readiness: 1.0,
            },
            PlannedSite {
                site_id: SiteId(site_id),
                origin,
                footprint: FootprintTiles { width: 1, depth: 1 },
                archetype: SiteArchetype::Factory,
                layer: LayerType::Surface,
                catalog_id: None,
                placement,
            },
            SiteFootprint {
                tiles: vec![tile],
                layer: LayerType::Surface,
            },
        ))
        .id()
    }

    #[test]
    fn phys08_dual_cite_in_module_docs() {
        let src = include_str!("demolish.rs");
        let prod = src.split_once("#[cfg(test)]").map_or(src, |(p, _)| p);
        assert!(prod.contains("PHYS-08") || prod.contains("V4-PHYS08"));
        assert!(prod.contains("14.5.3.5"));
        assert!(prod.contains("Destructible"));
        assert!(prod.contains("physics_integration"));
        assert!(prod.contains("gea_v2_e4"));
    }

    #[test]
    fn phys08_damage_stages_monotone_ladder() {
        assert_eq!(
            destructible_stage_from_integrity(1.0, DESTRUCTIBLE_COLLAPSE_INTEGRITY),
            DestructibleDamageStage::Intact
        );
        assert_eq!(
            destructible_stage_from_integrity(0.90, DESTRUCTIBLE_COLLAPSE_INTEGRITY),
            DestructibleDamageStage::Scarred
        );
        assert_eq!(
            destructible_stage_from_integrity(0.50, DESTRUCTIBLE_COLLAPSE_INTEGRITY),
            DestructibleDamageStage::Compromised
        );
        assert_eq!(
            destructible_stage_from_integrity(0.20, DESTRUCTIBLE_COLLAPSE_INTEGRITY),
            DestructibleDamageStage::Collapsed
        );
        let ranks = [
            DestructibleDamageStage::Intact.rank(),
            DestructibleDamageStage::Scarred.rank(),
            DestructibleDamageStage::Compromised.rank(),
            DestructibleDamageStage::Collapsed.rank(),
        ];
        assert!(ranks.windows(2).all(|w| w[0] < w[1]));
        assert!(DestructibleDamageStage::Collapsed.is_terminal());
        assert!(!DestructibleDamageStage::Compromised.is_terminal());
    }

    #[test]
    fn phys08_catastrophic_gate_matches_blast_demolish_threshold() {
        let gate = super::super::blast::BLAST_DEMOLISH_DAMAGE;
        assert!(!destructible_catastrophic_collapse(gate - 1.0, gate));
        assert!(destructible_catastrophic_collapse(gate, gate));
        assert!(destructible_catastrophic_collapse(gate + 50.0, gate));
    }

    #[test]
    fn phys08_execute_demolish_despawns_overlapping_site() {
        use bevy::ecs::system::SystemState;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let tile = IVec2::new(4, 6);
        spawn_site_with_placement(app.world_mut(), tile, 42, false);
        assert_eq!(
            app.world_mut()
                .query::<&ConstructionSite>()
                .iter(app.world())
                .count(),
            1
        );

        {
            let mut state: SystemState<(
                Commands,
                Query<(
                    Entity,
                    &ConstructionSite,
                    &PlannedSite,
                    &SiteFootprint,
                    Option<&BuildingDefinitionRef>,
                )>,
            )> = SystemState::new(app.world_mut());
            let (mut commands, sites) = state
                .get_mut(app.world_mut())
                .expect("demolish SystemState params");
            let (n, events) = execute_demolish_at_tile(
                &mut commands,
                BuildSiteTile {
                    x: tile.x as u32,
                    z: tile.y as u32,
                },
                &sites,
                None,
                None,
                false,
            );
            assert_eq!(n, 1);
            assert_eq!(events.len(), 1);
            state.apply(app.world_mut());
        }
        assert_eq!(
            app.world_mut()
                .query::<&ConstructionSite>()
                .iter(app.world())
                .count(),
            0,
            "terminal collapse must despawn via execute_demolish_at_tile"
        );
    }

    #[test]
    fn phys08_voxel_occupation_clears_only_via_execute_funnel() {
        use bevy::ecs::system::SystemState;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<TileOccupationBook>()
            .init_resource::<VoxelOccupationBook>();
        let tile = IVec2::new(8, 3);
        let site_id = 77u64;
        spawn_site_with_placement(app.world_mut(), tile, site_id, true);
        {
            let mut voxel = app.world_mut().resource_mut::<VoxelOccupationBook>();
            voxel.apply_site_commit(SiteId(site_id), &[(tile, 1.0)]);
            assert!(voxel.weight_at(tile) > 0.0);
        }
        {
            let mut state: SystemState<(
                Commands,
                Query<(
                    Entity,
                    &ConstructionSite,
                    &PlannedSite,
                    &SiteFootprint,
                    Option<&BuildingDefinitionRef>,
                )>,
                ResMut<TileOccupationBook>,
                ResMut<VoxelOccupationBook>,
            )> = SystemState::new(app.world_mut());
            let (mut commands, sites, mut tile_book, mut voxel) = state
                .get_mut(app.world_mut())
                .expect("demolish voxel SystemState params");
            let (n, _) = execute_demolish_at_tile(
                &mut commands,
                BuildSiteTile {
                    x: tile.x as u32,
                    z: tile.y as u32,
                },
                &sites,
                Some(&mut *tile_book),
                Some(&mut *voxel),
                true, // commit authority on → voxel sole writer
            );
            assert_eq!(n, 1);
            assert!(
                voxel.weight_at(tile) < 1e-5,
                "voxel cells must clear inside demolish funnel"
            );
            state.apply(app.world_mut());
        }

        // Source tripwire: this module is the only demolish writer of remove_site_commit.
        let prod = include_str!("demolish.rs")
            .split_once("#[cfg(test)]")
            .map(|(p, _)| p)
            .unwrap_or(include_str!("demolish.rs"));
        assert!(prod.contains("remove_site_commit"));
        assert!(prod.contains("replace_from_voxel"));
        assert!(
            prod.contains("commit_authority_enabled"),
            "voxel clear stays gated on commit authority"
        );
    }

    #[test]
    fn phys08_ingress_routes_collapse_through_demolish_funnel() {
        // Blast / fire may stage damage; terminal collapse must call execute_demolish_at_tile.
        // Read-only tripwires — do not edit those modules in this slice.
        let blast = include_str!("blast.rs");
        let blast_prod = blast.split_once("#[cfg(test)]").map_or(blast, |(p, _)| p);
        assert!(
            blast_prod.contains("execute_demolish_at_tile"),
            "blast terminal path must use demolish funnel"
        );
        assert!(
            !blast_prod.contains("remove_site_commit"),
            "blast must not write voxel book outside demolish"
        );

        let fire = include_str!("../systems/fire/entity_exposure.rs");
        let fire_prod = fire.split_once("#[cfg(test)]").map_or(fire, |(p, _)| p);
        assert!(
            fire_prod.contains("execute_demolish_at_tile"),
            "fire collapse must use demolish funnel"
        );
        assert!(
            fire_prod.contains("FireCollapseQueue") || fire_prod.contains("apply_fire_collapse"),
            "fire must stage collapse before demolish execute"
        );
        assert!(
            !fire_prod.contains("remove_site_commit"),
            "fire must not write voxel book outside demolish"
        );

        let pending = include_str!("build_interaction.rs");
        let pending_prod = pending
            .split_once("#[cfg(test)]")
            .map_or(pending, |(p, _)| p);
        assert!(
            pending_prod.contains("PendingEntryKind::Demolish")
                && pending_prod.contains("execute_demolish_at_tile"),
            "player demolish confirm must use execute funnel"
        );
    }

    #[test]
    fn phys08_collapsed_stage_is_execute_only() {
        let prod = include_str!("demolish.rs")
            .split_once("#[cfg(test)]")
            .map(|(p, _)| p)
            .unwrap_or(include_str!("demolish.rs"));
        assert!(prod.contains("DestructibleDamageStage"));
        assert!(prod.contains("execute_demolish_at_tile"));
        // No second despawn helper in production body.
        let despawn_hits = prod.matches(".despawn(").count();
        assert_eq!(
            despawn_hits, 1,
            "sole .despawn( must live inside execute_demolish_at_tile"
        );
    }
}
