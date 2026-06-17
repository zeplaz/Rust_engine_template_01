//! PLAN-VEG-RUNTIME-PROOF-001 — rollup witness for vegetation runtime proof ladder L0→L4.

use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

pub const VEG_RUNTIME_PROOF_LIVE_JSON: &str = "debug_runs/veg_runtime_proof_live.json";

fn repo_path(path: &str) -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|r| r.join(path))
        .unwrap_or_else(|| std::path::PathBuf::from(path))
}

fn read_witness(path: &str) -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(repo_path(path)).ok()?;
    serde_json::from_str(&raw).ok()
}

fn read_json_path(path: &str, pointer: &str) -> Option<serde_json::Value> {
    read_witness(path)?.pointer(pointer).cloned()
}

fn u64_at(doc: &serde_json::Value, key: &str) -> u64 {
    doc.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
}

fn bool_at(doc: &serde_json::Value, key: &str) -> bool {
    doc.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

/// WIT-RUST-002 / CDR-A-WIT-HON-ROLLUP-001 — L1 sim harness sub-rules (not top-level `green` alone).
#[must_use]
pub fn l1_sim_harness_sub_rules_ok(doc: &serde_json::Value) -> bool {
    bool_at(doc, "green")
        && u64_at(doc, "chunks_with_program") >= 16
        && u64_at(doc, "topology_tint_visible_chunks") >= 2
}

/// WIT-RUST-002 / CDR-A-LG4 — L3 LG-4 preview sub-rules aligned with WIT-GREEN-TINT-ZERO.
#[must_use]
pub fn l3_lg4_preview_sub_rules_ok(doc: &serde_json::Value) -> bool {
    let tint = u64_at(doc, "topology_tint_visible_chunks");
    let kinds = u64_at(doc, "topology_kind_count_visible");
    let pixel_wired = bool_at(doc, "pixel_heterogeneity_wired");
    bool_at(doc, "operator_visible")
        && tint >= 1
        && kinds >= 3
        && pixel_wired
}

/// WIT-RUST-002 — LG-5 atlas consumer sub-rules.
#[must_use]
fn lg5_sub_rules_ok(doc: &serde_json::Value) -> bool {
    bool_at(doc, "bevy_chunk_uv_stamp")
        && bool_at(doc, "registry_stamp")
        && bool_at(doc, "atlas_batch_green")
        && bool_at(doc, "atlas_expand_rollup_green")
        && doc
            .get("atlas_id")
            .and_then(|v| v.as_str())
            .is_some_and(|id| id == "landscape_lg5_expanded_v1")
}

#[must_use]
pub fn lg4_preview_child_sub_rules_ok() -> bool {
    read_witness("debug_runs/landscape_grammar_lg4_preview_live.json")
        .map(|doc| l3_lg4_preview_sub_rules_ok(&doc))
        .unwrap_or(false)
}

#[must_use]
pub fn veg_runtime_child_sub_rules_ok() -> bool {
    let l1 = read_witness("debug_runs/landscape_grammar_sim_harness_live.json")
        .map(|doc| l1_sim_harness_sub_rules_ok(&doc))
        .unwrap_or(false);
    let l2 = read_json_path(
        "debug_runs/stage5_full_app_live.json",
        "/ecology_rows_source",
    )
    .and_then(|v| v.as_str().map(|s| s == "live_landscape_program_on_chunk"))
    .unwrap_or(false);
    let l3 = lg4_preview_child_sub_rules_ok();
    let l4 = read_json_path(
        "debug_runs/play_scenario_live.json",
        "/veg_topology_visible_at_operational_zoom",
    )
    .and_then(|v| v.as_bool())
    .unwrap_or(false);
    let f03 =
        crate::gui::landscape_chunk_atlas_stamp::landscape_lg5_chunk_uv_stamp_witness_green();
    let lg5 = read_witness("debug_runs/landscape_grammar_lg5_live.json")
        .map(|doc| lg5_sub_rules_ok(&doc))
        .unwrap_or(false);
    l1 && l2 && l3 && l4 && f03 && lg5
}

#[must_use]
pub fn veg_runtime_proof_ladder_green() -> bool {
    refresh_veg_runtime_proof_live_witness()
}

#[must_use]
pub fn refresh_veg_runtime_proof_live_witness() -> bool {
    let l1_harness = read_witness("debug_runs/landscape_grammar_sim_harness_live.json")
        .map(|doc| l1_sim_harness_sub_rules_ok(&doc))
        .unwrap_or(false);
    let l2_fullapp = read_json_path(
        "debug_runs/stage5_full_app_live.json",
        "/ecology_rows_source",
    )
    .and_then(|v| v.as_str().map(|s| s == "live_landscape_program_on_chunk"))
    .unwrap_or(false);
    let l3_preview = lg4_preview_child_sub_rules_ok();
    let l4_play = read_json_path(
        "debug_runs/play_scenario_live.json",
        "/veg_topology_visible_at_operational_zoom",
    )
    .and_then(|v| v.as_bool())
    .unwrap_or(false);
    let f03_stamp =
        crate::gui::landscape_chunk_atlas_stamp::landscape_lg5_chunk_uv_stamp_witness_green();
    let lg5 = read_witness("debug_runs/landscape_grammar_lg5_live.json")
        .map(|doc| lg5_sub_rules_ok(&doc))
        .unwrap_or(false);

    let green = veg_runtime_child_sub_rules_ok();
    let body = serde_json::json!({
        "gate": "PLAN-VEG-RUNTIME-PROOF-001",
        "slice_id": "CDR-A-WIT-HON-ROLLUP-001",
        "green": green,
        "proof_grade": crate::dev::proof_grade::ProofGrade::HeadlessSim.as_str(),
        "plan": "src/dev/plan_veg_runtime_proof_001_v1.md",
        "ladder": {
            "L0_lib_tests": "cargo test -p proc_A_dine01 --lib landscape_grammar fire_ecology",
            "L1_sim_harness": l1_harness,
            "L2_fullapp_live_ecology": l2_fullapp,
            "L3_preview_pixel_heterogeneity": l3_preview,
            "L4_play_operational_zoom": l4_play,
        },
        "sub_rules_evaluated": true,
        "VEG-F03-REGISTRY-STAMP-001": f03_stamp,
        "VEG-LG5-WITNESS-001": lg5,
        "exit_predicate": {
            "witness": VEG_RUNTIME_PROOF_LIVE_JSON,
            "must": [{ "path": "green", "eq": true }],
        },
        "forbidden_exit": [
            "lib_test_only",
            "witness_counter_zero",
            "eval_math_without_render",
            "single_chunk_pilot",
        ],
        "live_sim_required": true,
        "operator_visible": l3_preview && l4_play,
    });
    let wrapped = wrap_debug_run(
        "PLAN-VEG-RUNTIME-PROOF-001",
        "refresh_veg_runtime_proof_live_witness",
        VEG_RUNTIME_PROOF_LIVE_JSON,
        body,
    );
    write_debug_run_json(VEG_RUNTIME_PROOF_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn veg_runtime_proof_live_witness_refresh() {
        let _ = refresh_veg_runtime_proof_live_witness();
        let raw = std::fs::read_to_string(VEG_RUNTIME_PROOF_LIVE_JSON).expect("witness");
        let doc: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(
            doc.get("gate").and_then(|v| v.as_str()),
            Some("PLAN-VEG-RUNTIME-PROOF-001")
        );
        assert_eq!(
            doc.get("sub_rules_evaluated").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert!(
            doc.pointer("/ladder/L1_sim_harness")
                .and_then(|v| v.as_bool())
                .is_some()
        );
    }

    #[test]
    fn l3_sub_rules_reject_tint_zero_even_if_green_true() {
        let doc = serde_json::json!({
            "green": true,
            "operator_visible": true,
            "topology_tint_visible_chunks": 0,
            "topology_kind_count_visible": 6,
            "pixel_heterogeneity_wired": true
        });
        assert!(!l3_lg4_preview_sub_rules_ok(&doc));
    }

    #[test]
    fn l3_sub_rules_require_pixel_heterogeneity_wired() {
        let doc = serde_json::json!({
            "green": true,
            "operator_visible": true,
            "topology_tint_visible_chunks": 2,
            "topology_kind_count_visible": 6,
            "pixel_heterogeneity_wired": false
        });
        assert!(!l3_lg4_preview_sub_rules_ok(&doc));
    }
}
