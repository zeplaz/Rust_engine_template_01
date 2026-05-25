//! GPU-side RGBA buffer handle used as an egui texture source (full-world CPU raster today).

use bevy::asset::RenderAssetUsages;
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use super::preview_render_contract::{PreviewAuthoritativeSurface, PreviewPathAuthority};
use super::render_target_barrier::{
    WorldPreviewGpuResizeQueue, WorldPreviewRenderTargetBindBarrier, WorldPreviewRenderTargetRegistry,
};
use crate::render::{trace_render_target, DebugRenderTraceConfig};
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

    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image.data = Some(vec![0; 4 * width as usize * height as usize]);
    image
}

/// Resize the RGBA preview buffer when `WorldGenParams` width/height changes.
pub fn sync_world_preview_texture_size(
    preview_ui: Res<super::WorldPreviewUiState>,
    world_gen: Res<crate::gui::editor::world_gen_ui::WorldGenUiState>,
    params: Res<WorldGenParams>,
    path: Res<PreviewPathAuthority>,
    mut preview: ResMut<WorldPreviewTexture>,
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
    mut images: ResMut<Assets<Image>>,
    mut preview_state: ResMut<WorldPreviewState>,
    chunks: Query<&Chunk, With<ChunkCellMatrix>>,
) {
    if !preview_ui.window_open && !world_gen.visible {
        return;
    }
    if preview.width == params.width && preview.height == params.height {
        return;
    }

    let width = params.width;
    let height = params.height;
    preview.width = width;
    preview.height = height;

    if path.authoritative_surface == PreviewAuthoritativeSurface::GpuRenderTarget {
        let coords = chunks.iter().map(|c| c.coord);
        invalidate_world(InvalidationReason::Tuning, &mut preview_state, coords);
        return;
    }

    let old_tex = preview.texture.clone();
    let old_front = swap.front.clone();
    let old_back = swap.back.clone();

    let new_front = images.add(rgba_preview_image(width, height));
    let new_back = images.add(rgba_preview_image(width, height));
    preview.texture = new_front.clone();

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
    if let Some(image) = images.get_mut(&front) {
        if let Some(data) = image.data.as_mut() {
            data.fill(0);
        }
    }
    if let Some(image) = images.get_mut(&back) {
        if let Some(data) = image.data.as_mut() {
            data.fill(0);
        }
    }

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
    preview_ready: Res<super::preview_readiness::WorldPreviewReady>,
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
    mut preview_tex: ResMut<WorldPreviewTexture>,
    mut target: ResMut<super::preview_render_contract::PreviewRenderTarget>,
    mut dbg: ResMut<super::preview_render_contract::PreviewPresentationDebug>,
    mut authority: ResMut<super::preview_render_contract::PreviewPathAuthority>,
    contract: Res<super::render_target_barrier::WorldPreviewRenderViewportContract>,
    bind_barrier: Res<super::render_target_barrier::WorldPreviewRenderTargetBindBarrier>,
    registry: Res<super::render_target_barrier::WorldPreviewRenderTargetRegistry>,
    images: Res<Assets<Image>>,
    mut lifecycle_signals: ResMut<super::preview_lifecycle::WorldPreviewLifecycleSignals>,
) {
    if !preview_ready.0 {
        return;
    }
    if bind_barrier.pending.is_some() || !contract.camera_ready {
        return;
    }
    let committed = registry.committed_image.clone();
    let needs_bind_present = committed != Handle::default() && swap.front != committed;
    if !swap.dirty && !needs_bind_present {
        return;
    }
    if swap.front == Handle::default() || swap.back == Handle::default() {
        if needs_bind_present {
            swap.front = committed.clone();
            preview_tex.texture = swap.front.clone();
            target.image = swap.front.clone();
            if let Some(image) = images.get(&swap.front) {
                target.size = UVec2::new(
                    image.texture_descriptor.size.width,
                    image.texture_descriptor.size.height,
                );
            }
            swap.dirty = false;
            super::preview_lifecycle::note_world_preview_present_committed(&mut lifecycle_signals);
        }
        return;
    }
    if needs_bind_present && swap.back == committed {
        let f = swap.front.clone();
        swap.front = swap.back.clone();
        swap.back = f;
    } else if needs_bind_present {
        swap.front = committed.clone();
    } else {
        let f = swap.front.clone();
        let b = swap.back.clone();
        swap.front = b;
        swap.back = f;
    }
    preview_tex.texture = swap.front.clone();
    target.image = swap.front.clone();
    if let Some(image) = images.get(&swap.front) {
        target.size = UVec2::new(
            image.texture_descriptor.size.width,
            image.texture_descriptor.size.height,
        );
    }
    swap.dirty = false;
    dbg.swap_count = dbg.swap_count.saturating_add(1);
    dbg.last_front_asset_id_bits =
        super::preview_render_contract::preview_image_asset_id_bits(&swap.front);
    dbg.last_back_asset_id_bits =
        super::preview_render_contract::preview_image_asset_id_bits(&swap.back);
    authority.gpu_present_count = dbg.swap_count;
    dbg.authoritative_surface = super::preview_render_contract::PreviewAuthoritativeSurface::GpuRenderTarget;
    super::preview_lifecycle::note_world_preview_raster_wrote(&mut lifecycle_signals);
    super::preview_lifecycle::note_world_preview_present_committed(&mut lifecycle_signals);
}

fn preview_image_extent(images: &Assets<Image>, handle: &Handle<Image>) -> Option<UVec2> {
    images.get(handle).map(|image| {
        let size = image.texture_descriptor.size;
        UVec2::new(size.width, size.height)
    })
}

/// Stage C — allocate GPU swap images one frame after resize intent was queued.
pub fn apply_world_preview_gpu_resize_request(
    cfg: Res<DebugRenderTraceConfig>,
    frame: Res<FrameCount>,
    path: Res<PreviewPathAuthority>,
    mut queue: ResMut<WorldPreviewGpuResizeQueue>,
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
    mut preview_tex: ResMut<WorldPreviewTexture>,
    mut images: ResMut<Assets<Image>>,
    mut bind_barrier: ResMut<WorldPreviewRenderTargetBindBarrier>,
    mut registry: ResMut<WorldPreviewRenderTargetRegistry>,
) {
    if path.authoritative_surface != PreviewAuthoritativeSurface::GpuRenderTarget {
        return;
    }
    let Some(extent) = queue.requested_extent else {
        return;
    };
    if frame.0 <= queue.frame_requested {
        return;
    }
    let needs_resize = preview_image_extent(&images, &swap.back)
        .map(|current| current != extent)
        .unwrap_or(true);
    if !needs_resize {
        queue.requested_extent = None;
        return;
    }

    let old_front = swap.front.clone();
    let old_back = swap.back.clone();
    let new_front = images.add(rgba_preview_image(extent.x, extent.y));
    let new_back = images.add(rgba_preview_image(extent.x, extent.y));
    swap.front = new_front;
    swap.back = new_back;
    swap.dirty = true;
    preview_tex.texture = swap.front.clone();
    preview_tex.width = extent.x;
    preview_tex.height = extent.y;
    if old_front != Handle::default() {
        let _ = images.remove(old_front.id());
    }
    if old_back != Handle::default() {
        let _ = images.remove(old_back.id());
    }
    // New images exist in `Assets` immediately; promote them so `registry` never keeps removed
    // `committed_image` handles (which would make `stale_texture_binding` latch until deferred commit).
    bind_barrier.pending = None;
    bind_barrier.bound = swap.back.clone();
    registry.committed_image = swap.back.clone();
    registry.committed_size = extent;
    registry.revision = if registry.revision == 0 {
        1
    } else {
        registry.revision.wrapping_add(1)
    };
    queue.requested_extent = None;
    if cfg.render_target_trace {
        trace_render_target(
            &cfg,
            &format!(
                "gpu_swap_resize extent={}x{} swap_dirty={} front_id={:?}",
                extent.x,
                extent.y,
                swap.dirty,
                swap.front.id(),
            ),
        );
    }
}

/// **D-3** — After CPU raster writes [`crate::gui::SwapImageBuffers::back`], swap handles so
/// [`WorldPreviewTexture::texture`] is the **front** egui samples (`base_visual_dev01_plan_status` § phase-d).
pub fn present_world_preview_swap_after_raster(
    preview_ready: Res<super::preview_readiness::WorldPreviewReady>,
    mut swap: ResMut<crate::gui::SwapImageBuffers>,
    mut preview_tex: ResMut<WorldPreviewTexture>,
    mut dbg: ResMut<super::preview_render_contract::PreviewPresentationDebug>,
    mut lifecycle_signals: ResMut<super::preview_lifecycle::WorldPreviewLifecycleSignals>,
) {
    if !preview_ready.0 {
        return;
    }
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
    super::preview_lifecycle::note_world_preview_present_committed(&mut lifecycle_signals);
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
