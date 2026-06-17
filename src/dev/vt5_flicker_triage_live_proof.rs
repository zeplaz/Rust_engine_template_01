//! CODER-A-VT5-TRIAGE-001 / VR-04 — VT-5 bootstrap burst deferral witness.

pub const VT5_FLICKER_TRIAGE_LIVE_JSON: &str = "debug_runs/vt5_flicker_triage_live.json";

#[must_use]
pub fn vr04_triage_lib_green() -> bool {
    crate::render::full_app_vt_ci_fixture_passes()
        && crate::render::vt5_spatial_eval_deferred(&[
            crate::render::sample_fire_row(bevy::math::IVec2::new(0, 0), 0.8),
            crate::render::sample_fire_row(bevy::math::IVec2::new(1, 0), 0.7),
        ])
        && !crate::render::passes_vt5_spatial_invariants(&[
            crate::render::sample_fire_row(bevy::math::IVec2::new(0, 0), 0.8),
            crate::render::sample_fire_row(bevy::math::IVec2::new(1, 0), 0.7),
        ])
}

#[must_use]
pub fn build_vt5_flicker_triage_payload() -> serde_json::Value {
    let lib_green = vr04_triage_lib_green();
    serde_json::json!({
        "gate": "CODER-A-VT5-TRIAGE-001",
        "slice_id": "CODER-A-VT5-TRIAGE-001",
        "vr04_id": "VR-04",
        "green": lib_green,
        "disposition": "bootstrap_defer",
        "not_full_app_gate": true,
        "fix": {
            "policy": "defer_vt5_until_fire_inst_ge",
            "min_eval_fire_instances": crate::render::VT5_MIN_EVAL_FIRE_INSTANCES,
            "collapsed_single_chunk_still_deferred": true,
            "particle_lane_sparse_deferred": true,
        },
        "lib_vt_ci_green": crate::render::full_app_vt_ci_fixture_passes(),
        "vr04_visual_confirm_pending": true,
        "operator_lane": "OPS-VT5-001",
        "docs": "src/dev/visual_run_blockers.md",
        "code": [
            "src/render/vt_spatial_invariants.rs",
            "src/render/vt_ci_matrix.rs",
        ],
    })
}

#[must_use]
pub fn refresh_vt5_flicker_triage_live_witness() -> bool {
    let body = build_vt5_flicker_triage_payload();
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CODER-A-VT5-TRIAGE-001",
        "refresh_vt5_flicker_triage_live_witness",
        VT5_FLICKER_TRIAGE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(VT5_FLICKER_TRIAGE_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vt5_flicker_triage_live_witness_green() {
        assert!(refresh_vt5_flicker_triage_live_witness());
        let raw = std::fs::read_to_string(VT5_FLICKER_TRIAGE_LIVE_JSON).expect("witness");
        let doc: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(doc.get("green").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            doc.get("disposition").and_then(|v| v.as_str()),
            Some("bootstrap_defer")
        );
        assert_eq!(
            doc.pointer("/fix/min_eval_fire_instances").and_then(|v| v.as_u64()),
            Some(3)
        );
    }
}
