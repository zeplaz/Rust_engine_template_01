//! DESTROY-T2 — PixelWorld ground material (bake albedo + destruction mask).
//!
//! Lit: the ground is Bevy's [`StandardMaterial`] (bake albedo as `base_color_texture`) extended
//! with the destruction mask, so it takes the scene sun ([`super::camera::PixelWorldSun`]),
//! ambient and directional shadow maps like every other voxel-view mesh. The old unlit material
//! drew the flat bake colour on tops and cliff sides alike, so relief read as one solid green.

use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

pub const GROUND_MASK_WGSL: &str = "shaders/pixel_world/ground_mask.wesl";

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct GroundMaskUniform {
    /// xy = width/height tiles; z = skirt Y; w unused
    pub extent_skirt: Vec4,
}

/// Destruction-mask extension (bindings 100+; 0..99 belong to [`StandardMaterial`]).
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct PixelWorldGroundExt {
    #[uniform(100)]
    pub uniform: GroundMaskUniform,
    #[texture(101)]
    #[sampler(102)]
    pub destruction_mask: Handle<Image>,
    /// GPU-B scorch authority ([`crate::render::pipelines::terrain_scorch_field::TerrainScorchField`]):
    /// live fire heat per tile, drawn as an ember glow on the voxel ground (same source as 2D).
    #[texture(103)]
    #[sampler(104)]
    pub scorch: Handle<Image>,
}

impl MaterialExtension for PixelWorldGroundExt {
    fn vertex_shader() -> ShaderRef {
        GROUND_MASK_WGSL.into()
    }

    fn fragment_shader() -> ShaderRef {
        GROUND_MASK_WGSL.into()
    }
}

/// Voxel ground material: lit standard surface + destruction mask.
pub type PixelWorldGroundMaterial = ExtendedMaterial<StandardMaterial, PixelWorldGroundExt>;

/// Point the ground material at the scorch field image (narrow: scorch -> ground material only).
pub fn sync_ground_scorch_texture(
    scorch: Option<Res<crate::render::pipelines::terrain_scorch_field::TerrainScorchField>>,
    handle: Res<PixelWorldGroundMaterialHandle>,
    mut materials: ResMut<Assets<PixelWorldGroundMaterial>>,
) {
    let Some(scorch) = scorch else {
        return;
    };
    if scorch.image == Handle::default() || handle.0 == Handle::default() {
        return;
    }
    let stale = materials.get(&handle.0).is_some_and(|m| m.extension.scorch != scorch.image);
    if stale {
        if let Some(mut m) = materials.get_mut(&handle.0) {
            m.extension.scorch = scorch.image.clone();
        }
    }
}

/// Matte ground surface for the bake albedo (`None` until the bake image exists).
#[must_use]
pub fn ground_base_material(albedo: Option<Handle<Image>>) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: albedo,
        perceptual_roughness: 0.95,
        reflectance: 0.12,
        ..default()
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

#[cfg(test)]
mod tests {
    /// Bevy 0.20 passes `.wgsl` to wgpu raw (no naga_oil): `#import`/`#ifdef` there is a parse
    /// error and the material silently never draws. Shaders that import Bevy modules must be
    /// `.wesl` with `import …;` / `@if(…)` (PLAN-CLOSE-VOXEL-WATER-v1 V1b).
    #[test]
    fn close_v1b_no_naga_oil_directives_in_wgsl() {
        fn walk(dir: &std::path::Path, bad: &mut Vec<String>) {
            for e in std::fs::read_dir(dir).expect("shader dir").flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, bad);
                } else if p.extension().is_some_and(|x| x == "wgsl") {
                    let src = std::fs::read_to_string(&p).unwrap_or_default();
                    for (i, l) in src.lines().enumerate() {
                        let t = l.trim_start();
                        if ["#import", "#ifdef", "#ifndef", "#define_import_path", "#if "]
                            .iter()
                            .any(|d| t.starts_with(d))
                        {
                            bad.push(format!("{}:{}", p.display(), i + 1));
                        }
                    }
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/shaders");
        let mut bad = Vec::new();
        walk(&root, &mut bad);
        assert!(bad.is_empty(), "naga_oil directives in raw .wgsl: {bad:?}");
    }
}
