//! World-preview texture source resolution.

use bevy::prelude::*;

use crate::gui::editor::world_preview::{
    PreviewAuthoritativeSurface, PreviewPathAuthority, WorldPreviewRenderTargetRegistry,
    WorldPreviewTexture,
};

use super::types::MapTextureSource;

#[must_use]
pub fn resolve_world_preview_texture_source(
    path: &PreviewPathAuthority,
    registry: &WorldPreviewRenderTargetRegistry,
    preview_tex: &WorldPreviewTexture,
) -> MapTextureSource {
    if path.authoritative_surface == PreviewAuthoritativeSurface::GpuRenderTarget
        && registry.committed_image != Handle::default()
    {
        MapTextureSource::GpuRenderTarget(registry.committed_image.clone())
    } else {
        MapTextureSource::SharedCpuRaster(preview_tex.texture.clone())
    }
}
