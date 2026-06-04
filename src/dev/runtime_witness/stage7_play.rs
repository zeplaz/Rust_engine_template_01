//! Stage 7 play witness — `debug_runs/stage7_play_live.json` (DEV-CONTAIN-005).

use bevy::prelude::*;

use crate::dev::stage7_play_witness::build_stage7_play_witness_payload;
use crate::economy::activation::ConcreteChainE2eWitness;
use crate::engine::states::BaseState;

use super::io::{write_enveloped_witness, write_enveloped_witness_unchecked};

pub const STAGE7_PLAY_LIVE_JSON: &str = "debug_runs/stage7_play_live.json";

#[derive(Resource, Debug)]
pub struct Stage7PlayLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for Stage7PlayLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 120,
            written: false,
        }
    }
}

#[must_use]
pub fn commit_stage7_play_witness(
    chain: &ConcreteChainE2eWitness,
    activation_green: bool,
    open_todos: u32,
    grid_overload_hook: bool,
    overload_events_total: u64,
) -> bool {
    let body = build_stage7_play_witness_payload(
        chain,
        activation_green,
        open_todos,
        grid_overload_hook,
        overload_events_total,
    );
    write_enveloped_witness_unchecked(
        "STAGE7_PLAY",
        "stage7_play_witness",
        STAGE7_PLAY_LIVE_JSON,
        body,
    )
}

pub fn write_stage7_play_witness_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<Stage7PlayLiveProofState>,
    chain: Res<ConcreteChainE2eWitness>,
    board: Option<Res<crate::dev::IndustrialActivationTodoBoard>>,
    industrial_witness: Option<Res<crate::dev::IndustrialActivationWitness>>,
    flow: Option<Res<crate::economy::resource_flow::ResourceFlowSimWitness>>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;

    let open_todos = board
        .as_ref()
        .map(|b| {
            b.status
                .iter()
                .filter(|s| **s != crate::dev::TodoStatus::Done)
                .count() as u32
        })
        .unwrap_or(0);
    let activation_green = open_todos == 0;
    let grid_overload_hook = industrial_witness
        .as_deref()
        .is_some_and(|w| w.grid_overload_hook);
    let overload_events_total = flow
        .as_deref()
        .map(|f| f.overload_events_total)
        .unwrap_or(0);

    let body = build_stage7_play_witness_payload(
        chain.as_ref(),
        activation_green,
        open_todos,
        grid_overload_hook,
        overload_events_total,
    );
    if write_enveloped_witness(
        "STAGE7_PLAY",
        "stage7_play_witness",
        STAGE7_PLAY_LIVE_JSON,
        body,
    ) {
        state.written = true;
    }
}
