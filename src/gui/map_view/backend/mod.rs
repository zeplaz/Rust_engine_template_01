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

/// CPU minimap raster handle — preserved for **effects / dev** lanes (`SharedCpuRaster` opt-in).
#[must_use]
pub fn minimap_cpu_raster_handle(fallback: &TileWorldFallbackState) -> Handle<Image> {
    if fallback.minimap_image != Handle::default() {
        fallback.minimap_image.clone()
    } else {
        fallback.image.clone()
    }
}

/// True when shell explicitly selects the CPU layered raster (VFX, diagnostics, legacy egui host).
#[inline]
#[must_use]
pub fn minimap_effects_cpu_raster_active(shell: &MinimapShellState) -> bool {
    shell.presentation_source == MinimapPresentationSource::SharedCpuRaster
}

/// Default simulation HUD path: GPU compositor RT, not CPU fallback.
#[inline]
#[must_use]
pub fn minimap_main_display_uses_gpu_compositor(shell: &MinimapShellState) -> bool {
    crate::render::minimap_compositor::minimap_gpu_compositor_runtime_enabled()
        && shell.presentation_source == MinimapPresentationSource::SharedRenderTargetImage
}

/// Best-effort texture for Bevy minimap chrome — GPU RT when composited, else main/CPU terrain.
#[must_use]
pub fn resolve_minimap_bevy_display_handle(
    shell: &MinimapShellState,
    fallback: &TileWorldFallbackState,
    registry: &MinimapRenderTargetRegistry,
    compositor_stamp: u64,
) -> Handle<Image> {
    if !shell.visible || shell.minimized {
        return Handle::default();
    }
    if minimap_main_display_uses_gpu_compositor(shell)
        && registry.committed_image != Handle::default()
        && compositor_stamp > 0
    {
        return registry.committed_image.clone();
    }
    if fallback.image != Handle::default() {
        return fallback.image.clone();
    }
    if fallback.minimap_image != Handle::default() {
        return fallback.minimap_image.clone();
    }
    if shell.presentation_source == MinimapPresentationSource::SharedRenderTargetImage
        && registry.committed_image != Handle::default()
    {
        return registry.committed_image.clone();
    }
    Handle::default()
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

/// Main minimap **display** authority (projection graph, readiness, perf policy).
///
/// - Simulation default → GPU RT when compositor env is on.
/// - [`MinimapPresentationSource::SharedCpuRaster`] → CPU raster (effects lane; not auto-fallback).
#[must_use]
pub fn resolve_minimap_texture_source(
    shell: &MinimapShellState,
    fallback: &TileWorldFallbackState,
    registry: &MinimapRenderTargetRegistry,
) -> MapTextureSource {
    if minimap_effects_cpu_raster_active(shell) {
        return MapTextureSource::SharedCpuRaster(minimap_cpu_raster_handle(fallback));
    }
    if minimap_main_display_uses_gpu_compositor(shell) {
        return MapTextureSource::GpuRenderTarget(registry.committed_image.clone());
    }
    match shell.presentation_source {
        MinimapPresentationSource::SharedRenderTargetImage
            if registry.committed_image != Handle::default() =>
        {
            MapTextureSource::GpuRenderTarget(registry.committed_image.clone())
        }
        MinimapPresentationSource::SharedRenderTargetImage | MinimapPresentationSource::SharedCpuRaster => {
            resolve_minimap_effects_cpu_raster_source(fallback)
        }
    }
}

/// Effects-only entry — same handle as [`resolve_minimap_texture_source`] when CPU raster is active.
#[inline]
#[must_use]
pub fn resolve_minimap_effects_cpu_raster_source(
    fallback: &TileWorldFallbackState,
) -> MapTextureSource {
    MapTextureSource::SharedCpuRaster(minimap_cpu_raster_handle(fallback))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::minimap_gpu_compositor_env_enabled;

    #[test]
    fn bevy_display_falls_back_to_main_terrain_before_compositor_stamp() {
        let shell = MinimapShellState {
            visible: true,
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        let mut images = Assets::<Image>::default();
        let main = images.add(Image::default());
        let fallback = TileWorldFallbackState {
            image: main.clone(),
            ..Default::default()
        };
        let mut registry = MinimapRenderTargetRegistry::default();
        registry.committed_image = images.add(Image::default());
        assert_eq!(
            resolve_minimap_bevy_display_handle(&shell, &fallback, &registry, 0),
            main
        );
    }

    #[test]
    fn main_sim_path_gpu_when_compositor_on() {
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        let fallback = TileWorldFallbackState::default();
        let mut registry = MinimapRenderTargetRegistry::default();
        let mut images = Assets::<Image>::default();
        registry.committed_image = images.add(Image::default());
        if minimap_gpu_compositor_env_enabled() {
            assert!(matches!(
                resolve_minimap_texture_source(&shell, &fallback, &registry),
                MapTextureSource::GpuRenderTarget(_)
            ));
        }
    }

    #[test]
    fn effects_opt_in_cpu_even_when_compositor_on() {
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedCpuRaster,
            ..Default::default()
        };
        let fallback = TileWorldFallbackState::default();
        let registry = MinimapRenderTargetRegistry::default();
        assert!(matches!(
            resolve_minimap_texture_source(&shell, &fallback, &registry),
            MapTextureSource::SharedCpuRaster(_)
        ));
    }
}
