//! Backend texture source selection for map-view consumers.

use bevy::prelude::*;

use crate::gui::editor::world_preview::{
    PreviewAuthoritativeSurface, PreviewPathAuthority, WorldPreviewRenderTargetRegistry,
    WorldPreviewTexture,
};
use crate::gui::{MinimapPresentationSource, MinimapShellState};
use crate::render::MinimapRenderTargetRegistry;
use crate::render::TileWorldFallbackState;

/// Authoritative pixel source for a map consumer (resolved by the backend, not egui).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapTextureSource {
    GpuRenderTarget(Handle<Image>),
    SharedCpuRaster(Handle<Image>),
}

impl MapTextureSource {
    #[must_use]
    pub fn handle(&self) -> &Handle<Image> {
        match self {
            Self::GpuRenderTarget(handle) | Self::SharedCpuRaster(handle) => handle,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::GpuRenderTarget(_) => "GpuRenderTarget",
            Self::SharedCpuRaster(_) => "SharedCpuRaster",
        }
    }
}

impl Default for MapTextureSource {
    fn default() -> Self {
        Self::SharedCpuRaster(Handle::default())
    }
}

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

#[must_use]
pub fn resolve_minimap_texture_source(
    shell: &MinimapShellState,
    fallback: &TileWorldFallbackState,
    registry: &MinimapRenderTargetRegistry,
) -> MapTextureSource {
    match shell.presentation_source {
        MinimapPresentationSource::SharedRenderTargetImage
            if registry.committed_image != Handle::default() =>
        {
            MapTextureSource::GpuRenderTarget(registry.committed_image.clone())
        }
        MinimapPresentationSource::SharedRenderTargetImage | MinimapPresentationSource::SharedCpuRaster => {
            let handle = if fallback.minimap_image != Handle::default() {
                fallback.minimap_image.clone()
            } else {
                fallback.image.clone()
            };
            MapTextureSource::SharedCpuRaster(handle)
        }
    }
}
