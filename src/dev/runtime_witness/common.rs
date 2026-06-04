//! Shared cadence state for periodic witness refresh systems.

use bevy::prelude::*;

/// Frames-between-writes cadence used by multiple live-proof systems.
#[derive(Resource, Debug, Clone)]
pub struct LiveProofCadence {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for LiveProofCadence {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 120,
            written: false,
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
