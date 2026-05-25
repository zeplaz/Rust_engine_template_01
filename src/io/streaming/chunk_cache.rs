//! In-memory chunk cache hot path for Wave C streaming.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use bevy::prelude::{IVec2, Resource};

use crate::io::save::{compress_payload, SavedChunkBody};

/// Disk tier thresholds are configurable; spill uses the same save artifact envelope.
pub const CHUNK_CACHE_DISK_TIER_OPEN: &str = "optional ChunkCache disk tier thresholds";

#[derive(Resource, Clone, Debug)]
pub struct ChunkCacheTierSettings {
    pub max_hot_entries: usize,
    pub spill_dir: PathBuf,
}

impl Default for ChunkCacheTierSettings {
    fn default() -> Self {
        Self {
            max_hot_entries: 256,
            spill_dir: PathBuf::from("cache/chunk_spill"),
        }
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct ChunkCacheDiskSpill {
    pub paths: HashMap<IVec2, PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkCacheEntry {
    pub coord: IVec2,
    pub material_names: Vec<String>,
    pub content_hash: u64,
    pub last_touch: u64,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct ChunkCache {
    pub entries: HashMap<IVec2, ChunkCacheEntry>,
    touch_seq: u64,
}

impl ChunkCache {
    pub fn upsert_from_saved_body(
        &mut self,
        coord: IVec2,
        body: &SavedChunkBody,
        tier: &ChunkCacheTierSettings,
        spill: &mut ChunkCacheDiskSpill,
    ) -> io::Result<()> {
        self.enforce_hot_tier(tier, spill)?;
        let material_names = body
            .cells
            .iter()
            .map(|cell| cell.material_name.clone())
            .collect();
        let content_hash = hash_saved_chunk_body(body);
        self.touch_seq = self.touch_seq.saturating_add(1);
        self.entries.insert(
            coord,
            ChunkCacheEntry {
                coord,
                material_names,
                content_hash,
                last_touch: self.touch_seq,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn get(&self, coord: IVec2) -> Option<&ChunkCacheEntry> {
        self.entries.get(&coord)
    }

    fn enforce_hot_tier(
        &mut self,
        tier: &ChunkCacheTierSettings,
        spill: &mut ChunkCacheDiskSpill,
    ) -> io::Result<()> {
        while self.entries.len() >= tier.max_hot_entries {
            let Some((coord, entry)) = self
                .entries
                .iter()
                .min_by_key(|(_, row)| row.last_touch)
                .map(|(coord, row)| (*coord, row.clone()))
            else {
                break;
            };
            self.spill_entry_to_disk(coord, &entry, tier, spill)?;
            self.entries.remove(&coord);
        }
        Ok(())
    }

    fn spill_entry_to_disk(
        &mut self,
        coord: IVec2,
        entry: &ChunkCacheEntry,
        tier: &ChunkCacheTierSettings,
        spill: &mut ChunkCacheDiskSpill,
    ) -> io::Result<()> {
        fs::create_dir_all(&tier.spill_dir)?;
        let path = tier
            .spill_dir
            .join(format!("chunk_{}_{}.ron", coord.x, coord.y));
        let body = SavedChunkBody {
            schema_version: crate::io::save::SAVED_CHUNK_BODY_SCHEMA_VERSION,
            chunk: [coord.x, coord.y],
            cells: entry
                .material_names
                .iter()
                .map(|name| crate::io::save::SavedTerrainCell {
                    material_name: name.clone(),
                    tags: Vec::new(),
                })
                .collect(),
        };
        let encoded = crate::io::save::encode_chunk_body_ron(&body)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let wrapped = compress_payload(&encoded);
        fs::write(&path, wrapped)?;
        spill.paths.insert(coord, path);
        Ok(())
    }

    pub fn promote_from_disk_spill(
        &mut self,
        coord: IVec2,
        spill: &mut ChunkCacheDiskSpill,
        tier: &ChunkCacheTierSettings,
    ) -> io::Result<bool> {
        let Some(path) = spill.paths.remove(&coord) else {
            return Ok(false);
        };
        let bytes = fs::read(&path)?;
        let payload = crate::io::save::unwrap_chunk_artifact_body(&bytes)?;
        let body = crate::io::save::decode_chunk_body_ron(payload)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.upsert_from_saved_body(coord, &body, tier, spill)?;
        let _ = fs::remove_file(path);
        Ok(true)
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
        let tier = ChunkCacheTierSettings {
            max_hot_entries: 8,
            spill_dir: std::env::temp_dir().join("chunk_cache_test_spill"),
        };
        let mut spill = ChunkCacheDiskSpill::default();
        let body = SavedChunkBody {
            schema_version: 1,
            chunk: [1, 2],
            cells: vec![crate::io::save::SavedTerrainCell {
                material_name: "grass".into(),
                tags: vec!["wet".into()],
            }],
        };
        cache
            .upsert_from_saved_body(IVec2::new(1, 2), &body, &tier, &mut spill)
            .unwrap();
        let entry = cache.get(IVec2::new(1, 2)).unwrap();
        assert_eq!(entry.material_names, vec!["grass".to_string()]);
    }

    #[test]
    fn chunk_cache_spills_lru_when_hot_tier_exceeded() {
        let dir = std::env::temp_dir().join(format!(
            "chunk_cache_spill_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let tier = ChunkCacheTierSettings {
            max_hot_entries: 1,
            spill_dir: dir.clone(),
        };
        let mut cache = ChunkCache::default();
        let mut spill = ChunkCacheDiskSpill::default();
        let body_a = SavedChunkBody {
            schema_version: 1,
            chunk: [0, 0],
            cells: vec![crate::io::save::SavedTerrainCell {
                material_name: "a".into(),
                tags: Vec::new(),
            }],
        };
        let body_b = SavedChunkBody {
            schema_version: 1,
            chunk: [1, 0],
            cells: vec![crate::io::save::SavedTerrainCell {
                material_name: "b".into(),
                tags: Vec::new(),
            }],
        };
        cache
            .upsert_from_saved_body(IVec2::ZERO, &body_a, &tier, &mut spill)
            .unwrap();
        cache
            .upsert_from_saved_body(IVec2::new(1, 0), &body_b, &tier, &mut spill)
            .unwrap();
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(spill.paths.len(), 1);
        let _ = fs::remove_dir_all(dir);
    }
}
