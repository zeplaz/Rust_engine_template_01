//! **GS-03 CLOSED** — per-tile instanced terrain draw **deleted**.
//!
//! Sole terrain display authority is the world bake texture
//! ([`plan_gpu_stubs_kill_v1.md`](../../dev/plan_gpu_stubs_kill_v1.md)).
//! [`TerrainTileInstance`] remains as the bake/stamp POD feed type.

use bevy::prelude::*;

pub use super::terrain_tile_instance::TerrainTileInstance;

/// Retired WGSL path — file may remain on disk; not loaded.
pub const TERRAIN_INSTANCED_WGSL: &str = "shaders/terrain/terrain_instanced.wgsl";

/// Retired camera host marker (kept so existing inserts compile; no draw pass reads it).
#[derive(Component, Clone, Copy, Default)]
pub struct TerrainInstancedRenderHost;

/// Retired globals resource (empty).
#[derive(Resource, Clone, Copy, Default)]
pub struct TerrainInstancedDrawGlobals {
    pub instance_count: u32,
}

/// Retired instance map (always empty).
#[derive(Resource, Clone, Default)]
pub struct TerrainInstanceMap {
    pub instances: Vec<TerrainTileInstance>,
    pub revision: u64,
}

/// No-op plugin — GS-03 deleted the Core2d instanced terrain pass.
pub struct TerrainInstancedDrawPlugin;

impl Plugin for TerrainInstancedDrawPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainInstanceMap>()
            .init_resource::<TerrainInstancedDrawGlobals>();
    }
}

/// No-op — render pass removed (GS-03).
pub fn register_terrain_instanced_draw(_app: &mut App) {}
