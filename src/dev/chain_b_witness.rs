//! CHAIN-B (@coder_b) lib witnesses — migration tail + cleanup + product fixes.

pub const CHAIN_B_WITNESS_JSON: &str = "debug_runs/chain_b_witness_live.json";

#[must_use]
pub fn cb_mig_001_audit_split_green() -> bool {
    include_str!("mig_a_adoption.rs").contains("MigAAuditPlugin")
        && include_str!("mig_a_audit.rs").contains("write_mig_a_slice_witnesses")
        && !include_str!("mig_a_adoption.rs").contains("fn write_mig_a_a11_depth_prepass_audit")
}

#[must_use]
pub fn cb_mig_002_dev_diagnostics_wired_green() -> bool {
    include_str!("../engine/engine_with_worldgen.rs").contains("DevDiagnosticsPlugin")
}

#[must_use]
pub fn cb_mig_003_render_schedule_event_green() -> bool {
    include_str!("diagnostics/subscribers.rs").contains("DiagnosticEvent::RenderSchedule")
        && include_str!("diagnostic_events.rs").contains("pub enum DiagnosticEvent")
}

#[must_use]
pub fn cb_cln_001_p0_coder_green() -> bool {
    super::cleanup_p0_witness::cln_p0_r1_legacy_engine_doc_green()
        && super::cleanup_p0_witness::cln_p0_s8_spacial_no_println_green()
        && super::cleanup_p0_witness::cln_p0_p10_solver_frequency_doc_green()
}

#[must_use]
pub fn cb_bq_001_f2_style_green() -> bool {
    crate::construction::procedural::bq_f2_style_001_witness_green()
}

#[must_use]
pub fn cb_bq_002_f3_slot_green() -> bool {
    crate::construction::procedural::bq_f3_slot_001_witness_green()
}

#[must_use]
pub fn cb_city_001_g0_s1c_split_green() -> bool {
    crate::construction::procedural::city_g0_s1c_split_witness_green()
}

#[must_use]
pub fn cb_rgr_001_witness_home_green() -> bool {
    include_str!("diagnostics/visual_readiness.rs").contains("CB-RGR-001")
        && include_str!("diagnostics/perf_attribution.rs").contains("perf_attribution_witness_json")
}

#[must_use]
pub fn cb_city_002_g2_c6_visual_green() -> bool {
    crate::strategic::settlement::city_c6_bsn_witness_green()
        && crate::strategic::settlement::block_street_visual_fixture_witness_green()
}

#[must_use]
pub fn chain_b_witness_green() -> bool {
    cb_mig_001_audit_split_green()
        && cb_mig_002_dev_diagnostics_wired_green()
        && cb_mig_003_render_schedule_event_green()
        && cb_cln_001_p0_coder_green()
        && cb_bq_001_f2_style_green()
        && cb_bq_002_f3_slot_green()
        && cb_city_001_g0_s1c_split_green()
        && cb_rgr_001_witness_home_green()
        && cb_city_002_g2_c6_visual_green()
}

#[must_use]
pub fn chain_b_witness_json() -> serde_json::Value {
    serde_json::json!({
        "schema": "chain_b_witness_v1",
        "green": chain_b_witness_green(),
        "slices": {
            "CB-MIG-001": cb_mig_001_audit_split_green(),
            "CB-MIG-002": cb_mig_002_dev_diagnostics_wired_green(),
            "CB-MIG-003": cb_mig_003_render_schedule_event_green(),
            "CB-CLN-001": cb_cln_001_p0_coder_green(),
            "CB-BQ-001": cb_bq_001_f2_style_green(),
            "CB-BQ-002": cb_bq_002_f3_slot_green(),
            "CB-CITY-001": cb_city_001_g0_s1c_split_green(),
            "CB-RGR-001": cb_rgr_001_witness_home_green(),
            "CB-CITY-002": cb_city_002_g2_c6_visual_green(),
        },
    })
}

#[must_use]
pub fn refresh_chain_b_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = chain_b_witness_json();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "CHAIN-B",
        "refresh_chain_b_witness",
        CHAIN_B_WITNESS_JSON,
        body,
    );
    write_debug_run_json(CHAIN_B_WITNESS_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_b_all_slices_green() {
        assert!(chain_b_witness_green(), "{}", chain_b_witness_json());
    }

    #[test]
    fn chain_b_witness_refresh_writes_json() {
        crate::dev::debug_run_envelope::reset_witness_refresh_gate_for_tests();
        assert!(refresh_chain_b_witness());
    }
}
