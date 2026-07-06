//! `ChunkCoord → Entity` lookup for bounded fire extract (no full-world `Query` iteration).
//!
//! Maintained via `Added`/`RemovedComponents<Chunk>` systems — not rebuilt each frame.

use bevy::prelude::*;
use rustc_hash::FxHashMap;

use crate::render::fire_chunk_runtime::ChunkCoord;

/// Chunk entity lookup for [`extract_fire_simulation_snapshot`](crate::render::extraction::extract_fire_simulation_snapshot).
#[derive(Resource, Default, Debug)]
pub struct ChunkFireEntityIndex {
    pub by_coord: FxHashMap<ChunkCoord, Entity>,
    by_entity: FxHashMap<Entity, ChunkCoord>,
    pub revision: u64,
}

impl ChunkFireEntityIndex {
    pub fn len(&self) -> usize {
        self.by_coord.len()
    }

    pub fn insert(&mut self, coord: ChunkCoord, entity: Entity) {
        if let Some(old) = self.by_coord.insert(coord, entity) {
            self.by_entity.remove(&old);
        }
        self.by_entity.insert(entity, coord);
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(coord) = self.by_entity.remove(&entity) {
            self.by_coord.remove(&coord);
        }
    }

    pub fn clear(&mut self) {
        self.by_coord.clear();
        self.by_entity.clear();
        self.revision = self.revision.wrapping_add(1);
    }
}

/// Bootstrap + track chunk spawns without scanning the world each frame.
pub fn sync_chunk_fire_entity_index_added(
    added: Query<(Entity, &crate::terrain::generation::Chunk), Added<crate::terrain::generation::Chunk>>,
    mut index: ResMut<ChunkFireEntityIndex>,
) {
    for (entity, chunk) in &added {
        index.insert(chunk.coord, entity);
    }
}

pub fn sync_chunk_fire_entity_index_removed(
    mut removed: RemovedComponents<crate::terrain::generation::Chunk>,
    mut index: ResMut<ChunkFireEntityIndex>,
) {
    for entity in removed.read() {
        index.remove_entity(entity);
    }
}

/// One-shot fill when slabs exist before the Added hook ran (e.g. test harness spawn).
pub fn bootstrap_chunk_fire_entity_index_if_empty(
    chunks: Query<(Entity, &crate::terrain::generation::Chunk)>,
    mut index: ResMut<ChunkFireEntityIndex>,
) {
    if !index.by_coord.is_empty() {
        return;
    }
    for (entity, chunk) in &chunks {
        index.insert(chunk.coord, entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::generation::Chunk;
    use bevy::prelude::{App, MinimalPlugins};

    #[test]
    fn chunk_fire_entity_index_insert_remove_clear() {
        let mut index = ChunkFireEntityIndex::default();
        let e0 = Entity::from_bits(1);
        let e1 = Entity::from_bits(2);
        let c0 = ChunkCoord::new(0, 0);
        let c1 = ChunkCoord::new(1, 0);
        index.insert(c0, e0);
        index.insert(c1, e1);
        assert_eq!(index.len(), 2);
        index.remove_entity(e0);
        assert_eq!(index.len(), 1);
        assert!(!index.by_coord.contains_key(&c0));
        let rev = index.revision;
        index.clear();
        assert_eq!(index.len(), 0);
        assert_ne!(index.revision, rev);
    }

    #[test]
    fn chunk_fire_entity_index_tracks_added_chunks() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ChunkFireEntityIndex>()
            .add_systems(
                Update,
                (
                    bootstrap_chunk_fire_entity_index_if_empty,
                    sync_chunk_fire_entity_index_added,
                    sync_chunk_fire_entity_index_removed,
                )
                    .chain(),
            );
        app.world_mut().spawn((
            Chunk {
                coord: ChunkCoord::new(2, 3),
            },
        ));
        app.update();
        let index = app.world().resource::<ChunkFireEntityIndex>();
        assert_eq!(index.len(), 1);
        assert!(index.by_coord.contains_key(&ChunkCoord::new(2, 3)));
    }
}
