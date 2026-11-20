//! Road width upgrade on executed network (PHASE2-BUILD-19).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::construction_pipeline::{ConstructionIntent, ConstructionPlanQueue, ConstructionType};
use super::corridor_network::CorridorNetwork;
use crate::infrastructure::RoadTier;
use super::roads::ActiveRoadPlacement;

/// Nearest committed road segment to `world` (whole line ends + its tier).
#[must_use]
pub fn nearest_executed_segment(
    world: Vec3,
    roads: &CorridorNetwork,
) -> Option<(BuildSiteTile, BuildSiteTile, RoadTier)> {
    let mut best: Option<(f32, BuildSiteTile, BuildSiteTile, RoadTier)> = None;
    for (seg, a, b) in roads.road_spans() {
        let d2 = point_segment_distance_sq(
            world.x,
            world.z,
            a.x as f32 + 0.5,
            a.z as f32 + 0.5,
            b.x as f32 + 0.5,
            b.z as f32 + 0.5,
        );
        if best.map_or(true, |(bd, ..)| d2 < bd) {
            best = Some((d2, seg.head(), seg.tail(), seg.kind.road_tier().unwrap_or_default()));
        }
    }
    best.map(|(_, h, t, tier)| (h, t, tier))
}

/// The next built tier up (a worn track becomes a dirt road; concrete stays concrete: motorways
/// are their own build).
#[must_use]
pub fn next_road_tier(tier: RoadTier) -> RoadTier {
    match tier {
        RoadTier::Track => RoadTier::Dirt,
        RoadTier::Dirt => RoadTier::Gravel,
        RoadTier::Gravel => RoadTier::Paved,
        RoadTier::Paved | RoadTier::Concrete => RoadTier::Concrete,
        RoadTier::Motorway => RoadTier::Motorway,
    }
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
    roads: &CorridorNetwork,
    queue: &mut ConstructionPlanQueue,
    placement: &mut ActiveRoadPlacement,
    params: &WorldGenParams,
) -> bool {
    let Some((head, tail, tier)) = nearest_executed_segment(world, roads) else {
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
        entity_type: ConstructionType::RoadSegment { head, tail, tier: next_road_tier(tier) },
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
    fn nearest_segment_finds_the_line_and_its_tier() {
        let mut roads = CorridorNetwork::default();
        let (h, t) = (BuildSiteTile { x: 2, z: 2 }, BuildSiteTile { x: 5, z: 2 });
        roads.add_segment(crate::construction::CorridorKind::Road { tier: RoadTier::Dirt, lanes: 1 }, h, t);
        let seg = nearest_executed_segment(Vec3::new(3.5, 0.0, 2.5), &roads);
        assert_eq!(seg, Some((h, t, RoadTier::Dirt)));
        assert_eq!(next_road_tier(RoadTier::Dirt), RoadTier::Gravel);
    }
}
