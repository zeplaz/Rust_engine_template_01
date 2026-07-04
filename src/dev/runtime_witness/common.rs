//! Shared cadence state for periodic witness refresh systems.

use bevy::prelude::*;

/// Frames-between-writes cadence used by multiple live-proof systems.
#[derive(Resource, Debug, Clone)]
pub struct LiveProofCadence {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
    /// Set each frame by [`arm_live_proof_cadence`]; read by `.run_if(live_proof_cadence_due)`.
    pub write_due: bool,
}

impl Default for LiveProofCadence {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 120,
            written: false,
            write_due: false,
        }
    }
}

/// Advance cadence; returns `true` when a write tick is due.
#[must_use]
pub fn tick_live_proof_cadence(state: &mut LiveProofCadence) -> bool {
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return false;
    }
    state.frames_since_write = 0;
    true
}

/// Arm global [`LiveProofCadence`] (MIG-A7).
pub fn arm_live_proof_cadence(state: &mut LiveProofCadence) {
    state.write_due = tick_live_proof_cadence(state);
}

/// Run condition for wave_c / wave_s / stage6 writers sharing [`LiveProofCadence`].
#[must_use]
pub fn live_proof_cadence_due(state: Res<LiveProofCadence>) -> bool {
    state.write_due
}

/// Arm inline cadence fields on bespoke witness state resources.
pub fn arm_witness_write_cadence(state: &mut WitnessWriteCadence) {
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    state.write_due = if state.frames_since_write >= state.write_interval {
        state.frames_since_write = 0;
        true
    } else {
        false
    };
}

/// Inline cadence fields shared by domain-specific live-proof resources.
#[derive(Debug, Clone, Default)]
pub struct WitnessWriteCadence {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
    pub write_due: bool,
}

impl WitnessWriteCadence {
    #[must_use]
    pub fn written(&self) -> bool {
        self.written
    }
}

/// Deprecated — prefer [`live_proof_cadence_due`] on [`LiveProofCadence`].
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct LiveProofWriteLatch(pub bool);

/// Deprecated — use [`arm_live_proof_cadence`].
pub fn arm_live_proof_cadence_with_latch(
    state: &mut LiveProofCadence,
    latch: &mut LiveProofWriteLatch,
) {
    latch.0 = tick_live_proof_cadence(state);
    state.write_due = latch.0;
}

/// Deprecated — use [`live_proof_cadence_due`].
#[must_use]
pub fn live_proof_write_latched(latch: Res<LiveProofWriteLatch>) -> bool {
    latch.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_proof_cadence_arms_write_due_on_interval() {
        let mut cadence = LiveProofCadence {
            write_interval: 3,
            ..Default::default()
        };
        arm_live_proof_cadence(&mut cadence);
        assert!(!cadence.write_due);
        arm_live_proof_cadence(&mut cadence);
        assert!(!cadence.write_due);
        arm_live_proof_cadence(&mut cadence);
        assert!(cadence.write_due);
    }

    #[test]
    fn witness_write_cadence_helper_matches_interval() {
        let mut cadence = WitnessWriteCadence {
            write_interval: 2,
            ..Default::default()
        };
        arm_witness_write_cadence(&mut cadence);
        assert!(!cadence.write_due);
        arm_witness_write_cadence(&mut cadence);
        assert!(cadence.write_due);
    }
}
