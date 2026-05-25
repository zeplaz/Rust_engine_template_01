//! E1 committed snapshot fence — render-visible extracts publish stamp before Update consumers run.

use bevy::prelude::*;

use crate::render::FireSimulationSnapshot;
use crate::render::Stage5ReadinessProfile;
use crate::systems::sim_control::SimStepStamp;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CommittedVisualSnapshotFence {
    pub fire: SimStepStamp,
}

pub fn commit_fire_visual_snapshot(
    sim: Res<FireSimulationSnapshot>,
    mut fence: ResMut<CommittedVisualSnapshotFence>,
    profile: Res<Stage5ReadinessProfile>,
) {
    let prev = fence.fire;
    fence.fire = sim.stamp;
    if *profile != Stage5ReadinessProfile::FULL_APP || prev == sim.stamp {
        return;
    }
    if !std::env::var("STAGE5_FENCE_VERBOSE")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        return;
    }
    info!(
        target: "stage5_fence::live",
        "STAGE5_FENCE_COMMIT fire_tick={} sim_time_micros={}",
        fence.fire.tick,
        fence.fire.sim_time_micros,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_fence_tracks_fire_visual_stamp() {
        let stamp = SimStepStamp::new(7, 42_000);
        let sim = FireSimulationSnapshot {
            stamp,
            instances: Vec::new(),
            chunk_heat: Vec::new(),
        };
        let fence = CommittedVisualSnapshotFence::default();
        let mut world = World::new();
        world.insert_resource(sim);
        world.insert_resource(fence);
        world.insert_resource(Stage5ReadinessProfile::default());
        let mut schedule = Schedule::default();
        schedule.add_systems(commit_fire_visual_snapshot);
        schedule.run(&mut world);
        let fence = world.resource::<CommittedVisualSnapshotFence>();
        assert_eq!(fence.fire, stamp);
    }
}
