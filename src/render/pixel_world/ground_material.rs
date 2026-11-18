//! DESTROY-T2 — PixelWorld ground material (bake albedo + destruction mask).

use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

pub const GROUND_MASK_WGSL: &str = "shaders/pixel_world/ground_mask.wgsl";

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct GroundMaskUniform {
    /// xy = width/height tiles; z = skirt Y; w unused
    pub extent_skirt: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PixelWorldGroundMaterial {
    #[uniform(0)]
    pub uniform: GroundMaskUniform,
    #[texture(1)]
    #[sampler(2)]
    pub albedo: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub destruction_mask: Handle<Image>,
}

impl Material for PixelWorldGroundMaterial {
    fn vertex_shader() -> ShaderRef {
        GROUND_MASK_WGSL.into()
    }

    fn fragment_shader() -> ShaderRef {
        GROUND_MASK_WGSL.into()
    }
}

pub struct PixelWorldGroundMaterialPlugin;

impl Plugin for PixelWorldGroundMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<PixelWorldGroundMaterial>::default());
    }
}

/// Shared handle for all ground chunk entities.
#[derive(Resource, Clone, Debug, Default)]
pub struct PixelWorldGroundMaterialHandle(pub Handle<PixelWorldGroundMaterial>);
