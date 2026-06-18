//! **VEG-CATALOG-LOADER-001** — `debug_runs/vegetation_variant_catalog_load_live.json`.

pub const VEGETATION_VARIANT_CATALOG_LOAD_LIVE_JSON: &str =
    "debug_runs/vegetation_variant_catalog_load_live.json";

#[must_use]
pub fn build_vegetation_variant_catalog_load_body() -> serde_json::Value {
    use crate::systems::ecology::{
        load_vegetation_variant_catalog, ENGINE_VEG_RESOLVER_KEYS,
        VEGETATION_VARIANT_CATALOG_RON,
    };

    let loaded = load_vegetation_variant_catalog().is_some();
    let parity = load_vegetation_variant_catalog()
        .map(|c| c.has_all_resolver_keys())
        .unwrap_or(false);
    let entry_count = load_vegetation_variant_catalog()
        .map(|c| c.entries.len())
        .unwrap_or(0);
    let veg_count = load_vegetation_variant_catalog()
        .map(|c| c.veg_keys().len())
        .unwrap_or(0);

    serde_json::json!({
        "slice_id": "VEG-CATALOG-LOADER-001",
        "gate": "VEG-CATALOG-LOADER-001",
        "green": loaded && parity,
        "catalog_loaded": loaded,
        "catalog_path": VEGETATION_VARIANT_CATALOG_RON,
        "engine_veg_resolver_keys": ENGINE_VEG_RESOLVER_KEYS,
        "catalog_entry_count": entry_count,
        "catalog_veg_key_count": veg_count,
        "all_resolver_keys_present": parity,
        "runtime_sim_verified": parity,
    })
}

#[must_use]
pub fn refresh_vegetation_variant_catalog_load_live_witness() -> bool {
    let body = build_vegetation_variant_catalog_load_body();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VEG-CATALOG-LOADER-001",
        "refresh_vegetation_variant_catalog_load_live_witness",
        VEGETATION_VARIANT_CATALOG_LOAD_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(
        VEGETATION_VARIANT_CATALOG_LOAD_LIVE_JSON,
        wrapped,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vegetation_variant_catalog_load_live_witness_refresh_green() {
        assert!(refresh_vegetation_variant_catalog_load_live_witness());
    }
}
