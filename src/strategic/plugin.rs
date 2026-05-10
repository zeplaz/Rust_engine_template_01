//! Bevy integration: [`ChunkStrategicOverlay`] on every chunk that has a [`ChunkCellMatrix`](crate::terrain::generation::ChunkCellMatrix).

use bevy::prelude::*;

use super::logistics_net::logistics_net_inject_into_overlays;
use super::transport_bridge::{
    inject_transport_scalar_fields_into_overlays, maintain_strategic_corridor_entities,
    sync_logistics_graph_from_transport, StrategicRasterConfig,
};
use super::{ChunkStrategicOverlay, LogisticsGraph};
use crate::systems::terrain::materialize_chunks;
use crate::systems::transport::{TransportCostWeights, TransportEdgeDirectory, TransportFieldStore};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

fn ensure_chunk_strategic_overlays(
    mut commands: Commands,
    q: Query<(Entity, &Chunk, &ChunkCellMatrix), Without<ChunkStrategicOverlay>>,
) {
    for (entity, chunk, matrix) in q.iter() {
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
            .init_resource::<TransportEdgeDirectory>()
            .init_resource::<TransportFieldStore>()
            .init_resource::<TransportCostWeights>()
            .add_systems(Update, ensure_chunk_strategic_overlays.after(materialize_chunks))
            .add_systems(
                Update,
                (
                    sync_logistics_graph_from_transport,
                    maintain_strategic_corridor_entities,
                )
                    .chain()
                    .after(ensure_chunk_strategic_overlays),
            )
            .add_systems(
                Update,
                inject_transport_scalar_fields_into_overlays
                    .after(ensure_chunk_strategic_overlays)
                    .after(maintain_strategic_corridor_entities)
                    .before(logistics_net_inject_into_overlays),
            )
            .add_systems(
                Update,
                logistics_net_inject_into_overlays.after(inject_transport_scalar_fields_into_overlays),
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
