//! **GPU-P1/P2** — lib witness for cadence, overlay defaults, and dirty-gate contracts.

pub const GPU_P1_P2_LIVE_JSON: &str = "debug_runs/gpu_p1_p2_001_live.json";

#[must_use]
pub fn gpu_p1d_fire_cadence_contract_ok() -> bool {
    use crate::render::{fire_extract_cadence_due, FireExtractCadence, FireExtractClock};

    let cadence = FireExtractCadence {
        min_interval_secs: 1.0,
        full_scan_on_sim_tick: false,
        residency_scoped: true,
    };
    let clock = FireExtractClock {
        last_full_extract_secs: 0.5,
        last_tick: 1,
        ..Default::default()
    };
    !fire_extract_cadence_due(&clock, &cadence, 0.6, true, false, false, false)
}

#[must_use]
pub fn gpu_p1e_sim_defaults_not_witness_harness_ok() -> bool {
    use crate::gui::{minimap_overlay_witness_harness, simulation_minimap_overlay_defaults};

    let sim = simulation_minimap_overlay_defaults();
    let harness = minimap_overlay_witness_harness();
    !sim.construction_heat
        && harness.construction_heat
        && harness.ecology_heat
        && !sim.ecology_heat
}

#[must_use]
pub fn gpu_p2b_overlay_eps_gate_ok() -> bool {
    use crate::render::{chunk_fire_heat_maps_differ, CHUNK_FIRE_HEAT_OVERLAY_EPS};
    use bevy::prelude::IVec2;
    use std::collections::HashMap;

    let mut prev = HashMap::new();
    prev.insert(IVec2::ZERO, 0.5);
    let mut next = prev.clone();
    next.insert(IVec2::ZERO, 0.5 + CHUNK_FIRE_HEAT_OVERLAY_EPS * 0.5);
    !chunk_fire_heat_maps_differ(&prev, &next)
}

#[must_use]
pub fn gpu_p1_p2_witness_green() -> bool {
    gpu_p1d_fire_cadence_contract_ok()
        && gpu_p1e_sim_defaults_not_witness_harness_ok()
        && gpu_p2b_overlay_eps_gate_ok()
}

#[must_use]
pub fn build_gpu_p1_p2_witness_body() -> serde_json::Value {
    serde_json::json!({
        "gate": "GPU-P1-P2-001",
        "green": gpu_p1_p2_witness_green(),
        "p1d_cadence_contract": gpu_p1d_fire_cadence_contract_ok(),
        "p1e_overlay_defaults": gpu_p1e_sim_defaults_not_witness_harness_ok(),
        "p2b_overlay_eps_gate": gpu_p2b_overlay_eps_gate_ok(),
        "todo_board": "src/dev/gpu_todos_v1.md",
    })
}

#[must_use]
pub fn refresh_gpu_p1_p2_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_gpu_p1_p2_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "GPU-P1-P2-001",
        "refresh_gpu_p1_p2_witness",
        GPU_P1_P2_LIVE_JSON,
        body,
    );
    write_debug_run_json(GPU_P1_P2_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_p1_p2_witness_reports_green() {
        assert!(gpu_p1_p2_witness_green());
    }

    #[test]
    fn gpu_p1_p2_refresh_witness_when_green() {
        if gpu_p1_p2_witness_green() {
            assert!(refresh_gpu_p1_p2_witness());
        }
    }
}
