//! DESTROY-T2 — PixelWorld ground material (bake albedo + destruction mask).

use bevy::pbr::{Material, MaterialPlugin};
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
