//! Bevy integration: [`ChunkStrategicOverlay`] on every chunk that has a [`ChunkCellMatrix`](crate::terrain::generation::ChunkCellMatrix).

use bevy::prelude::*;

use super::build_order::process_build_order_queue_system;
use super::frontline::derive_frontline_from_control_system;
use super::logistics_net::logistics_net_inject_into_overlays;
use super::network_flow::{
    network_digest_marks_flow_dirty_system, network_flow_chunk_local_solver_system,
    network_insulation_visibility_post_system, NetworkDirtyMask, NetworkFlowPrevSignatures,
    NETWORK_DIRTY_FLOW,
};
use super::schedule::{StrategicOverlayCouplingScratch, StrategicOverlayDisplayPolicy};
use super::transport_bridge::{
    apply_corridor_construction_book_to_entities, inject_transport_scalar_fields_into_overlays,
    maintain_strategic_corridor_entities, sync_logistics_graph_from_transport, StrategicRasterConfig,
};
use super::world_read_snapshot::world_read_snapshot_refresh_system;
use super::zones::apply_zones_to_strategic_overlays_system;
use super::{
    apply_site_zone_emitters_to_overlays_system, startup_spawn_operational_causality_anchors,
    sync_site_operational_dependency_links_apply_system, sync_zone_emitter_from_archetype_system,
    ApprovedBuildOrders, BuildOrderQueue, ChunkNetworkDigest, ChunkStrategicOverlay,
    CorridorConstructionBook, FrontlineState, InfrastructureGraph, LogisticsGraph,
    SiteConstructionBook, SiteIdIssuer, SpatialNetworkGraph, WorldFieldLayerConfig,
    WorldFieldLayerEpoch, WorldReadSnapshot,
    commit_construction_site_system, site_advance_planned_to_under_construction_system,
    site_construction_progression_system, site_provisioning_system,
    validate_committed_site_terrain_system,
};
use super::construction_book::{
    advance_corridor_construction_book_on_sim_tick, align_corridor_book_with_transport_directory,
    transport_directory_edge_signature, CorridorConstructionTickConfig,
};
use crate::systems::sim_control::{SimControlState, SimControlSystemSet};
use crate::systems::terrain::materialize_chunks;
use crate::systems::terrain::material_plugin::rebuild_dirty_chunks;
use crate::systems::transport::{TransportCostWeights, TransportEdgeDirectory, TransportFieldStore};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::{ChunkDependency, ChunkDirty, MaterializedChunk};

use super::spatial_network::rebuild_chunk_network_digest_system;

/// Ordering buckets for strategic field pipeline (`chunk_scheduler_runbook_v1` / transport coupling).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StrategicFieldPipeline {
    EnsureOverlays,
    /// Rebuild [`LogisticsGraph`] from transport + corridor entities.
    GraphSync,
    InjectTransportScalars,
    LogisticsNetInject,
    /// Network graph → chunk-local flow diffusion → overlay SOA only (no terrain entities).
    NetworkFlow,
    /// Zones + frontline + planner read model (after logistics + network flow baselines).
    ZoneAndReadModel,
}

/// Site authority pipeline — runs after [`StrategicFieldPipeline::ZoneAndReadModel`] (P2).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InfrastructureSiteSet {
    Planning,
    Validation,
    NetworkSolve,
    Logistics,
    Construction,
    Provisioning,
    OperationalZones,
    PreviewInvalidation,
}

fn ensure_network_dirty_mask_on_overlays(
    mut commands: Commands,
    q: Query<Entity, (With<ChunkStrategicOverlay>, Without<NetworkDirtyMask>)>,
) {
    for e in q.iter() {
        commands.entity(e).insert(NetworkDirtyMask {
            mask: NETWORK_DIRTY_FLOW,
        });
    }
}

/// When U7 terrain passes finish (`ChunkDirty` clears), schedule a **network flow** refresh only — no extra terrain passes.
fn terrain_rebuild_finished_marks_network_flow_dirty(
    mut q: Query<
        (Entity, &ChunkDirty, &mut NetworkDirtyMask),
        (
            With<ChunkStrategicOverlay>,
            With<ChunkDependency>,
            With<MaterializedChunk>,
        ),
    >,
    mut prev: Local<std::collections::HashMap<Entity, u8>>,
) {
    let mut seen = std::collections::HashSet::new();
    for (e, dirty, mut mask) in q.iter_mut() {
        seen.insert(e);
        let prior = *prev.get(&e).unwrap_or(&0);
        if prior != 0 && dirty.passes == 0 {
            mask.mask |= NETWORK_DIRTY_FLOW;
        }
        prev.insert(e, dirty.passes);
    }
    prev.retain(|e, _| seen.contains(e));
}

fn ensure_chunk_strategic_overlays(
    mut commands: Commands,
    q: Query<(Entity, &Chunk, &ChunkCellMatrix), Without<ChunkStrategicOverlay>>,
    mut config: ResMut<StrategicRasterConfig>,
    mut warned_mismatch: Local<bool>,
) {
    let mut first_size: Option<UVec2> = None;
    for (entity, chunk, matrix) in q.iter() {
        if first_size.is_none() {
            first_size = Some(matrix.size);
        } else if !*warned_mismatch && first_size != Some(matrix.size) {
            warn!(
                "ChunkCellMatrix sizes differ across chunks; StrategicRasterConfig uses first seen {:?}",
                first_size
            );
            *warned_mismatch = true;
        }
        let n = ChunkStrategicOverlay::new(chunk.coord, matrix.size).len_cells();
        let expected = (matrix.size.x as usize).saturating_mul(matrix.size.y as usize);
        if n != expected {
            warn!(
                "strategic overlay cell count mismatch chunk {:?}: got {} expected {}",
                chunk.coord, n, expected
            );
        }
        commands.entity(entity).insert((
            ChunkStrategicOverlay::new(chunk.coord, matrix.size),
            NetworkDirtyMask {
                mask: NETWORK_DIRTY_FLOW,
            },
        ));
    }
    if let Some(sz) = first_size {
        if config.cells_per_chunk != sz {
            config.cells_per_chunk = sz;
        }
    }
}

fn sync_construction_book_after_transport_changes(
    directory: Res<TransportEdgeDirectory>,
    mut book: ResMut<CorridorConstructionBook>,
    mut last_sig: Local<u64>,
) {
    let sig = transport_directory_edge_signature(&directory);
    if sig == *last_sig {
        return;
    }
    *last_sig = sig;
    align_corridor_book_with_transport_directory(&directory, &mut book);
}

/// Spawns and keeps **zeroed** operational field buffers aligned with terrain chunks.
///
/// Runs after [`materialize_chunks`](crate::systems::terrain::materialize_chunks) so first-time materialized
/// chunks receive an overlay in the same frame. Simulation systems (diffusion, unit influence, graphs → fields)
/// layered in later phases per `phased_engine_delivery_v1.md`.
pub struct StrategicFieldsPlugin;

impl Plugin for StrategicFieldsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimControlState>()
            .init_resource::<LogisticsGraph>()
            .init_resource::<StrategicRasterConfig>()
            .init_resource::<StrategicOverlayCouplingScratch>()
            .init_resource::<StrategicOverlayDisplayPolicy>()
            .init_resource::<CorridorConstructionBook>()
            .init_resource::<CorridorConstructionTickConfig>()
            .init_resource::<TransportEdgeDirectory>()
            .init_resource::<TransportFieldStore>()
            .init_resource::<TransportCostWeights>()
            .init_resource::<FrontlineState>()
            .init_resource::<WorldReadSnapshot>()
            .init_resource::<WorldFieldLayerEpoch>()
            .init_resource::<WorldFieldLayerConfig>()
            .init_resource::<BuildOrderQueue>()
            .init_resource::<ApprovedBuildOrders>()
            .init_resource::<NetworkFlowPrevSignatures>()
            .init_resource::<ChunkNetworkDigest>()
            .init_resource::<SpatialNetworkGraph>()
            .init_resource::<InfrastructureGraph>()
            .init_resource::<SiteConstructionBook>()
            .init_resource::<SiteIdIssuer>()
            .init_resource::<super::site::TileOccupationBook>()
            .init_resource::<super::settlement::BlockBook>()
            .init_resource::<super::settlement::DistrictBook>()
            .init_resource::<super::settlement::TownBook>()
            .add_plugins(super::settlement::SettlementPlugin)
            .add_systems(Startup, startup_spawn_operational_causality_anchors)
            .add_message::<super::CommitConstructionSiteEvent>()
            .configure_sets(
                Update,
                (
                    StrategicFieldPipeline::EnsureOverlays.after(materialize_chunks),
                    StrategicFieldPipeline::GraphSync
                        .after(StrategicFieldPipeline::EnsureOverlays)
                        .after(SimControlSystemSet::AdvanceSimTick),
                    StrategicFieldPipeline::InjectTransportScalars.after(StrategicFieldPipeline::GraphSync),
                    StrategicFieldPipeline::LogisticsNetInject
                        .after(StrategicFieldPipeline::InjectTransportScalars),
                    StrategicFieldPipeline::NetworkFlow.after(StrategicFieldPipeline::LogisticsNetInject),
                    StrategicFieldPipeline::ZoneAndReadModel.after(StrategicFieldPipeline::NetworkFlow),
                    InfrastructureSiteSet::Planning.after(StrategicFieldPipeline::ZoneAndReadModel),
                    InfrastructureSiteSet::Validation.after(InfrastructureSiteSet::Planning),
                    InfrastructureSiteSet::NetworkSolve.after(InfrastructureSiteSet::Validation),
                    InfrastructureSiteSet::Logistics.after(InfrastructureSiteSet::NetworkSolve),
                    InfrastructureSiteSet::Construction.after(InfrastructureSiteSet::Logistics),
                    InfrastructureSiteSet::Provisioning.after(InfrastructureSiteSet::Construction),
                    InfrastructureSiteSet::OperationalZones.after(InfrastructureSiteSet::Provisioning),
                    InfrastructureSiteSet::PreviewInvalidation.after(InfrastructureSiteSet::OperationalZones),
                ),
            )
            .add_systems(
                Update,
                advance_corridor_construction_book_on_sim_tick
                    .in_set(SimControlSystemSet::AdvanceSimTick)
                    .after(crate::systems::sim_control::advance_sim_tick),
            )
            .add_systems(
                Update,
                (
                    ensure_network_dirty_mask_on_overlays,
                    ensure_chunk_strategic_overlays,
                )
                    .chain()
                    .in_set(StrategicFieldPipeline::EnsureOverlays),
            )
            .add_systems(
                Update,
                (
                    sync_construction_book_after_transport_changes,
                    maintain_strategic_corridor_entities,
                    apply_corridor_construction_book_to_entities,
                    sync_logistics_graph_from_transport,
                )
                    .chain()
                    .in_set(StrategicFieldPipeline::GraphSync),
            )
            .add_systems(
                Update,
                inject_transport_scalar_fields_into_overlays
                    .in_set(StrategicFieldPipeline::InjectTransportScalars),
            )
            .add_systems(
                Update,
                logistics_net_inject_into_overlays.in_set(StrategicFieldPipeline::LogisticsNetInject),
            )
            // **SCH-W1-P2-001:** PostUpdate dirty mark → Update `NetworkFlow` next frame (1-frame lag by design).
            .add_systems(
                PostUpdate,
                terrain_rebuild_finished_marks_network_flow_dirty.after(rebuild_dirty_chunks),
            )
            .add_systems(
                Update,
                (
                    network_digest_marks_flow_dirty_system.after(rebuild_chunk_network_digest_system),
                    network_flow_chunk_local_solver_system,
                    network_insulation_visibility_post_system,
                )
                    .chain()
                    .in_set(StrategicFieldPipeline::NetworkFlow),
            )
            .add_systems(
                Update,
                (
                    apply_zones_to_strategic_overlays_system,
                    derive_frontline_from_control_system,
                    world_read_snapshot_refresh_system,
                    process_build_order_queue_system,
                )
                    .chain()
                    .in_set(StrategicFieldPipeline::ZoneAndReadModel),
            )
            .add_systems(
                Update,
                commit_construction_site_system.in_set(InfrastructureSiteSet::Planning),
            )
            .add_systems(
                Update,
                (
                    validate_committed_site_terrain_system,
                    site_advance_planned_to_under_construction_system,
                )
                    .chain()
                    .in_set(InfrastructureSiteSet::Validation),
            )
            .add_systems(
                Update,
                site_construction_progression_system.in_set(InfrastructureSiteSet::Construction),
            )
            .add_systems(
                Update,
                site_provisioning_system.in_set(InfrastructureSiteSet::Provisioning),
            )
            .add_systems(
                Update,
                sync_site_operational_dependency_links_apply_system
                    .after(site_provisioning_system)
                    .in_set(InfrastructureSiteSet::Provisioning),
            )
            .add_systems(
                Update,
                (
                    sync_zone_emitter_from_archetype_system,
                    apply_site_zone_emitters_to_overlays_system,
                )
                    .chain()
                    .in_set(InfrastructureSiteSet::OperationalZones),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::terrain::MaterialUnificationPlugin;
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use bevy::asset::AssetPlugin;
    use bevy::prelude::{IVec2, MinimalPlugins, UVec2};

    #[test]
    fn strategic_overlay_spawns_with_chunk_matrix() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .add_plugins(MaterialUnificationPlugin)
            .add_plugins(StrategicFieldsPlugin);

        let e = app
            .world_mut()
            .spawn((
                Chunk {
                    coord: IVec2::new(0, 1),
                },
                ChunkCellMatrix::new(UVec2::new(3, 2)),
            ))
            .id();

        app.update();

        let overlay = app.world().entity(e).get::<ChunkStrategicOverlay>().expect("overlay");
        assert_eq!(overlay.chunk_coord, IVec2::new(0, 1));
        assert_eq!(overlay.size, UVec2::new(3, 2));
        assert_eq!(overlay.len_cells(), 6);
        assert_eq!(overlay.faction_control.len(), 6);
        assert_eq!(overlay.threat.len(), 6);
        assert_eq!(overlay.routing_congestion.len(), 6);
        assert_eq!(overlay.power_flow.len(), 6);
        assert_eq!(overlay.logistics_flow.len(), 6);
        assert_eq!(overlay.control_pressure.len(), 6);
        assert_eq!(overlay.visibility.len(), 6);
    }

    /// **R4** — faction-slot field writers (`strategic_overlay` runbook).
    #[test]
    fn strategic_overlay_round4_faction_field_writes() {
        let mut o = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(2, 2));
        assert!(o.set_faction_threat(0, 0, 0.7).is_ok());
        assert!(o.set_recon_confidence(1, 0, 0.5).is_ok());
        assert!(o.set_artillery_danger(2, 1, 0.9).is_ok());
        assert!((o.threat[0][0] - 0.7).abs() < 1e-6);
        assert!(o.set_faction_threat(99, 0, 1.0).is_err());
        assert!(o.set_faction_threat(0, 20, 1.0).is_err());
    }
}
