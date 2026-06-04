//! Projection sampling into per-consumer [`ResolvedMapViewFrame`] values.

use bevy::math::{Rect, Vec2};
use bevy::prelude::*;

use super::backend::{
    resolve_minimap_texture_source, resolve_world_preview_texture_source, MapTextureSource,
};
use super::presentation::MapViewInstanceId;
use super::resolved::ResolvedMapViewFrame;
use crate::gui::editor::world_preview::{
    PreviewPathAuthority, WorldPreviewRenderTargetRegistry, WorldPreviewTexture,
};
use crate::render::MinimapRenderTargetRegistry;
use crate::gui::view_representation_snapshot::ViewRepresentationSnapshot;
use crate::gui::{MapPresentationDiagnostics, MinimapShellState};
use crate::render::{
    ResolvedViewports, SharedOverlayFieldBuffers, TileWorldFallbackRasterDirty,
    TileWorldFallbackState,
};

#[derive(Resource, Clone, Debug, Default)]
pub struct ResolvedMapViewFrames {
    pub world_preview: ResolvedMapViewFrame,
    pub minimap: ResolvedMapViewFrame,
    /// In-game / auxiliary map surfaces (simulation HUD map, tactical, etc.): not world preview.
    pub simulation_map: ResolvedMapViewFrame,
}

impl ResolvedMapViewFrames {
    #[must_use]
    pub fn get(&self, id: MapViewInstanceId) -> &ResolvedMapViewFrame {
        match id {
            MapViewInstanceId::WorldPreview => &self.world_preview,
            MapViewInstanceId::Minimap => &self.minimap,
            MapViewInstanceId::SimulationMap => &self.simulation_map,
            // Until each surface has its own resolved frame, share the simulation-map contract
            // (never alias [`world_preview`], which used to bleed preview extent/texture here).
            MapViewInstanceId::TacticalMap
            | MapViewInstanceId::FullscreenMap
            | MapViewInstanceId::CommanderMap
            | MapViewInstanceId::Stage7IntelMap => &self.simulation_map,
        }
    }
}

fn world_bounds_from_snapshot(snapshot: &ViewRepresentationSnapshot) -> Rect {
    let bounds = &snapshot.particle_bounds;
    if bounds.min.is_finite() && bounds.max.is_finite() && bounds.min != bounds.max {
        return Rect::from_corners(bounds.min, bounds.max);
    }
    snapshot.viewport
}

pub fn sync_resolved_map_view_frames(
    resolved: Res<ResolvedViewports>,
    preview_registry: Res<WorldPreviewRenderTargetRegistry>,
    minimap_registry: Res<MinimapRenderTargetRegistry>,
    path: Res<PreviewPathAuthority>,
    preview_tex: Res<WorldPreviewTexture>,
    minimap: Res<MinimapShellState>,
    fallback: Res<TileWorldFallbackState>,
    raster_dirty: Res<TileWorldFallbackRasterDirty>,
    overlay: Option<Res<SharedOverlayFieldBuffers>>,
    snapshot: Res<ViewRepresentationSnapshot>,
    mut frames: ResMut<ResolvedMapViewFrames>,
    mut map_pres: Option<ResMut<MapPresentationDiagnostics>>,
    update_attrib: Option<ResMut<crate::render::FrameUpdateAttrib>>,
) {
    let t0 = std::time::Instant::now();
    let overlay_revision = overlay.as_ref().map(|o| o.revision).unwrap_or(0);
    let world_bounds = world_bounds_from_snapshot(&snapshot);

    let preview_extent = if resolved.world_preview.valid {
        resolved.world_preview.physical_extent
    } else {
        UVec2::ONE
    };
    let preview_source = resolve_world_preview_texture_source(&path, &preview_registry, &preview_tex);
    // Do not fold `resolved.revision` (primary window / minimap churn) into world-preview
    // revision — that forces egui texture rebinds and flicker while the preview handle is stable.
    // Tie identity to render-target commits, panel extent, and GPU present ticks only.
    let preview_projection = preview_registry
        .revision
        .wrapping_mul(1_000_003)
        .wrapping_add(path.gpu_present_count as u64)
        .wrapping_add(preview_extent.x as u64)
        .wrapping_add((preview_extent.y as u64) << 32);

    frames.world_preview = ResolvedMapViewFrame {
        projection_revision: preview_projection,
        texture_source: preview_source,
        viewport_extent: preview_extent,
        overlay_revision,
        world_bounds,
    };

    let minimap_extent = if resolved.minimap_panel.valid {
        resolved.minimap_panel.physical_extent
    } else {
        UVec2::new(fallback.last_w.max(1), fallback.last_h.max(1))
    };
    let minimap_source = resolve_minimap_texture_source(&minimap, &fallback, &minimap_registry);
    let compositor_revision = minimap_registry.revision;
    let minimap_projection = compositor_revision
        .wrapping_mul(1_000_003)
        .wrapping_add(raster_dirty.revision())
        .wrapping_add(overlay_revision)
        .wrapping_add(minimap_extent.x as u64)
        .wrapping_add((minimap_extent.y as u64) << 32);

    frames.minimap = ResolvedMapViewFrame {
        projection_revision: minimap_projection,
        texture_source: minimap_source,
        viewport_extent: minimap_extent,
        overlay_revision,
        world_bounds: Rect::from_corners(
            Vec2::ZERO,
            Vec2::new(fallback.last_w.max(1) as f32, fallback.last_h.max(1) as f32),
        ),
    };

    let sim_extent = if resolved.simulation_map.valid {
        resolved.simulation_map.physical_extent
    } else {
        resolved
            .primary_window
            .physical_extent
            .max(UVec2::splat(1))
    };
    let sim_source = MapTextureSource::SharedCpuRaster(fallback.image.clone());
    let sim_projection = raster_dirty
        .revision()
        .wrapping_mul(1_000_003)
        .wrapping_add(preview_registry.revision)
        .wrapping_add(sim_extent.x as u64)
        .wrapping_add((sim_extent.y as u64) << 32);

    frames.simulation_map = ResolvedMapViewFrame {
        projection_revision: sim_projection,
        texture_source: sim_source,
        viewport_extent: sim_extent,
        overlay_revision,
        world_bounds,
    };

    if let Some(map_pres) = map_pres.as_mut() {
        map_pres.world_preview.viewport_extent = frames.world_preview.viewport_extent;
        map_pres.minimap.viewport_extent = frames.minimap.viewport_extent;
    }

    crate::render::record_viewport_sync_ms(
        update_attrib,
        t0.elapsed().as_secs_f32() * 1000.0,
    );
}
