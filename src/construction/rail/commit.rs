//! Commit rail path → construction plan queue.

use bevy::prelude::*;

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::junction::RailJunctionAuthority;
use super::pathing::world_xy_to_tile;
use super::placement::ActiveRailPlacement;
use super::super::construction_pipeline::{
    ConstructionIntent, ConstructionPlanQueue, ConstructionType, ExecutedRoadNetwork,
};

pub fn commit_rail_path_to_queue(
    placement: &mut ActiveRailPlacement,
    queue: &mut ConstructionPlanQueue,
    junctions: &mut RailJunctionAuthority,
    roads: &ExecutedRoadNetwork,
    params: &WorldGenParams,
    continuous: bool,
) {
    let segments: Vec<_> = placement
        .generated_segments
        .iter()
        .filter(|s| s.valid)
        .cloned()
        .collect();
    if segments.is_empty() {
        return;
    }
    for seg in &segments {
        let head = world_xy_to_tile(seg.start);
        let tail = world_xy_to_tile(seg.end);
        if head == tail {
            continue;
        }
        if roads.tiles.contains(&head) {
            junctions.register_switch(head, None);
        }
        if roads.tiles.contains(&tail) {
            junctions.register_switch(tail, None);
        }
        let center = Vec2::new(
            (seg.start.x + seg.end.x) * 0.5,
            (seg.start.z + seg.end.z) * 0.5,
        );
        queue.enqueue(ConstructionIntent {
            entity_type: ConstructionType::RailSegment { head, tail },
            world_position: center,
            rotation: 0.0,
        });
    }
    if continuous {
        if let Some(last) = segments.last() {
            placement.control_points = vec![last.end];
        } else {
            placement.control_points.clear();
        }
    } else {
        placement.control_points.clear();
    }
    placement.generated_segments.clear();
    let _ = params;
}
