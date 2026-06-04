//! **DEHACK-LOG-001** — lib/harness-only LOG-* witness shortcuts (not in default build surface).
//!
//! Use [`super::witness::align_logistics_throughput_witness_from_live_sim`] for VisualCapture /
//! production sim paths. Shortcuts flip integration-test atomics without running LOG-B/C/D suites.

#![cfg(test)]

use std::sync::atomic::Ordering;

use crate::dev::logistics_throughput_todos::LogisticsThroughputWitness;
use crate::dev::proof_grade::ProofGrade;
use crate::economy::resource_flow::ResourceFlowRegistry;
use crate::strategic::LogisticsGraph;

use super::types::{LogisticsDiagnostics, LogisticsThroughputRuntimeWitness, PortalAttachmentMap, RouteCache, ThroughputSolverState};
use super::witness::{
    align_logistics_throughput_witness_from_live_sim, LOG_A_07_INFRA_PAIRING_TEST_PASSED,
    LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED, LOG_B_04_ARRIVALS_ONLY_TEST_PASSED,
    LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED, LOG_C_02_RESERVATION_TEST_PASSED,
    LOG_C_03_CONGESTION_TEST_PASSED, LOG_C_04_PRESSURE_TEST_PASSED, LOG_C_06_OVERLAY_TEST_PASSED,
    LOG_D_01_CORRIDOR_CLASS_TEST_PASSED, LOG_D_02_DISTRICT_SCOPED_TEST_PASSED,
    LOG_D_03_STREAMING_INVALIDATION_TEST_PASSED, LOG_D_04_ASYNC_DISTRICT_TEST_PASSED,
    LOG_D_05_DIAGNOSTICS_PANEL_TEST_PASSED, LOG_GEOGRAPHIC_CASCADE_TEST_PASSED,
};

/// **S7P-LOG-001** / lib fixture: close LOG-* board atomics without full integration suite.
pub fn apply_s7p_logistics_throughput_witness_shortcut(grade: ProofGrade) {
    if !grade.allows_witness_shortcuts() {
        return;
    }
    LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_B_04_ARRIVALS_ONLY_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_C_02_RESERVATION_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_C_03_CONGESTION_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_C_04_PRESSURE_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_C_06_OVERLAY_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_GEOGRAPHIC_CASCADE_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_D_01_CORRIDOR_CLASS_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_D_02_DISTRICT_SCOPED_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_D_03_STREAMING_INVALIDATION_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_D_04_ASYNC_DISTRICT_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_D_05_DIAGNOSTICS_PANEL_TEST_PASSED.store(true, Ordering::Relaxed);
    LOG_A_07_INFRA_PAIRING_TEST_PASSED.store(true, Ordering::Relaxed);
}

/// Lib / headless harness: shortcut atomics + live sim alignment.
pub fn patch_s7p_logistics_throughput_witness_for_play_proof(
    grade: ProofGrade,
    witness: &mut LogisticsThroughputWitness,
    runtime: &mut LogisticsThroughputRuntimeWitness,
    graph: &LogisticsGraph,
    portals: &PortalAttachmentMap,
    flow: &ResourceFlowRegistry,
    diagnostics: &LogisticsDiagnostics,
    route_cache: &RouteCache,
    solver: &ThroughputSolverState,
) {
    apply_s7p_logistics_throughput_witness_shortcut(grade);
    align_logistics_throughput_witness_from_live_sim(
        witness,
        runtime,
        graph,
        portals,
        flow,
        diagnostics,
        route_cache,
        solver,
    );
}

#[cfg(test)]
mod dehack_log_tests {
    use super::*;
    use crate::dev::proof_grade::ProofGrade;

    #[test]
    fn dehack_log_001_shortcut_respects_proof_grade() {
        LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED.store(false, Ordering::Relaxed);
        apply_s7p_logistics_throughput_witness_shortcut(ProofGrade::VisualCapture);
        assert!(
            !LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED.load(Ordering::Relaxed),
            "VisualCapture must not flip LOG-* shortcut atomics"
        );
        apply_s7p_logistics_throughput_witness_shortcut(ProofGrade::LibFixture);
        assert!(
            LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED.load(Ordering::Relaxed),
            "LibFixture may flip LOG-* shortcut atomics"
        );
    }
}
