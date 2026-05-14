//! E1 committed snapshot fence — render-visible extracts publish stamp before Update consumers run.

use bevy::prelude::*;

use crate::render::sim_visual_extract::FireVisualFrame;
use crate::systems::sim_control::SimStepStamp;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommittedVisualSnapshotFence {
    pub fire: SimStepStamp,
}

pub fn commit_fire_visual_snapshot(
    fire: Res<FireVisualFrame>,
    mut fence: ResMut<CommittedVisualSnapshotFence>,
) {
    fence.fire = fire.stamp;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_fence_tracks_fire_visual_stamp() {
        let stamp = SimStepStamp::new(7, 42_000);
        let fire = FireVisualFrame {
            stamp,
            instances: Vec::new(),
            chunk_heat: Vec::new(),
        };
        let fence = CommittedVisualSnapshotFence::default();
        let mut world = World::new();
        world.insert_resource(fire);
        world.insert_resource(fence);
        let mut schedule = Schedule::default();
        schedule.add_systems(commit_fire_visual_snapshot);
        schedule.run(&mut world);
        let fence = world.resource::<CommittedVisualSnapshotFence>();
        assert_eq!(fence.fire, stamp);
    }
}
