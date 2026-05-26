//! Minimap compositor pass — overlay-aware composite into dedicated minimap RT.

use bevy::diagnostic::FrameCount;
use bevy::ecs::system::SystemParam;
use bevy::math::UVec2;
use bevy::prelude::*;

use crate::gui::{MapViewInstances, MinimapPresentationSource, MinimapShellState};
use crate::construction::site_phase_tile_instances::ConstructionPhaseGpuChannel;
use crate::render::{
    EcologyVisualSnapshot, LogisticsVisualSnapshot, MinimapOperationalSnapshot, ResolvedViewports,
    SharedOverlayFieldBuffers, TileWorldFallbackState,
};
use crate::strategic::{ConstructionPhase, CorridorConstructionBook};

use super::composite::{
    sync_minimap_terrain_storage, upload_minimap_heat_textures, MinimapCompositeDispatch,
    MinimapCompositeHeatTextures, MinimapCompositeParamsGpu,
};
use super::diagnostics::{
    composite_fingerprint, minimap_gpu_debug_logging_enabled, MinimapGpuCompositorDiagnostics,
    MinimapGpuDispatchReason, MinimapGpuSkipReason,
};
use super::render_target::{
    try_commit_minimap_render_target, MinimapGpuResizeQueue, MinimapRenderTargetBindBarrier,
    MinimapRenderTargetRegistry,
};

/// Force a composite when visible even if fingerprint unchanged (seconds).
const MINIMAP_GPU_MAX_STALE_SECS: f64 = 2.0;

/// Bundles optional heat sources so `run_minimap_compositor_pass` stays within Bevy param limits.
#[derive(SystemParam)]
pub struct MinimapCompositorHeatSources<'w> {
    pub overlay: Option<Res<'w, SharedOverlayFieldBuffers>>,
    pub logistics: Option<Res<'w, LogisticsVisualSnapshot>>,
    pub construction_book: Option<Res<'w, CorridorConstructionBook>>,
    pub ecology: Option<Res<'w, EcologyVisualSnapshot>>,
    pub operational: Option<Res<'w, MinimapOperationalSnapshot>>,
    pub construction_channel: Option<Res<'w, ConstructionPhaseGpuChannel>>,
    pub replay: Option<Res<'w, crate::systems::sim_frame_delta::CommittedSimReplayRing>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapCompositePath {
    #[default]
    CpuBridge,
    GpuCompute,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct MinimapCompositorState {
    pub stamp: u64,
    pub compositor_revision: u64,
    pub last_overlay_revision: u64,
    pub dual_minimap_present: bool,
    pub extent_match_px: f32,
    pub composite_path: MinimapCompositePath,
    pub logistics_rows: u32,
    pub construction_rows: u32,
    pub ecology_rows: u32,
    pub fow_rows: u32,
    pub ew_rows: u32,
    pub fire_heat_enabled: bool,
    pub logistics_heat_enabled: bool,
    pub construction_heat_enabled: bool,
    pub ecology_heat_enabled: bool,
    pub fow_heat_enabled: bool,
    pub ew_heat_enabled: bool,
    pub units_heat_enabled: bool,
    pub unit_marker_rows: u32,
    pub replay_scrub_enabled: bool,
}

#[must_use]
pub fn minimap_gpu_compositor_env_enabled() -> bool {
    match std::env::var("MINIMAP_GPU_COMPOSITOR").ok().as_deref() {
        None => true,
        Some("0") | Some("false") | Some("FALSE") | Some("no") | Some("NO") => false,
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES") => true,
        _ => true,
    }
}

pub fn queue_minimap_render_target_resize(
    resolved: Res<ResolvedViewports>,
    frame: Res<FrameCount>,
    mut queue: ResMut<MinimapGpuResizeQueue>,
) {
    if !minimap_gpu_compositor_env_enabled() {
        return;
    }
    if !resolved.minimap_panel.valid {
        return;
    }
    let extent = resolved.minimap_panel.physical_extent.max(UVec2::splat(1));
    if queue.requested_extent == Some(extent) {
        return;
    }
    queue.requested_extent = Some(extent);
    queue.frame_requested = frame.0;
}

pub fn apply_minimap_gpu_resize_request(
    frame: Res<FrameCount>,
    mut queue: ResMut<MinimapGpuResizeQueue>,
    mut images: ResMut<Assets<Image>>,
    mut bind_barrier: ResMut<MinimapRenderTargetBindBarrier>,
    registry: Res<MinimapRenderTargetRegistry>,
) {
    if !minimap_gpu_compositor_env_enabled() {
        return;
    }
    let Some(extent) = queue.requested_extent else {
        return;
    };
    if frame.0 <= queue.frame_requested {
        return;
    }

    let needs_alloc = registry.committed_size.max(extent) != extent
        || registry.committed_image == Handle::default()
        || images.get(&registry.committed_image).is_none();

    if !needs_alloc {
        queue.requested_extent = None;
        return;
    }

    let image = images.add(super::render_target::minimap_rgba_image(extent.x, extent.y));
    bind_barrier.request_resize(image, extent, queue.frame_requested);
    queue.requested_extent = None;
}

pub fn commit_minimap_render_target_bind_system(
    frame: Res<FrameCount>,
    images: Res<Assets<Image>>,
    mut bind_barrier: ResMut<MinimapRenderTargetBindBarrier>,
    mut registry: ResMut<MinimapRenderTargetRegistry>,
) {
    if !minimap_gpu_compositor_env_enabled() {
        return;
    }
    let _ = try_commit_minimap_render_target(
        bind_barrier.as_mut(),
        registry.as_mut(),
        &frame,
        &images,
    );
}

pub fn sync_minimap_presentation_source(
    mut shell: ResMut<MinimapShellState>,
    registry: Res<MinimapRenderTargetRegistry>,
) {
    if !minimap_gpu_compositor_env_enabled() {
        return;
    }
    if registry.committed_image != Handle::default() && registry.revision > 0 {
        shell.presentation_source = MinimapPresentationSource::SharedRenderTargetImage;
    }
}

pub fn run_minimap_compositor_pass(
    mut cadence: Local<f32>,
    time: Res<Time>,
    mut compositor: ResMut<MinimapCompositorState>,
    mut diagnostics: ResMut<MinimapGpuCompositorDiagnostics>,
    registry: Res<MinimapRenderTargetRegistry>,
    fallback: Res<TileWorldFallbackState>,
    raster_dirty: Option<Res<crate::render::TileWorldFallbackRasterDirty>>,
    heat_sources: MinimapCompositorHeatSources,
    map_views: Res<MapViewInstances>,
    resolved: Res<ResolvedViewports>,
    mut dispatch: ResMut<MinimapCompositeDispatch>,
    mut heat: ResMut<MinimapCompositeHeatTextures>,
    mut shell: ResMut<MinimapShellState>,
    mut images: ResMut<Assets<Image>>,
    visual_cadence: Res<crate::gui::VisualCadence>,
) {
    let now_secs = time.elapsed_secs_f64();
    diagnostics.refresh_budget_verdict(now_secs, visual_cadence.minimap_hz);

    if !minimap_gpu_compositor_env_enabled() {
        compositor.dual_minimap_present = false;
        compositor.composite_path = MinimapCompositePath::CpuBridge;
        dispatch.commit_stamp = 0;
        return;
    }
    if !shell.visible || shell.minimized {
        compositor.dual_minimap_present = false;
        diagnostics.record_skip(MinimapGpuSkipReason::ShellHidden);
        dispatch.commit_stamp = 0;
        return;
    }
    if registry.committed_image == Handle::default() {
        diagnostics.record_skip(MinimapGpuSkipReason::NoRenderTarget);
        dispatch.commit_stamp = 0;
        return;
    }

    *cadence += time.delta_secs();
    let hz = visual_cadence.minimap_hz.max(1.0);
    if *cadence < 1.0 / hz {
        return;
    }
    *cadence = 0.0;

    let terrain = if fallback.minimap_image != Handle::default() {
        fallback.minimap_image.clone()
    } else {
        fallback.image.clone()
    };
    if terrain == Handle::default() {
        dispatch.commit_stamp = 0;
        diagnostics.record_skip(MinimapGpuSkipReason::NoTerrain);
        return;
    }

    let overlay_revision = heat_sources
        .overlay
        .as_ref()
        .map(|o| o.revision)
        .unwrap_or(0);
    let logistics_rows_hint = heat_sources
        .logistics
        .as_ref()
        .map(|l| l.active_overlay_rows)
        .unwrap_or(0);
    let construction_rows_hint = heat_sources
        .construction_book
        .as_ref()
        .map(|b| {
            b.rows
                .values()
                .filter(|r| r.phase != ConstructionPhase::Completed)
                .count() as u32
        })
        .unwrap_or(0)
        .max(
            heat_sources
                .construction_channel
                .as_ref()
                .map(|c| c.instance_count)
                .unwrap_or(0),
        );
    let ecology_rows_hint = heat_sources
        .ecology
        .as_ref()
        .map(|e| e.ecology_chunk_count.max(e.chunk_rows.len() as u32))
        .unwrap_or(0);
    let fallback_revision = raster_dirty.as_ref().map(|r| r.revision()).unwrap_or(0);
    let overlays = map_views.minimap.overlays;
    let fingerprint = composite_fingerprint(
        &terrain,
        overlay_revision,
        logistics_rows_hint,
        construction_rows_hint,
        ecology_rows_hint,
        registry.revision,
        fallback_revision,
        overlays.fire_heat,
        overlays.logistics_heat,
        overlays.construction_heat,
        overlays.ecology_heat,
    );

    let dispatch_reason = if diagnostics.last_fingerprint == 0 {
        MinimapGpuDispatchReason::Initial
    } else if fingerprint != diagnostics.last_fingerprint {
        if registry.revision != compositor.compositor_revision {
            MinimapGpuDispatchReason::RtResize
        } else if overlay_revision != compositor.last_overlay_revision {
            MinimapGpuDispatchReason::OverlayChanged
        } else if logistics_rows_hint != compositor.logistics_rows
            || construction_rows_hint != compositor.construction_rows
            || ecology_rows_hint != compositor.ecology_rows
        {
            MinimapGpuDispatchReason::LogisticsChanged
        } else if overlays.fire_heat != compositor.fire_heat_enabled
            || overlays.logistics_heat != compositor.logistics_heat_enabled
            || overlays.construction_heat != compositor.construction_heat_enabled
            || overlays.ecology_heat != compositor.ecology_heat_enabled
        {
            MinimapGpuDispatchReason::ToggleChanged
        } else {
            MinimapGpuDispatchReason::TerrainChanged
        }
    } else if now_secs - diagnostics.last_commit_at_secs >= MINIMAP_GPU_MAX_STALE_SECS {
        MinimapGpuDispatchReason::StaleRefresh
    } else {
        diagnostics.record_skip(MinimapGpuSkipReason::NoChange);
        dispatch.commit_stamp = 0;
        return;
    };

    let max_commits_per_sec = hz * 1.25;
    let window_elapsed = (now_secs - diagnostics.window_start_secs).max(1e-3);
    let effective_hz = diagnostics.window_commits as f32 / window_elapsed as f32;
    if diagnostics.window_commits > 0 && effective_hz > max_commits_per_sec {
        diagnostics.record_skip(MinimapGpuSkipReason::RateCapped);
        if minimap_gpu_debug_logging_enabled() {
            warn!(
                "minimap GPU rate cap — effective_hz={effective_hz:.1} max={max_commits_per_sec:.1}"
            );
        }
        return;
    }

    let extent = registry.committed_size.max(UVec2::splat(1));
    if !sync_minimap_terrain_storage(&mut images, &mut heat, &terrain, extent) {
        dispatch.commit_stamp = 0;
        diagnostics.record_skip(MinimapGpuSkipReason::UploadFailed);
        return;
    }
    let (
        upload_ok,
        logistics_rows,
        construction_rows,
        ecology_rows,
        fow_rows,
        ew_rows,
        unit_marker_rows,
        replay_scrub_enabled,
    ) = upload_minimap_heat_textures(
        &mut images,
        &mut heat,
        heat_sources.overlay.as_deref(),
        heat_sources.logistics.as_deref(),
        heat_sources.construction_book.as_deref(),
        heat_sources.ecology.as_deref(),
        heat_sources.operational.as_deref(),
        heat_sources.construction_channel.as_deref(),
        heat_sources.replay.as_deref(),
        &map_views,
        &fallback,
        extent,
    );
    if !upload_ok {
        dispatch.commit_stamp = 0;
        diagnostics.record_skip(MinimapGpuSkipReason::UploadFailed);
        return;
    }

    compositor.stamp = compositor.stamp.wrapping_add(1);
    dispatch.terrain = terrain;
    dispatch.output = registry.committed_image.clone();
    dispatch.params = MinimapCompositeParamsGpu {
        fire_heat_enabled: overlays.fire_heat as u32,
        logistics_heat_enabled: overlays.logistics_heat as u32,
        construction_heat_enabled: overlays.construction_heat as u32,
        ecology_heat_enabled: overlays.ecology_heat as u32,
        fow_heat_enabled: overlays.fow as u32,
        ew_heat_enabled: overlays.ew as u32,
        overlay_revision,
        logistics_rows,
        construction_rows,
        ecology_rows,
        fow_rows,
        ew_rows,
    };
    dispatch.commit_stamp = compositor.stamp;

    diagnostics.record_commit(dispatch_reason, fingerprint, compositor.stamp, now_secs);
    if minimap_gpu_debug_logging_enabled() {
        info!(
            "minimap GPU commit stamp={} reason={:?} fp={fingerprint:#x} hz_target={hz:.1}",
            compositor.stamp, dispatch_reason
        );
    }

    compositor.compositor_revision = registry.revision;
    compositor.last_overlay_revision = overlay_revision;
    compositor.logistics_rows = logistics_rows;
    compositor.construction_rows = construction_rows;
    compositor.ecology_rows = ecology_rows;
    compositor.fow_rows = fow_rows;
    compositor.ew_rows = ew_rows;
    compositor.fire_heat_enabled = overlays.fire_heat;
    compositor.logistics_heat_enabled = overlays.logistics_heat;
    compositor.construction_heat_enabled = overlays.construction_heat;
    compositor.ecology_heat_enabled = overlays.ecology_heat;
    compositor.fow_heat_enabled = overlays.fow;
    compositor.ew_heat_enabled = overlays.ew;
    compositor.units_heat_enabled = overlays.units;
    compositor.unit_marker_rows = unit_marker_rows;
    compositor.replay_scrub_enabled = replay_scrub_enabled;
    compositor.composite_path = MinimapCompositePath::GpuCompute;
    shell.compositor_revision = compositor.compositor_revision;
    shell.cached_texture_revision = compositor.stamp;

    if resolved.minimap_panel.valid {
        let panel = resolved.minimap_panel.physical_extent;
        let reg = registry.committed_size;
        compositor.extent_match_px = (panel.x as f32 - reg.x as f32).abs()
            + (panel.y as f32 - reg.y as f32).abs();
    }
    compositor.dual_minimap_present = false;
}
