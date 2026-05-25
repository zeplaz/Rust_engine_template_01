//! CPU world-preview raster: avoid wiping the swap buffer when the viewport contract is invalid.

use bevy::prelude::*;

/// Tracks when the preview raster intentionally **skipped** a full clear because
/// [`crate::render::ResolvedViewports::world_preview`] was not yet valid (holds last texels).
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct PreviewRenderState {
    pub held_last_raster_due_to_invalid_viewport: bool,
}
