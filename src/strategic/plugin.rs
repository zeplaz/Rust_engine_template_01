//! Bevy integration: [`ChunkStrategicOverlay`] on every chunk that has a [`ChunkCellMatrix`](crate::terrain::generation::ChunkCellMatrix).

use bevy::prelude::*;

use super::logistics_net::logistics_net_inject_into_overlays;
use super::{ChunkStrategicOverlay, LogisticsGraph};
use crate::systems::terrain::materialize_chunks;
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
            .add_systems(Update, ensure_chunk_strategic_overlays.after(materialize_chunks))
            .add_systems(
                Update,
                logistics_net_inject_into_overlays.after(ensure_chunk_strategic_overlays),
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
    }
}
