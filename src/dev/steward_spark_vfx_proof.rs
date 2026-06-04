//! STEWARD-SPARK-VFX-001 — tactical fire-spark witness bundle check.

use std::path::PathBuf;

use serde_json::Value;

const STAGE5: &str = "debug_runs/stage5_full_app_live.json";

fn repo_root() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_json(rel: &str) -> Value {
    let path = repo_root().join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {rel}: {e}"))
}

fn pointer_bool(v: &Value, ptr: &str) -> bool {
    v.pointer(ptr)
        .and_then(|x| x.as_bool())
        .unwrap_or_else(|| panic!("missing or non-bool {ptr}"))
}

fn pointer_u64_positive(v: &Value, ptr: &str) {
    let n = v
        .pointer(ptr)
        .and_then(|x| x.as_u64())
        .unwrap_or_else(|| panic!("missing or non-number {ptr}"));
    assert!(n > 0, "{ptr} must be > 0, got {n}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// STEWARD-SPARK-VFX-001 — stage5 tactical fire spark harness gates.
    #[test]
    fn steward_spark_vfx_001_lib_bundle() {
        use crate::render::stage5_full_app_harness::refresh_p2_fire_spark_011_stage5_live_witness;
        use crate::render::FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA;

        assert!(refresh_p2_fire_spark_011_stage5_live_witness());

        let v = read_json(STAGE5);
        assert!(pointer_bool(&v, "/tactical_vfx_witness/all_green"));
        assert!(pointer_bool(&v, "/tactical_vfx_witness/fire_spark_rows_gt_0"));
        assert!(pointer_bool(&v, "/tactical_vfx_witness/fire_sparks_above_smoke"));
        assert!(pointer_bool(&v, "/tactical_vfx_witness/fire_tactical_zoom"));
        assert!(pointer_bool(&v, "/tactical_vfx_witness/fire_spark_011_green"));
        assert!(pointer_bool(&v, "/particle_routing/fire_spark_011_green"));
        assert!(pointer_bool(&v, "/stage5_closure/passes"));
        pointer_u64_positive(&v, "/particle_routing/fire_spark_rows");
        let zoom = v
            .pointer("/particle_routing/fire_spark_zoom_alpha")
            .and_then(|x| x.as_f64())
            .expect("fire_spark_zoom_alpha");
        assert!(
            (zoom - f64::from(FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA)).abs() < 1e-4,
            "expected P2-FIRE-SPARK-011 proof zoom 0.85, got {zoom}"
        );
        let proof_zoom = v
            .pointer("/particle_routing/fire_spark_tactical_proof_zoom_alpha")
            .and_then(|x| x.as_f64())
            .expect("fire_spark_tactical_proof_zoom_alpha");
        assert!(
            (proof_zoom - f64::from(FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA)).abs() < 1e-4
        );
    }
}
