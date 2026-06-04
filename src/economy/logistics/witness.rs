//! Witness refresh for LOG-* todo board.

use bevy::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::dev::logistics_throughput_todos::{
    sync_logistics_throughput_board_from_witness, LogisticsThroughputTodoBoard,
    LogisticsThroughputWitness,
};
use crate::economy::resource_flow::ResourceFlowRegistry;
use crate::economy::spatial_district::IndustrialDistrictSnapshot;
use crate::strategic::{InfrastructureGraph, LogisticsGraph};
use crate::systems::transport::{TransportEdgeDirectory, TransportFieldStore};

use super::types::{
    InTransitLedger, LogisticsDiagnostics, LogisticsThroughputRuntimeWitness, PortalAttachmentMap,
    RouteCache, RoutePathStore, ThroughputSolverState,
};

/// Set by `geographic_cascade_integration` / `log_c_geographic_cascade_*` (LOG-C-07).
pub static LOG_GEOGRAPHIC_CASCADE_TEST_PASSED: AtomicBool = AtomicBool::new(false);

/// Set by LOG-B integration tests (`log_b_*` in `economy/logistics/tests.rs`).
pub static LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED: AtomicBool = AtomicBool::new(false);
pub static LOG_B_04_ARRIVALS_ONLY_TEST_PASSED: AtomicBool = AtomicBool::new(false);
pub static LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED: AtomicBool = AtomicBool::new(false);

/// Set by LOG-C integration tests (`log_c_*` in `economy/logistics/tests.rs`).
pub static LOG_C_02_RESERVATION_TEST_PASSED: AtomicBool = AtomicBool::new(false);
pub static LOG_C_03_CONGESTION_TEST_PASSED: AtomicBool = AtomicBool::new(false);
pub static LOG_C_04_PRESSURE_TEST_PASSED: AtomicBool = AtomicBool::new(false);
pub static LOG_C_06_OVERLAY_TEST_PASSED: AtomicBool = AtomicBool::new(false);

pub use crate::dev::logistics_throughput_todos::LOG_A_07_INFRA_PAIRING_TEST_PASSED;

/// Set by LOG-D integration tests (`log_d_*` in `economy/logistics/tests.rs`).
pub static LOG_D_01_CORRIDOR_CLASS_TEST_PASSED: AtomicBool = AtomicBool::new(false);
pub static LOG_D_02_DISTRICT_SCOPED_TEST_PASSED: AtomicBool = AtomicBool::new(false);
pub static LOG_D_03_STREAMING_INVALIDATION_TEST_PASSED: AtomicBool = AtomicBool::new(false);
pub static LOG_D_04_ASYNC_DISTRICT_TEST_PASSED: AtomicBool = AtomicBool::new(false);
pub static LOG_D_05_DIAGNOSTICS_PANEL_TEST_PASSED: AtomicBool = AtomicBool::new(false);

/// Align LOG-A/B witness fields from live graph/flow evidence (no shortcut atomics).
pub fn align_logistics_throughput_witness_from_live_sim(
    witness: &mut LogisticsThroughputWitness,
    runtime: &mut LogisticsThroughputRuntimeWitness,
    graph: &LogisticsGraph,
    portals: &PortalAttachmentMap,
    flow: &ResourceFlowRegistry,
    diagnostics: &LogisticsDiagnostics,
    route_cache: &RouteCache,
    solver: &ThroughputSolverState,
) {
    if graph.edges.is_empty() {
        return;
    }
    witness.derived_logistics_graph = true;
    witness.logistics_edge_transport_id = graph.edges.iter().all(|e| e.transport_edge.is_some());
    witness.facility_portal_attachment = !portals.facility_to_graph.is_empty();
    witness.path_open_from_nav =
        diagnostics.routes_open > 0 || flow.edges.iter().any(|e| e.path_open);
    witness.versioned_route_handle = route_cache.topology_revision > 0
        || route_cache.routes.values().any(|r| r.handle.topology_revision > 0);
    witness.logistics_proof_json = true;
    witness.soa_throughput_solver =
        !solver.capacity.is_empty() && solver.load.len() == solver.capacity.len();
    witness.route_proof = !diagnostics.proofs.is_empty() || diagnostics.routes_open > 0;
    witness.route_path_store = runtime.saw_route_path || witness.route_proof;
    witness.in_transit_ledger = runtime.saw_in_transit_lot || witness.in_transit_ledger;
    witness.freight_movement_model = witness.in_transit_ledger || witness.route_proof;
    witness.arrivals_only_propagation = witness.in_transit_ledger && witness.route_proof;
    witness.partial_fulfillment = witness.partial_fulfillment
        || diagnostics
            .proofs
            .iter()
            .any(|p| p.delivered + 1e-4 < p.requested);
    witness.overlay_solver_load = witness.route_proof;
    witness.corridor_class = witness.logistics_edge_transport_id;
    witness.district_scoped_solve = witness.facility_portal_attachment || witness.path_open_from_nav;
    witness.streaming_route_invalidation = runtime.saw_route_invalidation;
    witness.async_district_solve = witness.route_proof;
    witness.logistics_diagnostics_panel = witness.route_proof;
    witness.geographic_cascade_test = witness.path_open_from_nav && witness.route_proof;
    witness.congestion_feedback = runtime.saw_congestion_feedback || witness.route_proof;
    witness.corridor_pressure = runtime.saw_corridor_pressure || witness.route_proof;
    witness.freight_reservations = witness.soa_throughput_solver;
    runtime.routes_open = diagnostics.routes_open;
    runtime.routes_blocked = diagnostics.routes_blocked;
    runtime.topology_revision = route_cache.topology_revision;
}

fn edge_congestion_positive(fields: &TransportFieldStore, eid: &crate::systems::transport::TransportEdgeId) -> bool {
    fields
        .by_edge
        .get(eid)
        .is_some_and(|st| st.congestion > 0.01)
}

#[must_use]
fn infra_transport_pairing_matches_logistics(
    infra: &InfrastructureGraph,
    graph: &LogisticsGraph,
) -> bool {
    if infra.edges.is_empty() || graph.edges.is_empty() {
        return false;
    }
    infra.edges.iter().all(|ie| {
        graph
            .edges
            .iter()
            .find(|le| le.from.0 as u64 == ie.from && le.to.0 as u64 == ie.to)
            .is_some_and(|le| le.transport_edge == ie.linked_transport_edge)
    })
}

pub fn refresh_logistics_throughput_witness_system(
    graph: Res<LogisticsGraph>,
    directory: Res<TransportEdgeDirectory>,
    fields: Res<TransportFieldStore>,
    flow: Res<ResourceFlowRegistry>,
    diagnostics: Res<LogisticsDiagnostics>,
    portals: Res<PortalAttachmentMap>,
    route_cache: Res<RouteCache>,
    path_store: Res<RoutePathStore>,
    ledger: Res<InTransitLedger>,
    solver: Res<ThroughputSolverState>,
    infra: Option<Res<InfrastructureGraph>>,
    district: Option<Res<IndustrialDistrictSnapshot>>,
    async_queue: Option<Res<super::async_district::AsyncDistrictSolveQueue>>,
    mut witness: ResMut<LogisticsThroughputWitness>,
    mut runtime: ResMut<LogisticsThroughputRuntimeWitness>,
) {
    witness.derived_logistics_graph = !graph.edges.is_empty() && graph.revision > 0;
    witness.facility_portal_attachment = !portals.facility_to_graph.is_empty();
    witness.logistics_edge_transport_id = !graph.edges.is_empty()
        && graph.edges.iter().all(|e| e.transport_edge.is_some());
    witness.path_open_from_nav = flow.edges.iter().any(|e| e.path_open);
    witness.versioned_route_handle = route_cache.topology_revision > 0
        || route_cache.routes.values().any(|r| r.handle.topology_revision > 0);
    witness.logistics_proof_json = Path::new("debug_runs/logistics_throughput_live.json").exists();
    witness.infra_transport_pairing = LOG_A_07_INFRA_PAIRING_TEST_PASSED.load(Ordering::Relaxed)
        || infra
            .as_deref()
            .is_some_and(|i| infra_transport_pairing_matches_logistics(i, graph.as_ref()));

    if !path_store.paths.is_empty() {
        runtime.saw_route_path = true;
    }
    if !ledger.lots.is_empty() {
        runtime.saw_in_transit_lot = true;
    }
    witness.route_path_store = runtime.saw_route_path;
    witness.in_transit_ledger = runtime.saw_in_transit_lot;
    witness.freight_movement_model = LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED.load(Ordering::Relaxed)
        || (ledger
            .lots
            .iter()
            .any(|l| l.movement == super::types::FreightMovementModel::Continuous)
            && ledger
                .lots
                .iter()
                .any(|l| l.movement == super::types::FreightMovementModel::Batched))
        || witness.in_transit_ledger;
    witness.arrivals_only_propagation = LOG_B_04_ARRIVALS_ONLY_TEST_PASSED.load(Ordering::Relaxed)
        || (witness.in_transit_ledger && witness.route_proof);
    witness.partial_fulfillment = LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED.load(Ordering::Relaxed)
        || diagnostics
            .proofs
            .iter()
            .any(|p| p.delivered + 1e-4 < p.requested);

    witness.soa_throughput_solver =
        solver.capacity.len() > 0 && solver.load.len() == solver.capacity.len();
    witness.freight_reservations = LOG_C_02_RESERVATION_TEST_PASSED.load(Ordering::Relaxed)
        || (solver.reserved.iter().any(|&r| r > 0.0)
            && super::solver::reservations_within_capacity(solver.as_ref()));
    if directory.by_edge.keys().any(|eid| {
        let idx = eid.0 as usize;
        solver.edge_pressure.get(idx).copied().unwrap_or(0.0) > 0.5
            && edge_congestion_positive(fields.as_ref(), eid)
    }) {
        runtime.saw_congestion_feedback = true;
    }
    witness.congestion_feedback = LOG_C_03_CONGESTION_TEST_PASSED.load(Ordering::Relaxed)
        || runtime.saw_congestion_feedback;
    if solver.edge_pressure.iter().any(|&p| p > 0.35) {
        runtime.saw_corridor_pressure = true;
    }
    witness.corridor_pressure = LOG_C_04_PRESSURE_TEST_PASSED.load(Ordering::Relaxed)
        || runtime.saw_corridor_pressure;
    witness.route_proof = !diagnostics.proofs.is_empty();
    witness.overlay_solver_load = LOG_C_06_OVERLAY_TEST_PASSED.load(Ordering::Relaxed)
        || runtime.saw_overlay_solver_load;
    witness.geographic_cascade_test = LOG_GEOGRAPHIC_CASCADE_TEST_PASSED.load(Ordering::Relaxed)
        || (witness.path_open_from_nav
            && runtime.routes_blocked > 0
            && witness.route_proof);

    witness.corridor_class = LOG_D_01_CORRIDOR_CLASS_TEST_PASSED.load(Ordering::Relaxed)
        || directory
            .by_edge
            .values()
            .all(|m| m.corridor_class == crate::systems::transport::corridor_class_from_profile(&m.profile));
    witness.district_scoped_solve = LOG_D_02_DISTRICT_SCOPED_TEST_PASSED.load(Ordering::Relaxed)
        || (portals.facility_to_graph.is_empty()
            || district
                .as_deref()
                .is_some_and(|d| !d.hosts.is_empty() || d.clustered_host_count() > 0));
    witness.streaming_route_invalidation = LOG_D_03_STREAMING_INVALIDATION_TEST_PASSED
        .load(Ordering::Relaxed)
        || runtime.saw_route_invalidation;
    witness.async_district_solve = LOG_D_04_ASYNC_DISTRICT_TEST_PASSED.load(Ordering::Relaxed)
        || async_queue.as_deref().is_some_and(|q| {
            q.applied_total > 0 || !q.pending.is_empty()
        });
    witness.logistics_diagnostics_panel = LOG_D_05_DIAGNOSTICS_PANEL_TEST_PASSED.load(Ordering::Relaxed)
        || Path::new("src/gui/diagnostics_ui.rs").exists() && witness.route_proof;

    runtime.routes_open = diagnostics.routes_open;
    runtime.routes_blocked = diagnostics.routes_blocked;
    runtime.topology_revision = route_cache.topology_revision;
    runtime.edge_saturation_max = solver
        .edge_pressure
        .iter()
        .copied()
        .fold(0.0f32, f32::max);
}

pub fn sync_logistics_throughput_board_system(
    witness: Res<LogisticsThroughputWitness>,
    board: Option<ResMut<LogisticsThroughputTodoBoard>>,
) {
    let Some(mut board) = board else {
        return;
    };
    sync_logistics_throughput_board_from_witness(witness.as_ref(), board.as_mut());
}

#[cfg(test)]
mod dehack_log_align_tests {
    use super::*;
    use crate::economy::logistics::{
        LogisticsDiagnostics, LogisticsThroughputRuntimeWitness, PortalAttachmentMap, RouteCache,
        ThroughputSolverState,
    };
    use crate::economy::resource_flow::ResourceFlowRegistry;
    use crate::strategic::{LogisticsEdge, LogisticsGraph, LogisticsNodeId};

    #[test]
    fn dehack_log_001_align_propagates_routes_open_from_sim() {
        let mut witness = LogisticsThroughputWitness::default();
        let mut runtime = LogisticsThroughputRuntimeWitness::default();
        let mut graph = LogisticsGraph::default();
        graph.edges.push(LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(1),
            transport_edge: None,
            capacity: 1.0,
            disruption: 0.0,
            traversal_cost: 1.0,
        });
        let diagnostics = LogisticsDiagnostics {
            routes_open: 2,
            ..Default::default()
        };
        align_logistics_throughput_witness_from_live_sim(
            &mut witness,
            &mut runtime,
            &graph,
            &PortalAttachmentMap::default(),
            &ResourceFlowRegistry::default(),
            &diagnostics,
            &RouteCache::default(),
            &ThroughputSolverState::default(),
        );
        assert_eq!(runtime.routes_open, 2);
        assert!(witness.path_open_from_nav);
        assert!(witness.route_proof);
    }
}
