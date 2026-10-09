//! Zone paint input: LMB paint, Alt+LMB drag, RMB undo, Shift+LMB commit to pending queue.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, SimulationMapPointerGate, SimulationToolPointer,
};
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::strategic::{
    evaluate_site_placement_at_world_tile, BuildSiteTile, FootprintTiles, StrategicRasterConfig,
};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::super::pending_construction::PendingConstructionQueue;
use super::super::roads::cursor_world_on_map;
use super::super::sessions::{ActiveToolSession, PlacementBrushMode};
use super::commit::commit_painted_zones_to_pending;
use super::placement::ActiveZonePaint;
use crate::strategic::settlement::{
    admin_ring_should_close, admin_zone_tile_from_world, append_admin_ring_vertex,
    arm_admin_zone_edit, closed_ring_vertices, commit_admin_zone_ring, delete_admin_zone,
    move_admin_ring_vertex, start_admin_zone_drag, AdminZoneDrag, DistrictId, InkSite, TownBook,
};
use crate::strategic::{ConstructionSite, SiteFootprint};

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
    let footprint = FootprintTiles { width: 1, depth: 1 };
    evaluate_site_placement_at_world_tile(tile, footprint, config, overlay).allows_commit
}

pub fn sync_active_zone_from_tool(tool: Res<ActiveBuildTool>, mut paint: ResMut<ActiveZonePaint>) {
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
    pointer_gate: Option<Res<SimulationMapPointerGate>>,
    mut last_tile: Local<Option<BuildSiteTile>>,
    admin: Option<Res<AdminZoneDrag>>,
) {
    if admin.as_ref().is_some_and(|drag| drag.armed) {
        return;
    }
    let BuildTool::Zone(zone) = tool.tool else {
        *last_tile = None;
        return;
    };

    if simulation_tool_pointer(pointer_gate.as_deref()) != SimulationToolPointer::Free {
        return;
    }

    let Ok(window) = win.single() else {
        return;
    };
    let Some(world) = cursor_world_on_map(
        &window,
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    ) else {
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
    let paint_drag = buttons.pressed(MouseButton::Left) && (paint.drag_active || alt_drag);

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

fn ink_sites_for_town(
    town: &crate::strategic::settlement::TownId,
    districts: &crate::strategic::settlement::DistrictBook,
    sites: &Query<(&ConstructionSite, &SiteFootprint)>,
) -> Vec<InkSite> {
    let mut rows = Vec::new();
    for (site, foot) in sites.iter() {
        let owner = districts
            .districts
            .values()
            .find(|district| district.site_ids.contains(&site.site_id))
            .map(|district| district.town_id.clone())
            .unwrap_or_else(|| town.clone());
        let tile = foot.tiles.first().copied().unwrap_or(IVec2::ZERO);
        rows.push(InkSite {
            id: site.site_id,
            town_id: owner,
            tile,
        });
    }
    rows
}

fn ring_vertex_at(
    districts: &crate::strategic::settlement::DistrictBook,
    town: &crate::strategic::settlement::TownId,
    tile: IVec2,
) -> Option<(DistrictId, usize)> {
    let mut rows: Vec<_> = districts
        .districts
        .values()
        .filter(|district| &district.town_id == town && district.ring.len() >= 3)
        .collect();
    rows.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    for district in rows {
        if let Some(index) = district.ring.iter().position(|vertex| *vertex == tile) {
            return Some((district.id.clone(), index));
        }
    }
    None
}

fn zone_at_tile(
    districts: &crate::strategic::settlement::DistrictBook,
    town: &crate::strategic::settlement::TownId,
    tile: IVec2,
) -> Option<DistrictId> {
    let mut rows: Vec<_> = districts
        .districts
        .values()
        .filter(|district| &district.town_id == town && district.contains_tile(tile))
        .map(|district| district.id.clone())
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows.into_iter().next()
}

fn finish_ring(
    districts: &mut crate::strategic::settlement::DistrictBook,
    drag: &mut AdminZoneDrag,
    sites: &Query<(&ConstructionSite, &SiteFootprint)>,
) {
    let Some(zone) = drag.zone.clone() else {
        return;
    };
    let Some(town) = districts
        .districts
        .get(&zone)
        .map(|district| district.town_id.clone())
    else {
        return;
    };
    let ring = closed_ring_vertices(&drag.vertices);
    let rows = ink_sites_for_town(&town, districts, sites);
    let _ = commit_admin_zone_ring(districts, &zone, &ring, &rows);
    drag.armed = false;
    drag.moving = None;
    drag.anchor = None;
    drag.cursor = None;
    drag.vertices.clear();
}

/// Same pointer as zone paint. Writes the district book only when the ring closes,
/// a vertex finishes moving, or the limit is deleted.
pub fn admin_zone_drag_input_system(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    tool: Res<ActiveBuildTool>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    drag: Option<ResMut<AdminZoneDrag>>,
    mut districts: Option<ResMut<crate::strategic::settlement::DistrictBook>>,
    towns: Option<Res<TownBook>>,
    sites: Query<(&ConstructionSite, &SiteFootprint)>,
) {
    let Some(mut drag) = drag else {
        return;
    };
    let Some(districts) = districts.as_mut() else {
        return;
    };
    let control = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let Ok(window) = win.single() else {
        return;
    };
    let Some(world) = cursor_world_on_map(
        &window,
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    ) else {
        return;
    };
    let tile = admin_zone_tile_from_world(world.x, world.z);
    let Some(town_id) = towns.as_ref().and_then(|book| book.default_town.clone()) else {
        return;
    };

    if !drag.armed {
        if !matches!(tool.tool, BuildTool::Zone(_)) || !control {
            return;
        }
        if buttons.just_pressed(MouseButton::Right) {
            if let Some(zone) = zone_at_tile(districts, &town_id, tile) {
                let _ = delete_admin_zone(districts, &zone);
            }
            return;
        }
        if !buttons.just_pressed(MouseButton::Left) {
            return;
        }
        if let Some((zone, index)) = ring_vertex_at(districts, &town_id, tile) {
            let ring = districts
                .districts
                .get(&zone)
                .map(|district| district.ring.clone())
                .unwrap_or_default();
            arm_admin_zone_edit(&mut drag, zone, ring, index);
            return;
        }
        let _zone = start_admin_zone_drag(districts, &mut drag, &town_id, tile);
        drag.vertices.clear();
        append_admin_ring_vertex(&mut drag, tile);
        drag.moving = None;
        return;
    }

    if buttons.just_pressed(MouseButton::Right) || keys.just_pressed(KeyCode::Delete) {
        if let Some(zone) = drag.zone.clone() {
            let _ = delete_admin_zone(districts, &zone);
        }
        *drag = AdminZoneDrag::default();
        return;
    }

    if drag.moving.is_some() {
        if buttons.pressed(MouseButton::Left) {
            if let Some(index) = drag.moving {
                move_admin_ring_vertex(&mut drag, index, tile);
            }
        }
        if buttons.just_released(MouseButton::Left) {
            finish_ring(districts, &mut drag, &sites);
        }
        return;
    }

    if buttons.just_pressed(MouseButton::Left) && admin_ring_should_close(&drag, tile) {
        finish_ring(districts, &mut drag, &sites);
        return;
    }

    let drawing = buttons.pressed(MouseButton::Left);
    if drawing && drag.vertices.last().copied() != Some(tile) {
        append_admin_ring_vertex(&mut drag, tile);
    }
}
