//! Construction stage witness — `debug_runs/construction_stage_live.json` (DEV-CONTAIN-002).

use bevy::prelude::*;

use crate::construction::build_construction_stage_proof_payload;
use crate::construction::{ConstructionHistory, ConstructionPathFeedback, RailJunctionAuthority};
use crate::dev::construction_live_todos::ConstructionLiveTodoBoard;
use crate::dev::construction_operational_todos::{
    ConstructionOperationalTodoBoard, ConstructionOperationalWitness,
};
use crate::dev::construction_p9_todos::{ConstructionP9TodoBoard, ConstructionP9Witness};
use crate::dev::construction_phase2_todos::{
    ConstructionPhase2TodoBoard, ConstructionPhase2Witness,
};
use crate::dev::construction_round2_todos::ConstructionRound2TodoBoard;
use crate::dev::construction_round3_todos::ConstructionRound3TodoBoard;
use crate::engine::states::BaseState;

use super::io::write_enveloped_witness;

pub const CONSTRUCTION_STAGE_JSON: &str = "debug_runs/construction_stage_live.json";

#[derive(Resource, Debug)]
pub struct ConstructionLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for ConstructionLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 90,
            written: false,
        }
    }
}

#[must_use]
pub fn commit_construction_stage_live_proof(
    build_board: Option<&ConstructionLiveTodoBoard>,
    phase2_board: Option<&ConstructionPhase2TodoBoard>,
    phase2_witness: Option<&ConstructionPhase2Witness>,
    p9_board: Option<&ConstructionP9TodoBoard>,
    p9_witness: Option<&ConstructionP9Witness>,
    round2_board: Option<&ConstructionRound2TodoBoard>,
    round3_board: Option<&ConstructionRound3TodoBoard>,
    operational_board: Option<&ConstructionOperationalTodoBoard>,
    operational_witness: Option<&ConstructionOperationalWitness>,
    stage_witness: Option<&crate::construction::ConstructionStageWitness>,
    history: Option<&ConstructionHistory>,
    path_feedback: Option<&ConstructionPathFeedback>,
    junction_count: usize,
) -> bool {
    let body = build_construction_stage_proof_payload(
        build_board,
        phase2_board,
        phase2_witness,
        p9_board,
        p9_witness,
        true,
        round2_board,
        round3_board,
        operational_board,
        operational_witness,
        stage_witness,
        history,
        path_feedback,
        junction_count,
    );
    write_enveloped_witness(
        "CONSTRUCTION_STAGE",
        "construction_live_proof",
        CONSTRUCTION_STAGE_JSON,
        body,
    )
}

pub fn write_construction_live_proof_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<ConstructionLiveProofState>,
    build_board: Option<Res<ConstructionLiveTodoBoard>>,
    phase2_board: Option<Res<ConstructionPhase2TodoBoard>>,
    phase2_witness: Option<Res<ConstructionPhase2Witness>>,
    p9_board: Option<Res<ConstructionP9TodoBoard>>,
    p9_witness: Option<Res<ConstructionP9Witness>>,
    round2_board: Option<Res<ConstructionRound2TodoBoard>>,
    round3_board: Option<Res<ConstructionRound3TodoBoard>>,
    operational_board: Option<Res<ConstructionOperationalTodoBoard>>,
    operational_witness: Option<Res<ConstructionOperationalWitness>>,
    stage_witness: Option<Res<crate::construction::ConstructionStageWitness>>,
    history: Option<Res<ConstructionHistory>>,
    path_feedback: Option<Res<ConstructionPathFeedback>>,
    junctions: Option<Res<RailJunctionAuthority>>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;

    let operational_for_payload = operational_witness.as_deref().map(|w| {
        let mut snap = w.clone();
        snap.proof_json = true;
        snap
    });

    let junction_count = junctions.as_deref().map(|j| j.junctions.len()).unwrap_or(0);
    let p9_for_payload = p9_witness.as_deref().map(|w| {
        let mut snap = w.clone();
        snap.construction_proof_json = true;
        snap
    });

    if commit_construction_stage_live_proof(
        build_board.as_deref(),
        phase2_board.as_deref(),
        phase2_witness.as_deref(),
        p9_board.as_deref(),
        p9_for_payload.as_ref(),
        round2_board.as_deref(),
        round3_board.as_deref(),
        operational_board.as_deref(),
        operational_for_payload.as_ref(),
        stage_witness.as_deref(),
        history.as_deref(),
        path_feedback.as_deref(),
        junction_count,
    ) {
        state.written = true;
    }
}
