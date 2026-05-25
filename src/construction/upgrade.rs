//! Road width upgrade on executed network (PHASE2-BUILD-19).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::construction_pipeline::{
    ConstructionIntent, ConstructionPlanQueue, ConstructionType, ExecutedRoadNetwork,
};
use super::roads::ActiveRoadPlacement;

#[must_use]
pub fn nearest_executed_segment(
    world: Vec3,
    roads: &ExecutedRoadNetwork,
) -> Option<(BuildSiteTile, BuildSiteTile)> {
    let tiles = &roads.tiles;
    if tiles.len() < 2 {
        return None;
    }
    let mut best: Option<(f32, usize)> = None;
    for i in 0..tiles.len().saturating_sub(1) {
        let a = tiles[i];
        let b = tiles[i + 1];
        let ax = a.x as f32 + 0.5;
        let az = a.z as f32 + 0.5;
        let bx = b.x as f32 + 0.5;
        let bz = b.z as f32 + 0.5;
        let d2 = point_segment_distance_sq(world.x, world.z, ax, az, bx, bz);
        if best.map_or(true, |(bd, _)| d2 < bd) {
            best = Some((d2, i));
        }
    }
    best.map(|(_, i)| (tiles[i], tiles[i + 1]))
}

#[inline]
fn point_segment_distance_sq(px: f32, pz: f32, ax: f32, az: f32, bx: f32, bz: f32) -> f32 {
    let dx = bx - ax;
    let dz = bz - az;
    let len_sq = dx * dx + dz * dz;
    if len_sq < 1e-6 {
        let ox = px - ax;
        let oz = pz - az;
        return ox * ox + oz * oz;
    }
    let t = ((px - ax) * dx + (pz - az) * dz) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let qx = ax + t * dx;
    let qz = az + t * dz;
    let ox = px - qx;
    let oz = pz - qz;
    ox * ox + oz * oz
}

/// Enqueue a rebuild plan for the nearest executed segment with increased width.
pub fn enqueue_road_upgrade(
    world: Vec3,
    roads: &ExecutedRoadNetwork,
    queue: &mut ConstructionPlanQueue,
    placement: &mut ActiveRoadPlacement,
    params: &WorldGenParams,
) -> bool {
    let Some((head, tail)) = nearest_executed_segment(world, roads) else {
        return false;
    };
    if !super::construction_pipeline::validate_road_segment(head, tail, params).valid {
        return false;
    }
    let center = Vec2::new(
        (head.x as f32 + tail.x as f32) * 0.5,
        (head.z as f32 + tail.z as f32) * 0.5,
    );
    queue.enqueue(ConstructionIntent {
        entity_type: ConstructionType::RoadSegment { head, tail },
        world_position: center,
        rotation: 0.0,
    });
    placement.width = (placement.width * 1.25).min(24.0);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_segment_finds_adjacent_pair() {
        let mut roads = ExecutedRoadNetwork::default();
        roads.tiles.push(BuildSiteTile { x: 2, z: 2 });
        roads.tiles.push(BuildSiteTile { x: 5, z: 2 });
        let seg = nearest_executed_segment(Vec3::new(3.5, 0.0, 2.5), &roads);
        assert_eq!(seg, Some((roads.tiles[0], roads.tiles[1])));
    }
}
