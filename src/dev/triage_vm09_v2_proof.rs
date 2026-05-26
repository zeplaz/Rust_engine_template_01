//! **TRIAGE-VM-09-v2** — invert bridge witness close-out (@coder P1).
//!
//! Plan: [`triage_vm09_v2_invert_bridge_plan_v1.md`](triage_vm09_v2_invert_bridge_plan_v1.md)

use std::path::PathBuf;

use serde_json::Value;

const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";

const V2_GATES: &[&str] = &[
    "/infrastructure_view_isolation_green",
    "/vm_09/triage_vm09_v2_green",
    "/vm_09/triage_vm09_coder_b_green",
    "/vm_a/dual_writer_pose_violation",
    "/vm_a/minimap_shell_wrote_map_camera_desired",
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

fn pointer_str(v: &Value, ptr: &str) -> String {
    v.pointer(ptr)
        .and_then(|x| x.as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| panic!("missing or non-string {ptr}"))
}

/// INFRA-VM09-STRAY-001 — production `ResMut<MapCameraDesired>` only in derive shim (+ tests).
#[must_use]
pub fn infra_vm09_stray_map_camera_writer_audit_green() -> bool {
    let root = repo_root();
    let map_camera = std::fs::read_to_string(root.join("src/gui/map_camera.rs"))
        .expect("read map_camera.rs");
    let derive_count = map_camera.matches("pub fn derive_map_camera_desired_from_view_authority").count();
    let resmut_count = map_camera.matches("ResMut<MapCameraDesired>").count();
    let harness_ok = !std::fs::read_to_string(root.join("src/render/stage5_full_app_harness.rs"))
        .expect("stage5 harness")
        .contains("mut desired: ResMut<crate::gui::MapCameraDesired>");
    derive_count == 1 && resmut_count <= 2 && harness_ok
}

/// Refreshes infrastructure witness + agent index.
pub fn refresh_triage_vm09_v2_live_witness() -> bool {
    use crate::dev::debug_run_envelope::refresh_agent_debug_index;
    use crate::render::refresh_infrastructure_view_isolation_live_witness;

    assert!(
        refresh_infrastructure_view_isolation_live_witness(),
        "TRIAGE-VM-09-v2 infrastructure witness"
    );
    refresh_agent_debug_index().expect("agent_debug_index");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **TRIAGE-VM-09-v2** — P1 close-out: invert bridge fields + VM-A guards.
    #[test]
    fn triage_vm09_v2_001_lib_bundle() {
        assert!(refresh_triage_vm09_v2_live_witness());
        let v = read_json(INFRA);
        for ptr in V2_GATES {
            let expected = ptr.contains("violation")
                || ptr.contains("minimap_shell_wrote_map_camera_desired");
            if expected {
                assert!(!pointer_bool(&v, ptr), "{INFRA} {ptr} must be false");
            } else {
                assert!(pointer_bool(&v, ptr), "{INFRA} {ptr} must be true");
            }
        }
        assert_eq!(
            pointer_str(&v, "/vm_09/invert_bridge"),
            "ViewProjectionAuthority_write_MapCameraDesired_derive",
            "invert_bridge label"
        );
    }
}
