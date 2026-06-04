//! Logistics throughput — transport-graph freight causality (LOG-A…D).

pub mod async_district;
pub mod portals;
pub mod witness;
#[cfg(test)]
pub mod witness_fixture;
pub mod witness_collectors;
pub mod propagation;
pub mod routes;
pub mod solver;
pub mod types;

pub use witness_collectors::LogisticsThroughputLiveProofState;
pub use portals::{
    register_facility_portals_system, rebuild_portal_attachment_map_system,
};
pub use routes::{refresh_resource_flow_routes_system, tile_node_key, topology_revision_u32};
pub use solver::{
    feedback_congestion_from_load_system, propagate_corridor_pressure_system,
    solve_throughput_greedy_system, sync_solver_capacity_from_graph_system,
};
pub use types::{
    FacilityPortal, FacilityPortalRegistered, FreightLot, FreightMovementModel,
    InTransitLedger, LogisticsDiagnostics, LogisticsThroughputRuntimeWitness,
    PortalAttachmentMap, RouteCache, RouteHandle, RoutePathStore, RouteProof,
    ThroughputSolverState, TransportNodeAnchor,
};
pub use witness::{
    align_logistics_throughput_witness_from_live_sim,
    refresh_logistics_throughput_witness_system, sync_logistics_throughput_board_system,
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
            .init_resource::<ThroughputSolverState>()
            .init_resource::<LogisticsDiagnostics>()
            .init_resource::<LogisticsThroughputRuntimeWitness>()
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
                    solve_throughput_greedy_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::ThroughputSolve)
                    .run_if(crate::economy::resource_flow::economy_sim_running),
            )
            .add_systems(
                Update,
                (
                    propagation::commit_freight_arrivals_system,
                    propagation::dispatch_freight_from_solver_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::FreightDispatch)
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
                async_district::apply_async_district_solve_results_system
                    .in_set(LogisticsSimulationSet::Witness),
            )
            .add_systems(
                Update,
                (
                    refresh_logistics_throughput_witness_system,
                    sync_logistics_throughput_board_system,
                    witness_collectors::write_logistics_throughput_live_proof_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::Witness),
            );
    }
}
