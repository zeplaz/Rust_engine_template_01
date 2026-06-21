//! **VEG-SHIP-CLOSE-001** — vegetation / landscape program close witness (T3).

pub const VEG_SHIP_CLOSE_JSON: &str = "debug_runs/veg_ship_close_live.json";

#[must_use]
pub fn refresh_veg_ship_close_witness() -> bool {
    use crate::dev::landscape_grammar_burn_live_proof::refresh_veg_burn_chain_live_witness;
    use crate::dev::minimap_topology_legend_live_proof::refresh_minimap_topology_legend_live_witness;
    use crate::dev::veg_resolver_parity_live_proof::refresh_veg_resolver_parity_live_witness;
    use crate::dev::vegetation_snapshot_roundtrip_live_proof::refresh_vegetation_snapshot_roundtrip_live_witness;
    use crate::dev::vegetation_variant_catalog_load_live_proof::refresh_vegetation_variant_catalog_load_live_witness;

    let catalog_ok = refresh_vegetation_variant_catalog_load_live_witness();
    let parity_ok = refresh_veg_resolver_parity_live_witness();
    let burn_ok = refresh_veg_burn_chain_live_witness();
    let minimap_ok = refresh_minimap_topology_legend_live_witness();
    let snapshot_ok = refresh_vegetation_snapshot_roundtrip_live_witness();
    let program_close_ok = std::path::Path::new("debug_runs/vegetation_program_close_live.json").is_file()
        && std::fs::read_to_string("debug_runs/vegetation_program_close_live.json")
            .ok()
            .and_then(|text| {
                serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.get("all_green").and_then(|g| g.as_bool()))
            })
            == Some(true);

    let green = catalog_ok && parity_ok && burn_ok && minimap_ok && snapshot_ok && program_close_ok;

    let body = serde_json::json!({
        "gate": "VEG-SHIP-CLOSE-001",
        "program_closed": green,
        "catalog_load": catalog_ok,
        "resolver_parity": parity_ok,
        "burn_chain": burn_ok,
        "minimap_topology_legend": minimap_ok,
        "snapshot_roundtrip": snapshot_ok,
        "vegetation_program_close": program_close_ok,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VEG-SHIP-CLOSE-001",
        "refresh_veg_ship_close_witness",
        VEG_SHIP_CLOSE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(VEG_SHIP_CLOSE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn veg_ship_close_witness_green() {
        assert!(refresh_veg_ship_close_witness());
    }
}
