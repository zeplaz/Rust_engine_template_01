//! Minimap GPU render-target registry — separate from world-preview authority (UX-E01 M1).

use bevy::diagnostic::FrameCount;
use bevy::math::UVec2;
use bevy::prelude::*;

#[must_use]
pub fn minimap_rgba_image(width: u32, height: u32) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    };
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("minimap_compositor_rt"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        },
        ..default()
    };
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image.data = Some(vec![0; 4 * width as usize * height as usize]);
    image
}

/// Deferred GPU resize intent from [`ResolvedViewports::minimap_panel`].
#[derive(Resource, Debug, Clone, Default)]
pub struct MinimapGpuResizeQueue {
    pub requested_extent: Option<UVec2>,
    pub frame_requested: u32,
}

/// Committed minimap render-target truth (Bevy `ImageNode` + map-view consumers).
#[derive(Resource, Debug, Clone, Default)]
pub struct MinimapRenderTargetRegistry {
    pub revision: u64,
    pub committed_image: Handle<Image>,
    pub committed_size: UVec2,
}

#[derive(Debug, Clone)]
pub struct PendingMinimapRenderTargetBind {
    pub target: Handle<Image>,
    pub size: UVec2,
    pub frame_requested: u32,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct MinimapRenderTargetBindBarrier {
    pub pending: Option<PendingMinimapRenderTargetBind>,
    pub bound: Handle<Image>,
}

impl MinimapRenderTargetBindBarrier {
    pub fn request_resize(&mut self, target: Handle<Image>, size: UVec2, frame_requested: u32) {
        if target == Handle::default() {
            return;
        }
        self.pending = Some(PendingMinimapRenderTargetBind {
            target,
            size,
            frame_requested,
        });
    }

    pub fn clear(&mut self) {
        self.pending = None;
        self.bound = Handle::default();
    }
}

#[must_use]
pub fn pending_minimap_render_target_bind_ready(
    pending: &PendingMinimapRenderTargetBind,
    frame: &FrameCount,
    images: &Assets<Image>,
) -> bool {
    frame.0 > pending.frame_requested && images.get(&pending.target).is_some()
}

pub fn try_commit_minimap_render_target(
    barrier: &mut MinimapRenderTargetBindBarrier,
    registry: &mut MinimapRenderTargetRegistry,
    frame: &FrameCount,
    images: &Assets<Image>,
) -> bool {
    let Some(pending) = barrier.pending.as_ref() else {
        return false;
    };
    if !pending_minimap_render_target_bind_ready(pending, frame, images) {
        return false;
    }
    let pending = barrier.pending.take().expect("checked above");
    barrier.bound = pending.target.clone();
    registry.committed_image = pending.target;
    registry.committed_size = pending.size;
    registry.revision = registry.revision.wrapping_add(1).max(1);
    true
}

#[must_use]
pub fn committed_minimap_render_target_handle(
    registry: &MinimapRenderTargetRegistry,
    images: &Assets<Image>,
) -> Option<Handle<Image>> {
    if registry.committed_image == Handle::default() {
        return None;
    }
    if images.get(&registry.committed_image).is_some() {
        Some(registry.committed_image.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::minimap_rgba_image;

    #[test]
    fn minimap_commit_waits_until_frame_after_resize_request() {
        let mut images = Assets::<Image>::default();
        let handle = images.add(minimap_rgba_image(64, 64));
        let mut barrier = MinimapRenderTargetBindBarrier::default();
        let mut registry = MinimapRenderTargetRegistry::default();
        barrier.request_resize(handle.clone(), UVec2::new(64, 64), 10);

        let frame = FrameCount(10);
        assert!(!try_commit_minimap_render_target(
            &mut barrier,
            &mut registry,
            &frame,
            &images,
        ));

        let frame = FrameCount(11);
        assert!(try_commit_minimap_render_target(
            &mut barrier,
            &mut registry,
            &frame,
            &images,
        ));
        assert_eq!(registry.committed_image, handle);
        assert_eq!(registry.revision, 1);
    }
}
