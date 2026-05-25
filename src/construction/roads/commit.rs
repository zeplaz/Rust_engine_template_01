//! Commit validated road path segments → [`ConstructionPlanQueue`] (sole road enqueue from path).

use bevy::prelude::*;

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::pathing::world_xy_to_tile;
use super::placement::ActiveRoadPlacement;
use crate::construction::construction_pipeline::{
    ConstructionIntent, ConstructionPlanQueue, ConstructionType,
};

/// Enqueue one plan per valid preview segment (no direct world spawn).
///
/// When `continuous` is true, keeps the last segment end as the next path anchor (Round 2).
pub fn commit_road_path_to_queue(
    placement: &mut ActiveRoadPlacement,
    queue: &mut ConstructionPlanQueue,
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
        let center = Vec2::new(
            (seg.start.x + seg.end.x) * 0.5,
            (seg.start.z + seg.end.z) * 0.5,
        );
        queue.enqueue(ConstructionIntent {
            entity_type: ConstructionType::RoadSegment { head, tail },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::construction_pipeline::ConstructionPlanQueue;

    #[test]
    fn commit_enqueues_valid_segments_only() {
        let mut placement = ActiveRoadPlacement::default();
        placement.generated_segments.push(super::super::placement::RoadSegmentPreview {
            start: Vec3::new(1.0, 0.0, 1.0),
            end: Vec3::new(4.0, 0.0, 1.0),
            width: 8.0,
            valid: true,
        });
        placement.generated_segments.push(super::super::placement::RoadSegmentPreview {
            start: Vec3::new(4.0, 0.0, 1.0),
            end: Vec3::new(4.0, 0.0, 1.0),
            width: 8.0,
            valid: false,
        });
        let mut queue = ConstructionPlanQueue::default();
        let params = WorldGenParams {
            width: 64,
            height: 64,
            ..Default::default()
        };
        commit_road_path_to_queue(&mut placement, &mut queue, &params, false);
        assert_eq!(queue.plans.len(), 1);
        assert!(placement.control_points.is_empty());
    }
}
