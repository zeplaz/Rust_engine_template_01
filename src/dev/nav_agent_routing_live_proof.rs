//! INFRA-E6-002 — `debug_runs/nav_agent_routing_live.json` lib refresh.

use serde_json::{json, Value};

use crate::economy::logistics::routes::nav_agent_routing_witness_payload;

pub const NAV_AGENT_ROUTING_LIVE_JSON: &str = "debug_runs/nav_agent_routing_live.json";

#[must_use]
pub fn build_nav_agent_routing_live_payload() -> Value {
    nav_agent_routing_witness_payload()
}

#[must_use]
pub fn refresh_nav_agent_routing_live_witness() -> bool {
    let body = build_nav_agent_routing_live_payload();
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "INFRA-E6-002",
        "refresh_nav_agent_routing_live_witness",
        NAV_AGENT_ROUTING_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(NAV_AGENT_ROUTING_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_agent_routing_live_witness_refresh_green() {
        assert!(refresh_nav_agent_routing_live_witness());
        let text = std::fs::read_to_string(NAV_AGENT_ROUTING_LIVE_JSON).expect("witness json");
        let v: Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(v.get("green").and_then(|x| x.as_bool()), Some(true));
    }
}
