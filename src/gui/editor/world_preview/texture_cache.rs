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
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
    mut images: ResMut<Assets<Image>>,
    mut preview_state: ResMut<WorldPreviewState>,
    chunks: Query<&Chunk, With<ChunkCellMatrix>>,
) {
    if preview.width == params.width && preview.height == params.height {
        return;
    }

    let width = params.width;
    let height = params.height;
    let old_tex = preview.texture.clone();
    let old_front = swap.front.clone();
    let old_back = swap.back.clone();

    let new_front = images.add(rgba_preview_image(width, height));
    let new_back = images.add(rgba_preview_image(width, height));
    preview.texture = new_front.clone();
    preview.width = width;
    preview.height = height;

    swap.front = new_front;
    swap.back = new_back;
    swap.dirty = false;

    let _ = images.remove(old_tex.id());
    if old_front != Handle::default() {
        let _ = images.remove(old_front.id());
    }
    if old_back != Handle::default() {
        let _ = images.remove(old_back.id());
    }

    let coords = chunks.iter().map(|c| c.coord);
    invalidate_world(InvalidationReason::Tuning, &mut preview_state, coords);
}

pub fn init_world_preview_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    params: Res<WorldGenParams>,
) {
    let front_img = rgba_preview_image(params.width, params.height);
    let back_img = rgba_preview_image(params.width, params.height);
    let front = images.add(front_img);
    let back = images.add(back_img);

    commands.insert_resource(WorldPreviewTexture {
        texture: front.clone(),
        width: params.width,
        height: params.height,
    });
    commands.insert_resource(crate::gui::SwapImageBuffers {
        front,
        back,
        dirty: false,
    });
}

/// **D-3** — After GPU offscreen camera writes [`crate::gui::SwapImageBuffers::back`], swap so
/// [`WorldPreviewTexture::texture`] is the stable front handle egui samples.
pub fn present_world_preview_gpu_swap(
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
    mut preview_tex: ResMut<WorldPreviewTexture>,
    mut target: ResMut<super::preview_render_contract::PreviewRenderTarget>,
    mut dbg: ResMut<super::preview_render_contract::PreviewPresentationDebug>,
    mut authority: ResMut<super::preview_render_contract::PreviewPathAuthority>,
) {
    if !swap.dirty {
        return;
    }
    if swap.front == Handle::default() || swap.back == Handle::default() {
        return;
    }
    let f = swap.front.clone();
    let b = swap.back.clone();
    swap.front = b;
    swap.back = f;
    preview_tex.texture = swap.front.clone();
    target.image = swap.front.clone();
    target.size = UVec2::new(preview_tex.width, preview_tex.height);
    swap.dirty = false;
    dbg.swap_count = dbg.swap_count.saturating_add(1);
    dbg.last_front_asset_id_bits =
        super::preview_render_contract::preview_image_asset_id_bits(&swap.front);
    dbg.last_back_asset_id_bits =
        super::preview_render_contract::preview_image_asset_id_bits(&swap.back);
    authority.gpu_present_count = dbg.swap_count;
    dbg.authoritative_surface = super::preview_render_contract::PreviewAuthoritativeSurface::GpuRenderTarget;
}

/// **D-3** — After CPU raster writes [`crate::gui::SwapImageBuffers::back`], swap handles so
/// [`WorldPreviewTexture::texture`] is the **front** egui samples (`base_visual_dev01_plan_status` § phase-d).
pub fn present_world_preview_swap_after_raster(
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
    mut preview_tex: ResMut<WorldPreviewTexture>,
    mut dbg: ResMut<super::preview_render_contract::PreviewPresentationDebug>,
) {
    if !swap.dirty {
        return;
    }
    if swap.front == Handle::default() || swap.back == Handle::default() {
        return;
    }
    let f = swap.front.clone();
    let b = swap.back.clone();
    swap.front = b;
    swap.back = f;
    preview_tex.texture = swap.front.clone();
    swap.dirty = false;
    dbg.swap_count = dbg.swap_count.saturating_add(1);
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
