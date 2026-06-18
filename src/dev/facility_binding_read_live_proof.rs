//! **COD-FACILITY-BINDING-READ-001** — optional `facility_binding` on BuildingGrammar (read-only).

pub const FACILITY_BINDING_READ_LIVE_JSON: &str = "debug_runs/facility_binding_read_live.json";

#[must_use]
pub fn build_facility_binding_read_body() -> serde_json::Value {
    let body = crate::construction::procedural::facility_binding_read_witness_body();
    serde_json::json!({
        "gate": "COD-FACILITY-BINDING-READ-001",
        "slice_id": "COD-FACILITY-BINDING-READ-001",
        "program_id": "PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001",
        "green": body.get("green").and_then(|v| v.as_bool()).unwrap_or(false),
        "read": body,
        "design": "src/dev/design_facility_binding_schema_v1.md",
        "schema": "tools/mcp/schemas/facility_binding_v1.schema.json",
        "cmcp_lane": "debug_runs/art_pipeline/dmcp_facility_binding_lane_live.json",
        "code": [
            "src/construction/procedural/building_grammar.rs",
        ],
    })
}

#[must_use]
pub fn refresh_facility_binding_read_live_witness() -> bool {
    let body = build_facility_binding_read_body();
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-FACILITY-BINDING-READ-001",
        "refresh_facility_binding_read_live_witness",
        FACILITY_BINDING_READ_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(FACILITY_BINDING_READ_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facility_binding_read_live_witness_green() {
        assert!(refresh_facility_binding_read_live_witness());
    }
}
