//! Canonical save manifest — names on disk, not runtime ids.

use serde::{Deserialize, Serialize};

pub const SAVE_WORLD_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Root manifest for one world save bundle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveWorldManifest {
    pub schema_version: u32,
    pub world_seed: u64,
    pub chunk_sets: Vec<ChunkSetRef>,
    pub registries: Vec<RegistrySnapshotRef>,
    pub overlays: Vec<OverlaySnapshotRef>,
}

/// Chunk payload artifact reference (incremental save slice).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkSetRef {
    pub chunk: [i32; 2],
    pub artifact_path: String,
}

/// Registry snapshot artifact reference (materials, tags, biome defs, …).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrySnapshotRef {
    pub registry_name: String,
    pub artifact_path: String,
}

/// Overlay / domain snapshot artifact reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlaySnapshotRef {
    pub overlay_name: String,
    pub artifact_path: String,
}

#[must_use]
pub fn build_save_world_manifest(
    world_seed: u64,
    chunk_sets: Vec<ChunkSetRef>,
    registries: Vec<RegistrySnapshotRef>,
    overlays: Vec<OverlaySnapshotRef>,
) -> SaveWorldManifest {
    SaveWorldManifest {
        schema_version: SAVE_WORLD_MANIFEST_SCHEMA_VERSION,
        world_seed,
        chunk_sets,
        registries,
        overlays,
    }
}
