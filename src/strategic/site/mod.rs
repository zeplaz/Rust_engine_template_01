//! Territorial **site** authority (P2-A…E): planned nodes, validation scores, logistics-fed build, provisioning gates.
//!
//! Schedule bucket: [`InfrastructureSiteSet`](crate::strategic::InfrastructureSiteSet) — wired after [`StrategicFieldPipeline::ZoneAndReadModel`](super::plugin::StrategicFieldPipeline) so transport overlays exist before site solve.

mod components;
mod events;
mod logistics;
mod overlays;
mod provisioning;
mod resources;
mod systems;
mod validation;

pub use components::*;
pub use events::*;
pub use logistics::site_construction_progression_system;
pub use provisioning::site_provisioning_system;
pub use resources::*;
pub use overlays::{
    apply_site_zone_emitters_to_overlays_system, sync_zone_emitter_from_archetype_system,
    zone_emitter_for_archetype,
};
pub use systems::{
    commit_construction_site_system, footprint_affected_chunk_coords,
    site_advance_planned_to_under_construction_system, validate_committed_site_terrain_system,
};
pub use validation::{
    evaluate_site_placement_at_world_tile,
    evaluate_site_placement_stubs, validate_network_access_for_site, validate_site_placement_stubs,
    validate_terrain_for_site, SitePlacementValidation,
};

/// Schedule buckets for site systems live on [`crate::strategic::InfrastructureSiteSet`](super::plugin::InfrastructureSiteSet).

#[cfg(test)]
mod tests {
    use crate::strategic::build_order::BuildSiteTile;
    use crate::strategic::construction_book::CorridorConstructionPhase;
    use crate::strategic::spatial_network::LayerType;
    use bevy::prelude::{App, IVec2, MinimalPlugins, UVec2, Update};

    use super::{
        commit_construction_site_system, footprint_affected_chunk_coords,
        site_phase_from_corridor_coarse, CommitConstructionSiteEvent, ConstructionSite, FootprintTiles,
        PlannedSite, SiteArchetype, SiteConstructionBook, SiteConstructionPhase, SiteConstructionStatus,
        SiteId, SiteIdIssuer,
    };

    #[test]
    fn corridor_to_site_coarse_mapping() {
        assert_eq!(
            site_phase_from_corridor_coarse(CorridorConstructionPhase::Planned),
            SiteConstructionPhase::Planned
        );
        assert_eq!(
            site_phase_from_corridor_coarse(CorridorConstructionPhase::InProgress),
            SiteConstructionPhase::UnderConstruction
        );
        assert_eq!(
            site_phase_from_corridor_coarse(CorridorConstructionPhase::Completed),
            SiteConstructionPhase::Operational
        );
    }

    #[test]
    fn operational_factor_defaults() {
        let mut book = SiteConstructionBook::default();
        let id = SiteId(1);
        book.by_site.insert(
            id,
            SiteConstructionStatus {
                phase: SiteConstructionPhase::Provisioning,
                progress: 0.5,
            },
        );
        assert!((book.operational_factor(id) - 0.5).abs() < 1e-6);
        assert_eq!(book.operational_factor(SiteId(999)), 1.0);
    }

    #[test]
    fn site_id_issuer_nonzero() {
        let mut iss = SiteIdIssuer::default();
        let a = iss.next();
        let b = iss.next();
        assert_ne!(a.0, 0);
        assert_ne!(a, b);
    }

    #[test]
    fn footprint_affected_chunk_coords_single_chunk() {
        let origin = BuildSiteTile { x: 5, z: 7 };
        let fp = FootprintTiles {
            width: 2,
            depth: 2,
        };
        let cells = UVec2::new(16, 16);
        let mut v = footprint_affected_chunk_coords(origin, fp, cells);
        v.sort_by_key(|c| (c.x, c.y));
        assert_eq!(v, vec![IVec2::new(0, 0)]);
    }

    #[test]
    fn footprint_affected_chunk_coords_spans_two_chunks() {
        let origin = BuildSiteTile { x: 15, z: 0 };
        let fp = FootprintTiles {
            width: 4,
            depth: 1,
        };
        let cells = UVec2::new(16, 16);
        let mut v = footprint_affected_chunk_coords(origin, fp, cells);
        v.sort_by_key(|c| (c.x, c.y));
        assert_eq!(v, vec![IVec2::new(0, 0), IVec2::new(1, 0)]);
    }

    #[test]
    fn commit_allocates_id_and_spawns_authority() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SiteConstructionBook>()
            .init_resource::<SiteIdIssuer>()
            .add_message::<CommitConstructionSiteEvent>()
            .add_systems(Update, commit_construction_site_system);

        let owner = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(CommitConstructionSiteEvent {
            site_id: SiteId::UNASSIGNED,
            owner,
            archetype: SiteArchetype::Factory,
            origin: BuildSiteTile { x: 0, z: 0 },
            footprint: FootprintTiles {
                width: 2,
                depth: 2,
            },
            layer: LayerType::Surface,
            catalog_id: None,
        });
        app.update();

        let book = app.world().resource::<SiteConstructionBook>();
        assert_eq!(book.by_site.len(), 1);
        {
            let world = app.world_mut();
            let mut q = world.query::<(&PlannedSite, &ConstructionSite)>();
            assert_eq!(q.iter(world).count(), 1);
            let (_p, c) = q.iter(world).next().expect("site bundle");
            assert_eq!(c.phase, SiteConstructionPhase::Planned);
            assert_ne!(c.site_id, 0);
        }
    }

    #[test]
    fn commit_invalidates_world_preview_when_config_and_state_present() {
        use crate::strategic::transport_bridge::StrategicRasterConfig;
        use crate::terrain::material::WorldPreviewState;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<WorldPreviewState>()
            .init_resource::<StrategicRasterConfig>()
            .init_resource::<SiteConstructionBook>()
            .init_resource::<SiteIdIssuer>()
            .add_message::<CommitConstructionSiteEvent>()
            .add_systems(Update, commit_construction_site_system);

        let owner = app.world_mut().spawn_empty().id();
        app.world_mut().write_message(CommitConstructionSiteEvent {
            site_id: SiteId::UNASSIGNED,
            owner,
            archetype: SiteArchetype::Factory,
            origin: BuildSiteTile { x: 0, z: 0 },
            footprint: FootprintTiles {
                width: 2,
                depth: 2,
            },
            layer: LayerType::Surface,
            catalog_id: None,
        });
        app.update();

        let preview = app.world().resource::<WorldPreviewState>();
        assert!(preview.epoch.0 > 0);
        assert!(!preview.dirty_queue.is_empty());
    }
}
