//! Logistics throughput — transport-graph freight causality (LOG-A…D).

pub mod async_district;
pub mod district_scope;
pub mod intent_draw;
pub mod orders;
pub mod portals;
pub mod propagation;
pub mod debris_block;
pub mod road_flow;
pub mod track_wear;
pub mod routes;
pub mod solver;
pub mod types;
pub mod diagnostics_panel;

pub use diagnostics_panel::collect_logistics_diagnostics_panel_system;
pub use portals::{rebuild_portal_attachment_map_system, register_facility_portals_system};
pub use propagation::{
    apply_pending_site_staging_debits_system, commit_freight_arrivals_system,
    deployable_haul_transit_ticks, dispatch_deployable_from_manufacturing_system,
    dispatch_freight_from_solver_system, freight_movement_for_transport, freight_transit_ticks,
};
pub use debris_block::{
    publish_debris_route_blocks_system, tile_debris_impassable, DebrisRouteBlockBook,
};
pub use routes::{
    accumulate_logistics_route_dirty_tiles_system, cached_route_intersects_dirty_tiles,
    collect_portal_entity_tiles, collect_portal_entity_tiles_from_world,
    flow_paths_match_nav_export, invalidate_routes_touching_dirty_tiles,
    refresh_resource_flow_routes_system, tile_node_key, topology_revision_u32,
};
pub use solver::{
    book_matches_soa_reserved, feedback_congestion_from_load_system,
    propagate_corridor_pressure_system, reservations_within_capacity, soa_solver_aligned,
    solve_throughput_greedy_system, sync_solver_capacity_from_graph_system,
    throughput_solve_max_edges, throughput_solve_unchanged, THROUGHPUT_SOLVE_BASE_EDGE_BUDGET,
};
pub use types::{
    DistrictSolveScope, FacilityPortal, FacilityPortalRegistered, FreightLot, FreightMovementModel,
    FreightReservation, FreightReservationBook, InTransitLedger, LogisticsDiagnostics,
    LogisticsThroughputStats, PendingSiteStagingDebits, PendingStagingDebit,
    LogisticsRouteDirtyTiles, PortalAttachmentMap, RouteCache, RouteHandle, RoutePath,
    RoutePathStore, RouteProof,
    SaturatedEdgeSample, SiteStagingStock, ThroughputSolverState, TransportNodeAnchor,
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
        crate::systems::sim_control::ensure_sim_step_clock(app);
        app.add_plugins(orders::LogisticsOrdersPlugin);
        app.init_resource::<PortalAttachmentMap>()
            .init_resource::<RouteCache>()
            .init_resource::<LogisticsRouteDirtyTiles>()
            .init_resource::<DebrisRouteBlockBook>()
            .init_resource::<RoutePathStore>()
            .init_resource::<InTransitLedger>()
            .init_resource::<SiteStagingStock>()
            .init_resource::<PendingSiteStagingDebits>()
            .init_resource::<ThroughputSolverState>()
            .init_resource::<FreightReservationBook>()
            .init_resource::<LogisticsDiagnostics>()
            .init_resource::<LogisticsThroughputStats>()
            .init_resource::<DistrictSolveScope>()
            .init_resource::<async_district::AsyncDistrictSolveQueue>()
            .init_resource::<intent_draw::AiConstructionIntent>()
            .init_resource::<intent_draw::LogisticsIntentDrawFrameRes>()
            .configure_sets(
                Update,
                (
                    LogisticsSimulationSet::PortalAttach.after(StrategicFieldPipeline::GraphSync),
                    LogisticsSimulationSet::RouteRefresh
                        .after(TransportSchedule::CostCache)
                        .after(LogisticsSimulationSet::PortalAttach),
                    LogisticsSimulationSet::SolverSync.after(LogisticsSimulationSet::RouteRefresh),
                    LogisticsSimulationSet::ThroughputSolve
                        .after(LogisticsSimulationSet::SolverSync)
                        .after(crate::systems::sim_control::SimControlSystemSet::AdvanceSimTick),
                    LogisticsSimulationSet::FreightDispatch
                        .after(LogisticsSimulationSet::ThroughputSolve),
                    LogisticsSimulationSet::FieldFeedback
                        .after(LogisticsSimulationSet::FreightDispatch),
                    LogisticsSimulationSet::CorridorPressure
                        .after(LogisticsSimulationSet::FieldFeedback),
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
                (
                    publish_debris_route_blocks_system,
                    accumulate_logistics_route_dirty_tiles_system,
                    refresh_resource_flow_routes_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::RouteRefresh),
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
                    .run_if(crate::systems::sim_control::sim_step_due),
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
                    .run_if(crate::systems::sim_control::sim_step_due),
            )
            .add_systems(
                Update,
                (
                    feedback_congestion_from_load_system,
                    propagate_corridor_pressure_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::FieldFeedback)
                    .run_if(crate::systems::sim_control::sim_step_due),
            )
            .add_systems(
                Update,
                (
                    async_district::apply_async_district_solve_results_system,
                    collect_logistics_diagnostics_panel_system,
                    async_district::promote_async_district_staging_system,
                )
                    .chain()
                    .in_set(LogisticsSimulationSet::Witness),
            )
            .add_systems(PostUpdate, intent_draw::sync_logistics_intent_draw_frame);
    }
}
