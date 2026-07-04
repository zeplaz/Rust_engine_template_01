//! Fire ecology witness — `debug_runs/fire_ecology_live.json` (DEV-CONTAIN-004).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::systems::fire::witness_collectors::{build_fire_ecology_proof_payload, FireEcologyWitness};

use super::common::WitnessWriteCadence;
use super::io::write_enveloped_witness;

pub const FIRE_ECOLOGY_JSON: &str = "debug_runs/fire_ecology_live.json";

#[derive(Resource, Debug)]
pub struct FireEcologyLiveProofState {
    pub cadence: WitnessWriteCadence,
}

impl Default for FireEcologyLiveProofState {
    fn default() -> Self {
        Self {
            cadence: WitnessWriteCadence {
                write_interval: 90,
                ..Default::default()
            },
        }
    }
}

impl FireEcologyLiveProofState {
    #[must_use]
    pub fn written(&self) -> bool {
        self.cadence.written
    }

    pub fn set_written(&mut self, value: bool) {
        self.cadence.written = value;
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

    witness.proof_json = true;
    if commit_fire_ecology_live_proof(witness.as_ref()) {
        state.set_written(true);
    }
}
