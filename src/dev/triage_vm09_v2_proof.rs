//! **TRIAGE-VM-09-v2** — invert bridge witness close-out (@coder P1).
//!
//! Plan: [`triage_vm09_v2_invert_bridge_plan_v1.md`](triage_vm09_v2_invert_bridge_plan_v1.md)

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// INFRA-VM09-STRAY-001 / RPC-2-004 — production `ResMut<MapCameraDesiredRes>` +
/// `Query<&mut MapCameraDesired>` only in derive (sole mirror writer).
#[must_use]
pub fn infra_vm09_stray_map_camera_writer_audit_green() -> bool {
    let root = repo_root();
    let map_camera = std::fs::read_to_string(root.join("src/gui/tactical/map_camera.rs"))
        .expect("read map_camera.rs");
    let derive_count = map_camera
        .matches("pub fn derive_map_camera_desired_from_view_authority")
        .count();
    let resmut_count = map_camera.matches("ResMut<MapCameraDesiredRes>").count();
    let query_mut_count = map_camera
        .matches("Query<&mut MapCameraDesired")
        .count();
    // ApplyInput must not desired→authority sync (RPC-2-002).
    let no_desired_to_auth_sync = !map_camera.contains("fn sync_map_camera_pose_to_view_authority");
    // Misleading pre-invert alias must stay retired (RPC-2-004).
    let no_compat_alias = !map_camera.contains("fn mirror_world_main_camera_from_map_desired");
    let harness_needle = "mut desired: ResMut<crate::gui::MapCameraDesired>";
    let harness_dir = root.join("src/render/stage5_full_app_harness");
    let harness_ok = if harness_dir.is_dir() {
        std::fs::read_dir(&harness_dir)
            .expect("stage5 harness dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            .all(|e| {
                !std::fs::read_to_string(e.path())
                    .unwrap_or_default()
                    .contains(harness_needle)
            })
    } else {
        // Legacy monolith path (pre RGR-H3 split).
        !std::fs::read_to_string(root.join("src/render/stage5_full_app_harness.rs"))
            .unwrap_or_default()
            .contains(harness_needle)
    };
    derive_count == 1
        && resmut_count == 1
        && query_mut_count == 1
        && no_desired_to_auth_sync
        && no_compat_alias
        && harness_ok
}

/// Refreshes infrastructure witness + agent index.
pub fn refresh_triage_vm09_v2_live_witness() -> bool {
    use crate::dev::debug_run_envelope::refresh_agent_debug_index;
    use crate::dev::runtime_witness::refresh_infrastructure_view_isolation_live_witness;

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
    use serde_json::Value;

    const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";

    const V2_GATES: &[&str] = &[
        "/infrastructure_view_isolation_green",
        "/vm_09/triage_vm09_v2_green",
        "/vm_09/triage_vm09_coder_b_green",
        "/vm_a/dual_writer_pose_violation",
        "/vm_a/minimap_shell_wrote_map_camera_desired",
    ];

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
