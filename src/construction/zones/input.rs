//! Zone paint input: LMB paint, Alt+LMB drag, RMB undo, Shift+LMB commit to pending queue.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::strategic::{
    evaluate_site_placement_at_world_tile, BuildSiteTile, FootprintTiles, StrategicRasterConfig,
};

use super::super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::super::sessions::{ActiveToolSession, PlacementBrushMode};
use super::super::pending_construction::PendingConstructionQueue;
use super::super::roads::cursor_world_on_map;
use super::commit::commit_painted_zones_to_pending;
use super::placement::ActiveZonePaint;

fn tile_from_world_xy(x: f32, z: f32) -> BuildSiteTile {
    BuildSiteTile {
        x: x.floor().max(0.0) as u32,
        z: z.floor().max(0.0) as u32,
    }
}

fn zone_tile_valid(
    tile: BuildSiteTile,
    config: Option<&StrategicRasterConfig>,
    overlay: &Query<&crate::strategic::ChunkStrategicOverlay>,
) -> bool {
    let footprint = FootprintTiles {
        width: 1,
        depth: 1,
    };
    evaluate_site_placement_at_world_tile(tile, footprint, config, overlay).allows_commit
}

pub fn sync_active_zone_from_tool(
    tool: Res<ActiveBuildTool>,
    mut paint: ResMut<ActiveZonePaint>,
) {
    match tool.tool {
        BuildTool::Zone(z) => {
            if paint.zone != Some(z) {
                paint.zone = Some(z);
                paint.clear();
            }
        }
        _ => {
            if paint.zone.is_some() {
                paint.zone = None;
                paint.clear();
            }
        }
    }
}

pub fn zone_paint_input_system(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    tool: Res<ActiveBuildTool>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    config: Option<Res<StrategicRasterConfig>>,
    overlay: Query<&crate::strategic::ChunkStrategicOverlay>,
    mut paint: ResMut<ActiveZonePaint>,
    mut pending: ResMut<PendingConstructionQueue>,
    session: Res<ActiveToolSession>,
    mut egui_ctx: EguiContexts,
    mut last_tile: Local<Option<BuildSiteTile>>,
) {
    let BuildTool::Zone(zone) = tool.tool else {
        *last_tile = None;
        return;
    };

    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };
    if ctx.egui_wants_pointer_input() {
        return;
    }

    let Ok(window) = win.single() else {
        return;
    };
    let Some(world) =
        cursor_world_on_map(
            &window,
            authority.as_deref(),
            desired.as_ref(),
            map_vp.as_ref(),
            params.as_ref(),
        )
    else {
        return;
    };
    let tile = tile_from_world_xy(world.x, world.z);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if buttons.just_pressed(MouseButton::Right) {
        paint.pop_last();
        return;
    }

    if buttons.just_pressed(MouseButton::Left) && shift {
        if !paint.painted.is_empty() {
            commit_painted_zones_to_pending(zone, &paint.painted, &mut pending);
            paint.clear();
            paint.zone = Some(zone);
        }
        return;
    }

    let alt_drag = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
    let paint_drag = buttons.pressed(MouseButton::Left)
        && (paint.drag_active || alt_drag);

    if buttons.just_pressed(MouseButton::Left) && !shift {
        if session.brush_mode == PlacementBrushMode::Rectangle {
            paint.rect_anchor = Some(tile);
            paint.drag_active = true;
            *last_tile = Some(tile);
            return;
        }
        if zone_tile_valid(tile, config.as_deref(), &overlay) {
            paint.push_unique(tile);
            paint.drag_active = true;
            *last_tile = Some(tile);
        }
        return;
    }

    if paint_drag {
        if last_tile.is_some_and(|t| t == tile) {
            return;
        }
        if zone_tile_valid(tile, config.as_deref(), &overlay) {
            paint.push_unique(tile);
            *last_tile = Some(tile);
        }
        return;
    }

    if buttons.just_released(MouseButton::Left) {
        if session.brush_mode == PlacementBrushMode::Rectangle {
            if let (Some(anchor), true) = (paint.rect_anchor, paint.drag_active) {
                paint.fill_rectangle(anchor, tile);
            }
            paint.rect_anchor = None;
        }
        if session.zone_auto_commit_on_release
            && paint.drag_active
            && !paint.painted.is_empty()
            && !shift
        {
            commit_painted_zones_to_pending(zone, &paint.painted, &mut pending);
            paint.clear();
            paint.zone = Some(zone);
        }
        paint.drag_active = false;
        *last_tile = None;
    }
}
