//! VM-B/C2: [`ResolvedViewports`] ↔ [`ViewProjectionAuthority`] (resolve writes cache; authority is post-commit truth).

use bevy::prelude::*;

use crate::gui::{MapViewInstances, SemanticViewportRect, SimulationMapViewport};
use crate::render::ResolvedViewport;
use crate::render::ResolvedViewports;

use super::authority::{ViewAuthorityWriter, ViewProjectionAuthority};
use super::ids::ViewSurfaceId;
use super::layers::{RenderViewportContract, SemanticViewportRect as RuntimeSemanticRect, ViewRenderTargetDesc};
use super::surface::ViewSurface;

#[must_use]
pub fn render_contract_from_resolved(resolved: &ResolvedViewport) -> RenderViewportContract {
    RenderViewportContract {
        logical_size: resolved.logical_size,
        physical_extent: resolved.physical_extent,
        valid: resolved.valid,
        target: ViewRenderTargetDesc::None,
    }
}

/// VM-C2: derive legacy [`ResolvedViewport`] from authority render contract.
#[must_use]
pub fn resolved_viewport_from_render_contract(render: &RenderViewportContract) -> ResolvedViewport {
    let half = render.logical_size * 0.5;
    ResolvedViewport {
        logical_size: render.logical_size,
        physical_extent: render.physical_extent,
        world_extent: render.physical_extent,
        half_extents: half,
        valid: render.valid,
    }
}

#[must_use]
pub fn resolved_viewport_from_surface(surface: &ViewSurface) -> ResolvedViewport {
    resolved_viewport_from_render_contract(&surface.render)
}

impl ViewProjectionAuthority {
    /// Read model for consumers migrating off [`ResolvedViewports`].
    #[must_use]
    pub fn resolved_viewport(&self, id: ViewSurfaceId) -> ResolvedViewport {
        self.surface(id)
            .map(resolved_viewport_from_surface)
            .unwrap_or_default()
    }
}

/// Post UI measure: commit sim-map hole into authority (C2 — dedup with [`AuthoritativeViewport`]).
pub fn commit_simulation_map_hole_to_authority(
    sim: Res<SimulationMapViewport>,
    semantic: Res<SemanticViewportRect>,
    mut authority: ResMut<ViewProjectionAuthority>,
) {
    if !sim.is_adequate_for_camera() {
        return;
    }
    let w = (sim.max.x - sim.min.x).max(1.0);
    let h = (sim.max.y - sim.min.y).max(1.0);
    let logical = Vec2::new(w, h);
    let physical = UVec2::new(w.round() as u32, h.round() as u32);
    let render = RenderViewportContract {
        logical_size: logical,
        physical_extent: physical,
        valid: true,
        target: ViewRenderTargetDesc::PrimaryWindowSubrect {
            min: sim.min,
            max: sim.max,
        },
    };
    authority.commit_render_contract(
        ViewSurfaceId::SimulationMap,
        render.clone(),
        ViewAuthorityWriter::ViewportPipeline,
    );
    authority.commit_render_contract(
        ViewSurfaceId::WorldMain,
        render,
        ViewAuthorityWriter::ViewportPipeline,
    );
    if semantic.valid {
        if let Some(surface) = authority.surfaces.get_mut(&ViewSurfaceId::SimulationMap) {
            surface.semantic = Some(RuntimeSemanticRect {
                rect: Rect::from_corners(semantic.min, semantic.max),
                valid: true,
            });
        }
    }
}

fn commit_resolved_surface(
    authority: &mut ViewProjectionAuthority,
    id: ViewSurfaceId,
    resolved: &ResolvedViewport,
) {
    authority.commit_render_contract(
        id,
        render_contract_from_resolved(resolved),
        ViewAuthorityWriter::ViewportPipeline,
    );
}

/// After viewport resolve: commit all contracts to authority (VM-B).
pub fn commit_resolved_viewports_to_authority(
    resolved: Res<ResolvedViewports>,
    mut authority: ResMut<ViewProjectionAuthority>,
) {
    commit_resolved_surface(
        authority.as_mut(),
        ViewSurfaceId::WorldPreview,
        &resolved.world_preview,
    );
    commit_resolved_surface(
        authority.as_mut(),
        ViewSurfaceId::Minimap,
        &resolved.minimap_panel,
    );
    commit_resolved_surface(
        authority.as_mut(),
        ViewSurfaceId::SimulationMap,
        &resolved.simulation_map,
    );
    // WorldMain tactical hole uses the same resolved sim-map extent as the bridge.
    commit_resolved_surface(
        authority.as_mut(),
        ViewSurfaceId::WorldMain,
        &resolved.simulation_map,
    );
}

/// After pipeline commit: refresh [`ResolvedViewports`] read cache from authority (VM-C2).
pub fn sync_resolved_viewports_from_authority(
    authority: Res<ViewProjectionAuthority>,
    mut resolved: ResMut<ResolvedViewports>,
) {
    let mut changed = false;
    let wp = authority.resolved_viewport(ViewSurfaceId::WorldPreview);
    if wp.valid && resolved.world_preview != wp {
        resolved.world_preview = wp;
        changed = true;
    }
    let mm = authority.resolved_viewport(ViewSurfaceId::Minimap);
    if mm.valid && resolved.minimap_panel != mm {
        resolved.minimap_panel = mm;
        changed = true;
    }
    let sm = authority.resolved_viewport(ViewSurfaceId::SimulationMap);
    if sm.valid && resolved.simulation_map != sm {
        resolved.simulation_map = sm;
        changed = true;
    }
    if changed {
        resolved.revision = resolved.revision.wrapping_add(1);
    }
}

/// One-way presentation mirror: map view panel sizes follow authority (not reverse sync).
pub fn apply_map_view_extents_from_authority(
    authority: Res<ViewProjectionAuthority>,
    mut map_views: ResMut<MapViewInstances>,
) {
    if let Some(surface) = authority.surface(ViewSurfaceId::WorldPreview) {
        if surface.render.valid {
            map_views.world_preview.viewport_size = surface.render.logical_size;
        }
    }
    if let Some(surface) = authority.surface(ViewSurfaceId::Minimap) {
        if surface.render.valid {
            map_views.minimap.viewport_size = surface.render.logical_size;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::UVec2;

    #[test]
    fn render_contract_from_resolved_copies_extent() {
        let rv = ResolvedViewport {
            logical_size: Vec2::new(400.0, 300.0),
            physical_extent: UVec2::new(400, 300),
            world_extent: UVec2::new(400, 300),
            half_extents: Vec2::new(200.0, 150.0),
            valid: true,
        };
        let c = render_contract_from_resolved(&rv);
        assert!(c.valid);
        assert_eq!(c.logical_size, Vec2::new(400.0, 300.0));
    }

    #[test]
    fn resolved_viewport_roundtrip_from_render_contract() {
        let c = RenderViewportContract {
            logical_size: Vec2::new(320.0, 240.0),
            physical_extent: UVec2::new(320, 240),
            valid: true,
            target: ViewRenderTargetDesc::None,
        };
        let rv = resolved_viewport_from_render_contract(&c);
        assert!(rv.valid);
        assert_eq!(rv.logical_size, c.logical_size);
    }
}
