//! Resolved viewport contracts for render + particle consumers.
//!
//! **Single source of truth:** preview and minimap layout gates read [`ResolvedViewports`] only
//! (logical size, physical extent, half-extents, `valid`). Do not duplicate projection sizing in
//! GUI-only structs; push requests through [`ViewportAuthority`] and resolve here.

use bevy::log::info;
use bevy::math::{UVec2, Vec2};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

use crate::gui::{
    MinimapShellState, PreviewAuthoritativeSurface, PreviewCameraState, PreviewPathAuthority,
    PreviewRenderMode, SimulationMapViewport, ViewportAuthority,
    ViewRepresentationSystemSet, VIEWPORT_PRIORITY_PREVIEW,
};

const LAYOUT_EPS: f32 = 0.5;
/// Minimum logical px for the simulation map hole — smaller reads as a layout defect.
const SIM_MAP_MIN_LOGICAL_EXTENT: f32 = 8.0;

/// True when the sim-map UI hole is invalid or spills outside the primary window (not “smaller than fullscreen”).
#[must_use]
fn simulation_map_viewport_defect(sim: &SimulationMapViewport, primary_logical: Vec2) -> bool {
    if !sim.valid {
        return false;
    }
    let w = (sim.max.x - sim.min.x).max(0.0);
    let h = (sim.max.y - sim.min.y).max(0.0);
    if w < SIM_MAP_MIN_LOGICAL_EXTENT || h < SIM_MAP_MIN_LOGICAL_EXTENT {
        return true;
    }
    sim.min.x < -2.0
        || sim.min.y < -2.0
        || sim.max.x > primary_logical.x + 2.0
        || sim.max.y > primary_logical.y + 2.0
}

#[derive(Resource, Default)]
struct ViewportResolveLogCache {
    last: HashMap<&'static str, (Vec2, UVec2)>,
}

/// Committed viewport contract for one consumer surface.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedViewport {
    pub logical_size: Vec2,
    pub physical_extent: UVec2,
    pub world_extent: UVec2,
    pub half_extents: Vec2,
    pub valid: bool,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct ResolvedViewports {
    pub world_preview: ResolvedViewport,
    pub minimap_panel: ResolvedViewport,
    pub simulation_map: ResolvedViewport,
    pub primary_window: ResolvedViewport,
    pub revision: u64,
}

/// Read-only mismatch flags for dev / VT-4 consumers.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct ViewportPresentationMismatch {
    pub world_preview_extent_mismatch: bool,
    pub minimap_panel_extent_mismatch: bool,
    pub simulation_map_extent_mismatch: bool,
    pub stale_texture_binding: bool,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViewportPipelineSet {
    Resolve,
}

pub struct ViewportPipelinePlugin;

impl Plugin for ViewportPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResolvedViewports>()
            .init_resource::<ViewportPresentationMismatch>()
            .init_resource::<ViewportResolveLogCache>()
            .configure_sets(
                Update,
                ViewportPipelineSet::Resolve.in_set(ViewRepresentationSystemSet::ResolveViewport),
            )
            .add_systems(
                Update,
                (
                    resolve_preview_viewport_requests,
                    resolve_primary_and_simulation_viewports,
                    resolve_minimap_panel_viewport,
                    crate::render::view_runtime::commit_resolved_viewports_to_authority,
                    crate::render::view_runtime::sync_resolved_viewports_from_authority,
                    crate::render::view_runtime::apply_map_view_extents_from_authority,
                    clear_viewport_requests_after_resolve,
                )
                    .chain()
                    .in_set(ViewportPipelineSet::Resolve),
            );
    }
}

fn resolve_preview_viewport_requests(
    pending_layout: Res<crate::gui::hud::PendingHudLayoutCommit>,
    mut authority: ResMut<ViewportAuthority>,
    windows: Query<&Window, With<PrimaryWindow>>,
    path: Res<PreviewPathAuthority>,
    preview_cam: Res<PreviewCameraState>,
    mut resolved: ResMut<ResolvedViewports>,
    mut mismatch: ResMut<ViewportPresentationMismatch>,
    mut log_cache: ResMut<ViewportResolveLogCache>,
) {
    if pending_layout.drag_active {
        mismatch.world_preview_extent_mismatch = false;
        return;
    }
    let Some(req) = authority
        .pending
        .iter()
        .filter(|request| request.priority == VIEWPORT_PRIORITY_PREVIEW)
        .max_by(|left, right| {
            left.logical_rect
                .area()
                .partial_cmp(&right.logical_rect.area())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
    else {
        // No preview request this frame: **do not** clear `world_preview` to default.
        // Empty `pending` is normal between UI submissions; clearing caused `projection_ready`
        // to flap false and `wp_half_x=0` in readiness logs while the surface was still valid.
        mismatch.world_preview_extent_mismatch = false;
        authority.resolved = None;
        return;
    };

    let logical = clamp_viewport_request(req.logical_rect, &windows);
    let logical_size = Vec2::new(logical.width().max(1.0), logical.height().max(1.0));
    let gpu = path.authoritative_surface == PreviewAuthoritativeSurface::GpuRenderTarget
        && preview_cam.mode == PreviewRenderMode::GpuRenderTarget;
    let physical = if gpu {
        UVec2::new(
            logical_size.x.round().max(1.0) as u32,
            logical_size.y.round().max(1.0) as u32,
        )
    } else if req.world_extent == UVec2::ZERO {
        UVec2::new(
            logical_size.x.round().max(1.0) as u32,
            logical_size.y.round().max(1.0) as u32,
        )
    } else {
        req.world_extent
    };

    let world_extent = if req.world_extent == UVec2::ZERO {
        physical
    } else {
        req.world_extent
    };
    let next_preview = ResolvedViewport {
        logical_size,
        physical_extent: physical,
        world_extent,
        half_extents: Vec2::new(logical_size.x * 0.5, logical_size.y * 0.5),
        valid: true,
    };
    let preview_changed = resolved.world_preview != next_preview;
    resolved.world_preview = next_preview;
    mismatch.world_preview_extent_mismatch = gpu
        && physical != UVec2::ZERO
        && physical
            != UVec2::new(
                logical_size.x.round().max(1.0) as u32,
                logical_size.y.round().max(1.0) as u32,
            );

    authority.requested = Some(req);
    authority.resolved = Some(crate::gui::ResolvedViewport {
        logical,
        physical,
        world_extent: resolved.world_preview.world_extent,
    });
    if preview_changed {
        authority.revision = authority.revision.wrapping_add(1);
        bump_resolved_revision(&mut resolved);
    }
    log_resolved_viewport(
        "world_preview",
        &resolved.world_preview,
        resolved.revision,
        &mut log_cache,
    );
}

fn resolve_primary_and_simulation_viewports(
    windows: Query<&Window, With<PrimaryWindow>>,
    sim_map: Res<SimulationMapViewport>,
    authority: Res<crate::render::view_runtime::ViewProjectionAuthority>,
    mut resolved: ResMut<ResolvedViewports>,
    mut mismatch: ResMut<ViewportPresentationMismatch>,
    mut log_cache: ResMut<ViewportResolveLogCache>,
) {
    let mut primary = ResolvedViewport::default();
    if let Ok(win) = windows.single() {
        let w = win.width().max(1.0);
        let h = win.height().max(1.0);
        primary.logical_size = Vec2::new(w, h);
        primary.physical_extent = UVec2::new(w.round() as u32, h.round() as u32);
        primary.world_extent = primary.physical_extent;
        primary.half_extents = Vec2::new(w * 0.5, h * 0.5);
        primary.valid = true;
    }

    let mut sim = authority
        .resolved_viewport(crate::render::view_runtime::ViewSurfaceId::SimulationMap);
    if !sim.valid && sim_map.is_adequate_for_camera() {
        let w = (sim_map.max.x - sim_map.min.x).max(1.0);
        let h = (sim_map.max.y - sim_map.min.y).max(1.0);
        sim.logical_size = Vec2::new(w, h);
        sim.physical_extent = UVec2::new(w.round() as u32, h.round() as u32);
        sim.world_extent = sim.physical_extent;
        sim.half_extents = Vec2::new(w * 0.5, h * 0.5);
        sim.valid = true;
    }

    // Sub-viewport sim map hole is expected (HUD chrome); only flag real defects.
    mismatch.simulation_map_extent_mismatch =
        simulation_map_viewport_defect(sim_map.as_ref(), primary.logical_size);

    let primary_changed = resolved.primary_window != primary;
    let sim_changed = resolved.simulation_map != sim;
    resolved.primary_window = primary;
    resolved.simulation_map = sim;
    if primary_changed || sim_changed {
        bump_resolved_revision(&mut resolved);
    }
    if sim_map.is_adequate_for_camera() {
        crate::gui::hud::trace_viewport_authority(
            crate::gui::hud::ViewportAuthoritySource::ResolvedViewport,
            sim_map.min,
            sim_map.max,
            true,
        );
    }
    log_resolved_viewport(
        "primary_window",
        &resolved.primary_window,
        resolved.revision,
        &mut log_cache,
    );
    log_resolved_viewport(
        "simulation_map",
        &resolved.simulation_map,
        resolved.revision,
        &mut log_cache,
    );
}

fn resolve_minimap_panel_viewport(
    shell: Res<MinimapShellState>,
    pending_layout: Res<crate::gui::hud::PendingHudLayoutCommit>,
    mut resolved: ResMut<ResolvedViewports>,
    mut mismatch: ResMut<ViewportPresentationMismatch>,
    mut log_cache: ResMut<ViewportResolveLogCache>,
) {
    if pending_layout.drag_active {
        mismatch.minimap_panel_extent_mismatch = false;
        return;
    }
    if !shell.panel_viewport_suggestion_active {
        resolved.minimap_panel = ResolvedViewport::default();
        mismatch.minimap_panel_extent_mismatch = false;
        return;
    }

    let mut logical = shell.panel_viewport_suggestion_logical_size;
    logical.x = logical.x.clamp(180.0, 720.0);
    logical.y = logical.y.clamp(160.0, 720.0);
    let physical = UVec2::new(
        logical.x.round().max(1.0) as u32,
        logical.y.round().max(1.0) as u32,
    );

    if resolved.minimap_panel.valid {
        let prev = resolved.minimap_panel.logical_size;
        mismatch.minimap_panel_extent_mismatch =
            (prev - logical).length_squared() > LAYOUT_EPS * LAYOUT_EPS;
    } else {
        mismatch.minimap_panel_extent_mismatch = false;
    }

    let next_minimap = ResolvedViewport {
        logical_size: logical,
        physical_extent: physical,
        world_extent: physical,
        half_extents: Vec2::new(logical.x * 0.5, logical.y * 0.5),
        valid: true,
    };
    let minimap_changed = resolved.minimap_panel != next_minimap;
    resolved.minimap_panel = next_minimap;
    if minimap_changed {
        bump_resolved_revision(&mut resolved);
    }
    log_resolved_viewport(
        "minimap_panel",
        &resolved.minimap_panel,
        resolved.revision,
        &mut log_cache,
    );
}

fn clear_viewport_requests_after_resolve(mut authority: ResMut<ViewportAuthority>) {
    authority.pending.clear();
}

fn clamp_viewport_request(
    mut logical: bevy_egui::egui::Rect,
    windows: &Query<&Window, With<PrimaryWindow>>,
) -> bevy_egui::egui::Rect {
    if let Ok(window) = windows.single() {
        let max_w = window.width().max(1.0);
        let max_h = window.height().max(1.0);
        logical.max.x = logical.min.x + logical.width().clamp(1.0, max_w);
        logical.max.y = logical.min.y + logical.height().clamp(1.0, max_h);
    }
    logical
}

fn bump_resolved_revision(resolved: &mut ResolvedViewports) {
    resolved.revision = resolved.revision.wrapping_add(1);
}

fn log_resolved_viewport(
    label: &'static str,
    viewport: &ResolvedViewport,
    revision: u64,
    cache: &mut ViewportResolveLogCache,
) {
    if !viewport.valid {
        return;
    }
    let key = (viewport.logical_size, viewport.physical_extent);
    if cache.last.get(label) == Some(&key) {
        return;
    }
    cache.last.insert(label, key);
    info!(
        target: "proc_A_dine01::render::viewport_pipeline",
        "resolved viewport={label} revision={revision} logical=({:.1},{:.1}) physical={}x{}",
        viewport.logical_size.x,
        viewport.logical_size.y,
        viewport.physical_extent.x,
        viewport.physical_extent.y,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_map_subviewport_is_not_extent_mismatch() {
        let sim = SimulationMapViewport {
            valid: true,
            min: Vec2::new(100.0, 80.0),
            max: Vec2::new(900.0, 400.0),
        };
        let primary = Vec2::new(1280.0, 720.0);
        assert!(!simulation_map_viewport_defect(&sim, primary));
    }

    #[test]
    fn simulation_map_tiny_or_oob_is_defect() {
        let tiny = SimulationMapViewport {
            valid: true,
            min: Vec2::ZERO,
            max: Vec2::new(4.0, 4.0),
        };
        assert!(simulation_map_viewport_defect(&tiny, Vec2::new(800.0, 600.0)));
        let oob = SimulationMapViewport {
            valid: true,
            min: Vec2::new(-10.0, 0.0),
            max: Vec2::new(100.0, 100.0),
        };
        assert!(simulation_map_viewport_defect(&oob, Vec2::new(800.0, 600.0)));
    }
}

pub fn resolved_particle_half_extents(resolved: &ResolvedViewports) -> (f32, f32) {
    if resolved.simulation_map.valid {
        return (
            resolved.simulation_map.half_extents.x.max(240.0),
            resolved.simulation_map.half_extents.y.max(240.0),
        );
    }
    if resolved.primary_window.valid {
        return (
            resolved.primary_window.half_extents.x.max(240.0),
            resolved.primary_window.half_extents.y.max(240.0),
        );
    }
    (960.0, 540.0)
}
