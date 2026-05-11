//! Bevy integration: [`ChunkStrategicOverlay`] on every chunk that has a [`ChunkCellMatrix`](crate::terrain::generation::ChunkCellMatrix).

use bevy::prelude::*;

use super::build_order::process_build_order_queue_system;
use super::frontline::derive_frontline_from_control_system;
use super::logistics_net::logistics_net_inject_into_overlays;
use super::schedule::{StrategicOverlayCouplingScratch, StrategicOverlayDisplayPolicy};
use super::transport_bridge::{
    apply_corridor_construction_book_to_entities, inject_transport_scalar_fields_into_overlays,
    maintain_strategic_corridor_entities, sync_logistics_graph_from_transport, StrategicRasterConfig,
};
use super::world_read_snapshot::world_read_snapshot_refresh_system;
use super::zones::apply_zones_to_strategic_overlays_system;
use super::{
    ApprovedBuildOrders, BuildOrderQueue, ChunkStrategicOverlay, CorridorConstructionBook, FrontlineState,
    LogisticsGraph, WorldFieldLayerConfig, WorldFieldLayerEpoch, WorldReadSnapshot,
};
use super::construction_book::{
    align_corridor_book_with_transport_directory, transport_directory_edge_signature,
};
use crate::systems::terrain::materialize_chunks;
use crate::systems::transport::{TransportCostWeights, TransportEdgeDirectory, TransportFieldStore};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Ordering buckets for strategic field pipeline (`chunk_scheduler_runbook_v1` / transport coupling).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum StrategicFieldPipeline {
    EnsureOverlays,
    /// Rebuild [`LogisticsGraph`] from transport + corridor entities.
    GraphSync,
    InjectTransportScalars,
    LogisticsNetInject,
    /// Zones + frontline + planner read model (after logistics paints baselines).
    ZoneAndReadModel,
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
        commands
            .entity(entity)
            .insert(ChunkStrategicOverlay::new(chunk.coord, matrix.size));
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
        app.init_resource::<LogisticsGraph>()
            .init_resource::<StrategicRasterConfig>()
            .init_resource::<StrategicOverlayCouplingScratch>()
            .init_resource::<StrategicOverlayDisplayPolicy>()
            .init_resource::<CorridorConstructionBook>()
            .init_resource::<TransportEdgeDirectory>()
            .init_resource::<TransportFieldStore>()
            .init_resource::<TransportCostWeights>()
            .init_resource::<FrontlineState>()
            .init_resource::<WorldReadSnapshot>()
            .init_resource::<WorldFieldLayerEpoch>()
            .init_resource::<WorldFieldLayerConfig>()
            .init_resource::<BuildOrderQueue>()
            .init_resource::<ApprovedBuildOrders>()
            .configure_sets(
                Update,
                (
                    StrategicFieldPipeline::EnsureOverlays.after(materialize_chunks),
                    StrategicFieldPipeline::GraphSync.after(StrategicFieldPipeline::EnsureOverlays),
                    StrategicFieldPipeline::InjectTransportScalars.after(StrategicFieldPipeline::GraphSync),
                    StrategicFieldPipeline::LogisticsNetInject
                        .after(StrategicFieldPipeline::InjectTransportScalars),
                    StrategicFieldPipeline::ZoneAndReadModel
                        .after(StrategicFieldPipeline::LogisticsNetInject),
                ),
            )
            .add_systems(
                Update,
                ensure_chunk_strategic_overlays.in_set(StrategicFieldPipeline::EnsureOverlays),
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
        assert_eq!(overlay.ew_denial.len(), 6);

        let cfg = app.world().resource::<StrategicRasterConfig>();
        assert_eq!(cfg.cells_per_chunk, UVec2::new(3, 2));
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
