//! Minimap Bevy chrome display handle + texel extent authority.

use bevy::prelude::*;

use crate::gui::{MinimapPresentationSource, MinimapShellState};
use crate::render::MinimapRenderTargetRegistry;
use crate::render::TileWorldFallbackState;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

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

/// Pixel extent of the image [`resolve_minimap_bevy_display_handle`] binds (GPU RT vs world raster).
///
/// Bevy `ImageNode.rect` must stay within this size — using world `320×320` on a `260×220` GPU RT
/// samples outside the texture and shows black bars on the right/bottom.
#[must_use]
pub fn minimap_bevy_display_texel_extent(
    shell: &MinimapShellState,
    registry: &MinimapRenderTargetRegistry,
    compositor_stamp: u64,
    world_params: &WorldGenParams,
) -> UVec2 {
    if minimap_main_display_uses_gpu_compositor(shell)
        && registry.committed_image != Handle::default()
        && compositor_stamp > 0
    {
        registry.committed_size.max(UVec2::ONE)
    } else {
        UVec2::new(world_params.width.max(1), world_params.height.max(1))
    }
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
