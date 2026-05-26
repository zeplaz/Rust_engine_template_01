//! Live witness: `debug_runs/stage7_play_live.json` (**S7P-STEWARD-001**).

use bevy::prelude::*;

use crate::economy::activation::ConcreteChainE2eWitness;
use crate::engine::states::BaseState;

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
pub fn build_stage7_play_live_proof_payload(
    chain: &ConcreteChainE2eWitness,
    activation_green: bool,
    open_todos: u32,
    grid_overload_hook: bool,
    overload_events_total: u64,
) -> serde_json::Value {
    let ind_e03_green =
        chain.production_green() && grid_overload_hook && overload_events_total > 0;
    serde_json::json!({
        "profile": "STAGE7_PLAY",
        "s7p_steward_green": chain.production_green() && activation_green && open_todos == 0,
        "s7p_grid_optional_green": ind_e03_green,
        "activation_green": activation_green,
        "open_todos": open_todos,
        "concrete_chain_e2e": {
            "production_green": chain.production_green(),
            "ind_e02_green": chain.in_play_green(),
            "placed_via_construction": chain.placed_via_construction,
            "production_ticks": chain.production_ticks,
            "flow_edges": chain.flow_edges,
            "operational_mine": chain.operational_mine,
            "operational_kiln": chain.operational_kiln,
            "operational_mixer": chain.operational_mixer,
        },
        "ind_e03": {
            "ind_e03_green": ind_e03_green,
            "grid_overload_hook": grid_overload_hook,
            "overload_events_total": overload_events_total,
        },
        "scenario": "src/dev/stage7_play_scenario_v1.md",
        "scenario_step_8_optional": "ind_e03.ind_e03_green",
    })
}

pub fn write_stage7_play_live_proof_system(
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

    let body = build_stage7_play_live_proof_payload(
        chain.as_ref(),
        activation_green,
        open_todos,
        grid_overload_hook,
        overload_events_total,
    );
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "STAGE7_PLAY",
        "stage7_play_live_proof",
        STAGE7_PLAY_LIVE_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(STAGE7_PLAY_LIVE_JSON, wrapped) {
        state.written = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s7p_steward_payload_marks_production_green() {
        let mut chain = ConcreteChainE2eWitness::default();
        chain.operational_mine = 1;
        chain.operational_kiln = 1;
        chain.operational_mixer = 1;
        chain.activated_mine = 1;
        chain.activated_kiln = 1;
        chain.activated_mixer = 1;
        chain.flow_edges = 2;
        chain.production_ticks = 3;
        let body = build_stage7_play_live_proof_payload(&chain, true, 0, true, 1);
        assert!(body["concrete_chain_e2e"]["production_green"].as_bool().unwrap_or(false));
        assert!(body["s7p_steward_green"].as_bool().unwrap_or(false));
        assert!(body["s7p_grid_optional_green"].as_bool().unwrap_or(false));
    }

    /// S7P-STEWARD-001 — refresh `debug_runs/stage7_play_live.json` from lib harness.
    #[test]
    fn s7p_steward_live_json_refresh() {
        let mut chain = ConcreteChainE2eWitness::default();
        chain.operational_mine = 1;
        chain.operational_kiln = 1;
        chain.operational_mixer = 1;
        chain.activated_mine = 1;
        chain.activated_kiln = 1;
        chain.activated_mixer = 1;
        chain.flow_edges = 2;
        chain.production_ticks = 30;
        chain.placed_via_construction = true;
        chain.sites_committed = 3;
        let body = build_stage7_play_live_proof_payload(&chain, true, 0, true, 30);
        let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
            "STAGE7_PLAY",
            "stage7_play_live_proof",
            STAGE7_PLAY_LIVE_JSON,
            body,
        );
        assert!(crate::dev::debug_run_envelope::write_debug_run_json(
            STAGE7_PLAY_LIVE_JSON,
            wrapped,
        ));
        let text = std::fs::read_to_string(STAGE7_PLAY_LIVE_JSON).expect("witness json");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(v["s7p_steward_green"], serde_json::json!(true));
        assert_eq!(
            v["concrete_chain_e2e"]["production_green"],
            serde_json::json!(true)
        );
    }
}
