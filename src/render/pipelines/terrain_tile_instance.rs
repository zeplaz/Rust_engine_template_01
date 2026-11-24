//! Shared POD for material-index bake / stamp feeds (GS-03: drawn instanced path deleted).

use bytemuck::{Pod, Zeroable};

/// One world tile material index for GPU bake control / stamp feeds.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct TerrainTileInstance {
    pub world_pos: [f32; 2],
    pub material_index: u32,
    pub _pad: u32,
}
