//! Per-view fire selection for [`crate::render::extraction::RenderProjectionGraph`] (IN-C05).

use bevy::math::{Rect, Vec2};
use bevy::prelude::*;

use crate::gui::{
    RepresentationResult, ViewId, ViewManager, WorldLodBand, WorldPresentationMode,
};
use crate::render::extraction::fire_view_extract::FireVisualFramesByView;
use crate::render::extraction::sim_visual_extract::FireVisualFrame;
use crate::render::pixel_world::PixelWorldCameraFrame;
use crate::render::view_runtime::PerViewRepresentationPolicy;

/// Tiles added around the voxel camera's ground footprint: plumes and embers are drawn lifted off
/// the ground, so fire just outside the footprint can still be on screen.
const PIXEL_WORLD_FIRE_FOOTPRINT_MARGIN_TILES: f32 = 8.0;

/// What the tactical fire views (`WorldMain` / `SimulationMap`) are looking at, when the 2D view
/// state is not the camera that is actually drawn.
///
/// In PixelWorld3D the flat camera is inactive and the visible region is the perspective
/// footprint, not the ortho `visible_world_rect`. [`publish_fire_view_override_from_pixel_world`] is
/// the sole writer; the one fire fill ([`crate::render::fire_vfx::emit`]) is unchanged and reads the
/// per-view frames those consumers build. `None` fields mean "use the view-manager state".
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct FireViewOverride {
    /// Tile-space ground rect the camera sees (padded).
    pub footprint: Option<Rect>,
    /// World LOD band for the tactical views, chosen from the camera's px-per-tile.
    pub band: Option<WorldLodBand>,
}

/// LOD band for the voxel view from its zoom (screen px per tile at the focus; the view runs
/// 1.5-64). Same band names the flat view uses, so one LOD policy serves both.
#[must_use]
pub fn pixel_world_fire_band(px_per_tile: f32) -> WorldLodBand {
    if px_per_tile >= 6.0 {
        WorldLodBand::LocalTactical
    } else if px_per_tile >= 2.5 {
        WorldLodBand::Operational
    } else {
        WorldLodBand::Strategic
    }
}

/// Publish the voxel camera's ground footprint + band for fire extract (PixelWorld3D only).
pub fn publish_fire_view_override_from_pixel_world(
    mode: Option<Res<WorldPresentationMode>>,
    cam: Option<Res<PixelWorldCameraFrame>>,
    mut out: ResMut<FireViewOverride>,
) {
    let next = match cam.as_deref() {
        Some(c) if c.active && mode.as_deref().is_some_and(|m| m.is_pixel_world()) => {
            let pad = Vec2::splat(PIXEL_WORLD_FIRE_FOOTPRINT_MARGIN_TILES);
            FireViewOverride {
                footprint: Some(Rect::from_corners(c.ground_min - pad, c.ground_max + pad)),
                band: Some(pixel_world_fire_band(c.px_per_tile)),
            }
        }
        _ => FireViewOverride::default(),
    };
    if *out != next {
        *out = next;
    }
}

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
    use super::*;
    use crate::gui::{
        OverlayMask, ViewCameraState, ViewInstance, ViewProjection, ViewRenderPolicy,
        ViewRenderTarget, VIEW_NO_ENTITY,
    };
    use crate::render::extraction::sim_visual_extract::FireVisualGpuInstance;

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
                        weather: false,
                        fow: false,
                        ew: false,
                        units: false,
                        replay_scrub: false,
                    },
                },
                ..Default::default()
            },
        }
    }

    fn voxel_camera(px_per_tile: f32) -> PixelWorldCameraFrame {
        PixelWorldCameraFrame {
            active: true,
            focus: Vec2::new(330.0, 330.0),
            ground_min: Vec2::new(300.0, 305.0),
            ground_max: Vec2::new(360.0, 350.0),
            px_per_tile,
            ..Default::default()
        }
    }

    fn publish(mode: Option<WorldPresentationMode>, cam: PixelWorldCameraFrame) -> FireViewOverride {
        use bevy::ecs::system::RunSystemOnce;
        let mut world = World::new();
        if let Some(m) = mode {
            world.insert_resource(m);
        }
        world.insert_resource(cam);
        world.init_resource::<FireViewOverride>();
        world.run_system_once(publish_fire_view_override_from_pixel_world).unwrap();
        *world.resource::<FireViewOverride>()
    }

    #[test]
    fn override_published_only_while_pixel_world_is_active() {
        use crate::gui::WorldPresentationKind;
        let voxel = WorldPresentationMode { kind: WorldPresentationKind::PixelWorld3D };
        let flat = WorldPresentationMode { kind: WorldPresentationKind::TacticalFlat2D };
        let on = publish(Some(voxel), voxel_camera(20.0));
        let rect = on.footprint.expect("footprint while voxel view is active");
        assert!(rect.contains(Vec2::new(300.0, 305.0)) && rect.min.x < 300.0, "padded footprint");
        assert_eq!(on.band, Some(WorldLodBand::LocalTactical));
        assert_eq!(publish(Some(voxel), voxel_camera(1.6)).band, Some(WorldLodBand::Strategic));
        assert_eq!(publish(Some(flat), voxel_camera(20.0)), FireViewOverride::default());
        assert_eq!(publish(None, voxel_camera(20.0)), FireViewOverride::default());
    }

    /// The one particle fill yields rows for burning cells in PixelWorld3D: chunk selected by the
    /// voxel footprint, row emitted without any 2D camera input.
    #[test]
    fn pixel_world_mode_fills_particle_rows_for_burning_cells() {
        use crate::gui::WorldPresentationKind;
        use crate::render::extraction::RenderProjectionGraph;
        use crate::render::fire_vfx::{update_world_fire_particles_from_projection, WorldFireParticleFrame};
        use crate::render::ExtractedCameraMetrics;
        use bevy::math::{IVec2, Vec4};

        let voxel = WorldPresentationMode { kind: WorldPresentationKind::PixelWorld3D };
        let over = publish(Some(voxel), voxel_camera(20.0));
        let chunk = IVec2::new(5, 5);
        let chunk_world = Vec2::new(5.0 * 64.0 + 8.0, 5.0 * 64.0 + 8.0);
        assert!(over.footprint.unwrap().contains(chunk_world), "burning chunk inside the voxel footprint");

        let mut row = crate::render::extraction::sim_visual_extract::FireVisualGpuInstance::default();
        row.chunk_xy_heat_lum = Vec4::new(chunk.x as f32, chunk.y as f32, 0.85, 1.0);
        row.world_xyz_radius = Vec4::new(chunk_world.x, chunk_world.y, 0.0, 32.0);
        row.smoke_ember_vis_priority = Vec4::new(0.3, 0.4, 0.0, 1.0);
        let mut graph = RenderProjectionGraph::default();
        graph.fire.gpu_instance_capacity = 64;
        graph.fire.instance_buffer = vec![row];
        let mut frame = WorldFireParticleFrame::default();
        update_world_fire_particles_from_projection(&graph, &mut frame, None, ExtractedCameraMetrics::default(), None);
        assert!(!frame.instances.is_empty(), "burning cell must yield particle rows");
        assert_eq!(frame.spark_witness.rows, frame.instances.len());
        assert!(frame.instances.iter().all(|r| {
            let (ground, h) = crate::render::fire_vfx::particle_anim::particle_ground_and_height_m(r);
            (ground - chunk_world).length() < 40.0 && (0.0..120.0).contains(&h)
        }));
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
