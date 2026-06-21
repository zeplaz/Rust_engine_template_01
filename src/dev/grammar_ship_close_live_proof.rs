//! **GRAMMAR-SHIP-CLOSE-001** — grammar program close witness (T2).

pub const GRAMMAR_SHIP_CLOSE_JSON: &str = "debug_runs/agent_ops/grammar_ship_close_live.json";

#[must_use]
pub fn refresh_grammar_ship_close_witness() -> bool {
    use crate::construction::building_set::building_set_coverage_witness_green;
    use crate::construction::pilot_catalog_parity_witness_green;
    use crate::construction::procedural::{
        facility_binding_read_witness_green, load_building_grammar_registry,
    };

    let pilot_ok = pilot_catalog_parity_witness_green();
    let set_ok = building_set_coverage_witness_green();
    let registry = load_building_grammar_registry();
    let grammar_count_ok =
        registry.load_errors.is_empty() && registry.grammars.len() >= 4;
    let g1_ok = facility_binding_read_witness_green();
    let green = pilot_ok && set_ok && grammar_count_ok && g1_ok;

    let body = serde_json::json!({
        "gate": "GRAMMAR-SHIP-CLOSE-001",
        "program_closed": green,
        "pilot_catalog_parity": pilot_ok,
        "building_set_coverage": set_ok,
        "grammar_registry_count": registry.grammars.len(),
        "grammar_count_ok": grammar_count_ok,
        "facility_binding_g1": g1_ok,
        "civic_block_ron_on_disk": std::path::Path::new(
            "assets/configs/buildings/grammars/civic_block_v1.ron",
        )
        .is_file(),
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "GRAMMAR-SHIP-CLOSE-001",
        "refresh_grammar_ship_close_witness",
        GRAMMAR_SHIP_CLOSE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(GRAMMAR_SHIP_CLOSE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_ship_close_witness_green() {
        assert!(refresh_grammar_ship_close_witness());
    }
}
