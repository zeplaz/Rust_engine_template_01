//! Generic instanced-quad GPU row layout (backend transport).
//!
//! Fire VFX maps semantic fields in `fire_vfx::pack` — byte stride must stay 32 until
//! all WGSL + registry consumers flip together.

use bevy::math::{Mat4, Vec4};
use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};

/// One instanced billboard source row: world position + four custom scalars per vec4 lane.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct GpuInstancedQuadInstance {
    /// `.xyz` world position; `.w` = lane-specific scalar (fire: heat).
    pub position_xyz_custom0: Vec4,
    /// Four lane scalars (fire: ember, class_id, world half-edge, smoke).
    pub custom1_custom2_custom3_custom4: Vec4,
}

/// Expanded corner vertex for instanced-quad raster passes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct GpuInstancedQuadVertex {
    /// World `xy`, `z` = custom0, `w` = custom1 (fire: heat, ember).
    pub world_xy_custom01: Vec4,
}

impl GpuInstancedQuadVertex {
    #[must_use]
    pub fn from_corner(world_x: f32, world_y: f32, custom0: f32, custom1: f32) -> Self {
        Self {
            world_xy_custom01: Vec4::new(world_x, world_y, custom0, custom1),
        }
    }
}

/// Shared view globals layout for fire/water particle raster WGSL (RTT-B5-001).
#[derive(Clone, Copy, Debug, Default, PartialEq, ShaderType)]
pub struct ParticleViewGlobals {
    pub view_proj: Mat4,
    pub vertex_count: u32,
    pub time_secs: f32,
    pub zoom_alpha: f32,
    pub _pad: f32,
}

/// Generic expand-pass uniform fields (bind group 0, bindings 0–4 in fire expand shader).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParticleSystemUniforms {
    pub instance_count: u32,
    pub max_instances: u32,
    pub time_secs: f32,
    pub camera_zoom: f32,
    pub zoom_alpha: f32,
}

/// Fire-only extension packed after system uniforms in the same GPU uniform block (migration).
#[allow(dead_code)] // constructed via `WorldFireParticleDrawUniforms::fire_extension` (tests / future WGSL split)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FireSparkDrawExtension {
    pub spark_sim_enabled: f32,
}
