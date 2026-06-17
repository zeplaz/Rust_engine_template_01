//! CDR-B-MAP-STAMP-CONTRACT-001 — LG-5 map-stamp contract witness (APS E5 consumer parity).

pub const LANDSCAPE_MAP_STAMP_CONTRACT_LIVE_JSON: &str =
    "debug_runs/landscape_map_stamp_contract_live.json";

#[must_use]
pub fn landscape_map_stamp_contract_witness_green() -> bool {
    crate::gui::landscape_chunk_atlas_stamp::landscape_lg5_chunk_uv_stamp_witness_green()
        && crate::construction::procedural::landscape_tile_resolver_witness_green()
        && crate::systems::ecology::landscape_lg5_registry_stamped()
}

#[must_use]
pub fn refresh_landscape_map_stamp_contract_live_witness() -> bool {
    let stamp_ok =
        crate::gui::landscape_chunk_atlas_stamp::landscape_lg5_chunk_uv_stamp_witness_green();
    let resolver_ok = crate::construction::procedural::landscape_tile_resolver_witness_green();
    let registry_ok = crate::systems::ecology::landscape_lg5_registry_stamped();
    let green = stamp_ok && resolver_ok && registry_ok;
    let body = serde_json::json!({
        "gate": "CDR-B-MAP-STAMP-CONTRACT-001",
        "slice_id": "CDR-B-MAP-STAMP-CONTRACT-001",
        "green": green,
        "engine_authority": "landscape_chunk_atlas_stamp",
        "charter": "src/dev/plan_aps_veg_parity_engine_authority_v1.md",
        "atlas_domain": "landscape",
        "index_path": "assets/configs/landscape/_landscape_atlas_index.ron",
        "variant_key_source": "topology_kind_to_variant_key | veg_topo_* extract",
        "chunk_stamp_wired": stamp_ok,
        "resolver_wired": resolver_ok,
        "registry_stamped": registry_ok,
        "representation_result_is_building_only": true,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CDR-B-MAP-STAMP-CONTRACT-001",
        "refresh_landscape_map_stamp_contract_live_witness",
        LANDSCAPE_MAP_STAMP_CONTRACT_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(LANDSCAPE_MAP_STAMP_CONTRACT_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscape_map_stamp_contract_live_witness_green() {
        assert!(refresh_landscape_map_stamp_contract_live_witness());
    }
}
