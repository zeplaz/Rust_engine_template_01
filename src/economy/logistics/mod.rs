//! Logistics throughput — transport-graph freight causality (LOG-A…D).

pub mod async_district;
pub mod district_scope;
pub mod portals;
pub mod witness;
#[cfg(test)]
pub mod witness_fixture;
pub mod witness_collectors;
pub mod propagation;
pub mod routes;
pub mod solver;
pub mod types;

pub use witness_collectors::{
    build_logistics_throughput_proof_payload, InfraE502ProofExtension,
    LogisticsThroughputLiveProofState, LOGISTICS_THROUGHPUT_JSON,
};
pub use portals::{
    register_facility_portals_system, rebuild_portal_attachment_map_system,
};
pub use routes::{
    collect_portal_entity_tiles, collect_portal_entity_tiles_from_world,
    flow_paths_match_nav_export, infra_e5_002_graph_only_paths_green,
    nav_agent_routing_witness_payload, refresh_resource_flow_routes_system, tile_node_key,
    topology_revision_u32,
};
pub use solver::{
    book_matches_soa_reserved, feedback_congestion_from_load_system,
    propagate_corridor_pressure_system, reservations_within_capacity, soa_solver_aligned,
    solve_throughput_greedy_system, sync_solver_capacity_from_graph_system,
};
pub use types::{
    DistrictSolveScope, FacilityPortal, FacilityPortalRegistered, FreightLot, FreightMovementModel,
    FreightReservation, FreightReservationBook, InTransitLedger, LogisticsDiagnostics,
    LogisticsThroughputRuntimeWitness, PendingSiteStagingDebits, PendingStagingDebit,
    PortalAttachmentMap, RouteCache, RouteHandle, RoutePath, RoutePathStore, RouteProof,
    SaturatedEdgeSample, SiteStagingStock, ThroughputSolverState, TransportNodeAnchor,
};
pub use propagation::{
    apply_pending_site_staging_debits_system, commit_freight_arrivals_system,
    deployable_haul_transit_ticks, dispatch_deployable_from_manufacturing_system,
    dispatch_freight_from_solver_system, freight_movement_for_transport, freight_transit_ticks,
};
pub use witness::{
    align_logistics_throughput_witness_from_live_sim,
    collect_logistics_diagnostics_panel_system,
    refresh_logistics_throughput_witness_system,
    sync_logistics_throughput_board_system,
    INFRA_E5_002_GRAPH_ONLY_LATCH,
    LOG_B_02_IN_TRANSIT_LEDGER_TEST_PASSED,
    LOG_B_03_FREIGHT_MOVEMENT_TEST_PASSED,
    LOG_B_04_ARRIVALS_ONLY_TEST_PASSED,
    LOG_B_05_PARTIAL_FULFILLMENT_TEST_PASSED,
    LOG_C_01_SOA_TEST_PASSED,
    LOG_C_02_RESERVATION_TEST_PASSED,
    LOG_C_03_CONGESTION_TEST_PASSED,
    LOG_C_04_PRESSURE_TEST_PASSED,
    LOG_C_06_OVERLAY_TEST_PASSED,
    LOG_D_01_CORRIDOR_CLASS_TEST_PASSED,
    LOG_D_02_DISTRICT_SCOPED_TEST_PASSED,
    LOG_D_03_STREAMING_INVALIDATION_TEST_PASSED,
    LOG_D_04_ASYNC_DISTRICT_TEST_PASSED,
    LOG_D_05_DIAGNOSTICS_PANEL_TEST_PASSED,
    LOG_GEOGRAPHIC_CASCADE_TEST_PASSED,
};

use bevy::prelude::*;

use crate::strategic::StrategicFieldPipeline;
use crate::systems::transport::TransportSchedule;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogisticsSimulationSet {
    PortalAttach,
    RouteRefresh,
    SolverSync,
    ThroughputSolve,
    FreightDispatch,
    FieldFeedback,
    CorridorPressure,
    Witness,
}

#[cfg(test)]
mod tests;

pub struct LogisticsThroughputPlugin;

impl Plugin for LogisticsThroughputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PortalAttachmentMap>()
            .init_resource::<RouteCache>()
            .init_resource::<RoutePathStore>()
            .init_resource::<InTransitLedger>()
            .init_resource::<SiteStagingStock>()
            .init_resource::<PendingSiteStagingDebits>()
            .init_resource::<ThroughputSolverState>()
            .init_resource::<FreightReservationBook>()
            .init_resource::<LogisticsDiagnostics>()
            .init_resource::<LogisticsThroughputRuntimeWitness>()
            .init_resource::<DistrictSolveScope>()
            .init_resource::<async_district::AsyncDistrictSolveQueue>()
            .init_resource::<witness_collectors::LogisticsThroughputLiveProofState>()
            .init_resource::<crate::dev::logistics_throughput_todos::LogisticsThroughputWitness>()
            .configure_sets(
                Update,
                (
                    LogisticsSimulationSet::PortalAttach
                        .after(StrategicFieldPipeline::GraphSync),
                    LogisticsSimulationSet::RouteRefresh
                        .after(TransportSchedule::CostCache)
                        .after(LogisticsSimulationSet::PortalAttach),
                    LogisticsSimulationSet::SolverSync.after(LogisticsSimulationSet::RouteRefresh),
                    LogisticsSimulationSet::ThroughputSolve.after(LogisticsSimulationSet::SolverSync),
                    LogisticsSimulationSet::FreightDispatch.after(LogisticsSimulationSet::ThroughputSolve),
                    LogisticsSimulationSet::FieldFeedback.after(LogisticsSimulationSet::FreightDispatch),
                    LogisticsSimulationSet::CorridorPressure.after(LogisticsSimulationSet::FieldFeedback),
                    LogisticsSimulationSet::Witness.after(LogisticsSimulationSet::CorridorPressure),
                ),
            )
            .add_systems(
                Update,
                (
                    register_facility_portals_system,
                    rebuild_portal_attachment_map_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::PortalAttach),
            )
            .add_systems(
                Update,
                refresh_resource_flow_routes_system.in_set(LogisticsSimulationSet::RouteRefresh),
            )
            .add_systems(
                Update,
                (
                    sync_solver_capacity_from_graph_system,
                    district_scope::apply_district_solve_scope_system,
                    solve_throughput_greedy_system,
                    async_district::enqueue_async_district_solve_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::ThroughputSolve)
                    .run_if(crate::economy::resource_flow::economy_sim_running),
            )
            .add_systems(
                Update,
                (
                    propagation::commit_freight_arrivals_system,
                    propagation::apply_pending_site_staging_debits_system,
                    propagation::dispatch_freight_from_solver_system,
                    propagation::dispatch_deployable_from_manufacturing_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::FreightDispatch)
                    // After plant credit so haul sees ManufacturingOutputBuffers same tick.
                    .after(crate::entities::production::core::tick_manufacturing_nodes)
                    .run_if(crate::economy::resource_flow::economy_sim_running),
            )
            .add_systems(
                Update,
                (
                    feedback_congestion_from_load_system,
                    propagate_corridor_pressure_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::FieldFeedback)
                    .run_if(crate::economy::resource_flow::economy_sim_running),
            )
            .add_systems(
                Update,
                (
                    async_district::apply_async_district_solve_results_system,
                    collect_logistics_diagnostics_panel_system,
                    refresh_logistics_throughput_witness_system,
                    sync_logistics_throughput_board_system,
                    witness_collectors::write_logistics_throughput_live_proof_system
                        .run_if(crate::dev::runtime_witness::logistics_throughput_live_proof_due),
                    async_district::promote_async_district_staging_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::Witness),
            );
    }
}
