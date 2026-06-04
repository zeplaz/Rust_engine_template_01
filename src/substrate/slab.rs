//! `ChunkKey` + generic `ChunkSlab<T>` paging storage (WSS-PLAN-002).

use std::collections::{HashMap, HashSet};

use bevy::math::IVec2;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ChunkKey {
    pub x: i32,
    pub y: i32,
}

impl ChunkKey {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<IVec2> for ChunkKey {
    fn from(v: IVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<ChunkKey> for IVec2 {
    fn from(k: ChunkKey) -> Self {
        IVec2::new(k.x, k.y)
    }
}

/// Generic paged storage — one pattern for all persistent domains.
#[derive(Resource, Debug)]
pub struct ChunkSlab<T> {
    pub chunks: HashMap<ChunkKey, T>,
    pub resident: HashSet<ChunkKey>,
    pub dirty: HashSet<ChunkKey>,
}

impl<T> Default for ChunkSlab<T> {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            resident: HashSet::new(),
            dirty: HashSet::new(),
        }
    }
}

impl<T> ChunkSlab<T> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    #[must_use]
    pub fn contains(&self, key: ChunkKey) -> bool {
        self.chunks.contains_key(&key)
    }

    #[must_use]
    pub fn get(&self, key: ChunkKey) -> Option<&T> {
        self.chunks.get(&key)
    }

    pub fn get_mut(&mut self, key: ChunkKey) -> Option<&mut T> {
        self.dirty.insert(key);
        self.chunks.get_mut(&key)
    }

    pub fn insert(&mut self, key: ChunkKey, value: T) {
        self.dirty.insert(key);
        self.chunks.insert(key, value);
    }

    pub fn mark_dirty(&mut self, key: ChunkKey) {
        if self.chunks.contains_key(&key) {
            self.dirty.insert(key);
        }
    }

    pub fn clear_dirty(&mut self, key: ChunkKey) {
        self.dirty.remove(&key);
    }

    #[must_use]
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    pub fn set_resident(&mut self, key: ChunkKey, resident: bool) {
        if resident {
            self.resident.insert(key);
        } else {
            self.resident.remove(&key);
        }
    }

    pub fn clear_resident(&mut self, key: ChunkKey) {
        self.resident.remove(&key);
    }

    #[must_use]
    pub fn is_resident(&self, key: ChunkKey) -> bool {
        self.resident.contains(&key)
    }

    #[must_use]
    pub fn resident_count(&self) -> usize {
        self.resident.len()
    }
}
