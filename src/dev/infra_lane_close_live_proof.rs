//! **T8 infra lane close** — WC-D04 + VM-09-v2 + VM-10 lockstep rollup.

pub const INFRA_LANE_CLOSE_LIVE_JSON: &str = "debug_runs/infra_lane_close_live.json";

#[must_use]
pub fn refresh_infra_lane_close_witness() -> bool {
    use crate::dev::runtime_witness::{
        refresh_infrastructure_view_isolation_live_witness,
        refresh_wc_d04_stage6_virtualization_live_witness,
    };
    use crate::dev::triage_vm09_v2_proof::infra_vm09_stray_map_camera_writer_audit_green;

    let wc_ok = refresh_wc_d04_stage6_virtualization_live_witness();
    let vm_ok = refresh_infrastructure_view_isolation_live_witness();
    let stray_ok = infra_vm09_stray_map_camera_writer_audit_green();

    let infra_path = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("debug_runs/infrastructure_view_isolation_live.json");
    let vm10_ok = infra_path
        .is_file()
        .then(|| {
            std::fs::read_to_string(&infra_path).ok().and_then(|text| {
                serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| {
                        v.pointer("/vm_10/minimap_lockstep_suspect")
                            .and_then(|x| x.as_bool())
                            .map(|s| !s)
                    })
            })
        })
        .flatten()
        .unwrap_or(false);

    let green = wc_ok && vm_ok && stray_ok && vm10_ok;
    let body = serde_json::json!({
        "gate": "INFRA-LANE-CLOSE-001",
        "wc_d04_green": wc_ok,
        "vm_09_v2_green": vm_ok,
        "vm_10_lockstep_ok": vm10_ok,
        "stray_writer_audit_green": stray_ok,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "INFRA-LANE-CLOSE-001",
        "refresh_infra_lane_close_witness",
        INFRA_LANE_CLOSE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(INFRA_LANE_CLOSE_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infra_lane_close_witness_green() {
        assert!(refresh_infra_lane_close_witness());
    }
}
