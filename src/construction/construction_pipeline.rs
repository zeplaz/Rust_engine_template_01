//! Minimal construction spine — **RoadSegment** vertical slice.
//!
//! UI / preview enqueue [`ConstructionPlan`] rows; only [`execute_construction_plans_system`] mutates
//! transport topology and sim road markers.

use bevy::prelude::*;

use crate::strategic::{
    align_corridor_book_with_transport_directory, ChunkStrategicOverlay, CorridorConstructionBook,
    NetworkDirtyMask, NETWORK_DIRTY_CONNECTIVITY, NETWORK_DIRTY_FLOW,
};
use crate::systems::transport::bake_snapshot_from_ordered_tile_markers;
use crate::systems::transport::{
    hydrate_transport_from_snapshot, TransportEdgeDirectory, TransportEdgeId, TransportFieldStore,
    TransportTopology,
};

use super::corridor_transport::{
    apply_selected_rail_profile_to_edge, find_edge_id_for_segment, planned_construction_record,
    transport_edge_from_binding, RailProfileSelection, SimCorridorEdgeBinding,
};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use crate::strategic::BuildSiteTile;
use super::terrain_conform::conform_world_y;

pub const ROAD_MARKER_Y_SCALE: f32 = 20.0;
pub const ROAD_MARKER_Y_BIAS: f32 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionType {
    RoadSegment {
        head: BuildSiteTile,
        tail: BuildSiteTile,
    },
    RailSegment {
        head: BuildSiteTile,
        tail: BuildSiteTile,
    },
}

#[derive(Clone, Debug)]
pub struct ConstructionIntent {
    pub entity_type: ConstructionType,
    pub world_position: Vec2,
    pub rotation: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionStatus {
    Requested,
    Validated,
    Building,
    Completed,
    Failed,
}

#[derive(Clone, Debug)]
pub struct ConstructionPlan {
    pub id: u64,
    pub intent: ConstructionIntent,
    pub status: ConstructionStatus,
    pub progress: f32,
}

#[derive(Clone, Debug, Default)]
pub struct ConstructionValidation {
    pub valid: bool,
    pub required_actions: Vec<String>,
}

#[derive(Resource, Default, Debug)]
pub struct ConstructionPlanQueue {
    pub plans: Vec<ConstructionPlan>,
    next_id: u64,
}

impl ConstructionPlanQueue {
    pub fn enqueue(&mut self, intent: ConstructionIntent) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.plans.push(ConstructionPlan {
            id,
            intent,
            status: ConstructionStatus::Requested,
            progress: 0.0,
        });
        id
    }
}

/// Bumps when execution mutates sim road / transport truth (presentation may subscribe).
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct ConstructionWorldRevision {
    pub revision: u64,
}

/// Authoritative ordered road tiles applied by execution.
#[derive(Resource, Debug, Default, Clone)]
pub struct ExecutedRoadNetwork {
    pub tiles: Vec<BuildSiteTile>,
    pub marker_seq: u32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SimRoadSegmentMarker;

#[derive(Component, Debug, Clone, Copy)]
pub struct SimRailSegmentMarker;

#[must_use]
pub fn validate_road_segment(
    head: BuildSiteTile,
    tail: BuildSiteTile,
    params: &WorldGenParams,
) -> ConstructionValidation {
    let mut required_actions = Vec::new();
    if head == tail {
        return ConstructionValidation {
            valid: false,
            required_actions: vec!["segment needs distinct endpoints".into()],
        };
    }
    if params.width == 0 || params.height == 0 {
        return ConstructionValidation {
            valid: false,
            required_actions: vec!["world bounds unavailable".into()],
        };
    }
    for tile in [head, tail] {
        if tile.x >= params.width || tile.z >= params.height {
            required_actions.push(format!("tile ({},{}) outside world", tile.x, tile.z));
        }
    }
    ConstructionValidation {
        valid: required_actions.is_empty(),
        required_actions,
    }
}

pub fn validate_construction_plans_system(
    mut queue: ResMut<ConstructionPlanQueue>,
    params: Res<WorldGenParams>,
    rail: Res<super::rail::ActiveRailPlacement>,
) {
    for plan in &mut queue.plans {
        if plan.status != ConstructionStatus::Requested {
            continue;
        }
        let validation = match plan.intent.entity_type {
            ConstructionType::RoadSegment { head, tail } => validate_road_segment(head, tail, &params),
            ConstructionType::RailSegment { head, tail } => {
                super::rail::validate_rail_segment(head, tail, 0.0, 0.0, rail.max_slope, &params)
            }
        };
        plan.status = if validation.valid {
            ConstructionStatus::Validated
        } else {
            ConstructionStatus::Failed
        };
    }
}

struct SegmentApplyResult {
    tiles_added: Vec<BuildSiteTile>,
    marker_entities: Vec<Entity>,
    edge_id: Option<TransportEdgeId>,
    construction_record: Option<crate::systems::transport::TransportConstructionRecord>,
}

pub fn execute_construction_plans_system(
    mut commands: Commands,
    mut queue: ResMut<ConstructionPlanQueue>,
    mut roads: ResMut<ExecutedRoadNetwork>,
    mut revision: ResMut<ConstructionWorldRevision>,
    mut topology: ResMut<TransportTopology>,
    mut fields: ResMut<TransportFieldStore>,
    mut directory: ResMut<TransportEdgeDirectory>,
    mut corridor_book: ResMut<CorridorConstructionBook>,
    mut overlays: Query<&mut NetworkDirtyMask, With<ChunkStrategicOverlay>>,
    mut history: ResMut<super::history::ConstructionHistory>,
    mut intersections: ResMut<super::roads::IntersectionRegistry>,
    params: Res<WorldGenParams>,
    rail: Res<super::rail::ActiveRailPlacement>,
    raster_cfg: Option<Res<crate::strategic::StrategicRasterConfig>>,
    mut hydro_queue: Option<ResMut<crate::substrate::hydrology::HydrologyEventQueue>>,
    mut hydro_coupling: Option<ResMut<crate::substrate::hydrology::HydrologyConstructionCouplingWitness>>,
) {
    let mut executed = false;
    let cells_per_chunk = raster_cfg
        .as_deref()
        .map(|c| c.cells_per_chunk)
        .unwrap_or(bevy::math::UVec2::new(32, 32));
    for plan in &mut queue.plans {
        if plan.status != ConstructionStatus::Validated {
            continue;
        }
        plan.status = ConstructionStatus::Building;
        let endpoints = match plan.intent.entity_type {
            ConstructionType::RoadSegment { head, tail } | ConstructionType::RailSegment { head, tail } => {
                (head, tail)
            }
        };
        let result = match plan.intent.entity_type {
            ConstructionType::RoadSegment { head, tail } => apply_road_segment_to_world(
                &mut commands,
                &mut roads,
                &params,
                &mut topology,
                &mut fields,
                &mut directory,
                &mut corridor_book,
                &mut overlays,
                head,
                tail,
                SimRoadSegmentMarker,
                false,
                None,
            ),
            ConstructionType::RailSegment { head, tail } => {
                let hy = conform_world_y(head.x as f32 + 0.5, head.z as f32 + 0.5, &params);
                let ty = conform_world_y(tail.x as f32 + 0.5, tail.z as f32 + 0.5, &params);
                if !super::rail::validate_rail_segment(head, tail, hy, ty, rail.max_slope, &params).valid
                {
                    None
                } else {
                    apply_road_segment_to_world(
                        &mut commands,
                        &mut roads,
                        &params,
                        &mut topology,
                        &mut fields,
                        &mut directory,
                        &mut corridor_book,
                        &mut overlays,
                        head,
                        tail,
                        SimRailSegmentMarker,
                        true,
                        Some(rail.profile_id.as_str()),
                    )
                }
            }
        };
        if let Some(apply) = result {
            plan.status = ConstructionStatus::Completed;
            plan.progress = 1.0;
            executed = true;
            let (head, tail) = endpoints;
            for &entity in &apply.marker_entities {
                intersections.register_or_extend(head, entity);
                if tail != head {
                    intersections.register_or_extend(tail, entity);
                }
            }
            super::history::record_road_execution(
                history.as_mut(),
                apply.tiles_added.clone(),
                apply.marker_entities,
                apply.edge_id,
                apply.construction_record,
            );
            if let (Some(hydro), Some(coupling)) = (hydro_queue.as_mut(), hydro_coupling.as_mut()) {
                super::hydro_coupling::emit_road_execute_hydro_dirty(
                    hydro,
                    coupling,
                    plan.id,
                    &apply.tiles_added,
                    cells_per_chunk,
                );
            }
        } else {
            plan.status = ConstructionStatus::Failed;
        }
    }
    if executed {
        revision.revision = revision.revision.wrapping_add(1);
    }
}

fn apply_road_segment_to_world<M: Component + Copy>(
    commands: &mut Commands,
    roads: &mut ExecutedRoadNetwork,
    params: &WorldGenParams,
    topology: &mut TransportTopology,
    fields: &mut TransportFieldStore,
    directory: &mut TransportEdgeDirectory,
    corridor_book: &mut CorridorConstructionBook,
    overlays: &mut Query<&mut NetworkDirtyMask, With<ChunkStrategicOverlay>>,
    head: BuildSiteTile,
    tail: BuildSiteTile,
    marker_component: M,
    is_rail: bool,
    rail_profile_id: Option<&str>,
) -> Option<SegmentApplyResult> {
    let validation = validate_road_segment(head, tail, params);
    if !validation.valid {
        if !validation.required_actions.is_empty() {
            bevy::log::debug!(
                target: "construction::pipeline",
                ?head,
                ?tail,
                actions = ?validation.required_actions,
                "road segment rejected"
            );
        }
        return None;
    }
    let mut tiles_added = Vec::new();
    let len_before = roads.tiles.len();
    append_road_tile(roads, head);
    append_road_tile(roads, tail);
    if roads.tiles.len() > len_before {
        tiles_added.extend_from_slice(&roads.tiles[len_before..]);
    }
    let marker_tiles: Vec<(u32, u32)> = roads
        .tiles
        .iter()
        .map(|tile| (tile.x, tile.z))
        .collect();
    let snap = bake_snapshot_from_ordered_tile_markers(
        &marker_tiles,
        |_x, _z| 0.5,
        ROAD_MARKER_Y_SCALE,
        ROAD_MARKER_Y_BIAS,
    );
    if snap.edges.is_empty() {
        return None;
    }
    let edge_count_before = topology.neighbors.len();
    if hydrate_transport_from_snapshot(topology, fields, directory, &snap).is_err() {
        return None;
    }
    let edge_count_after = topology.neighbors.len();
    align_corridor_book_with_transport_directory(directory, corridor_book);

    let segment_edge = find_edge_id_for_segment(directory, head, tail);
    if let Some(edge_id) = segment_edge {
        corridor_book.plan_edge(edge_id);
        if is_rail {
            let selection = RailProfileSelection {
                profile_id: rail_profile_id.unwrap_or("default_rail").to_owned(),
            };
            apply_selected_rail_profile_to_edge(directory, edge_id, &selection, None);
        }
    } else if let Some(edge_id) = directory
        .by_edge
        .keys()
        .max_by_key(|id| id.0)
        .copied()
        .filter(|_| edge_count_after > edge_count_before)
    {
        corridor_book.plan_edge(edge_id);
        if is_rail {
            let selection = RailProfileSelection {
                profile_id: rail_profile_id.unwrap_or("default_rail").to_owned(),
            };
            apply_selected_rail_profile_to_edge(directory, edge_id, &selection, None);
        }
    }
    let bound_edge = segment_edge.or_else(|| {
        directory
            .by_edge
            .keys()
            .max_by_key(|id| id.0)
            .copied()
            .filter(|_| edge_count_after > edge_count_before)
    });
    let construction_record = bound_edge.map(planned_construction_record);
    for mut mask in overlays.iter_mut() {
        mask.mask |= NETWORK_DIRTY_FLOW | NETWORK_DIRTY_CONNECTIVITY;
    }
    let mut marker_entities = vec![spawn_path_marker(
        commands,
        roads,
        head,
        marker_component,
        "path",
        bound_edge,
    )];
    if tail != head {
        marker_entities.push(spawn_path_marker(
            commands,
            roads,
            tail,
            marker_component,
            "path",
            bound_edge,
        ));
    }
    Some(SegmentApplyResult {
        tiles_added,
        marker_entities,
        edge_id: bound_edge,
        construction_record,
    })
}

fn append_road_tile(roads: &mut ExecutedRoadNetwork, tile: BuildSiteTile) {
    if roads.tiles.last() == Some(&tile) {
        return;
    }
    roads.tiles.push(tile);
}

/// Re-apply road tiles + transport + endpoint markers after undo (Ctrl+Y redo).
pub(crate) fn replay_road_tiles_for_redo<M: Component + Copy>(
    commands: &mut Commands,
    roads: &mut ExecutedRoadNetwork,
    params: &WorldGenParams,
    topology: &mut TransportTopology,
    fields: &mut TransportFieldStore,
    directory: &mut TransportEdgeDirectory,
    corridor_book: &mut CorridorConstructionBook,
    overlays: &mut Query<&mut NetworkDirtyMask, With<ChunkStrategicOverlay>>,
    tiles_added: &[BuildSiteTile],
    marker_component: M,
) -> Vec<Entity> {
    if tiles_added.is_empty() {
        return Vec::new();
    }
    for tile in tiles_added {
        if !roads.tiles.contains(tile) {
            append_road_tile(roads, *tile);
        }
    }
    let marker_tiles: Vec<(u32, u32)> = roads.tiles.iter().map(|t| (t.x, t.z)).collect();
    let snap = bake_snapshot_from_ordered_tile_markers(
        &marker_tiles,
        |_x, _z| 0.5,
        ROAD_MARKER_Y_SCALE,
        ROAD_MARKER_Y_BIAS,
    );
    if !snap.edges.is_empty()
        && hydrate_transport_from_snapshot(topology, fields, directory, &snap).is_ok()
    {
        align_corridor_book_with_transport_directory(directory, corridor_book);
        for mut mask in overlays.iter_mut() {
            mask.mask |= NETWORK_DIRTY_FLOW | NETWORK_DIRTY_CONNECTIVITY;
        }
    }
    let head = tiles_added[0];
    let tail = *tiles_added.last().unwrap_or(&head);
    let mut marker_entities = vec![spawn_path_marker(
        commands,
        roads,
        head,
        marker_component,
        "path",
        None,
    )];
    if tail != head {
        marker_entities.push(spawn_path_marker(
            commands,
            roads,
            tail,
            marker_component,
            "path",
            None,
        ));
    }
    let _ = params;
    marker_entities
}

fn spawn_path_marker<M: Component + Copy>(
    commands: &mut Commands,
    roads: &mut ExecutedRoadNetwork,
    tile: BuildSiteTile,
    marker: M,
    label: &str,
    edge_id: Option<TransportEdgeId>,
) -> Entity {
    let seq = roads.marker_seq;
    roads.marker_seq = roads.marker_seq.saturating_add(1);
    let y = 0.5 * ROAD_MARKER_Y_SCALE + ROAD_MARKER_Y_BIAS;
    let entity = commands
        .spawn((
            marker,
            Transform::from_translation(Vec3::new(tile.x as f32, y, tile.z as f32)),
            Name::new(format!("Sim {label} ({},{}) seq={seq}", tile.x, tile.z)),
        ))
        .id();
    if let Some(eid) = edge_id {
        let binding = SimCorridorEdgeBinding { edge_id: eid };
        debug_assert_eq!(transport_edge_from_binding(&binding), eid);
        commands.entity(entity).insert(binding);
    }
    entity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn road_segment_validation_rejects_identical_tiles() {
        let tile = BuildSiteTile { x: 2, z: 3 };
        let params = WorldGenParams {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let validation = validate_road_segment(tile, tile, &params);
        assert!(!validation.valid);
    }

    #[test]
    fn road_segment_validation_accepts_in_bounds_pair() {
        let head = BuildSiteTile { x: 1, z: 1 };
        let tail = BuildSiteTile { x: 2, z: 1 };
        let params = WorldGenParams {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let validation = validate_road_segment(head, tail, &params);
        assert!(validation.valid);
    }
}
