//! Fire ecology witness — `debug_runs/fire_ecology_live.json` (DEV-CONTAIN-004).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::systems::fire::witness_collectors::{build_fire_ecology_proof_payload, FireEcologyWitness};

use super::io::write_enveloped_witness;

pub const FIRE_ECOLOGY_JSON: &str = "debug_runs/fire_ecology_live.json";

#[derive(Resource, Debug)]
pub struct FireEcologyLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for FireEcologyLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 90,
            written: false,
        }
    }
}

#[must_use]
pub fn commit_fire_ecology_live_proof(witness: &FireEcologyWitness) -> bool {
    write_enveloped_witness(
        "FIRE_ECOLOGY_F1",
        "fire_ecology_live_proof",
        FIRE_ECOLOGY_JSON,
        build_fire_ecology_proof_payload(witness),
    )
}

pub fn write_fire_ecology_live_proof_system(
    base: Option<Res<State<BaseState>>>,
    mut state: ResMut<FireEcologyLiveProofState>,
    mut witness: ResMut<FireEcologyWitness>,
) {
    if !matches!(base.as_deref().map(|s| s.get()), Some(BaseState::Simulation)) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;

    witness.proof_json = true;
    if commit_fire_ecology_live_proof(witness.as_ref()) {
        state.written = true;
    }
}
