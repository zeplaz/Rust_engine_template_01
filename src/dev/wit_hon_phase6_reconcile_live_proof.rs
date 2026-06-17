//! CDR-B-WIT-HON-PHASE6-001 — reconcile phase6 done rows vs on-disk witness honesty.

pub const WIT_HON_PHASE6_RECONCILE_LIVE_JSON: &str =
    "debug_runs/wit_hon_phase6_reconcile_live.json";

const PHASE6_ROWS: &[(&str, &str, &str)] = &[
    (
        "BUILD-READ-CONSUMER-MCP-001",
        "debug_runs/aps_dna_consumer_rust_live.json",
        "green",
    ),
    (
        "BUILD-READ-VISUAL-001",
        "debug_runs/build_read_visual_001_live.json",
        "runtime_sim_verified",
    ),
    (
        "PG-QUALITY-001",
        "debug_runs/grammar_diversity_witness.json",
        "green",
    ),
    (
        "LG-4-PREVIEW-001",
        "debug_runs/landscape_grammar_lg4_preview_live.json",
        "operator_visible",
    ),
    (
        "VEG-PROGRAM-CLOSE-001",
        "debug_runs/vegetation_program_close_live.json",
        "all_green",
    ),
];

#[must_use]
fn witness_bool_field(path: &str, field: &str) -> Option<bool> {
    let raw = std::fs::read_to_string(path).ok()?;
    let doc: serde_json::Value = serde_json::from_str(&raw).ok()?;
    doc.get(field).and_then(|v| v.as_bool())
}

#[must_use]
pub fn wit_hon_phase6_reconcile_green() -> bool {
    PHASE6_ROWS.iter().all(|(_, path, field)| {
        witness_bool_field(path, field).unwrap_or(false)
    })
}

#[must_use]
pub fn refresh_wit_hon_phase6_reconcile_live_witness() -> bool {
    let mut rows = serde_json::Map::new();
    let mut all_ok = true;
    for (task_id, path, field) in PHASE6_ROWS {
        let ok = witness_bool_field(path, field).unwrap_or(false);
        all_ok &= ok;
        rows.insert(
            (*task_id).into(),
            serde_json::json!({
                "witness_path": path,
                "field": field,
                "ok": ok,
            }),
        );
    }
    let body = serde_json::json!({
        "gate": "CDR-B-WIT-HON-PHASE6-001",
        "slice_id": "CDR-B-WIT-HON-PHASE6-001",
        "green": all_ok,
        "phase6_rows_checked": rows.len(),
        "rows": rows,
        "queue_doc": "tools/orchestrator/queues/post_drain_phase6_coder_queue.json",
        "note": "Done rows must have honest on-disk witnesses — no lib-only green without field truth",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CDR-B-WIT-HON-PHASE6-001",
        "refresh_wit_hon_phase6_reconcile_live_witness",
        WIT_HON_PHASE6_RECONCILE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(WIT_HON_PHASE6_RECONCILE_LIVE_JSON, wrapped)
        && all_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wit_hon_phase6_reconcile_live_witness_green() {
        assert!(refresh_wit_hon_phase6_reconcile_live_witness());
    }
}
