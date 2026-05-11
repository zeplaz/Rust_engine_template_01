//! GPU-side RGBA buffer handle used as an egui texture source (full-world CPU raster today).

use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::{invalidate_world, InvalidationReason, WorldPreviewState};

pub fn rgba_preview_image(width: u32, height: u32) -> Image {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.data = Some(vec![0; 4 * width as usize * height as usize]);
    image
}

/// Resize the RGBA preview buffer when `WorldGenParams` width/height changes.
pub fn sync_world_preview_texture_size(
    params: Res<WorldGenParams>,
    mut preview: ResMut<WorldPreviewTexture>,
    mut images: ResMut<Assets<Image>>,
    mut preview_state: ResMut<WorldPreviewState>,
    chunks: Query<&Chunk, With<ChunkCellMatrix>>,
) {
    if preview.width == params.width && preview.height == params.height {
        return;
    }

    let width = params.width;
    let height = params.height;
    let old = preview.texture.clone();
    let image = rgba_preview_image(width, height);
    preview.texture = images.add(image);
    preview.width = width;
    preview.height = height;

    let _ = images.remove(old.id());

    let coords = chunks.iter().map(|c| c.coord);
    invalidate_world(InvalidationReason::Tuning, &mut preview_state, coords);
}

pub fn init_world_preview_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    params: Res<WorldGenParams>,
) {
    let image = rgba_preview_image(params.width, params.height);
    let texture_handle = images.add(image);

    commands.insert_resource(WorldPreviewTexture {
        texture: texture_handle,
        width: params.width,
        height: params.height,
    });
}

#[derive(Resource)]
pub struct WorldPreviewTexture {
    pub texture: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

impl Default for WorldPreviewTexture {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            width: 512,
            height: 512,
        }
    }
}
