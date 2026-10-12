//! Construction visual request buffer → unified draw pass (Round 3-C).

use bevy::prelude::*;

use crate::construction::map_egui_projection::map_zoom_screen_scale;
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
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
    pub color: crate::gui::UiRgba,
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
    /// Place-settle pulse (and similar) — when set, egui paints this RGBA instead of color_kind tokens.
    pub fill_override: Option<crate::gui::UiRgba>,
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
    pub control_points: Vec<(Vec3, crate::gui::UiRgba)>,
    pub footprint_tiles: Vec<FootprintTileRequest>,
    /// Tiles of the live ghost that take the gold lock ring (`BuildPlacementMode::Adjust` only).
    pub footprint_lock_tiles: Vec<IVec2>,
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
        self.footprint_lock_tiles.clear();
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
    fill_override: Option<crate::gui::UiRgba>,
) {
    for &(tile, w) in weights {
        if w > 0.001 {
            out.push(FootprintTileRequest {
                tile,
                color_kind: kind,
                weight: w.clamp(0.05, 1.0),
                fill_override,
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
    let occupy_fill = if super::preview_commit_guard::report_is_footprint_occupied(&preview.report)
    {
        let [r, g, b, a] = super::phase_fill::occupancy_reject_rgba();
        Some(crate::gui::UiRgba::from_rgba_unmultiplied(r, g, b, a))
    } else {
        None
    };
    if let Some(origin) = ghost.origin {
        let before = requests.footprint_tiles.len();
        if let Some(snapshot) = placement_snapshot_for_building(&tool, &registry, &ghost, origin) {
            push_weighted_footprint_tiles(
                &mut requests.footprint_tiles,
                &snapshot.weights,
                ghost_kind,
                occupy_fill,
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
                    fill_override: occupy_fill,
                });
            }
        }
        if ghost.placement_mode == crate::construction::build_state::BuildPlacementMode::Adjust {
            let locked: Vec<IVec2> = requests.footprint_tiles[before..]
                .iter()
                .map(|tile| tile.tile)
                .collect();
            requests.footprint_lock_tiles.extend(locked);
        }
    }
    for row in &staged.rows {
        let kind = color_kind_for_staged_validity(row.validity);
        push_weighted_footprint_tiles(
            &mut requests.footprint_tiles,
            &row.footprint_weights,
            kind,
            None,
        );
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
            footprint: FootprintTiles { width: 3, depth: 2 },
            ..Default::default()
        });
        app.insert_resource(BuildPlacementPreview::default());
        app.insert_resource(crate::construction::build_tool_authority::ActiveBuildTool::default());
        app.insert_resource(
            crate::construction::building_definitions::BuildingDefinitionRegistry::default(),
        );
        app.insert_resource(
            crate::construction::staged_ghost_panel::StagedPlacementBook::default(),
        );
        app.insert_resource(
            crate::construction::tile_visual::ConstructionTileVisualSettings::default(),
        );
        app.insert_resource(ConstructionVisualRequests::default());
        app.add_systems(Update, sync_footprint_visual_requests);
        app.update();
        let reqs = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(reqs.footprint_tiles.len(), 6);
        assert!(reqs
            .footprint_tiles
            .iter()
            .all(|t| (t.weight - 1.0).abs() < 1e-5));
        assert!(reqs.footprint_lock_tiles.is_empty());
    }

    #[test]
    fn locked_adjust_ghost_selects_lock_ring_token() {
        let mut app = App::new();
        app.insert_resource(BuildStripState {
            active: ToolContext::Industry,
            ..Default::default()
        });
        app.insert_resource(BuildGhostState {
            origin: Some(BuildSiteTile { x: 4, z: 7 }),
            footprint: FootprintTiles { width: 3, depth: 2 },
            placement_mode: crate::construction::build_state::BuildPlacementMode::Adjust,
            ..Default::default()
        });
        app.insert_resource(BuildPlacementPreview::default());
        app.insert_resource(crate::construction::build_tool_authority::ActiveBuildTool::default());
        app.insert_resource(
            crate::construction::building_definitions::BuildingDefinitionRegistry::default(),
        );
        app.insert_resource(
            crate::construction::staged_ghost_panel::StagedPlacementBook::default(),
        );
        app.insert_resource(
            crate::construction::tile_visual::ConstructionTileVisualSettings::default(),
        );
        app.insert_resource(ConstructionVisualRequests::default());
        app.add_systems(Update, sync_footprint_visual_requests);
        app.update();
        let reqs = app.world().resource::<ConstructionVisualRequests>();
        assert_eq!(reqs.footprint_lock_tiles.len(), 6);
        let stroke = crate::construction::ghost_visual::footprint_lock_ring_stroke(true)
            .expect("locked ghost selects the gold ring");
        assert!((stroke.width - 2.0).abs() < 1e-5);
        assert_eq!(
            stroke.color,
            crate::construction::ghost_visual::footprint_lock_ring_color()
        );
        assert!(crate::construction::ghost_visual::footprint_lock_ring_stroke(false).is_none());
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
    weighted
        .weights
        .iter()
        .any(|(_, w)| *w >= WEIGHT_OMIT_THRESHOLD && *w > 0.0 && *w < 1.0)
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
            crate::gui::UiRgba::from_rgba_unmultiplied(200, 160, 255, 220),
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
