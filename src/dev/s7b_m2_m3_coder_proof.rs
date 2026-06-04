//! **S7B-M2-001** → **S7B-M3-001** — @coder witness bundle.
//!
//! Plan: [`stage7_behavioral_implementation_plan_v1.md`](stage7_behavioral_implementation_plan_v1.md)

use std::path::PathBuf;

use serde_json::Value;

const BEHAVIORAL: &str = "debug_runs/stage7_behavioral_live.json";
const MINIMAP: &str = "debug_runs/minimap_compositor_live.json";
const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";

const M2_M3_BEHAVIORAL: &[&str] = &[
    "/s7b_m2_green",
    "/s7b_m3_green",
    "/s7b_steward_green",
    "/dispatch_delay_ticks",
    "/stale_intel_surface",
    "/recon_overlay_enabled",
    "/logistics_stress_overlay_enabled",
];

const M3_MINIMAP: &[&str] = &[
    "/ui_w3_m3_001/green",
    "/logistics_rows",
    "/construction_rows",
];

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

fn pointer_u64(v: &Value, ptr: &str) -> u64 {
    v.pointer(ptr)
        .and_then(|x| x.as_u64())
        .unwrap_or_else(|| panic!("missing or non-number {ptr}"))
}

/// M2 dispatch delay + M3 overlay readers + minimap cross-check.
pub fn refresh_s7b_m2_m3_001_live_witness() -> bool {
    use crate::dev::stage7_behavioral_witness::{
        refresh_s7b_m2_001_live_witness, refresh_s7b_m3_001_live_witness,
    };
    use crate::render::minimap_compositor::refresh_ui_w3_m3_001_live_witness;

    assert!(
        refresh_s7b_m2_001_live_witness(),
        "S7B-M2-001 dispatch delay witness"
    );
    assert!(
        refresh_s7b_m3_001_live_witness(),
        "S7B-M3-001 overlay reader witness"
    );
    assert!(
        refresh_ui_w3_m3_001_live_witness(),
        "minimap M3 compositor witness"
    );
    use crate::dev::stage7_behavioral_witness::refresh_s7b_m3_steward_remedy_001_live_witness;
    assert!(
        refresh_s7b_m3_steward_remedy_001_live_witness(),
        "S7B-M3-STEWARD-REMEDY-001 rollup on disk"
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **S7B-M2-001** → **S7B-M3-001** — full coder witness refresh.
    #[test]
    fn s7b_m2_m3_001_lib_bundle() {
        assert!(refresh_s7b_m2_m3_001_live_witness());

        let behavioral = read_json(BEHAVIORAL);
        for ptr in M2_M3_BEHAVIORAL {
            if *ptr == "/dispatch_delay_ticks" {
                assert_eq!(
                    pointer_u64(&behavioral, ptr),
                    8,
                    "fixed-tick delay per D-S7-04"
                );
            } else {
                assert!(pointer_bool(&behavioral, ptr), "{BEHAVIORAL} {ptr}");
            }
        }
        assert!(
            behavioral["recon_overlay_sample_count"]
                .as_u64()
                .unwrap_or(0)
                > 0
        );
        assert!(
            behavioral["logistics_stress_sample_count"]
                .as_u64()
                .unwrap_or(0)
                > 0
        );

        let minimap = read_json(MINIMAP);
        for ptr in M3_MINIMAP {
            if ptr.ends_with("/green") {
                assert!(pointer_bool(&minimap, ptr), "{MINIMAP} {ptr}");
            }
        }
        assert!(pointer_u64(&minimap, "/logistics_rows") >= 2);
        assert!(pointer_u64(&minimap, "/construction_rows") > 0);

        let infra = read_json(INFRA);
        assert!(
            pointer_bool(&infra, "/vm_09/triage_vm09_v2_green"),
            "TRIAGE-VM-09-v2 prereq for M2+"
        );
    }
}
