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
