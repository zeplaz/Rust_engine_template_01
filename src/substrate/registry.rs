//! `WorldSubstrateRegistry` + paging/persist stubs (WSS-PLAN-002).

use std::collections::HashMap;

use bevy::math::DVec2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::substrate::slab::{ChunkKey, ChunkSlab};
use crate::substrate::types::{DynamicOverlaySlice, WorldChunkState};

/// Persisted slab slice for one chunk (PR-4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersistChunkRecord {
    pub dynamic: DynamicOverlaySlice,
    pub version: u32,
}

/// In-memory save slot — dirty resident keys flushed from slab.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SubstratePersistSnapshot {
    pub chunks: HashMap<ChunkKey, PersistChunkRecord>,
}

#[derive(Resource, Debug, Default)]
pub struct ChunkPagingState {
    pub focus: DVec2,
    pub radius_chunks: i32,
}

#[derive(Resource, Debug, Default)]
pub struct SubstratePersistBook {
    pub last_flush_tick: u64,
    pub pending_slots: u32,
    /// PR-4 in-memory save slots (dirty resident keys flushed on write).
    pub snapshots: Vec<SubstratePersistSnapshot>,
}

#[derive(Resource, Debug)]
pub struct WorldSubstrateRegistry {
    pub chunks: ChunkSlab<WorldChunkState>,
    pub paging: ChunkPagingState,
    pub persist: SubstratePersistBook,
}

impl Default for WorldSubstrateRegistry {
    fn default() -> Self {
        Self {
            chunks: ChunkSlab::default(),
            paging: ChunkPagingState::default(),
            persist: SubstratePersistBook::default(),
        }
    }
}

#[derive(Resource, Debug)]
pub struct WssSubstrateWitness {
    pub hydrate_wired: bool,
    pub paging_wired: bool,
    /// SubstratePlugin does not reorder `ChunkEnvironmentSet` in PR-1.
    pub chunk_environment_order_preserved: bool,
}

impl Default for WssSubstrateWitness {
    fn default() -> Self {
        Self {
            hydrate_wired: false,
            paging_wired: false,
            chunk_environment_order_preserved: true,
        }
    }
}

impl WorldSubstrateRegistry {
    #[must_use]
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
}
