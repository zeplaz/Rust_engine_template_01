//! Viewport + render-target lifecycle: UI intent → deferred GPU allocation → committed bind.
//!
//! UI writes desired extent only. GPU swap images are allocated on a later frame. The preview
//! offscreen camera binds [`RenderTarget::Image`] only after the handle is registered.

use bevy::diagnostic::FrameCount;
use bevy::math::UVec2;
use bevy::prelude::*;

/// Lifecycle signals for preview viewport / render-target changes.
#[derive(Message, Debug, Clone)]
pub enum WorldPreviewViewportEvent {
    ResizeRequested {
        size: UVec2,
        frame_requested: u32,
    },
    ResizeCommitted {
        size: UVec2,
        image: Handle<Image>,
        revision: u64,
    },
}

/// Deferred GPU resize intent from [`super::viewport_authority::WorldPreviewViewportAuthority`].
#[derive(Resource, Debug, Clone, Default)]
pub struct WorldPreviewGpuResizeQueue {
    pub requested_extent: Option<UVec2>,
    pub frame_requested: u32,
}

/// Committed GPU render-target truth (camera + presentation consumers read this).
#[derive(Resource, Debug, Clone, Default)]
pub struct WorldPreviewRenderTargetRegistry {
    pub revision: u64,
    pub committed_image: Handle<Image>,
    pub committed_size: UVec2,
}

/// GPU-safe render viewport snapshot for consumers after bind.
#[derive(Resource, Debug, Clone, Default)]
pub struct WorldPreviewRenderViewportContract {
    pub size: UVec2,
    pub render_target: Handle<Image>,
    pub camera_ready: bool,
    pub version: u64,
}

pub fn sync_world_preview_render_viewport_contract(
    registry: Res<WorldPreviewRenderTargetRegistry>,
    images: Res<Assets<Image>>,
    mut contract: ResMut<WorldPreviewRenderViewportContract>,
) {
    contract.size = registry.committed_size;
    contract.render_target = registry.committed_image.clone();
    contract.version = registry.revision;
    contract.camera_ready = registry.committed_image != Handle::default()
        && images.get(&registry.committed_image).is_some();
}

/// Pending offscreen render target after swap-buffer resize.
#[derive(Debug, Clone)]
pub struct PendingRenderTargetBind {
    pub target: Handle<Image>,
    pub size: UVec2,
    pub frame_requested: u32,
}

/// Last successfully bound target plus any in-flight resize request.
#[derive(Resource, Debug, Clone, Default)]
pub struct WorldPreviewRenderTargetBindBarrier {
    pub pending: Option<PendingRenderTargetBind>,
    pub bound: Handle<Image>,
}

impl WorldPreviewRenderTargetBindBarrier {
    pub fn request_resize(&mut self, target: Handle<Image>, size: UVec2, frame_requested: u32) {
        if target == Handle::default() {
            return;
        }
        self.pending = Some(PendingRenderTargetBind {
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

/// True when the pending bind may be committed (strictly after the request frame).
#[must_use]
pub fn pending_render_target_bind_ready(
    pending: &PendingRenderTargetBind,
    frame: &FrameCount,
    images: &Assets<Image>,
) -> bool {
    frame.0 > pending.frame_requested && images.get(&pending.target).is_some()
}

/// Promote a ready pending bind into the committed registry.
pub fn try_commit_world_preview_render_target(
    barrier: &mut WorldPreviewRenderTargetBindBarrier,
    registry: &mut WorldPreviewRenderTargetRegistry,
    frame: &FrameCount,
    images: &Assets<Image>,
) -> Option<WorldPreviewViewportEvent> {
    let pending = barrier.pending.as_ref()?;
    if !pending_render_target_bind_ready(pending, frame, images) {
        return None;
    }
    let pending = barrier.pending.take().expect("checked above");
    barrier.bound = pending.target.clone();
    registry.committed_image = pending.target;
    registry.committed_size = pending.size;
    registry.revision = registry.revision.wrapping_add(1);
    Some(WorldPreviewViewportEvent::ResizeCommitted {
        size: registry.committed_size,
        image: registry.committed_image.clone(),
        revision: registry.revision,
    })
}

/// Handle the preview GPU camera may bind this frame (committed registry only).
#[must_use]
pub fn committed_render_target_handle(
    registry: &WorldPreviewRenderTargetRegistry,
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

    use super::super::texture_cache::rgba_preview_image;

    #[test]
    fn commit_waits_until_frame_after_resize_request() {
        let mut images = Assets::<Image>::default();
        let handle = images.add(rgba_preview_image(64, 64));
        let mut barrier = WorldPreviewRenderTargetBindBarrier::default();
        let mut registry = WorldPreviewRenderTargetRegistry::default();
        barrier.request_resize(handle.clone(), UVec2::new(64, 64), 10);

        let frame = FrameCount(10);
        assert!(try_commit_world_preview_render_target(
            &mut barrier,
            &mut registry,
            &frame,
            &images,
        )
        .is_none());

        let frame = FrameCount(11);
        assert!(try_commit_world_preview_render_target(
            &mut barrier,
            &mut registry,
            &frame,
            &images,
        )
        .is_some());
        assert_eq!(registry.committed_image, handle);
        assert_eq!(registry.revision, 1);
    }
}
