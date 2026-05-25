//! Per-view fire selection for [`super::extraction::RenderProjectionGraph`] (IN-C05).

use crate::gui::{RepresentationResult, ViewId, ViewManager};

use super::fire_view_extract::FireVisualFramesByView;
use super::sim_visual_extract::FireVisualFrame;
use super::view_runtime::PerViewRepresentationPolicy;

/// Tactical GPU projection source: WorldMain when fire overlay enabled, else SimulationMap.
#[must_use]
pub fn projection_fire_source_view(manager: Option<&ViewManager>) -> ViewId {
    if let Some(manager) = manager {
        if let Some(view) = manager.view(ViewId::WorldMain) {
            if view.render_policy.overlays.bits.fire_heat {
                return ViewId::WorldMain;
            }
        }
        if let Some(view) = manager.view(ViewId::SimulationMap) {
            if view.render_policy.overlays.bits.fire_heat {
                return ViewId::SimulationMap;
            }
        }
    }
    ViewId::WorldMain
}

/// Build the fire frame fed into the projection graph from per-view extracts + global policy.
#[must_use]
pub fn fire_frame_for_projection_graph(
    by_view: &FireVisualFramesByView,
    manager: Option<&ViewManager>,
    per_view_policy: &PerViewRepresentationPolicy,
    global_policy: &RepresentationResult,
) -> FireVisualFrame {
    let source = projection_fire_source_view(manager);
    let mut frame = by_view
        .by_id
        .get(&source)
        .cloned()
        .unwrap_or_default();
    if !global_policy.extract_plan.fire_instances {
        frame.instances.clear();
    }
    if !global_policy.visibility.fire_chunk_heat {
        frame.chunk_heat.clear();
    }
    let cap = per_view_policy.fire_cap_for_view_id(source);
    if frame.instances.len() > cap {
        frame.instances.truncate(cap);
    }
    frame
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::gui::{
        OverlayMask, ViewCameraState, ViewInstance, ViewProjection, ViewRenderPolicy,
        ViewRenderTarget, VIEW_NO_ENTITY,
    };
    use crate::render::sim_visual_extract::FireVisualGpuInstance;

    fn view_with_fire_overlay(id: ViewId, fire_heat: bool) -> ViewInstance {
        let camera = ViewCameraState::default();
        ViewInstance {
            id,
            camera_entity: VIEW_NO_ENTITY,
            render_target: ViewRenderTarget::None,
            camera,
            projection: ViewProjection::default(),
            interaction_state: Default::default(),
            viewport_rect: Rect::from_center_size(Vec2::ZERO, Vec2::ONE),
            render_policy: ViewRenderPolicy {
                overlays: OverlayMask {
                    bits: crate::gui::MinimapOverlayMask {
                        fire_heat,
                        logistics_heat: false,
                        construction_heat: false,
                        ecology_heat: false,
                    },
                },
                ..Default::default()
            },
        }
    }

    #[test]
    fn projection_source_prefers_world_main_when_fire_overlay_on() {
        let mut manager = ViewManager::default();
        manager
            .views
            .insert(ViewId::WorldMain, view_with_fire_overlay(ViewId::WorldMain, true));
        assert_eq!(
            projection_fire_source_view(Some(&manager)),
            ViewId::WorldMain
        );
    }

    #[test]
    fn fire_frame_for_projection_respects_global_extract_off() {
        let mut by_view = FireVisualFramesByView::default();
        let mut frame = FireVisualFrame::default();
        frame.instances.push(FireVisualGpuInstance::default());
        by_view.by_id.insert(ViewId::WorldMain, frame);
        let policy = PerViewRepresentationPolicy::default();
        let mut global = RepresentationResult::default();
        global.extract_plan.fire_instances = false;
        let out = fire_frame_for_projection_graph(&by_view, None, &policy, &global);
        assert!(out.instances.is_empty());
    }
}
