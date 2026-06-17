//! CDR-B E5 resolver gate — P0 parity + P1 consumer / tile / map-stamp witnesses.

pub const CODER_B_E5_RESOLVER_GATE_LIVE_JSON: &str =
    "debug_runs/coder_b_e5_resolver_gate_live.json";

#[must_use]
pub fn refresh_coder_b_e5_resolver_gate_witnesses() -> bool {
    let parity =
        crate::dev::veg_resolver_parity_live_proof::refresh_veg_resolver_parity_live_witness();
    let build_consumer =
        crate::dev::aps_dna_consumer_live_proof::refresh_aps_dna_consumer_rust_live_witness();
    let tile_resolver =
        crate::construction::procedural::landscape_tile_resolver_witness_green();
    let map_stamp =
        crate::dev::landscape_map_stamp_contract_live_proof::refresh_landscape_map_stamp_contract_live_witness(
        );

    let green = parity && build_consumer && tile_resolver && map_stamp;
    let body = serde_json::json!({
        "gate": "CDR-B-E5-RESOLVER-GATE-001",
        "green": green,
        "charter": "src/dev/plan_aps_veg_parity_engine_authority_v1.md",
        "slices": {
            "CDR-B-VEG-RESOLVER-PARITY-001": parity,
            "CDR-B-BUILD-CONSUMER-MCP-001": build_consumer,
            "CDR-B-TILE-RESOLVER-VEG-001": tile_resolver,
            "CDR-B-MAP-STAMP-CONTRACT-001": map_stamp,
        },
        "witness_paths": {
            "parity": "debug_runs/art_pipeline/veg_resolver_parity_live.json",
            "build_consumer": "debug_runs/aps_dna_consumer_rust_live.json",
            "map_stamp": "debug_runs/landscape_map_stamp_contract_live.json",
        },
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CDR-B-E5-RESOLVER-GATE-001",
        "refresh_coder_b_e5_resolver_gate_witnesses",
        CODER_B_E5_RESOLVER_GATE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(CODER_B_E5_RESOLVER_GATE_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coder_b_e5_resolver_gate_witness_bundle_green() {
        assert!(refresh_coder_b_e5_resolver_gate_witnesses());
    }
}
