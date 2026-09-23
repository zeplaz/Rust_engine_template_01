//! **EFFECTS-SYSTEM-ES-7** — Hanabi L6 contract witness (`debug_runs/effects_system_es7_live.json`).
//!
//! Scoped green: mapping + gates + no-writeback. `particles_rendered: false` until ES-7-4 capture.

use crate::render::witness::hanabi_witness::{
    hanabi_es7_contract_json, hanabi_l3_plugin_wired, hanabi_minimap_bleed_free,
    hanabi_no_sim_writeback, hanabi_spike_report_present,
};
use crate::render::fx_spine::hanabi_embellishment::hanabi_burst_consumer_present;

pub const EFFECTS_SYSTEM_ES7_LIVE_JSON: &str = "debug_runs/effects_system_es7_live.json";

#[must_use]
pub fn effects_system_es7_witness_body() -> serde_json::Value {
    let contract = hanabi_es7_contract_json();
    let spike = hanabi_spike_report_present();
    let consumer = hanabi_burst_consumer_present();
    let no_wb = hanabi_no_sim_writeback();
    let no_bleed = hanabi_minimap_bleed_free();
    let wired = hanabi_l3_plugin_wired();
    let mapped_lt = contract["mapped_spawns_localtactical"].as_u64().unwrap_or(0);
    let mapped_op = contract["mapped_spawns_operational"].as_u64().unwrap_or(0);
    let peak = contract["peak_instances_cap"].as_u64().unwrap_or(99);

    let green = spike
        && consumer
        && no_wb
        && no_bleed
        && !wired
        && mapped_lt > 0
        && mapped_op == 0
        && peak <= 20
        && contract["particles_rendered"] == false;

    serde_json::json!({
        "program_id": "EFFECTS-SYSTEM-UNIFY-001",
        "phase": "ES-7",
        "slice": "ES-7-001",
        "status": if green { "done" } else { "failed" },
        "green": green,
        "hanabi_no_sim_writeback": no_wb,
        "hanabi_minimap_bleed_free": no_bleed,
        "burst_hint_consumer_present": consumer,
        "hanabi_l3_wired": wired,
        "particles_rendered": false,
        "contract": contract,
        "deferred": {
            "slice": "ES-7-4",
            "note": "Emitter visual capture needs --features hanabi_l3 + RUST_ENGINE_HANABI_L3=1 + operator eyes",
        },
        "plan_note": "ES-7-1..3 contract closed. Hanabi ≠ weather. Parallel OK with GPU precip.",
    })
}

#[must_use]
pub fn refresh_effects_system_es7_live_witness() -> bool {
    let body = effects_system_es7_witness_body();
    if !body.get("green").and_then(|v| v.as_bool()).unwrap_or(false) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "EFFECTS-SYSTEM-ES-7-001",
        "refresh_effects_system_es7_live_witness",
        EFFECTS_SYSTEM_ES7_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(EFFECTS_SYSTEM_ES7_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_system_es7_contract_witness_green() {
        assert!(refresh_effects_system_es7_live_witness());
        let body = effects_system_es7_witness_body();
        assert_eq!(body["hanabi_no_sim_writeback"], true);
        assert_eq!(body["particles_rendered"], false);
        assert_eq!(body["hanabi_l3_wired"], false);
    }
}
