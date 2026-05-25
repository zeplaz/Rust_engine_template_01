//! VM-08 / VM-10 / VM-11 fire + overlay isolation witness (IN-C03, IN-C04).

use bevy::prelude::*;

use crate::gui::{MapViewInstances, ViewId, ViewIsolationDiagnostics, ViewManager};
use crate::render::fire_view_extract::FireVisualFramesByView;
use crate::render::view_fire_projection::projection_fire_source_view;

use super::ids::ViewSurfaceId;
use super::per_view_policy::PerViewRepresentationPolicy;

#[derive(Resource, Clone, Debug)]
pub struct ViewFireIsolationWitness {
    pub projection_source: ViewId,
    pub per_view_fire_instances: Vec<(ViewId, usize)>,
    pub per_view_chunk_heat: Vec<(ViewId, usize)>,
    pub vm08_overlay_masks_aligned: bool,
    pub vm10_minimap_lockstep: bool,
    pub vm10_preview_lockstep: bool,
    pub vm11_minimap_cap_respected: bool,
    pub vm11_preview_cap_respected: bool,
}

impl Default for ViewFireIsolationWitness {
    fn default() -> Self {
        Self {
            projection_source: ViewId::WorldMain,
            per_view_fire_instances: Vec::new(),
            per_view_chunk_heat: Vec::new(),
            vm08_overlay_masks_aligned: false,
            vm10_minimap_lockstep: false,
            vm10_preview_lockstep: false,
            vm11_minimap_cap_respected: true,
            vm11_preview_cap_respected: true,
        }
    }
}

#[must_use]
pub fn overlay_masks_aligned_with_map_views(
    manager: &ViewManager,
    map_views: &MapViewInstances,
) -> bool {
    let checks = [
        (ViewId::WorldPreview, map_views.world_preview.overlays),
        (ViewId::Minimap, map_views.minimap.overlays),
    ];
    checks.iter().all(|(id, mask)| {
        manager
            .view(*id)
            .is_none_or(|v| v.render_policy.overlays.bits == *mask)
    })
}

pub fn refresh_view_fire_isolation_witness(
    by_view: Res<FireVisualFramesByView>,
    manager: Res<ViewManager>,
    map_views: Res<MapViewInstances>,
    isolation: Res<ViewIsolationDiagnostics>,
    policy: Res<PerViewRepresentationPolicy>,
    mut witness: ResMut<ViewFireIsolationWitness>,
) {
    witness.projection_source = projection_fire_source_view(Some(manager.as_ref()));
    witness.per_view_fire_instances = by_view
        .by_id
        .iter()
        .map(|(id, frame)| (*id, frame.instances.len()))
        .collect();
    witness.per_view_chunk_heat = by_view
        .by_id
        .iter()
        .map(|(id, frame)| (*id, frame.chunk_heat.len()))
        .collect();
    witness.vm08_overlay_masks_aligned =
        overlay_masks_aligned_with_map_views(manager.as_ref(), map_views.as_ref());
    witness.vm10_minimap_lockstep = isolation.minimap_main_lockstep_suspect;
    witness.vm10_preview_lockstep = isolation.preview_main_lockstep_suspect;
    let minimap_inst = by_view
        .by_id
        .get(&ViewId::Minimap)
        .map(|f| f.instances.len())
        .unwrap_or(0);
    let preview_inst = by_view
        .by_id
        .get(&ViewId::WorldPreview)
        .map(|f| f.instances.len())
        .unwrap_or(0);
    witness.vm11_minimap_cap_respected =
        minimap_inst <= policy.fire_cap(ViewSurfaceId::Minimap);
    witness.vm11_preview_cap_respected =
        preview_inst <= policy.fire_cap(ViewSurfaceId::WorldPreview);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{
        OverlayMask, ViewCameraState, ViewInstance, ViewManager, ViewProjection, ViewRenderPolicy,
        ViewRenderTarget, VIEW_NO_ENTITY,
    };

    #[test]
    fn overlay_masks_aligned_when_manager_matches_map_views() {
        let mut manager = ViewManager::default();
        let mask = crate::gui::MinimapOverlayMask {
            fire_heat: false,
            logistics_heat: false,
            construction_heat: false,
            ecology_heat: false,
        };
        let camera = ViewCameraState::default();
        manager.views.insert(
            ViewId::Minimap,
            ViewInstance {
                id: ViewId::Minimap,
                camera_entity: VIEW_NO_ENTITY,
                render_target: ViewRenderTarget::None,
                camera,
                projection: ViewProjection::default(),
                interaction_state: Default::default(),
                viewport_rect: Rect::from_center_size(Vec2::ZERO, Vec2::ONE),
                render_policy: ViewRenderPolicy {
                    overlays: OverlayMask { bits: mask },
                    ..Default::default()
                },
            },
        );
        let mut map_views = MapViewInstances::default();
        map_views.minimap.overlays = mask;
        assert!(overlay_masks_aligned_with_map_views(&manager, &map_views));
    }
}
