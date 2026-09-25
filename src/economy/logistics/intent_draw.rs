//! Read-only haul routes and AI site targets on the sim map.
//!
//! Sole writers:
//! - [`record_ai_construction_intent_system`] → [`AiConstructionIntent`]
//! - [`sync_logistics_intent_draw_frame`] → [`LogisticsIntentDrawFrame`]
//!
//! [`LogisticsGraph`], [`InTransitLedger`], [`FreightLot`], and [`SiteStagingStock`]
//! are read, never written. AI commits stay on [`CommitConstructionSiteEvent`].
//! This pass does not touch the player two-click ghost.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::ai::construction::ConstructionAiOwner;
use crate::construction::ConstructionMapProjection;
use crate::engine::states::BaseState;
use crate::entities::production::core::{BUFFER_TAG_DRAGON_TEETH_UNIT, BUFFER_TAG_MINE_UNIT};
use crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate;
use crate::gui::{MainWorldCameraOrthoTrace, MapCameraDesiredRes, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::strategic::{
    BuildSiteTile, CommitConstructionSiteEvent, FootprintTiles, LogisticsGraph, LogisticsNodeId,
};
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId, TransportEdgeMeta};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::types::{FreightLot, InTransitLedger, RoutePathStore};

/// Sand — dragon-teeth haul. Not the player valid-ghost green.
pub const HAUL_DRAGON_TEETH_RGBA: [u8; 4] = [196, 154, 72, 220];
/// Clay — mine haul.
pub const HAUL_MINE_RGBA: [u8; 4] = [210, 92, 64, 220];
/// Cool gray-blue — concrete already on the logistics graph.
pub const HAUL_CONCRETE_RGBA: [u8; 4] = [120, 164, 188, 210];
/// Violet stroke — AI placement intent. Not the player ghost fill.
pub const AI_SITE_INTENT_RGBA: [u8; 4] = [224, 90, 216, 210];

/// Frames the AI target stays painted after the shared commit event (~3s at 60 Hz).
pub const AI_INTENT_HOLD_FRAMES: u32 = 180;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HaulKind {
    DragonTeeth,
    Mine,
    Concrete,
}

impl HaulKind {
    #[must_use]
    pub fn rgba(self) -> [u8; 4] {
        match self {
            Self::DragonTeeth => HAUL_DRAGON_TEETH_RGBA,
            Self::Mine => HAUL_MINE_RGBA,
            Self::Concrete => HAUL_CONCRETE_RGBA,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HaulPolyline {
    pub kind: HaulKind,
    pub points: Vec<Vec3>,
    pub amount: f32,
}

/// One AI site the player should see as a stroke, not a filled player ghost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AiSiteIntent {
    pub origin: BuildSiteTile,
    pub footprint: FootprintTiles,
    pub hold_frames: u32,
}

/// Presentation copy of the latest AI commit. Sole writer: [`record_ai_construction_intent_system`].
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct AiConstructionIntent {
    pub marker: Option<AiSiteIntent>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LogisticsIntentDrawFrame {
    pub haul: Vec<HaulPolyline>,
    pub ai_sites: Vec<AiSiteIntent>,
}

/// Sole writer of [`LogisticsIntentDrawFrame`].
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct LogisticsIntentDrawFrameRes(pub LogisticsIntentDrawFrame);

/// Paint when the cursor is in the play area and egui is not holding the pointer.
#[must_use]
pub fn logistics_intent_draw_allowed(
    gate: &SimulationMapPointerGate,
    egui_wants_pointer: bool,
) -> bool {
    gate.in_play_area && !gate.egui_blocks && !egui_wants_pointer
}

#[must_use]
pub fn haul_kind_for_tag(tag: &str) -> Option<HaulKind> {
    if tag == BUFFER_TAG_DRAGON_TEETH_UNIT {
        Some(HaulKind::DragonTeeth)
    } else if tag == BUFFER_TAG_MINE_UNIT {
        Some(HaulKind::Mine)
    } else if tag.to_ascii_lowercase().contains("concrete") {
        Some(HaulKind::Concrete)
    } else {
        None
    }
}

#[must_use]
pub fn build_logistics_intent_draw(
    ledger: &InTransitLedger,
    paths: &RoutePathStore,
    graph: Option<&LogisticsGraph>,
    directory: Option<&TransportEdgeDirectory>,
    cells_per_chunk: UVec2,
    intent: Option<&AiConstructionIntent>,
) -> LogisticsIntentDrawFrame {
    let mut frame = LogisticsIntentDrawFrame::default();
    for lot in &ledger.lots {
        if let Some(poly) = haul_polyline_for_lot(lot, paths, graph, directory, cells_per_chunk) {
            frame.haul.push(poly);
        }
    }
    if let Some(marker) = intent.and_then(|i| i.marker) {
        if marker.hold_frames > 0 && marker.footprint.width > 0 && marker.footprint.depth > 0 {
            frame.ai_sites.push(marker);
        }
    }
    frame
}

fn haul_polyline_for_lot(
    lot: &FreightLot,
    paths: &RoutePathStore,
    graph: Option<&LogisticsGraph>,
    directory: Option<&TransportEdgeDirectory>,
    cells_per_chunk: UVec2,
) -> Option<HaulPolyline> {
    if lot.amount <= f32::EPSILON {
        return None;
    }
    let kind = haul_kind_for_tag(&lot.buffer_tag)?;
    let edge_ids = paths.edge_slice(lot.path);
    if edge_ids.is_empty() {
        return None;
    }
    if kind == HaulKind::Concrete && !path_on_logistics_graph(graph, edge_ids) {
        return None;
    }
    let start = lot.progress_edge as usize;
    if start >= edge_ids.len() {
        return None;
    }
    let mut points = Vec::new();
    for id in &edge_ids[start..] {
        append_polyline(
            &mut points,
            &edge_world_points(*id, directory, graph, cells_per_chunk),
        );
    }
    if points.len() < 2 {
        return None;
    }
    Some(HaulPolyline {
        kind,
        points,
        amount: lot.amount,
    })
}

fn path_on_logistics_graph(graph: Option<&LogisticsGraph>, edge_ids: &[TransportEdgeId]) -> bool {
    let Some(graph) = graph else {
        return false;
    };
    if edge_ids.is_empty() || graph.edges.is_empty() {
        return false;
    }
    edge_ids.iter().all(|id| {
        graph
            .edges
            .iter()
            .any(|edge| edge.transport_edge == Some(*id))
    })
}

fn edge_world_points(
    id: TransportEdgeId,
    directory: Option<&TransportEdgeDirectory>,
    graph: Option<&LogisticsGraph>,
    cells_per_chunk: UVec2,
) -> Vec<Vec3> {
    if let Some(meta) = directory.and_then(|dir| dir.by_edge.get(&id)) {
        let from_meta = polyline_from_meta(meta);
        if from_meta.len() >= 2 {
            return from_meta;
        }
    }
    graph_edge_segment(graph, id, cells_per_chunk)
}

fn polyline_from_meta(meta: &TransportEdgeMeta) -> Vec<Vec3> {
    if meta.control_points.len() >= 2 {
        return meta
            .control_points
            .iter()
            .map(|p| Vec3::new(p[0], p[1], p[2]))
            .collect();
    }
    match (
        parse_tile_node_key(&meta.head_key),
        parse_tile_node_key(&meta.tail_key),
    ) {
        (Some((x0, z0)), Some((x1, z1))) => vec![tile_center(x0, z0), tile_center(x1, z1)],
        _ => Vec::new(),
    }
}

fn graph_edge_segment(
    graph: Option<&LogisticsGraph>,
    id: TransportEdgeId,
    cells_per_chunk: UVec2,
) -> Vec<Vec3> {
    let Some(graph) = graph else {
        return Vec::new();
    };
    let Some(edge) = graph
        .edges
        .iter()
        .find(|edge| edge.transport_edge == Some(id))
    else {
        return Vec::new();
    };
    match (
        node_world(graph, edge.from, cells_per_chunk),
        node_world(graph, edge.to, cells_per_chunk),
    ) {
        (Some(a), Some(b)) => vec![a, b],
        _ => Vec::new(),
    }
}

fn node_world(graph: &LogisticsGraph, id: LogisticsNodeId, cells: UVec2) -> Option<Vec3> {
    let node = graph.nodes.iter().find(|n| n.id == id)?;
    let key = node.anchor?;
    let sx = cells.x.max(1);
    let sy = cells.y.max(1);
    let lx = key.cell_index % sx;
    let ly = key.cell_index / sx;
    Some(Vec3::new(
        key.chunk.x as f32 * sx as f32 + lx as f32 + 0.5,
        0.2,
        key.chunk.y as f32 * sy as f32 + ly as f32 + 0.5,
    ))
}

fn parse_tile_node_key(key: &str) -> Option<(u32, u32)> {
    let rest = key.strip_prefix('t')?;
    let (xs, zs) = rest.split_once('_')?;
    Some((xs.parse().ok()?, zs.parse().ok()?))
}

fn tile_center(x: u32, z: u32) -> Vec3 {
    Vec3::new(x as f32 + 0.5, 0.2, z as f32 + 0.5)
}

fn append_polyline(out: &mut Vec<Vec3>, segment: &[Vec3]) {
    if segment.is_empty() {
        return;
    }
    if let Some(last) = out.last().copied() {
        if last.distance_squared(segment[0]) < 1e-4 {
            out.extend_from_slice(&segment[1..]);
            return;
        }
    }
    out.extend_from_slice(segment);
}

/// Sole writer of [`AiConstructionIntent`]. Reads the shared commit funnel; does not send events.
pub fn record_ai_construction_intent_system(
    mut reader: MessageReader<CommitConstructionSiteEvent>,
    owner: Res<ConstructionAiOwner>,
    mut intent: ResMut<AiConstructionIntent>,
) {
    let mut refreshed = false;
    for ev in reader.read() {
        if ev.owner != owner.0 {
            continue;
        }
        intent.marker = Some(AiSiteIntent {
            origin: ev.origin,
            footprint: ev.footprint,
            hold_frames: AI_INTENT_HOLD_FRAMES,
        });
        refreshed = true;
    }
    if refreshed {
        return;
    }
    let expired = intent.marker.as_mut().is_some_and(|marker| {
        marker.hold_frames = marker.hold_frames.saturating_sub(1);
        marker.hold_frames == 0
    });
    if expired {
        intent.marker = None;
    }
}

/// Sole writer of [`LogisticsIntentDrawFrameRes`]. Sim authorities are read-only.
pub fn sync_logistics_intent_draw_frame(
    ledger: Res<InTransitLedger>,
    paths: Res<RoutePathStore>,
    graph: Option<Res<LogisticsGraph>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    raster: Option<Res<crate::strategic::StrategicRasterConfig>>,
    intent: Option<Res<AiConstructionIntent>>,
    mut frame: ResMut<LogisticsIntentDrawFrameRes>,
) {
    let cells = raster
        .as_deref()
        .map(|cfg| cfg.cells_per_chunk.max(UVec2::ONE))
        .unwrap_or(UVec2::new(32, 32));
    frame.0 = build_logistics_intent_draw(
        ledger.as_ref(),
        paths.as_ref(),
        graph.as_deref(),
        directory.as_deref(),
        cells,
        intent.as_deref(),
    );
}

fn play_map_session(state: Option<&State<BaseState>>) -> bool {
    state.is_some_and(|s| matches!(s.get(), BaseState::Simulation | BaseState::Editor))
}

/// Strokes haul polylines and the AI site outline. Off outside the play area or when egui has the pointer.
pub fn draw_logistics_intent_overlay_egui(
    mut contexts: EguiContexts,
    frame: Res<LogisticsIntentDrawFrameRes>,
    gate: Option<Res<SimulationMapPointerGate>>,
    state: Option<Res<State<BaseState>>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Option<Res<MapCameraDesiredRes>>,
    map_vp: Option<Res<SimulationMapViewport>>,
    ortho: Option<Res<MainWorldCameraOrthoTrace>>,
    params: Option<Res<WorldGenParams>>,
) -> Result {
    if !play_map_session(state.as_deref()) {
        return Ok(());
    }
    let Some(gate) = gate.as_deref() else {
        return Ok(());
    };
    let Some(desired) = desired.as_deref() else {
        return Ok(());
    };
    let Some(map_vp) = map_vp.as_deref() else {
        return Ok(());
    };
    let Some(params) = params.as_deref() else {
        return Ok(());
    };
    if frame.0.haul.is_empty() && frame.0.ai_sites.is_empty() {
        return Ok(());
    }
    if !map_vp.is_adequate_for_camera() {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    if !logistics_intent_draw_allowed(gate, ctx.egui_wants_pointer_input()) {
        return Ok(());
    }
    let ortho_ref = ortho.as_deref();
    let authority_ref = authority.as_deref();
    let proj = ConstructionMapProjection::resolve(authority_ref, desired, map_vp, params);
    let tile_px = match (
        proj.world_to_egui_rendered(Vec3::new(0.5, 0.0, 0.5), ortho_ref),
        proj.world_to_egui_rendered(Vec3::new(1.5, 0.0, 0.5), ortho_ref),
    ) {
        (Some(p0), Some(p1)) => (p1 - p0).length().max(4.0 * proj.zoom_screen_scale()),
        _ => 24.0 * proj.zoom_screen_scale(),
    };
    let world_to_screen = |world: Vec3| proj.world_to_egui_rendered(world, ortho_ref);
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("logistics_intent_draw"),
    ));
    for haul in &frame.0.haul {
        let color = rgba_color(haul.kind.rgba());
        let stroke = egui::Stroke::new(2.0, color);
        for pair in haul.points.windows(2) {
            let Some(a) = world_to_screen(pair[0]) else {
                continue;
            };
            let Some(b) = world_to_screen(pair[1]) else {
                continue;
            };
            painter.line_segment([a, b], stroke);
        }
    }
    let ai_stroke = egui::Stroke::new(1.75, rgba_color(AI_SITE_INTENT_RGBA));
    for site in &frame.0.ai_sites {
        for dz in 0..site.footprint.depth {
            for dx in 0..site.footprint.width {
                let world = tile_center(
                    site.origin.x.saturating_add(dx),
                    site.origin.z.saturating_add(dz),
                );
                let Some(screen) = world_to_screen(world) else {
                    continue;
                };
                let side = tile_px * 0.92;
                let rect = egui::Rect::from_center_size(screen, egui::vec2(side, side));
                painter.rect_stroke(rect, 0.0, ai_stroke, egui::epaint::StrokeKind::Inside);
            }
        }
    }
    Ok(())
}

fn rgba_color(rgba: [u8; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::{BuildGhostState, BuildPlacementMode};
    /// Player valid-ghost fill (`ghost_visual::footprint_valid_color`). AI stroke must not reuse it.
    use crate::economy::logistics::types::{FreightMovementModel, RouteHandle};
    use crate::strategic::{LayerType, SiteArchetype, SiteId};
    use crate::systems::transport::TransportEdgeMeta;

    fn lot(tag: &str, path_first: u32, edge_count: u16, progress: u16) -> FreightLot {
        FreightLot {
            destination: Entity::PLACEHOLDER,
            buffer_tag: tag.to_string(),
            amount: 4.0,
            route: RouteHandle {
                id: 1,
                topology_revision: 1,
            },
            path: crate::economy::logistics::types::RoutePath {
                first_edge: path_first,
                edge_count,
            },
            progress_edge: progress,
            remaining_ticks: 2,
            movement: FreightMovementModel::Continuous,
        }
    }

    fn edge_meta(points: &[[f32; 3]]) -> TransportEdgeMeta {
        TransportEdgeMeta {
            control_points: points.to_vec(),
            ..TransportEdgeMeta::default()
        }
    }

    #[test]
    fn fixture_haul_routes_ai_marker_and_pointer_gate_no_window() {
        let mut paths = RoutePathStore::default();
        let dragon_path = paths.insert_path(&[TransportEdgeId(1), TransportEdgeId(2)]);
        let mine_path = paths.insert_path(&[TransportEdgeId(1)]);
        let concrete_on = paths.insert_path(&[TransportEdgeId(1)]);
        let concrete_off = paths.insert_path(&[TransportEdgeId(9)]);
        let steel = paths.insert_path(&[TransportEdgeId(1)]);

        let mut directory = TransportEdgeDirectory::default();
        directory.by_edge.insert(
            TransportEdgeId(1),
            edge_meta(&[[0.0, 0.2, 0.0], [4.0, 0.2, 0.0]]),
        );
        directory.by_edge.insert(
            TransportEdgeId(2),
            edge_meta(&[[4.0, 0.2, 0.0], [8.0, 0.2, 2.0]]),
        );
        directory.by_edge.insert(
            TransportEdgeId(9),
            edge_meta(&[[1.0, 0.2, 1.0], [2.0, 0.2, 2.0]]),
        );

        let graph = LogisticsGraph {
            revision: 1,
            nodes: Vec::new(),
            edges: vec![crate::strategic::LogisticsEdge {
                from: LogisticsNodeId(0),
                to: LogisticsNodeId(1),
                transport_edge: Some(TransportEdgeId(1)),
                capacity: 1.0,
                disruption: 0.0,
                traversal_cost: 1.0,
            }],
        };

        let ledger = InTransitLedger {
            lots: vec![
                {
                    let mut row = lot(
                        BUFFER_TAG_DRAGON_TEETH_UNIT,
                        dragon_path.first_edge,
                        dragon_path.edge_count,
                        1,
                    );
                    row.path = dragon_path;
                    row
                },
                {
                    let mut row = lot(
                        BUFFER_TAG_MINE_UNIT,
                        mine_path.first_edge,
                        mine_path.edge_count,
                        0,
                    );
                    row.path = mine_path;
                    row
                },
                {
                    let mut row = lot(
                        "concrete_portland",
                        concrete_on.first_edge,
                        concrete_on.edge_count,
                        0,
                    );
                    row.path = concrete_on;
                    row
                },
                {
                    let mut row = lot(
                        "concrete_portland",
                        concrete_off.first_edge,
                        concrete_off.edge_count,
                        0,
                    );
                    row.path = concrete_off;
                    row
                },
                {
                    let mut row = lot("steel", steel.first_edge, steel.edge_count, 0);
                    row.path = steel;
                    row
                },
            ],
        };

        let intent = AiConstructionIntent {
            marker: Some(AiSiteIntent {
                origin: BuildSiteTile { x: 6, z: 3 },
                footprint: FootprintTiles { width: 2, depth: 1 },
                hold_frames: AI_INTENT_HOLD_FRAMES,
            }),
        };

        let frame = build_logistics_intent_draw(
            &ledger,
            &paths,
            Some(&graph),
            Some(&directory),
            UVec2::new(32, 32),
            Some(&intent),
        );
        let again = build_logistics_intent_draw(
            &ledger,
            &paths,
            Some(&graph),
            Some(&directory),
            UVec2::new(32, 32),
            Some(&intent),
        );
        assert_eq!(frame, again);
        assert_eq!(
            frame.haul.iter().map(|h| h.kind).collect::<Vec<_>>(),
            vec![HaulKind::DragonTeeth, HaulKind::Mine, HaulKind::Concrete]
        );
        assert_eq!(
            frame.haul[0].points,
            vec![Vec3::new(4.0, 0.2, 0.0), Vec3::new(8.0, 0.2, 2.0)]
        );
        assert_eq!(
            frame.haul[1].points,
            vec![Vec3::new(0.0, 0.2, 0.0), Vec3::new(4.0, 0.2, 0.0)]
        );
        assert_eq!(frame.haul[2].points, frame.haul[1].points);
        assert_eq!(frame.ai_sites.len(), 1);
        assert_eq!(frame.ai_sites[0].origin, BuildSiteTile { x: 6, z: 3 });
        assert_ne!(AI_SITE_INTENT_RGBA, HaulKind::DragonTeeth.rgba());
        const PLAYER_GHOST_VALID_RGBA: [u8; 4] = [48, 140, 72, 220];
        assert_ne!(AI_SITE_INTENT_RGBA, PLAYER_GHOST_VALID_RGBA);

        let mut gate = SimulationMapPointerGate::default();
        gate.in_play_area = true;
        assert!(logistics_intent_draw_allowed(&gate, false));
        assert!(!logistics_intent_draw_allowed(&gate, true));
        gate.egui_blocks = true;
        assert!(!logistics_intent_draw_allowed(&gate, false));
        gate.egui_blocks = false;
        gate.in_play_area = false;
        assert!(!logistics_intent_draw_allowed(&gate, false));

        let mut app = App::new();
        let ai = app.world_mut().spawn_empty().id();
        let player_owner = app.world_mut().spawn_empty().id();
        app.add_plugins(MinimalPlugins)
            .add_message::<CommitConstructionSiteEvent>()
            .init_resource::<AiConstructionIntent>()
            .insert_resource(ConstructionAiOwner(ai))
            .insert_resource(BuildGhostState {
                placement_mode: BuildPlacementMode::Adjust,
                origin: Some(BuildSiteTile { x: 1, z: 1 }),
                ..BuildGhostState::default()
            })
            .add_systems(Update, record_ai_construction_intent_system);
        app.world_mut().write_message(CommitConstructionSiteEvent {
            site_id: SiteId::UNASSIGNED,
            owner: player_owner,
            archetype: SiteArchetype::FuelDepot,
            origin: BuildSiteTile { x: 1, z: 1 },
            footprint: FootprintTiles { width: 1, depth: 1 },
            layer: LayerType::Surface,
            catalog_id: None,
            placement: None,
        });
        app.world_mut().write_message(CommitConstructionSiteEvent {
            site_id: SiteId::UNASSIGNED,
            owner: ai,
            archetype: SiteArchetype::DragonTeeth,
            origin: BuildSiteTile { x: 12, z: 4 },
            footprint: FootprintTiles { width: 2, depth: 2 },
            layer: LayerType::Surface,
            catalog_id: None,
            placement: None,
        });
        app.update();
        let recorded = app.world().resource::<AiConstructionIntent>();
        assert_eq!(
            recorded
                .marker
                .map(|m| (m.origin, m.footprint, m.hold_frames)),
            Some((
                BuildSiteTile { x: 12, z: 4 },
                FootprintTiles { width: 2, depth: 2 },
                AI_INTENT_HOLD_FRAMES
            ))
        );
        let ghost = app.world().resource::<BuildGhostState>();
        assert_eq!(ghost.placement_mode, BuildPlacementMode::Adjust);
        assert_eq!(ghost.origin, Some(BuildSiteTile { x: 1, z: 1 }));
    }
}
