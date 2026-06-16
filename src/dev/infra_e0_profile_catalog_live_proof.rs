//! **INFRA-E0-003** — corridor profile RON catalog load witness.

pub const INFRA_E0_PROFILE_CATALOG_LIVE_JSON: &str =
    "debug_runs/infra_e0_profile_catalog_live.json";

#[must_use]
pub fn refresh_infra_e0_profile_catalog_live_witness() -> bool {
    use crate::infrastructure::profiles::ProfileRegistry;

    let registry = ProfileRegistry::load_default_example().ok();
    let road_count = registry
        .as_ref()
        .map(|r| r.resolve("default_road").is_some())
        .unwrap_or(false);
    let green = registry.is_some() && road_count;
    let body = serde_json::json!({
        "gate": "INFRA-E0-003",
        "green": green,
        "profile_registry_loaded": registry.is_some(),
        "local_road_resolved": road_count,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "INFRA-E0-003",
        "refresh_infra_e0_profile_catalog_live_witness",
        INFRA_E0_PROFILE_CATALOG_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(INFRA_E0_PROFILE_CATALOG_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infra_e0_profile_catalog_live_witness_green() {
        assert!(refresh_infra_e0_profile_catalog_live_witness());
    }
}
