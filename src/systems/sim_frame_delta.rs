//! Deterministic sim cadence helpers — frame deltas, replay ring, replication interest.
//!
//! Ref-books **LOOP-03** · LEGACY `game_loop` / `time_measurement` (GEA 4e I §8.5.5 / §8.5,
//! gea_v1_e4 p522 / p514): after a debugger hitch or load stall, wall-clock dt must be
//! **clamped** before it advances sim time — see [`clamp_sim_frame_delta_secs`]. Running
//! averages of frame dt are **display-only** ([`DisplayFrameDtAverage`]); never feed them
//! into [`crate::systems::sim_control::advance_sim_tick`].
//!
//! Replay / scrub stamps stay on **u64** [`SimStepStamp`] ticks (LOOP-04 / LOOP-01) — not f32
//! clocks.

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::systems::sim_control::{SimStepStamp, SIM_STEP_MAX_PER_FRAME};

/// Max wall-clock seconds that may contribute to sim time in one Update (GEA 8.5.5).
///
/// Aligned with [`SIM_STEP_MAX_PER_FRAME`] so a 1× hitch cannot inject more sim time than the
/// fixed-step catch-up budget already allows.
pub const SIM_FRAME_DT_CLAMP_SECS: f32 = SIM_STEP_MAX_PER_FRAME as f32;

/// Clamp a raw Bevy/`Time` delta for **sim advance** after stalls, loads, or breakpoints.
#[inline]
#[must_use]
pub fn clamp_sim_frame_delta_secs(raw_secs: f32) -> f32 {
    if !raw_secs.is_finite() {
        return 0.0;
    }
    raw_secs.clamp(0.0, SIM_FRAME_DT_CLAMP_SECS)
}

/// Display-only exponential moving average of wall-clock frame dt (GEA 8.5.2.3).
///
/// Never use [`Self::secs`] to drive gameplay clocks — sim truth is u64 ticks / micros.
#[derive(Resource, Debug, Clone, Copy)]
pub struct DisplayFrameDtAverage {
    pub secs: f32,
    pub alpha: f32,
}

impl Default for DisplayFrameDtAverage {
    fn default() -> Self {
        Self {
            secs: 0.0,
            alpha: 0.1,
        }
    }
}

impl DisplayFrameDtAverage {
    /// Fold one raw wall-clock sample (unclamped is fine — this is HUD/debug only).
    pub fn push_sample(&mut self, raw_secs: f32) {
        if !raw_secs.is_finite() || raw_secs < 0.0 {
            return;
        }
        if self.secs <= 0.0 {
            self.secs = raw_secs;
            return;
        }
        let a = self.alpha.clamp(0.0, 1.0);
        self.secs = self.secs * (1.0 - a) + raw_secs * a;
    }
}

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
#[derive(Resource, Debug, Clone)]
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

impl Default for CommittedSimReplayRing {
    fn default() -> Self {
        // Runtime default must hold at least a short replay window so minimap scrub can activate.
        Self::with_capacity(64)
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
            .init_resource::<DisplayFrameDtAverage>()
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

    #[test]
    fn long_frame_dt_clamps_to_catch_up_budget() {
        assert_eq!(clamp_sim_frame_delta_secs(0.016), 0.016);
        assert_eq!(
            clamp_sim_frame_delta_secs(120.0),
            SIM_FRAME_DT_CLAMP_SECS
        );
        assert_eq!(clamp_sim_frame_delta_secs(-1.0), 0.0);
        assert_eq!(clamp_sim_frame_delta_secs(f32::INFINITY), 0.0);
        assert_eq!(SIM_FRAME_DT_CLAMP_SECS, SIM_STEP_MAX_PER_FRAME as f32);
    }

    #[test]
    fn display_dt_average_is_display_only_smoother() {
        let mut avg = DisplayFrameDtAverage {
            secs: 0.0,
            alpha: 0.5,
        };
        avg.push_sample(0.010);
        avg.push_sample(0.030);
        assert!((avg.secs - 0.020).abs() < 1e-6);
    }
}
