//! INFRA-E4 + CDR-B-INFRA-OVERLAY-POLISH-001 — `debug_runs/utility_network_live.json` lib refresh.

pub const UTILITY_NETWORK_LIVE_JSON: &str = "debug_runs/utility_network_live.json";

#[must_use]
pub fn refresh_utility_network_live_witness() -> bool {
    let mut body =
        crate::infrastructure::utility::refresh_utility_network_live_witness_payload();
    let polish = crate::render::infrastructure_overlay_polish_witness_fields(
        &crate::render::InfrastructureOverlaySettings::default(),
    );
    if let Some(obj) = body.as_object_mut() {
        if let Some(polish_obj) = polish.as_object() {
            for (k, v) in polish_obj {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    if body
        .get("overlay_readability_polish")
        .and_then(|v| v.as_bool())
        != Some(true)
    {
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
        let raw = std::fs::read_to_string(UTILITY_NETWORK_LIVE_JSON).expect("witness");
        let w: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(
            w.get("overlay_readability_polish").and_then(|v| v.as_bool()),
            Some(true),
            "CDR-B-INFRA-OVERLAY-POLISH-001"
        );
        assert_eq!(
            w.get("power_edges_from_graph").and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
