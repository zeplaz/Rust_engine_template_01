//! VM-08 / VM-10 / VM-11 fire + overlay isolation witness (IN-C03, IN-C04).

use bevy::prelude::*;

use crate::gui::{MapViewInstances, ViewId, ViewManager};


#[derive(Resource, Clone, Debug)]
pub struct ViewFireIsolationStats {
    pub projection_source: ViewId,
    pub per_view_fire_instances: Vec<(ViewId, usize)>,
    pub per_view_chunk_heat: Vec<(ViewId, usize)>,
    pub vm08_overlay_masks_aligned: bool,
    pub vm10_minimap_lockstep: bool,
    pub vm10_preview_lockstep: bool,
    pub vm11_minimap_cap_respected: bool,
    pub vm11_preview_cap_respected: bool,
    /// F7-A-001 (FIRE-P7): per-view extract rows ⊆ [`VisibleFireChunkSet`] (vm08 fire isolation).
    pub f7_a_per_view_extract_bounded: bool,
}

impl Default for ViewFireIsolationStats {
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
            f7_a_per_view_extract_bounded: false,
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
            weather: false,
            fow: false,
            ew: false,
            units: false,
            replay_scrub: false,
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
