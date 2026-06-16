//! Construction visual request buffer → unified draw pass (Round 3-C).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::construction::map_egui_projection::{map_zoom_screen_scale, world_to_sim_map_egui};
use crate::gui::{MapCameraDesired, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::strategic::BuildSiteTile;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

#[derive(Clone, Copy, Debug)]
pub enum VisualPathKind {
    Road,
    Rail,
}

#[derive(Clone, Debug)]
pub struct PathLineRequest {
    pub kind: VisualPathKind,
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
    pub valid: bool,
    pub slope_ok: bool,
    pub committed: bool,
}

#[derive(Clone, Debug)]
pub struct ZoneTileRequest {
    pub center: Vec3,
    pub color: egui::Color32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FootprintTileColorKind {
    Valid,
    Risky,
    Invalid,
}

#[derive(Clone, Debug)]
pub struct FootprintTileRequest {
    pub tile: IVec2,
    pub color_kind: FootprintTileColorKind,
    /// Partial-alpha weight from parametric raster (`α = weight`, capped at 1).
    pub weight: f32,
}

/// BUILD-READ-SITE-v0-002 — dashed site composition border.
#[derive(Clone, Debug)]
pub struct SiteStubBoxRequest {
    pub origin: BuildSiteTile,
    pub width: u32,
    pub depth: u32,
}

#[derive(Clone, Debug)]
pub struct SiteZoneLabelRequest {
    pub world: Vec3,
    pub text: String,
}

#[derive(Resource, Debug, Default)]
pub struct ConstructionVisualRequests {
    pub paths: Vec<PathLineRequest>,
    pub zone_tiles: Vec<ZoneTileRequest>,
    pub control_points: Vec<(Vec3, egui::Color32)>,
    pub footprint_tiles: Vec<FootprintTileRequest>,
    pub corridor_paths: Vec<super::round4_corridor::CorridorPhasePathRequest>,
    pub site_stub_boxes: Vec<SiteStubBoxRequest>,
    pub site_zone_labels: Vec<SiteZoneLabelRequest>,
}

impl ConstructionVisualRequests {
    pub fn clear(&mut self) {
        self.paths.clear();
        self.zone_tiles.clear();
        self.control_points.clear();
        self.footprint_tiles.clear();
        self.corridor_paths.clear();
        self.site_stub_boxes.clear();
        self.site_zone_labels.clear();
    }
}

fn color_kind_for_staged_validity(
    validity: crate::construction::staged_ghost_panel::StagedValidity,
) -> FootprintTileColorKind {
    use crate::construction::staged_ghost_panel::StagedValidity;
    match validity {
        StagedValidity::Ok => FootprintTileColorKind::Valid,
        StagedValidity::Warn => FootprintTileColorKind::Risky,
        StagedValidity::Bad => FootprintTileColorKind::Invalid,
    }
}

fn push_weighted_footprint_tiles(
    out: &mut Vec<FootprintTileRequest>,
    weights: &[(IVec2, f32)],
    kind: FootprintTileColorKind,
) {
    for &(tile, w) in weights {
        if w > 0.001 {
            out.push(FootprintTileRequest {
                tile,
                color_kind: kind,
                weight: w.clamp(0.05, 1.0),
            });
        }
    }
}

/// Fill per-tile footprint requests from active ghost + staged book (view-only, partial-alpha).
pub fn sync_footprint_visual_requests(
    strip: Res<crate::construction::build_strip::BuildStripState>,
    ghost: Res<crate::construction::build_state::BuildGhostState>,
    preview: Res<crate::construction::build_state::BuildPlacementPreview>,
    tool: Res<crate::construction::build_tool_authority::ActiveBuildTool>,
    registry: Res<crate::construction::building_definitions::BuildingDefinitionRegistry>,
    staged: Res<crate::construction::staged_ghost_panel::StagedPlacementBook>,
    settings: Res<crate::construction::tile_visual::ConstructionTileVisualSettings>,
    mut requests: ResMut<ConstructionVisualRequests>,
) {
    use crate::construction::build_confidence::{confidence_from_validation, BuildConfidence};
    use crate::construction::build_interaction::placement_snapshot_for_building;
    use crate::construction::build_strip::ToolContext;
    use crate::construction::building_catalog::FootprintMatrix;
    if strip.active == ToolContext::None || !settings.show_occupation_tiles {
        return;
    }
    let ghost_kind = match confidence_from_validation(&preview.report) {
        BuildConfidence::Perfect | BuildConfidence::Good => FootprintTileColorKind::Valid,
        BuildConfidence::Risky => FootprintTileColorKind::Risky,
        BuildConfidence::Invalid => FootprintTileColorKind::Invalid,
    };
    if let Some(origin) = ghost.origin {
        if let Some(snapshot) = placement_snapshot_for_building(&tool, &registry, &ghost, origin) {
            push_weighted_footprint_tiles(
                &mut requests.footprint_tiles,
                &snapshot.weights,
                ghost_kind,
            );
        } else {
            let ox = origin.x as i32;
            let oz = origin.z as i32;
            let matrix = tool
                .building_intent
                .as_ref()
                .map(|i| i.footprint.clone())
                .unwrap_or_else(|| {
                    FootprintMatrix::from_size(ghost.footprint.width, ghost.footprint.depth, true)
                });
            for (dx, dz) in matrix.occupied_local_offsets() {
                let tile = IVec2::new(ox + dx as i32, oz + dz as i32);
                requests.footprint_tiles.push(FootprintTileRequest {
                    tile,
                    color_kind: ghost_kind,
                    weight: 1.0,
                });
            }
        }
    }
    for row in &staged.rows {
        let kind = color_kind_for_staged_validity(row.validity);
        push_weighted_footprint_tiles(&mut requests.footprint_tiles, &row.footprint_weights, kind);
    }
}

#[cfg(test)]
mod footprint_tests {
    use super::*;
    use crate::construction::build_state::{BuildGhostState, BuildPlacementPreview};
    use crate::construction::build_strip::{BuildStripState, ToolContext};
    use crate::strategic::BuildSiteTile;
    use crate::strategic::FootprintTiles;
    use bevy::app::App;

    #[test]
    fn footprint_tile_requests_3x2_emits_six() {
        let mut app = App::new();
        app.insert_resource(BuildStripState {
            active: ToolContext::Industry,
            ..Default::default()
        });
        app.insert_resource(BuildGhostState {
            origin: Some(BuildSiteTile { x: 4, z: 7 }),
            footprint: FootprintTiles {
                width: 3,
                depth: 2,
            },
            ..Default::default()
        });
        app.insert_resource(BuildPlacementPreview::default());
        app.insert_resource(crate::construction::build_tool_authority::ActiveBuildTool::default());
        app.insert_resource(crate::construction::building_definitions::BuildingDefinitionRegistry::default());
        app.insert_resource(crate::construction::staged_ghost_panel::StagedPlacementBook::default());
        app.insert_resource(crate::construction::tile_visual::ConstructionTileVisualSettings::default());
        app.insert_resource(ConstructionVisualRequests::default());
        app.add_systems(Update, sync_footprint_visual_requests);
        app.update();
        let reqs = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(reqs.footprint_tiles.len(), 6);
        assert!(reqs.footprint_tiles.iter().all(|t| (t.weight - 1.0).abs() < 1e-5));
    }

    #[test]
    fn partial_alpha_parametric_witness() {
        assert!(partial_alpha_parametric_witness_green());
    }
}

/// **PARAM-005** — weighted raster produces sub-unity tile weights for map overlay.
#[must_use]
pub fn partial_alpha_parametric_witness_green() -> bool {
    use crate::construction::building_catalog::FootprintMatrix;
    use crate::construction::weighted_footprint::{
        rasterize_with_effective_scale, PlacementParams, WEIGHT_OMIT_THRESHOLD,
    };

    let base = FootprintMatrix::from_size(2, 2, true);
    let origin = BuildSiteTile { x: 1, z: 1 };
    let (_, weighted) =
        rasterize_with_effective_scale(&base, PlacementParams::identity(1.24), origin);
    weighted.weights.iter().any(|(_, w)| {
        *w >= WEIGHT_OMIT_THRESHOLD && *w > 0.0 && *w < 1.0
    })
}

pub fn draw_construction_visual_requests_egui(
    mut contexts: bevy_egui::EguiContexts,
    requests: Res<ConstructionVisualRequests>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
) -> Result {
    if requests.paths.is_empty()
        && requests.zone_tiles.is_empty()
        && requests.control_points.is_empty()
        && requests.footprint_tiles.is_empty()
        && requests.corridor_paths.is_empty()
        && requests.site_stub_boxes.is_empty()
        && requests.site_zone_labels.is_empty()
    {
        return Ok(());
    }
    if !map_vp.is_adequate_for_camera() {
        return Ok(());
    }
    let zoom = map_zoom_screen_scale(authority.as_deref(), desired.as_ref());
    let ctx = contexts.ctx_mut()?;
    let layer = egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("construction_visual_authority"),
    );
    let painter = ctx.layer_painter(layer);

    let tile_px = crate::construction::map_egui_projection::tile_screen_extent(
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    );
    for z in &requests.zone_tiles {
        if let Some(screen) =
            world_to_sim_map_egui(
                z.center,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        {
            let side = tile_px * 0.92;
            let rect = egui::Rect::from_center_size(screen, egui::vec2(side, side));
            painter.rect_filled(rect, 1.0, z.color);
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(220)),
                egui::epaint::StrokeKind::Inside,
            );
        }
    }

    let dash = egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(180, 200, 255, 200));
    for stub in &requests.site_stub_boxes {
        for edge in 0..stub.width {
            for &dz in &[0, stub.depth.saturating_sub(1)] {
                let world = Vec3::new(
                    stub.origin.x as f32 + edge as f32 + 0.5,
                    0.05,
                    stub.origin.z as f32 + dz as f32 + 0.5,
                );
                if let Some(screen) = world_to_sim_map_egui(
                    world,
                    authority.as_deref(),
                    desired.as_ref(),
                    map_vp.as_ref(),
                    params.as_ref(),
                ) {
                    let side = tile_px * 0.92;
                    let rect = egui::Rect::from_center_size(screen, egui::vec2(side, side));
                    painter.rect_stroke(rect, 0.0, dash, egui::epaint::StrokeKind::Inside);
                }
            }
        }
        for edge in 0..stub.depth {
            for &dx in &[0, stub.width.saturating_sub(1)] {
                let world = Vec3::new(
                    stub.origin.x as f32 + dx as f32 + 0.5,
                    0.05,
                    stub.origin.z as f32 + edge as f32 + 0.5,
                );
                if let Some(screen) = world_to_sim_map_egui(
                    world,
                    authority.as_deref(),
                    desired.as_ref(),
                    map_vp.as_ref(),
                    params.as_ref(),
                ) {
                    let side = tile_px * 0.92;
                    let rect = egui::Rect::from_center_size(screen, egui::vec2(side, side));
                    painter.rect_stroke(rect, 0.0, dash, egui::epaint::StrokeKind::Inside);
                }
            }
        }
    }

    for label in &requests.site_zone_labels {
        if let Some(screen) = world_to_sim_map_egui(
            label.world,
            authority.as_deref(),
            desired.as_ref(),
            map_vp.as_ref(),
            params.as_ref(),
        ) {
            painter.text(
                screen,
                egui::Align2::CENTER_CENTER,
                &label.text,
                egui::FontId::proportional(11.0),
                egui::Color32::from_rgba_unmultiplied(220, 230, 255, 230),
            );
        }
    }

    super::round4_corridor::draw_corridor_phase_paths(
        &painter,
        &requests.corridor_paths,
        |world| {
            world_to_sim_map_egui(
                world,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        },
        zoom,
    );

    for path in &requests.paths {
        let Some(a) =
            world_to_sim_map_egui(
                path.start,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        else {
            continue;
        };
        let Some(b) =
            world_to_sim_map_egui(
                path.end,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        else {
            continue;
        };
        let color = match (path.kind, path.valid, path.slope_ok, path.committed) {
            (VisualPathKind::Road, true, _, true) => super::ghost_visual::road_committed_color(),
            (VisualPathKind::Road, true, _, false) => super::ghost_visual::road_segment_color(true),
            (VisualPathKind::Rail, true, true, _) => {
                egui::Color32::from_rgba_unmultiplied(180, 120, 255, 150)
            }
            (VisualPathKind::Rail, _, false, _) => egui::Color32::from_rgba_unmultiplied(255, 140, 60, 180),
            _ => egui::Color32::from_rgba_unmultiplied(240, 90, 90, 160),
        };
        let stroke_w = (path.width * 0.5 * zoom).clamp(1.0, 48.0);
        painter.line_segment([a, b], egui::Stroke::new(stroke_w, color));
    }

    for (p, color) in &requests.control_points {
        if let Some(screen) =
            world_to_sim_map_egui(
                *p,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        {
            let r = (5.0 * zoom.sqrt()).clamp(3.0, 14.0);
            painter.circle_filled(screen, r, *color);
        }
    }

    for tile in &requests.footprint_tiles {
        let world = Vec3::new(tile.tile.x as f32 + 0.5, 0.0, tile.tile.y as f32 + 0.5);
        let Some(screen) = world_to_sim_map_egui(
            world,
            authority.as_deref(),
            desired.as_ref(),
            map_vp.as_ref(),
            params.as_ref(),
        ) else {
            continue;
        };
        let base = match tile.color_kind {
            FootprintTileColorKind::Valid => super::ghost_visual::footprint_valid_color(),
            FootprintTileColorKind::Risky => super::ghost_visual::footprint_risky_color(),
            FootprintTileColorKind::Invalid => super::ghost_visual::footprint_invalid_color(),
        };
        let alpha = (base.a() as f32 * tile.weight).round() as u8;
        let color = egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), alpha);
        let side = tile_px * 0.92;
        let rect = egui::Rect::from_center_size(screen, egui::vec2(side, side));
        painter.rect_filled(rect, 1.0, color);
    }
    Ok(())
}

pub fn sync_road_visual_requests(
    tool: Res<crate::construction::build_tool_authority::ActiveBuildTool>,
    placement: Res<crate::construction::roads::ActiveRoadPlacement>,
    roads: Res<crate::construction::construction_pipeline::ExecutedRoadNetwork>,
    mut requests: ResMut<ConstructionVisualRequests>,
) {
    use crate::construction::build_tool_authority::BuildTool;
    use crate::construction::ghost_visual::road_control_point_color;

    sync_executed_road_paths(&roads, &mut requests);

    if !matches!(tool.tool, BuildTool::Road(_)) {
        return;
    }
    use crate::construction::tile_visual::build_site_tiles_between;
    for seg in &placement.generated_segments {
        requests.paths.push(PathLineRequest {
            kind: VisualPathKind::Road,
            start: seg.start,
            end: seg.end,
            width: placement.width,
            valid: seg.valid,
            slope_ok: true,
            committed: false,
        });
        let a = BuildSiteTile {
            x: seg.start.x.floor() as u32,
            z: seg.start.z.floor() as u32,
        };
        let b = BuildSiteTile {
            x: seg.end.x.floor() as u32,
            z: seg.end.z.floor() as u32,
        };
        let color = super::ghost_visual::road_segment_color(seg.valid);
        for t in build_site_tiles_between(a, b) {
            requests.zone_tiles.push(ZoneTileRequest {
                center: Vec3::new(t.x as f32 + 0.5, 0.0, t.z as f32 + 0.5),
                color,
            });
        }
    }
    let c = road_control_point_color();
    for p in &placement.control_points {
        requests.control_points.push((*p, c));
    }
}

fn sync_executed_road_paths(
    roads: &crate::construction::construction_pipeline::ExecutedRoadNetwork,
    requests: &mut ConstructionVisualRequests,
) {
    use crate::construction::ghost_visual::road_committed_color;
    use crate::construction::tile_visual::build_site_tiles_between;

    let tiles: Vec<BuildSiteTile> = roads.tiles.clone();
    for w in tiles.windows(2) {
        let a = w[0];
        let b = w[1];
        if a == b {
            continue;
        }
        requests.paths.push(PathLineRequest {
            kind: VisualPathKind::Road,
            start: Vec3::new(a.x as f32 + 0.5, 0.0, a.z as f32 + 0.5),
            end: Vec3::new(b.x as f32 + 0.5, 0.0, b.z as f32 + 0.5),
            width: 8.0,
            valid: true,
            slope_ok: true,
            committed: true,
        });
        for t in build_site_tiles_between(a, b) {
            requests.zone_tiles.push(ZoneTileRequest {
                center: Vec3::new(t.x as f32 + 0.5, 0.0, t.z as f32 + 0.5),
                color: road_committed_color(),
            });
        }
    }
}

pub fn sync_rail_visual_requests(
    tool: Res<crate::construction::build_tool_authority::ActiveBuildTool>,
    placement: Res<crate::construction::rail::ActiveRailPlacement>,
    mut requests: ResMut<ConstructionVisualRequests>,
) {
    use crate::construction::build_tool_authority::BuildTool;

    if !matches!(tool.tool, BuildTool::Rail(_)) {
        return;
    }
    for seg in &placement.generated_segments {
        requests.paths.push(PathLineRequest {
            kind: VisualPathKind::Rail,
            start: seg.start,
            end: seg.end,
            width: seg.width,
            valid: seg.valid,
            slope_ok: seg.slope_ok,
            committed: false,
        });
    }
    for p in &placement.control_points {
        requests.control_points.push((
            *p,
            egui::Color32::from_rgba_unmultiplied(200, 160, 255, 220),
        ));
    }
}

pub fn sync_zone_visual_requests(
    tool: Res<crate::construction::build_tool_authority::ActiveBuildTool>,
    paint: Res<crate::construction::zones::ActiveZonePaint>,
    mut requests: ResMut<ConstructionVisualRequests>,
) {
    use crate::construction::build_tool_authority::BuildTool;
    use crate::construction::zones::zone_fill;

    let BuildTool::Zone(zone) = tool.tool else {
        return;
    };
    let fill = zone_fill(zone);
    for tile in &paint.painted {
        requests.zone_tiles.push(ZoneTileRequest {
            center: Vec3::new(tile.x as f32 + 0.5, 0.05, tile.z as f32 + 0.5),
            color: fill,
        });
    }
}

pub fn clear_construction_visual_requests(mut requests: ResMut<ConstructionVisualRequests>) {
    requests.clear();
}
