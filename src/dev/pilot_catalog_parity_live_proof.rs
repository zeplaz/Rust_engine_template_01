//! **BUILD-READ-PILOT-001** — refresh `debug_runs/pilot_catalog_parity_live.json`.

pub const PILOT_CATALOG_PARITY_LIVE_JSON: &str = "debug_runs/pilot_catalog_parity_live.json";

#[must_use]
pub fn refresh_pilot_catalog_parity_live_witness() -> bool {
    let green = crate::construction::pilot_catalog_parity_witness_green();
    let catalog = crate::construction::PilotCatalog::load_from_disk();
    let body = serde_json::json!({
        "gate": "BUILD-READ-PILOT-001",
        "green": green,
        "pilot_count": catalog.pilots.len(),
        "load_errors": catalog.load_errors,
    });
    if !green {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "BUILD-READ-PILOT-001",
        "refresh_pilot_catalog_parity_live_witness",
        PILOT_CATALOG_PARITY_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(PILOT_CATALOG_PARITY_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pilot_catalog_parity_live_witness_refresh_green() {
        assert!(refresh_pilot_catalog_parity_live_witness());
    }
}
