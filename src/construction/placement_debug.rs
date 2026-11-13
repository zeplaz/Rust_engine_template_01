//! Construction placement / footprint visual debug (operator + agent triage).
//!
//! Surfaces pick vs ghost alignment: pick uses live camera after viewport sync;
//! footprint tiles use egui + live `world_to_viewport` (GPU instancing disabled).
//!
//! **HUD-A-003:** floating Window is editor-only (`product_shell_active`).

/// Sim must not show this floating debug Window (editor product shell only).
#[allow(dead_code)] // HUD-A-003 witness greps this const; schedule uses product_shell_active
pub const PLACEMENT_DEBUG_WINDOW_IN_SIM: bool = false;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::construction::build_interaction::placement_snapshot_for_building;
use crate::construction::build_state::BuildGhostState;
use crate::construction::build_tool_authority::ActiveBuildTool;
use crate::construction::building_definitions::BuildingDefinitionRegistry;
use crate::construction::footprint_tile_instances::FootprintTileStats;
use crate::construction::map_egui_projection::ConstructionMapProjection;
use crate::engine::launch_args::EngineLaunchArgs;
use crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate;
use crate::gui::{
    camera_map_plane_vec3_to_logical_screen, map_camera_desired_from_presentation,
    map_camera_pose_for_presentation, sim_map_projection_frame,
    sim_map_screen_to_world_xy_in_frame, sim_map_world_vec3_to_screen,
    sim_map_world_vec3_to_screen_rendered, MainWorldCamera, MainWorldCameraOrthoTrace,
    MapCameraDesiredRes, SimulationMapViewport, TileDebugDrawGlobals,
};
use crate::render::view_runtime::{ViewProjectionAuthority, ViewSurfaceId};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Toggle in-window or via `CONSTRUCTION_PLACEMENT_DEBUG=1`.
#[derive(Resource, Clone, Debug)]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub struct ConstructionPlacementDebugOverlay {
    pub enabled: bool,
    /// Draw cursor vs projected ghost markers on the map.
    pub show_crosshairs: bool,
}

impl Default for ConstructionPlacementDebugOverlay {
    fn default() -> Self {
        Self {
            enabled: std::env::var("CONSTRUCTION_PLACEMENT_DEBUG")
                .ok()
                .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "on")),
            show_crosshairs: true,
        }
    }
}

/// PostUpdate snapshot for overlay + agent triage (written after pick + camera sync).
#[derive(Resource, Clone, Debug, Default)]
pub struct ConstructionPlacementDebugProbe {
    pub cursor_logical: Option<Vec2>,
    pub pick_world_rendered: Option<Vec2>,
    pub pick_world_manual: Option<Vec2>,
    pub pick_delta_world: Option<f32>,
    pub ghost_tile: Option<(u32, u32)>,
    pub ghost_center_world_xy: Option<Vec2>,
    pub ghost_screen_camera: Option<Vec2>,
    pub ghost_screen_egui: Option<Vec2>,
    pub ghost_delta_camera_vs_egui_px: Option<f32>,
    pub cursor_reproject_from_pick_camera: Option<Vec2>,
    pub cursor_reproject_delta_px: Option<f32>,
    pub camera_viewport_phys: Option<[u32; 4]>,
    pub ortho_fixed_wh: Option<Vec2>,
    pub rtt_fill_valid: bool,
    pub pointer_in_play_area: bool,
    pub pointer_chrome_blocks: bool,
    pub egui_blocks: bool,
    pub os_cursor_hidden: bool,
    /// BUILD-READ-DEBUG-001 — OS cursor visible (inverse of hidden).
    pub cursor_visible: bool,
    /// BUILD-READ-DEBUG-001 — alias of rendered pick world XY.
    pub pick_world: Option<Vec2>,
    /// BUILD-READ-DEBUG-001 — last LMB screen position on map pick.
    pub click_screen: Option<Vec2>,
    /// BUILD-READ-DEBUG-001 — tile targeted by last LMB (lock or place).
    pub action_tile: Option<(u32, u32)>,
    pub view_proj_w: f32,
    pub gpu_footprint_active: bool,
    pub egui_footprint_active: bool,
    pub footprint_tile_count: u32,
    pub scale_factor: f32,
    pub weight_tile_count: u32,
    pub site_stub_preset: Option<String>,
    pub site_zone_label_count: u32,
    pub transform_translation: Option<Vec2>,
    pub transform_yaw_deg: f32,
    pub desired_yaw_deg: f32,
    pub authority_yaw_deg: Option<f32>,
    pub camera_authoritative: bool,
    pub projection_screen_rect: Option<[f32; 4]>,
    pub projection_visible_wh: Option<Vec2>,
}

pub fn sync_construction_placement_debug_probe(
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    pointer_gate: Res<SimulationMapPointerGate>,
    params: Res<WorldGenParams>,
    ghost: Res<BuildGhostState>,
    ortho: Res<MainWorldCameraOrthoTrace>,
    globals: Option<Res<TileDebugDrawGlobals>>,
    footprint: Option<Res<FootprintTileStats>>,
    site_stub: Option<Res<super::site_stub_overlay::SiteStubOverlayState>>,
    tool: Res<ActiveBuildTool>,
    registry: Res<BuildingDefinitionRegistry>,
    cam_q: Query<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
    mut probe: ResMut<ConstructionPlacementDebugProbe>,
) {
    *probe = ConstructionPlacementDebugProbe::default();

    let Ok(window) = win.single() else {
        return;
    };
    probe.cursor_logical = window.cursor_position();
    probe.rtt_fill_valid = map_vp.valid;
    probe.pointer_in_play_area = pointer_gate.in_play_area;
    probe.pointer_chrome_blocks = pointer_gate.chrome_blocks;
    probe.egui_blocks = pointer_gate.egui_blocks;
    probe.os_cursor_hidden = !pointer_gate.os_cursor_visible;
    probe.cursor_visible = pointer_gate.os_cursor_visible;
    probe.ortho_fixed_wh = Some(Vec2::new(ortho.fixed_width, ortho.fixed_height));
    probe.click_screen = ghost.last_click_screen;
    probe.action_tile = ghost
        .last_action_tile
        .map(|t| (t.x, t.z))
        .or_else(|| ghost.origin.map(|o| (o.x, o.z)));

    if let Ok((camera, xf)) = cam_q.single() {
        probe.transform_translation = Some(xf.translation().truncate());
        probe.transform_yaw_deg = xf.rotation().to_euler(EulerRot::ZYX).0.to_degrees();
        if let Some(vp) = camera.viewport.as_ref() {
            probe.camera_viewport_phys = Some([
                vp.physical_position.x,
                vp.physical_position.y,
                vp.physical_size.x,
                vp.physical_size.y,
            ]);
        }
    }

    probe.desired_yaw_deg = desired.rotation.to_euler(EulerRot::ZYX).0.to_degrees();
    probe.authority_yaw_deg = authority.as_ref().and_then(|a| {
        a.surface(ViewSurfaceId::SimulationMap)
            .or_else(|| a.surface(ViewSurfaceId::WorldMain))
            .map(|s| s.camera.rotation.to_degrees())
    });

    if let Some(g) = globals.as_ref() {
        probe.view_proj_w = g.view_proj.w_axis.w;
    }
    if let Some(f) = footprint.as_ref() {
        probe.gpu_footprint_active = f.gpu_path_active;
        probe.egui_footprint_active = f.egui_path_active;
        probe.footprint_tile_count = f.instance_count;
    }
    probe.scale_factor = ghost.scale_factor;
    if let Some(origin) = ghost.origin {
        if let Some(snap) = placement_snapshot_for_building(&tool, &registry, &ghost, origin) {
            probe.weight_tile_count = snap.weights.len() as u32;
        }
    }
    if let Some(stub) = site_stub.as_ref() {
        probe.site_stub_preset = stub.preset_id.clone();
        probe.site_zone_label_count = stub.zone_labels.len() as u32;
    }

    let proj = ConstructionMapProjection::resolve(
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    );

    let presentation = cam_q
        .single()
        .ok()
        .map(|(_, xf)| map_camera_pose_for_presentation(xf, desired.as_ref()));

    if let Some(cursor) = probe.cursor_logical {
        if let Ok((camera, xf)) = cam_q.single() {
            if let Some(frame) =
                sim_map_projection_frame(camera, map_vp.as_ref(), window, Some(ortho.as_ref()))
            {
                probe.camera_authoritative = frame.camera_authoritative;
                probe.projection_screen_rect = Some([
                    frame.screen_rect.min.x,
                    frame.screen_rect.min.y,
                    frame.screen_rect.max.x,
                    frame.screen_rect.max.y,
                ]);
                probe.projection_visible_wh = Some(Vec2::new(frame.visible_w, frame.visible_h));
                if let Some(pose) = presentation.as_ref() {
                    probe.pick_world_manual =
                        Some(sim_map_screen_to_world_xy_in_frame(cursor, pose, &frame));
                }
            }
            probe.pick_world_rendered =
                proj.cursor_world_xy_rendered(cursor, camera, xf, window, Some(ortho.as_ref()));
            if let Some(world) = probe.pick_world_rendered {
                probe.cursor_reproject_from_pick_camera = camera_map_plane_vec3_to_logical_screen(
                    camera,
                    xf,
                    Vec3::new(world.x, 0.0, world.y),
                );
                probe.cursor_reproject_delta_px = probe
                    .cursor_reproject_from_pick_camera
                    .map(|p| p.distance(cursor));
            }
        } else {
            probe.pick_world_manual = proj.cursor_world_xy(cursor);
        }
        if let (Some(a), Some(b)) = (probe.pick_world_rendered, probe.pick_world_manual) {
            probe.pick_delta_world = Some(a.distance(b));
        }
    }
    probe.pick_world = probe.pick_world_rendered;

    if let Some(origin) = ghost.origin {
        probe.ghost_tile = Some((origin.x, origin.z));
        if ghost.placement_mode == super::build_state::BuildPlacementMode::Adjust {
            probe.action_tile = Some((origin.x, origin.z));
        }
        let center_xy = Vec2::new(origin.x as f32 + 0.5, origin.z as f32 + 0.5);
        probe.ghost_center_world_xy = Some(center_xy);
        let center_v3 = Vec3::new(center_xy.x, 0.0, center_xy.y);
        probe.ghost_screen_egui = if let Ok((camera, xf)) = cam_q.single() {
            sim_map_world_vec3_to_screen_rendered(
                center_v3,
                desired.as_ref(),
                map_vp.as_ref(),
                camera,
                xf,
                window,
                Some(ortho.as_ref()),
            )
            .map(|p| Vec2::new(p.x, p.y))
        } else {
            presentation
                .as_ref()
                .and_then(|pose| {
                    let compat = map_camera_desired_from_presentation(pose);
                    sim_map_world_vec3_to_screen(
                        center_v3,
                        &compat,
                        map_vp.as_ref(),
                        params.width as f32,
                        params.height as f32,
                    )
                    .map(|p| Vec2::new(p.x, p.y))
                })
                .or_else(|| proj.world_to_screen(center_v3).map(|p| Vec2::new(p.x, p.y)))
        };
        if let Ok((camera, xf)) = cam_q.single() {
            probe.ghost_screen_camera =
                camera_map_plane_vec3_to_logical_screen(camera, xf, center_v3);
        }
        if let (Some(cam), Some(eg)) = (probe.ghost_screen_camera, probe.ghost_screen_egui) {
            probe.ghost_delta_camera_vs_egui_px = Some(cam.distance(eg));
        }
    }
}

#[inline]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
fn overlay_visible(
    overlay: &ConstructionPlacementDebugOverlay,
    _launch: Option<&EngineLaunchArgs>,
) -> bool {
    // `--test visual` used to force this window. That painted crosshairs into
    // water and sky and was logged as operator bug B1. Explicit enable only.
    overlay.enabled
}

/// Retired egui placement debug paint — probe sync remains scheduled.
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn draw_construction_placement_debug_overlay() -> Result { Ok(()) }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::launch_args::TestScene;

    #[test]
    fn visual_test_scene_does_not_force_placement_debug() {
        let off = ConstructionPlacementDebugOverlay {
            enabled: false,
            show_crosshairs: true,
        };
        let mut launch = EngineLaunchArgs::default();
        launch.test_scene = TestScene::Visual;
        assert!(!overlay_visible(&off, Some(&launch)));
        let on = ConstructionPlacementDebugOverlay {
            enabled: true,
            show_crosshairs: true,
        };
        assert!(overlay_visible(&on, Some(&launch)));
        assert!(overlay_visible(&on, None));
    }
}
