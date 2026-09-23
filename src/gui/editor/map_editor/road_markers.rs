//! Road place / undo / markers (M4, R9).

use bevy::prelude::*;

use crate::engine::BaseState;
use crate::systems::transport::{
    bake_snapshot_from_ordered_markers_with_world_positions, TransportNetworkSnapshot,
};
use crate::terrain::generation::world_generator_enhanced::{Height, TileMarker, WorldMarker};

use super::brush::{MapEditorTool, MapEditorToolKind};
use super::HEIGHT_WORLD_SCALE;

/// **R9:** undo last road stroke (stack captured **before** each mouse-down on the minimap).
#[derive(Message)]
pub struct MapEditorRoadUndoRequest;

/// Live **preview** polyline from markers (R9 ghost) — not hydrated until **Bake**.
#[derive(Resource, Clone, Debug, Default)]
pub struct RoadAuthoringGhostPreview {
    pub snapshot: Option<TransportNetworkSnapshot>,
}

/// One undo frame: full marker set **before** a placement action.
#[derive(Clone, Debug, Default)]
pub struct RoadMarkerUndoFrame {
    pub entries: Vec<(u32, u32, u32, Vec3)>,
}

impl RoadMarkerUndoFrame {
    pub(crate) fn capture(
        q: &Query<(&MapEditorRoadMarkerV1, &Transform), Without<TileMarker>>,
    ) -> Self {
        let mut rows: Vec<_> = q
            .iter()
            .map(|(m, t)| (m.placement_seq, m.tile_x, m.tile_z, t.translation))
            .collect();
        rows.sort_by_key(|(seq, _, _, _)| *seq);
        Self {
            entries: rows
                .into_iter()
                .map(|(seq, tx, tz, pos)| (seq, tx, tz, pos))
                .collect(),
        }
    }
}

#[derive(Resource, Debug)]
pub struct MapEditorRoadUndoStack {
    pub frames: Vec<RoadMarkerUndoFrame>,
    pub max_frames: usize,
}

impl Default for MapEditorRoadUndoStack {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            max_frames: 50,
        }
    }
}

impl MapEditorRoadUndoStack {
    pub(crate) fn push_frame(&mut self, frame: RoadMarkerUndoFrame) {
        while self.frames.len() >= self.max_frames {
            self.frames.remove(0);
        }
        self.frames.push(frame);
    }
}

/// Monotonic **click order** for the current editor session (reset when entering editor).
/// Drives bake polyline order — **R9**; see `r9_authoring_bake_order_steps_v1.md`.
#[derive(Resource, Default, Debug)]
pub struct MapEditorRoadPlacementSeq {
    pub next: u32,
}

/// Tile-aligned **road placeholder** for map editor M4. Does not replace `entities::structure` `Road` stubs;
/// `placement_seq` is **authoring order** for [`bake_snapshot_from_ordered_tile_markers`] (not lexicographic).
#[derive(Component, Clone, Copy, Debug)]
pub struct MapEditorRoadMarkerV1 {
    pub tile_x: u32,
    pub tile_z: u32,
    pub placement_seq: u32,
}

pub(crate) fn height_at_tile(
    tiles: &Query<
        (&Transform, &Height),
        (With<TileMarker>, Without<MapEditorRoadMarkerV1>),
    >,
    tx: u32,
    tz: u32,
) -> f32 {
    for (tf, h) in tiles.iter() {
        if tf.translation.x.round() as u32 == tx && tf.translation.z.round() as u32 == tz {
            return h.0;
        }
    }
    0.0
}

fn despawn_road_markers_at(
    commands: &mut Commands,
    road_q: &Query<(Entity, &MapEditorRoadMarkerV1)>,
    tx: u32,
    tz: u32,
) {
    let victims: Vec<Entity> = road_q
        .iter()
        .filter(|(_, m)| m.tile_x == tx && m.tile_z == tz)
        .map(|(e, _)| e)
        .collect();
    for e in victims {
        commands.entity(e).despawn();
    }
}

pub(crate) fn place_road_marker(
    commands: &mut Commands,
    world_roots: &Query<Entity, With<WorldMarker>>,
    road_q: &Query<(Entity, &MapEditorRoadMarkerV1)>,
    placement: &mut MapEditorRoadPlacementSeq,
    tx: u32,
    tz: u32,
    height_normalized: f32,
) {
    let Ok(world_root) = world_roots.single() else {
        warn!("Map editor road: expected exactly one WorldMarker");
        return;
    };
    despawn_road_markers_at(commands, road_q, tx, tz);
    let seq = placement.next;
    placement.next = placement.next.saturating_add(1);
    let y = height_normalized * HEIGHT_WORLD_SCALE + 0.25;
    commands.entity(world_root).with_children(|parent| {
        parent.spawn((
            MapEditorRoadMarkerV1 {
                tile_x: tx,
                tile_z: tz,
                placement_seq: seq,
            },
            Transform::from_translation(Vec3::new(tx as f32, y, tz as f32)),
            Name::new(format!("Road marker v1 ({tx},{tz}) seq={seq}")),
        ));
    });
}

/// While primary is held, extends polyline road placement between hovered minimap tiles.
#[derive(Resource, Default)]
pub struct MapEditorRoadDragState {
    pub last_hover_tile: Option<(u32, u32)>,
}

pub(crate) fn road_authoring_ghost_refresh(
    base: Res<State<BaseState>>,
    tool: Res<MapEditorTool>,
    markers: Query<(&MapEditorRoadMarkerV1, &Transform)>,
    mut ghost: ResMut<RoadAuthoringGhostPreview>,
) {
    if base.get() != &BaseState::Editor || tool.kind != MapEditorToolKind::Road {
        ghost.snapshot = None;
        return;
    }
    let mut rows: Vec<(u32, u32, u32, Vec3)> = markers
        .iter()
        .map(|(m, t)| (m.placement_seq, m.tile_x, m.tile_z, t.translation))
        .collect();
    rows.sort_by_key(|(seq, _, _, _)| *seq);
    let with_pos: Vec<(u32, u32, Vec3)> = rows.into_iter().map(|(_, x, z, p)| (x, z, p)).collect();
    let snap = bake_snapshot_from_ordered_markers_with_world_positions(&with_pos);
    ghost.snapshot = if snap.edges.is_empty() {
        None
    } else {
        Some(snap)
    };
}

pub(crate) fn map_editor_road_undo(
    mut events: MessageReader<MapEditorRoadUndoRequest>,
    mut commands: Commands,
    world_roots: Query<Entity, With<WorldMarker>>,
    road_entities: Query<(Entity, &MapEditorRoadMarkerV1)>,
    mut stack: ResMut<MapEditorRoadUndoStack>,
    mut placement: ResMut<MapEditorRoadPlacementSeq>,
) {
    for _ in events.read() {
        let Some(frame) = stack.frames.pop() else {
            continue;
        };
        let Ok(world_root) = world_roots.single() else {
            warn!("Map editor undo: expected exactly one WorldMarker");
            continue;
        };
        let to_remove: Vec<Entity> = road_entities.iter().map(|(e, _)| e).collect();
        for e in to_remove {
            commands.entity(e).despawn();
        }
        for (seq, tx, tz, pos) in &frame.entries {
            commands.entity(world_root).with_children(|parent| {
                parent.spawn((
                    MapEditorRoadMarkerV1 {
                        tile_x: *tx,
                        tile_z: *tz,
                        placement_seq: *seq,
                    },
                    Transform::from_translation(*pos),
                    Name::new(format!("Road marker v1 ({tx},{tz}) seq={seq}")),
                ));
            });
        }
        placement.next = frame
            .entries
            .iter()
            .map(|(s, _, _, _)| *s)
            .max()
            .map(|m| m.saturating_add(1))
            .unwrap_or(0);
    }
}
