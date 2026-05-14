//! In-memory chunk cache hot path for Wave C streaming.

use std::collections::HashMap;

use bevy::prelude::{IVec2, Resource};

use crate::io::save::SavedChunkBody;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkCacheEntry {
    pub coord: IVec2,
    pub material_names: Vec<String>,
    pub content_hash: u64,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct ChunkCache {
    pub entries: HashMap<IVec2, ChunkCacheEntry>,
}

impl ChunkCache {
    pub fn upsert_from_saved_body(&mut self, coord: IVec2, body: &SavedChunkBody) {
        let material_names = body
            .cells
            .iter()
            .map(|cell| cell.material_name.clone())
            .collect();
        let content_hash = hash_saved_chunk_body(body);
        self.entries.insert(
            coord,
            ChunkCacheEntry {
                coord,
                material_names,
                content_hash,
            },
        );
    }

    #[must_use]
    pub fn get(&self, coord: IVec2) -> Option<&ChunkCacheEntry> {
        self.entries.get(&coord)
    }
}

#[must_use]
pub fn hash_saved_chunk_body(body: &SavedChunkBody) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    body.schema_version.hash(&mut hasher);
    body.chunk.hash(&mut hasher);
    for cell in &body.cells {
        cell.material_name.hash(&mut hasher);
        for tag in &cell.tags {
            tag.hash(&mut hasher);
        }
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_cache_upsert_reads_material_names() {
        let mut cache = ChunkCache::default();
        let body = SavedChunkBody {
            schema_version: 1,
            chunk: [1, 2],
            cells: vec![crate::io::save::SavedTerrainCell {
                material_name: "grass".into(),
                tags: vec!["wet".into()],
            }],
        };
        cache.upsert_from_saved_body(IVec2::new(1, 2), &body);
        let entry = cache.get(IVec2::new(1, 2)).unwrap();
        assert_eq!(entry.material_names, vec!["grass".to_string()]);
    }
}
