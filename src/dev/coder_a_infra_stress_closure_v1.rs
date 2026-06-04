//! **@coder A** — `coder_a.active` infra/stress closure (INFRA-VM-DEEP + STAGE6-OPS-WITNESS).

use serde_json::Value;

const INFRA: &str = "debug_runs/infrastructure_view_isolation_live.json";
const STAGE6: &str = "debug_runs/stage6_virtualization_live.json";

fn repo_root() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
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

/// Refresh lib witnesses for both active infra/stress gates.
#[must_use]
pub fn refresh_coder_a_infra_stress_closure() -> bool {
    use crate::dev::runtime_witness::{
        refresh_infrastructure_view_isolation_live_witness,
        refresh_wc_d04_stage6_virtualization_live_witness,
    };

    assert!(
        refresh_infrastructure_view_isolation_live_witness(),
        "INFRA-VM-DEEP-001 lib path"
    );
    assert!(
        refresh_wc_d04_stage6_virtualization_live_witness(),
        "STAGE6-OPS-WITNESS-001 lib path (wc_d04 bundle)"
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coder_a_infra_stress_active_closure() {
        assert!(refresh_coder_a_infra_stress_closure());

        let infra = read_json(INFRA);
        assert!(pointer_bool(&infra, "/infra_vm_deep_001/green"));
        assert_eq!(
            infra["infra_vm_deep_001"]["source"],
            Value::String("lib_refresh".into())
        );
        assert_eq!(
            infra["infra_vm_deep_001"]["sim_time_written"],
            Value::Bool(false)
        );
        assert!(infra["infra_vm_deep_001"]["sim_trace"]["per_view_fire_instances"].is_object());
        assert!(pointer_bool(
            &infra,
            "/triage_phase_d_parity_001/stress/s1_world_main_simulation_map_masks"
        ));

        let stage6 = read_json(STAGE6);
        assert!(pointer_bool(&stage6, "/stage6_ops_witness_001/green"));
        assert_eq!(
            stage6["stage6_ops_witness_001"]["source"],
            Value::String("lib_refresh".into())
        );
        assert_eq!(
            stage6["stage6_ops_witness_001"]["gpu_upload_bytes_frame"]
                .as_u64()
                .unwrap_or(0),
            4096
        );
    }
}
