//! INFRA-E4 — `debug_runs/utility_network_live.json` lib refresh.

pub const UTILITY_NETWORK_LIVE_JSON: &str = "debug_runs/utility_network_live.json";

#[must_use]
pub fn refresh_utility_network_live_witness() -> bool {
    let body = crate::infrastructure::utility::graph::refresh_utility_network_live_witness_payload();
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "INFRA-E4-002",
        "refresh_utility_network_live_witness",
        UTILITY_NETWORK_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(UTILITY_NETWORK_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_network_live_witness_refresh_green() {
        assert!(refresh_utility_network_live_witness());
    }
}
