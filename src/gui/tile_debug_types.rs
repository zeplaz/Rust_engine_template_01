//! Shared types for GPU-style tile debug overlays (logical instances + settings).

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};

/// View bucket for instance streams (preview/minimap reserved for future Bevy targets).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileDebugViewId {
    WorldMain,
    WorldPreview,
    Minimap,
    SimulationMap,
}

/// Marker on the gameplay [`MainWorldCamera`](crate::gui::map_camera::MainWorldCamera): Core2d
/// tile-debug instancing runs only on this view.
#[derive(Component, Clone, Copy, Default, Reflect, bevy::render::extract_component::ExtractComponent)]
#[reflect(Component)]
pub struct TileDebugRenderHost;

#[derive(Resource, Clone, Copy, Default, ExtractResource, ShaderType)]
pub struct TileDebugDrawGlobals {
    pub view_proj: Mat4,
    pub instance_count: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

pub mod tile_flags {
    pub const FOCUS: u32 = 1 << 0;
    pub const FIRE: u32 = 1 << 1;
    pub const SELECT: u32 = 1 << 2;
    /// Chunk exists in terrain query (same signal as the legacy gizmo “terrain green”).
    pub const TERRAIN: u32 = 1 << 3;
    /// Construction footprint — valid placement (Visual Aid v2 VA2).
    pub const FOOTPRINT_VALID: u32 = 1 << 4;
    pub const FOOTPRINT_RISKY: u32 = 1 << 5;
    pub const FOOTPRINT_INVALID: u32 = 1 << 6;
    /// Active construction site tile — `lod` carries [`SiteConstructionPhase`] discriminant.
    pub const CONSTRUCTION_SITE: u32 = 1 << 7;
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct TileDebugInstance {
    pub world_pos: [f32; 2],
    pub size: f32,
    pub lod: u32,
    pub flags: u32,
}

#[derive(Resource, Default, Debug, Clone, ExtractResource)]
pub struct TileDebugInstanceMap {
    pub per_view: HashMap<TileDebugViewId, Vec<TileDebugInstance>>,
}

#[derive(Resource, Debug, Clone)]
pub struct TileGpuDebugSettings {
    /// When true, replaces per-tile gizmo rects with GPU instanced quads (single draw).
    pub use_batched_mesh_overlay: bool,
    pub max_instances: usize,
}

impl Default for TileGpuDebugSettings {
    fn default() -> Self {
        Self {
            // Off until [`BaseState::Simulation`] (`enable_tile_gpu_instanced_authoritative`) or test harness.
            use_batched_mesh_overlay: false,
            max_instances: 4096,
        }
    }
}

/// Dev-only: force fire tint on tile debug instances (triage when sim ↔ visual linkage is suspect).
#[derive(Resource, Debug, Clone)]
pub struct FireDebugOverride {
    pub force_visible: bool,
}

impl Default for FireDebugOverride {
    fn default() -> Self {
        Self {
            force_visible: false,
        }
    }
}

/// When true, keep the legacy gizmo overlay for chunk debug.
pub fn tile_debug_use_gizmos_instead(settings: Res<TileGpuDebugSettings>) -> bool {
    !settings.use_batched_mesh_overlay
}

/// Instanced path has at least one tile (skip duplicate CPU/egui overlays for that channel).
#[must_use]
pub fn tile_debug_instanced_has_instances(
    map: &TileDebugInstanceMap,
    settings: &TileGpuDebugSettings,
) -> bool {
    settings.use_batched_mesh_overlay
        && map
            .per_view
            .values()
            .any(|instances| !instances.is_empty())
}

/// Construction phase tiles are on the GPU instanced path — skip egui phase quads.
#[must_use]
pub fn construction_phase_on_instanced_path(
    map: &TileDebugInstanceMap,
    settings: &TileGpuDebugSettings,
) -> bool {
    settings.use_batched_mesh_overlay
        && map.per_view.values().any(|instances| {
            instances
                .iter()
                .any(|i| (i.flags & tile_flags::CONSTRUCTION_SITE) != 0)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_debug_instance_pod_layout() {
        // [f32;2] + f32 + u32 + u32 = 20 bytes (padding after `size` before `lod`).
        assert_eq!(std::mem::size_of::<TileDebugInstance>(), 20);
        assert_eq!(std::mem::align_of::<TileDebugInstance>(), 4);
    }
}
