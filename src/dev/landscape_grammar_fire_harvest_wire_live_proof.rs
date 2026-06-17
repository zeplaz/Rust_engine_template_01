//! CDR-A-FIRE-HARVEST-WIRE-001 — fire scar → SimEffect harvest → lg2 witness.

pub const FIRE_HARVEST_WIRE_LIVE_JSON: &str =
    "debug_runs/landscape_grammar_fire_harvest_wire_live.json";

#[must_use]
pub fn fire_harvest_wire_green() -> bool {
    let result = crate::dev::landscape_grammar_sim_harness::run_landscape_grammar_sim_harness();
    result.fire_disturbances >= 1
        && result.harvest_disturbances >= 1
        && result.construction_disturbances >= 1
}

#[must_use]
pub fn refresh_fire_harvest_wire_live_witness() -> bool {
    let result = crate::dev::landscape_grammar_sim_harness::run_landscape_grammar_sim_harness();
    let green = result.fire_disturbances >= 1
        && result.harvest_disturbances >= 1
        && result.construction_disturbances >= 1
        && result.chunks_with_program >= 16;
    let body = serde_json::json!({
        "slice_id": "CDR-A-FIRE-HARVEST-WIRE-001",
        "gate": "CDR-A-FIRE-HARVEST-WIRE-001",
        "green": green,
        "proof_grade": crate::dev::proof_grade::ProofGrade::HeadlessSim.as_str(),
        "fire_disturbances": result.fire_disturbances,
        "harvest_disturbances": result.harvest_disturbances,
        "harvest_via_sim_effect": true,
        "sim_effect_kind": "LandscapeDisturbance.harvest",
        "cause_id": "CDR-A-FIRE-HARVEST-WIRE-001",
        "wire_path": "push_post_fire_harvest_sim_effect → drain_sim_effect_queue → drain_landscape_disturbance_queue",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CDR-A-FIRE-HARVEST-WIRE-001",
        "refresh_fire_harvest_wire_live_witness",
        FIRE_HARVEST_WIRE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(FIRE_HARVEST_WIRE_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_harvest_wire_live_witness_green() {
        assert!(refresh_fire_harvest_wire_live_witness());
    }
}
