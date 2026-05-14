//! Deterministic sim cadence helpers — frame deltas, replay ring, replication interest.

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::systems::sim_control::SimStepStamp;

/// Monotonic delta between two committed sim steps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SimFrameDelta {
    pub from: SimStepStamp,
    pub to: SimStepStamp,
}

impl SimFrameDelta {
    #[must_use]
    pub fn new(from: SimStepStamp, to: SimStepStamp) -> Self {
        Self { from, to }
    }

    #[must_use]
    pub fn tick_span(&self) -> u64 {
        self.to.tick.saturating_sub(self.from.tick)
    }
}

/// Ring buffer of committed visual stamps for replay / scrub / rewind hooks.
#[derive(Resource, Debug, Clone, Default)]
pub struct CommittedSimReplayRing {
    pub stamps: VecDeque<SimStepStamp>,
    pub capacity: usize,
    pub last_delta: SimFrameDelta,
}

impl CommittedSimReplayRing {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            stamps: VecDeque::new(),
            capacity: capacity.max(1),
            last_delta: SimFrameDelta::default(),
        }
    }

    pub fn record_commit(&mut self, stamp: SimStepStamp) {
        let from = self.stamps.back().copied().unwrap_or(stamp);
        self.last_delta = SimFrameDelta::new(from, stamp);
        if self.stamps.back().is_some_and(|prev| *prev == stamp) {
            return;
        }
        self.stamps.push_back(stamp);
        while self.stamps.len() > self.capacity {
            self.stamps.pop_front();
        }
    }

    #[must_use]
    pub fn latest(&self) -> Option<SimStepStamp> {
        self.stamps.back().copied()
    }
}

/// Interest-managed replication policy stub (distance + simulation weight).
#[derive(Resource, Clone, Copy, Debug)]
pub struct ReplicationInterestPolicy {
    pub max_tick_lag: u64,
    pub distance_weight: f32,
    pub simulation_weight: f32,
}

impl Default for ReplicationInterestPolicy {
    fn default() -> Self {
        Self {
            max_tick_lag: 4,
            distance_weight: -0.5,
            simulation_weight: 1.25,
        }
    }
}

impl ReplicationInterestPolicy {
    #[must_use]
    pub fn replication_score(self, tick_lag: u64, distance: f32, sim_importance: f32) -> f32 {
        if tick_lag > self.max_tick_lag {
            return f32::NEG_INFINITY;
        }
        self.distance_weight * distance + self.simulation_weight * sim_importance
    }
}

pub fn record_committed_sim_replay_stamp(
    fence: Option<Res<crate::render::CommittedVisualSnapshotFence>>,
    mut replay: ResMut<CommittedSimReplayRing>,
) {
    let Some(fence) = fence else {
        return;
    };
    if fence.fire.tick == 0 && fence.fire.sim_time_micros == 0 {
        return;
    }
    replay.record_commit(fence.fire);
}

pub struct SimFrameDeltaPlugin;

impl Plugin for SimFrameDeltaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommittedSimReplayRing>()
            .init_resource::<ReplicationInterestPolicy>()
            .add_systems(
                Update,
                record_committed_sim_replay_stamp
                    .after(crate::render::extraction::FireVisualFrameSet::BuildProfiles),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_ring_records_monotonic_deltas() {
        let mut ring = CommittedSimReplayRing::with_capacity(4);
        let a = SimStepStamp::new(1, 1_000);
        let b = SimStepStamp::new(2, 2_000);
        ring.record_commit(a);
        ring.record_commit(b);
        assert_eq!(ring.latest(), Some(b));
        assert_eq!(ring.last_delta.tick_span(), 1);
    }

    #[test]
    fn replication_interest_rejects_stale_ticks() {
        let policy = ReplicationInterestPolicy {
            max_tick_lag: 2,
            ..Default::default()
        };
        assert!(policy.replication_score(3, 1.0, 1.0).is_infinite());
        assert!(policy.replication_score(1, 1.0, 1.0).is_finite());
    }
}
