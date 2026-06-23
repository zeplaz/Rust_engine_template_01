//! PERF-INSTR-VFX-001 — triage witness for VFX `--test` perf investigation (before/after p50/p95).

use serde_json::{json, Value};

pub const TRIAGE_PERF_VFX_FIX_LIVE_JSON: &str = "debug_runs/triage_perf_vfx_fix_2026-06-11_live.json";

/// Baseline captured from pre–Phase-1 instrumentation (`sim_spectrum_analytics_live.json`, 41 frames).
const BASELINE_BEFORE: &str = r#"{
  "frame_wall_ms": { "p50_ms": 233.84, "p95_ms": 250.0 },
  "after_map_camera_smooth_ms": { "p50_ms": 1.49, "p95_ms": 215.14 },
  "after_fire_build_ms": { "p50_ms": 220.28, "p95_ms": 365.65 },
  "post_world_repr_ms": { "p50_ms": 214.4, "p95_ms": 224.8 },
  "note": "Stall checkpoints mis-ordered — slice attribution conflated fire/view-sync into map_camera and fire_build."
}"#;

#[must_use]
pub fn triage_perf_vfx_fix_witness_body(rolling: &Value, frames_sampled: u64) -> Value {
    let after = json!({
        "frame_wall_ms": rolling.get("frame_wall_ms").cloned().unwrap_or(Value::Null),
        "after_map_camera_smooth_ms": rolling
            .get("after_map_camera_smooth_ms")
            .cloned()
            .unwrap_or_else(|| rolling.get("map_camera_chain_ms").cloned().unwrap_or(Value::Null)),
        "after_fire_build_ms": rolling
            .get("after_fire_build_ms")
            .cloned()
            .unwrap_or_else(|| rolling.get("view_fire_ms").cloned().unwrap_or(Value::Null)),
        "post_world_repr_ms": rolling.get("post_world_repr_ms").cloned().unwrap_or(Value::Null),
        "substage_ms": rolling.get("substage_ms").cloned().unwrap_or(Value::Null),
    });

    let owners_ge_50ms = owners_at_or_above_p50(&after, 50.0);

    let before: Value = serde_json::from_str(BASELINE_BEFORE).unwrap_or(Value::Null);

    json!({
        "schema_version": 1,
        "lane": "PERF-INSTR-VFX-001",
        "phase": "phase2a_2d_shipped",
        "frames_sampled": frames_sampled,
        "before": before,
        "after": after,
        "owners_ge_50ms_p50": owners_ge_50ms,
        "measurement_note": "Stall substage p50 can overlap the same wall interval across labels; trust PerfScope lines (upd_fire_sim_snapshot, upd_world_repr_frame) and fire_extract.fingerprint_skipped for steady frames.",
        "phase2_shipped": [
            "2A fire extract fingerprint",
            "2B world repr stamp skip",
            "2C map camera at-rest early exit",
            "2D camera zoom bounds derive no-op",
        ],
        "phase2_next": [
            "2E fire_sim_snapshot stall attribution vs upd_fire_sim_snapshot scope",
            "2F map_apply_input wall — systems between ApplyInput and DeriveDesired checkpoints",
            "display acceptance: steady_wall_p50_ms <= 33",
        ],
        "acceptance_targets": {
            "steady_wall_p50_ms": 33.0,
            "slice_p50_ms": 5.0,
            "main_camera_transform_changed_steady_pct": 90.0,
        },
    })
}

fn owners_at_or_above_p50(after: &Value, threshold_ms: f32) -> Value {
    let mut hits: Vec<Value> = Vec::new();

    for key in [
        "after_map_camera_smooth_ms",
        "after_fire_build_ms",
        "post_world_repr_ms",
    ] {
        if let Some(p50) = after
            .get(key)
            .and_then(|v| v.get("p50_ms"))
            .and_then(|v| v.as_f64())
        {
            if p50 >= f64::from(threshold_ms) {
                hits.push(json!({ "slice": key, "p50_ms": p50 }));
            }
        }
    }

    if let Some(sub) = after.get("substage_ms").and_then(|v| v.as_object()) {
        for (label, summary) in sub {
            if let Some(p50) = summary.get("p50_ms").and_then(|v| v.as_f64()) {
                if p50 >= f64::from(threshold_ms) {
                    hits.push(json!({ "slice": label, "p50_ms": p50 }));
                }
            }
        }
    }

    hits.sort_by(|a, b| {
        b.get("p50_ms")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            .partial_cmp(&a.get("p50_ms").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Value::Array(hits)
}

#[must_use]
pub fn write_triage_perf_vfx_fix_live_witness(rolling: &Value, frames_sampled: u64) -> bool {
    let body = triage_perf_vfx_fix_witness_body(rolling, frames_sampled);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "PERF-INSTR-VFX-001",
        "write_triage_perf_vfx_fix_live_witness",
        TRIAGE_PERF_VFX_FIX_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(TRIAGE_PERF_VFX_FIX_LIVE_JSON, wrapped)
}

const SLICE_P50_TARGET_MS: f64 = 5.0;
const WALL_P50_TARGET_MS: f64 = 33.0;

/// Phase 2A–2D PerfScope slices (not stall substage labels — those overlap wall intervals).
#[must_use]
pub fn triage_perf_slice_p50_green(after: &Value) -> bool {
    for key in [
        "after_map_camera_smooth_ms",
        "after_fire_build_ms",
    ] {
        if let Some(p50) = after
            .get(key)
            .and_then(|v| v.get("p50_ms"))
            .and_then(|v| v.as_f64())
        {
            if p50 > SLICE_P50_TARGET_MS {
                return false;
            }
        }
    }
    true
}

#[must_use]
pub fn triage_perf_wall_p50_green(after: &Value) -> bool {
    after
        .get("frame_wall_ms")
        .and_then(|v| v.get("p50_ms"))
        .and_then(|v| v.as_f64())
        .is_some_and(|p50| p50 <= WALL_P50_TARGET_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triage_witness_on_disk_has_green_perfscope_slices() {
        let raw = std::fs::read_to_string(TRIAGE_PERF_VFX_FIX_LIVE_JSON).expect("witness");
        let v: Value = serde_json::from_str(&raw).expect("json");
        let after = v.get("after").expect("after block");
        assert!(
            triage_perf_slice_p50_green(after),
            "PerfScope slice p50 should be <= {SLICE_P50_TARGET_MS}ms after phase 2A-2D"
        );
    }
}
