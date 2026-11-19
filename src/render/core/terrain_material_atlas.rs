//! Deterministic GPU RGBA atlas from [`MaterialRegistry`] preview colors.
//!
//! **GPU-ARRAY.1** — also packs a `Texture2DArray` (one layer per material) so bake /
//! PixelWorld consumers can sample without 2D-atlas mip UV bleed.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::RenderApp;

use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::material::{MaterialId, MaterialRegistry};

const ATLAS_TILE_PX: u32 = 8;

/// GPU terrain material atlas — 2D grid (compat) + Texture2DArray (GPU-ARRAY).
#[derive(Resource, Clone, Debug, ExtractResource)]
#[extract_app(RenderApp)]
pub struct TerrainMaterialAtlasGpu {
    /// Legacy 2D grid atlas (cols×rows cells). Prefer [`Self::array_image`] for new paths.
    pub image: Handle<Image>,
    /// GPU-ARRAY: one `tile_px²` layer per material id (stable sort).
    pub array_image: Handle<Image>,
    pub revision: u64,
    pub cols: u32,
    pub rows: u32,
    pub material_count: u32,
    pub cell_uv: [f32; 2],
    pub tile_px: u32,
}

impl Default for TerrainMaterialAtlasGpu {
    fn default() -> Self {
        Self {
            image: Handle::default(),
            array_image: Handle::default(),
            revision: 0,
            cols: 1,
            rows: 1,
            material_count: 0,
            cell_uv: [1.0, 1.0],
            tile_px: ATLAS_TILE_PX,
        }
    }
}

#[must_use]
pub fn atlas_uv_for_material(atlas: &TerrainMaterialAtlasGpu, material: MaterialId) -> [f32; 4] {
    let idx = material.0 as u32;
    if atlas.material_count == 0 || atlas.cols == 0 {
        return [0.0, 0.0, atlas.cell_uv[0], atlas.cell_uv[1]];
    }
    let col = idx % atlas.cols;
    let row = idx / atlas.cols;
    let u0 = col as f32 * atlas.cell_uv[0];
    let v0 = row as f32 * atlas.cell_uv[1];
    [u0, v0, atlas.cell_uv[0], atlas.cell_uv[1]]
}

/// Pack sorted material preview colors into a `Texture2DArray` (layer = material slot).
#[must_use]
pub fn pack_material_preview_texture_array(
    colors: &[[u8; 4]],
    tile_px: u32,
) -> Image {
    let tile = tile_px.max(1);
    let layers = colors.len().max(1) as u32;
    let layer_bytes = (tile * tile * 4) as usize;
    let mut data = vec![0u8; layer_bytes * layers as usize];
    for (layer, color) in colors.iter().enumerate() {
        let base = layer * layer_bytes;
        for i in 0..(tile * tile) as usize {
            let o = base + i * 4;
            data[o..o + 4].copy_from_slice(color);
        }
    }
    let size = Extent3d {
        width: tile,
        height: tile,
        depth_or_array_layers: layers,
    };
    // wgpu/Bevy: array textures are D2 with depth_or_array_layers > 1 (not a D2Array dimension).
    let mut image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: Some("terrain_material_array"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };
    image.data = Some(data);
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

pub fn rebuild_terrain_material_atlas(
    registry: &MaterialRegistry,
    images: &mut Assets<Image>,
    atlas: &mut TerrainMaterialAtlasGpu,
) {
    let count = registry.materials.len().max(1) as u32;
    let cols = count.isqrt().max(1);
    let rows = count.div_ceil(cols);
    let w = cols * ATLAS_TILE_PX;
    let h = rows * ATLAS_TILE_PX;
    let mut data = vec![0u8; (w * h * 4) as usize];
    let mut layer_colors: Vec<[u8; 4]> = Vec::with_capacity(count as usize);

    let mut indexed: Vec<(u16, usize)> = registry
        .materials
        .iter()
        .enumerate()
        .map(|(i, _m)| (MaterialId(i as u16).0, i))
        .collect();
    indexed.sort_by_key(|(id, _)| *id);

    for (slot, (_id, mat_i)) in indexed.iter().enumerate() {
        let col = (slot as u32) % cols;
        let row = (slot as u32) / cols;
        let color = registry.materials[*mat_i].preview_color;
        layer_colors.push(color);
        for dy in 0..ATLAS_TILE_PX {
            for dx in 0..ATLAS_TILE_PX {
                let px = col * ATLAS_TILE_PX + dx;
                let py = row * ATLAS_TILE_PX + dy;
                let i = ((py * w + px) * 4) as usize;
                if i + 3 < data.len() {
                    data[i..i + 4].copy_from_slice(&color);
                }
            }
        }
    }
    if layer_colors.is_empty() {
        layer_colors.push([0, 0, 0, 255]);
    }

    let size = Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: bevy::render::render_resource::TextureDescriptor {
            label: Some("terrain_material_atlas"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };
    image.data = Some(data);
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        ..default()
    });

    let handle = atlas.image.clone();
    if handle == Handle::default() || images.get(&handle).is_none() {
        atlas.image = images.add(image);
    } else if let Some(mut existing) = images.get_mut(&handle) {
        *existing = image;
    }

    // GPU-ARRAY.1 — Texture2DArray companion (production: terrain bake RTT samples this).
    let array = pack_material_preview_texture_array(&layer_colors, ATLAS_TILE_PX);
    let array_handle = atlas.array_image.clone();
    if array_handle == Handle::default() || images.get(&array_handle).is_none() {
        atlas.array_image = images.add(array);
    } else if let Some(mut existing) = images.get_mut(&array_handle) {
        *existing = array;
    }

    atlas.revision = atlas.revision.wrapping_add(1);
    atlas.cols = cols;
    atlas.rows = rows;
    atlas.material_count = count;
    atlas.cell_uv = [1.0 / cols as f32, 1.0 / rows as f32];
    atlas.tile_px = ATLAS_TILE_PX;
}

pub fn sync_terrain_material_atlas_from_registry(
    handles: Option<Res<TerrainRegistriesHandles>>,
    materials: Res<Assets<MaterialRegistry>>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<TerrainMaterialAtlasGpu>,
) {
    let Some(handles) = handles else {
        return;
    };
    let Some(reg) = materials.get(&handles.material_registry) else {
        return;
    };
    // Registry is static after load: rebuilding every frame re-uploads (new GpuImage) the atlas
    // each frame and invalidates every bind group that samples it.
    if atlas.material_count > 0
        && atlas.array_image != Handle::default()
        && !materials.is_changed()
        && !handles.is_changed()
    {
        return;
    }
    rebuild_terrain_material_atlas(reg, images.as_mut(), atlas.as_mut());
}

pub struct TerrainMaterialAtlasPlugin;

impl Plugin for TerrainMaterialAtlasPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainMaterialAtlasGpu>()
            .add_plugins(ExtractResourcePlugin::<TerrainMaterialAtlasGpu>::default())
            .add_systems(
                Update,
                sync_terrain_material_atlas_from_registry.run_if(
                    |handles: Option<Res<TerrainRegistriesHandles>>,
                     materials: Res<Assets<MaterialRegistry>>| {
                        handles
                            .as_ref()
                            .and_then(|h| materials.get(&h.material_registry))
                            .is_some()
                    },
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID;
    use crate::terrain::material::MaterialDef;

    fn sample_registry() -> MaterialRegistry {
        MaterialRegistry {
            schema_version: 2,
            materials: vec![
                MaterialDef {
                    name: "grass".into(),
                    family: DEFAULT_TERRAIN_FAMILY_ID,
                    tags: vec![],
                    properties: serde_json::json!({}),
                    preview_color: [40, 160, 60, 255],
                },
                MaterialDef {
                    name: "sand".into(),
                    family: DEFAULT_TERRAIN_FAMILY_ID,
                    tags: vec![],
                    properties: serde_json::json!({}),
                    preview_color: [220, 200, 120, 255],
                },
            ],
            name_to_id: Default::default(),
            collision: Vec::new(),
        }
    }

    #[test]
    fn uv_round_trip_for_sample_materials() {
        let mut images = Assets::<Image>::default();
        let mut atlas = TerrainMaterialAtlasGpu::default();
        let reg = sample_registry();
        rebuild_terrain_material_atlas(&reg, &mut images, &mut atlas);
        assert!(atlas.material_count >= 2);
        assert_ne!(atlas.image, Handle::default());
        let uv0 = atlas_uv_for_material(&atlas, MaterialId(0));
        let uv1 = atlas_uv_for_material(&atlas, MaterialId(1));
        assert!(uv0[2] > 0.0 && uv0[3] > 0.0);
        assert!(uv1[0] >= uv0[0] || uv1[1] > uv0[1]);
    }

    #[test]
    fn gpu_array_packs_texture_2d_array_companion() {
        let mut images = Assets::<Image>::default();
        let mut atlas = TerrainMaterialAtlasGpu::default();
        rebuild_terrain_material_atlas(&sample_registry(), &mut images, &mut atlas);
        assert_ne!(atlas.array_image, Handle::default());
        let arr = images.get(&atlas.array_image).expect("array image");
        assert_eq!(arr.texture_descriptor.dimension, TextureDimension::D2);
        assert_eq!(
            arr.texture_descriptor.size.depth_or_array_layers,
            atlas.material_count
        );
        assert_eq!(arr.texture_descriptor.size.width, ATLAS_TILE_PX);
    }
}
