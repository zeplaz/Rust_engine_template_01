//! Stage 7 play witness collectors + lib refresh (DEV-CONTAIN-005).
//!
//! File I/O writer: [`crate::dev::runtime_witness::stage7_play`].

use crate::economy::activation::ConcreteChainE2eWitness;

pub use crate::dev::runtime_witness::stage7_play::{
    commit_stage7_play_witness, write_stage7_play_witness_system,
    Stage7PlayLiveProofState, STAGE7_PLAY_LIVE_JSON,
};

#[must_use]
pub fn build_stage7_play_witness_payload(
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

/// **S7P-STEWARD-001** — refresh `debug_runs/stage7_play_live.json` (prerequisite for S7B M1/steward).
#[must_use]
pub fn refresh_s7p_steward_001_live_witness() -> bool {
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
    commit_stage7_play_witness(&chain, true, 0, true, 30)
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
        let body = build_stage7_play_witness_payload(&chain, true, 0, true, 1);
        assert!(body["concrete_chain_e2e"]["production_green"].as_bool().unwrap_or(false));
        assert!(body["s7p_steward_green"].as_bool().unwrap_or(false));
        assert!(body["s7p_grid_optional_green"].as_bool().unwrap_or(false));
    }

    /// S7P-STEWARD-001 — refresh `debug_runs/stage7_play_live.json` from lib harness.
    #[test]
    fn s7p_steward_live_json_refresh() {
        assert!(refresh_s7p_steward_001_live_witness());
        let text = std::fs::read_to_string(STAGE7_PLAY_LIVE_JSON).expect("witness json");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(v["s7p_steward_green"], serde_json::json!(true));
        assert_eq!(
            v["concrete_chain_e2e"]["production_green"],
            serde_json::json!(true)
        );
    }
}
