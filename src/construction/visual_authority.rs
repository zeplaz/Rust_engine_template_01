//! Construction visual request buffer → unified draw pass (Round 3-C).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;

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
    /// BUILD-UX-1 — facing/nose tiles (rotation chevron) for the live ghost.
    pub footprint_facing_tiles: Vec<IVec2>,
    /// Ebbtide-style column stacks — presentation only ([`super::voxel_massing`]).
    /// Frame `clear()` resets the buffer; **content** is written only by
    /// `voxel_massing::sync_voxel_massing_visual_requests` (after footprint sync).
    pub massing_columns: Vec<super::voxel_massing::VoxelColumnRequest>,
    /// RND-09 instance batch — packed by the same sole writer when column count ≥
    /// [`super::voxel_massing::MASSING_INSTANCE_BATCH_THRESHOLD`]; empty below gate.
    pub massing_instances: Vec<super::voxel_massing::MassingColumnInstance>,
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
        self.footprint_facing_tiles.clear();
        self.massing_columns.clear();
        self.massing_instances.clear();
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

/// Quarter-turn facing unit on the map plane (0=+Z, 1=+X, 2=−Z, 3=−X).
#[must_use]
pub fn facing_delta(rotation_quarter_turns: u8) -> IVec2 {
    match rotation_quarter_turns % 4 {
        1 => IVec2::new(1, 0),
        2 => IVec2::new(0, -1),
        3 => IVec2::new(-1, 0),
        _ => IVec2::new(0, 1),
    }
}

/// Nose tiles: footprint cells farthest along facing, plus one step beyond for the chevron.
#[must_use]
pub fn facing_nose_tiles(weights: &[(IVec2, f32)], rotation_quarter_turns: u8) -> Vec<IVec2> {
    let dir = facing_delta(rotation_quarter_turns);
    let occupied: Vec<IVec2> = weights
        .iter()
        .filter(|(_, w)| *w > 0.001)
        .map(|(t, _)| *t)
        .collect();
    if occupied.is_empty() {
        return Vec::new();
    }
    let score = |t: IVec2| t.x * dir.x + t.y * dir.y;
    let best = occupied.iter().copied().map(score).max().unwrap_or(0);
    let mut nose: Vec<IVec2> = occupied.into_iter().filter(|t| score(*t) == best).collect();
    nose.sort_by_key(|t| (t.x, t.y));
    // Chevron tip just past the footprint edge.
    if let Some(&edge) = nose.first() {
        let tip = edge + dir;
        if !nose.contains(&tip) {
            nose.push(tip);
        }
    }
    nose
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
        // BUILD-UX-1: facing chevron always when ghost has tiles (rotation readable).
        let weights: Vec<(IVec2, f32)> = requests.footprint_tiles[before..]
            .iter()
            .map(|t| (t.tile, t.weight))
            .collect();
        requests
            .footprint_facing_tiles
            .extend(facing_nose_tiles(&weights, ghost.rotation_quarter_turns));
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
    fn facing_nose_moves_with_rotation() {
        let weights = vec![
            (IVec2::new(0, 0), 1.0),
            (IVec2::new(1, 0), 1.0),
            (IVec2::new(0, 1), 1.0),
            (IVec2::new(1, 1), 1.0),
        ];
        let n0 = facing_nose_tiles(&weights, 0);
        let n1 = facing_nose_tiles(&weights, 1);
        assert!(n0.contains(&IVec2::new(0, 1)) || n0.contains(&IVec2::new(1, 1)));
        assert!(n0.iter().any(|t| *t == IVec2::new(0, 2) || *t == IVec2::new(1, 2)));
        assert!(n1.contains(&IVec2::new(1, 0)) || n1.contains(&IVec2::new(1, 1)));
        assert_ne!(n0, n1);
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


}




pub fn sync_road_visual_requests(
    tool: Res<crate::construction::build_tool_authority::ActiveBuildTool>,
    placement: Res<crate::construction::roads::ActiveRoadPlacement>,
    roads: Res<crate::construction::CorridorNetwork>,
    mut requests: ResMut<ConstructionVisualRequests>,
) {
    use crate::construction::build_tool_authority::BuildTool;
    use crate::construction::ghost_visual::road_control_point_color;

    sync_executed_road_paths(&roads, &mut requests);

    if !matches!(tool.tool, BuildTool::Road(_) | BuildTool::Firebreak) {
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
    roads: &crate::construction::CorridorNetwork,
    requests: &mut ConstructionVisualRequests,
) {
    use crate::construction::ghost_visual::road_committed_color;
    use crate::construction::tile_visual::build_site_tiles_between;

    for (_, a, b) in roads.road_spans() {
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
    // Committed rail draws as rail (it used to land in the road tile list and draw as road).
    for (_, a, b) in roads.rail_spans() {
        requests.paths.push(PathLineRequest {
            kind: VisualPathKind::Rail,
            start: Vec3::new(a.x as f32 + 0.5, 0.0, a.z as f32 + 0.5),
            end: Vec3::new(b.x as f32 + 0.5, 0.0, b.z as f32 + 0.5),
            width: 6.0,
            valid: true,
            slope_ok: true,
            committed: true,
        });
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
