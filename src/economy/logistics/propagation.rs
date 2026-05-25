//! Freight ledger propagation (LOG-B).

use bevy::prelude::*;

use crate::economy::resource_flow::{ResourceFlowNode, ResourceFlowRegistry, TransportMode};

use super::types::{
    FreightLot, FreightMovementModel, InTransitLedger, LogisticsDiagnostics, RouteCache,
    RoutePathStore, ThroughputSolverState,
};

/// LOG-B-03: movement model from transport mode.
#[must_use]
pub fn freight_movement_for_transport(mode: TransportMode) -> FreightMovementModel {
    match mode {
        TransportMode::Rail => FreightMovementModel::Batched,
        _ => FreightMovementModel::Continuous,
    }
}

/// LOG-B-03: ETA ticks from compact path length and movement model.
#[must_use]
pub fn freight_transit_ticks(path_len: usize, movement: FreightMovementModel) -> u16 {
    match movement {
        FreightMovementModel::Continuous => path_len.max(1) as u16,
        FreightMovementModel::Batched => (path_len.max(1) * 2) as u16,
    }
}

pub fn commit_freight_arrivals_system(
    mut ledger: ResMut<InTransitLedger>,
    mut nodes: Query<&mut ResourceFlowNode>,
) {
    let mut i = 0;
    while i < ledger.lots.len() {
        if ledger.lots[i].remaining_ticks > 0 {
            ledger.lots[i].remaining_ticks -= 1;
            i += 1;
            continue;
        }
        let lot = ledger.lots[i].clone();
        if let Ok(mut node) = nodes.get_mut(lot.destination) {
            *node
                .buffer_by_tag
                .entry(lot.buffer_tag.clone())
                .or_insert(0.0) += lot.amount;
        }
        ledger.lots.swap_remove(i);
    }
}

pub fn dispatch_freight_from_solver_system(
    flow: Res<ResourceFlowRegistry>,
    route_cache: Res<RouteCache>,
    path_store: Res<RoutePathStore>,
    solver: Res<ThroughputSolverState>,
    diagnostics: Res<LogisticsDiagnostics>,
    mut ledger: ResMut<InTransitLedger>,
    mut nodes: Query<&mut ResourceFlowNode>,
) {
    for edge in flow.edges.iter() {
        if !edge.path_open {
            continue;
        }
        let delivered = diagnostics
            .proofs
            .iter()
            .rev()
            .find(|p| p.requested == edge.max_rate)
            .map(|p| p.delivered)
            .unwrap_or(edge.max_rate);
        if delivered <= 0.0 {
            continue;
        }
        let Some(tag) = edge.buffer_tag.as_ref() else {
            continue;
        };
        let Ok(mut from_node) = nodes.get_mut(edge.from) else {
            continue;
        };
        let available = from_node.buffer_by_tag.get(tag).copied().unwrap_or(0.0);
        let ship = available.min(delivered);
        if ship <= 0.0 {
            continue;
        }
        from_node.buffer_by_tag.insert(tag.clone(), available - ship);

        let path_len = route_cache
            .routes
            .get(&(edge.from, edge.to))
            .map(|c| path_store.edge_slice(c.path).len())
            .unwrap_or(1);
        let movement = freight_movement_for_transport(edge.transport_mode);
        let ticks = freight_transit_ticks(path_len, movement);

        let cached = route_cache.routes.get(&(edge.from, edge.to));
        ledger.lots.push(FreightLot {
            destination: edge.to,
            buffer_tag: tag.clone(),
            amount: ship,
            route: edge.route_handle.unwrap_or(super::types::RouteHandle {
                id: 0,
                topology_revision: solver.topology_revision,
            }),
            path: cached.map(|c| c.path).unwrap_or(super::types::RoutePath {
                first_edge: 0,
                edge_count: 0,
            }),
            progress_edge: 0,
            remaining_ticks: ticks,
            movement,
        });
    }
}
