//! Minimap texture source resolution for projection / compositor paths.

use bevy::prelude::*;

use crate::gui::{MinimapPresentationSource, MinimapShellState};
use crate::render::MinimapRenderTargetRegistry;
use crate::render::TileWorldFallbackState;

use super::display::{
    minimap_cpu_raster_handle, minimap_effects_cpu_raster_active,
    minimap_main_display_uses_gpu_compositor,
};
use super::types::MapTextureSource;

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
