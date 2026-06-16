//! **CONSTRUCTION-PLACEMENT-001** — refresh `debug_runs/construction_placement_live.json`.

pub const CONSTRUCTION_PLACEMENT_LIVE_JSON: &str = "debug_runs/construction_placement_live.json";

#[must_use]
pub fn refresh_construction_placement_live_witness() -> bool {
    let body = crate::construction::construction_placement_001_witness_json();
    if !body.get("green").and_then(|v| v.as_bool()).unwrap_or(false) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CONSTRUCTION-PLACEMENT-001",
        "refresh_construction_placement_live_witness",
        CONSTRUCTION_PLACEMENT_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(CONSTRUCTION_PLACEMENT_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_placement_live_witness_refresh_green() {
        assert!(refresh_construction_placement_live_witness());
        let raw = std::fs::read_to_string(CONSTRUCTION_PLACEMENT_LIVE_JSON).expect("read");
        let doc: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(
            doc.pointer("/map_pick_closure_001/green")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            doc.get("footprint_projection_ok").and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
