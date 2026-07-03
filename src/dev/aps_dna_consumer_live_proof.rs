//! BUILD-READ-CONSUMER-MCP-001 — APS snapshot DNA+β consumer witness (Rust commit path).

const LIVE_JSON: &str = "debug_runs/aps_dna_consumer_rust_live.json";

#[must_use]
pub fn aps_dna_consumer_rust_witness_green() -> bool {
    crate::construction::procedural::build_read_consumer_mcp_001_witness_green()
}

#[must_use]
pub fn refresh_aps_dna_consumer_rust_live_witness() -> bool {
    use crate::construction::PilotCatalog;

    let preset_id = PilotCatalog::load_from_disk()
        .first_grammar_arch_dna_preset_id()
        .unwrap_or_default();
    let consumer = crate::construction::procedural::arch_dna_consumer_from_preset_id(&preset_id);
    let green = aps_dna_consumer_rust_witness_green();
    let body = serde_json::json!({
        "gate_id": "BUILD-READ-CONSUMER-MCP-001",
        "task_id": "BUILD-READ-CONSUMER-MCP-001",
        "slice_id": "CDR-B-BUILD-CONSUMER-MCP-001",
        "green": green,
        "mcp_contract": "debug_runs/aps_dna_consumer_contract_live.json",
        "rust_consumer": "src/construction/procedural/arch_build_grammar_v0.rs::load_arch_dna_preset",
        "preset_id": preset_id,
        "consumer_ok": consumer.is_ok(),
        "consumer_wired": consumer.as_ref().map_or(false, |c| {
            crate::construction::procedural::arch_dna_consumer_wired(c)
        }),
        "commit_path": "parametric_commit::procedural_building_request_from_commit + footprint_grid_for_assembly",
        "snapshot_fields": [
            "arch_build_grammar_preset_id",
            "arch_build_grammar_id",
            "arch_dna",
            "pressure_field"
        ],
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "BUILD-READ-CONSUMER-MCP-001",
        "refresh_aps_dna_consumer_rust_live_witness",
        LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aps_dna_consumer_rust_live_witness_green() {
        assert!(refresh_aps_dna_consumer_rust_live_witness());
    }
}
