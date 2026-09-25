//! Freight ledger propagation (LOG-B) + deployable staging haul (COD-DEPLOYABLE-LOGISTICS-001).

use bevy::prelude::*;

use crate::economy::resource_flow::{ResourceFlowNode, ResourceFlowRegistry, TransportMode};
use crate::entities::production::core::{
    is_deployable_staging_buffer_tag, ManufacturingOutputBuffers, BUFFER_TAG_DRAGON_TEETH_UNIT,
    BUFFER_TAG_MINE_UNIT,
};

use super::types::{
    FreightLot, FreightMovementModel, InTransitLedger, LogisticsDiagnostics, PendingSiteStagingDebits,
    RouteCache, RouteHandle, RoutePath, RoutePathStore, SiteStagingStock, ThroughputSolverState,
};

/// Deployable tags hauled from plant buffers into the ledger (then staging).
const DEPLOYABLE_HAUL_TAGS: &[&str] = &[BUFFER_TAG_DRAGON_TEETH_UNIT, BUFFER_TAG_MINE_UNIT];

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

/// Minimal transit for plant→depot deployable haul (no parallel freight bus).
#[must_use]
pub fn deployable_haul_transit_ticks() -> u16 {
    freight_transit_ticks(1, FreightMovementModel::Continuous)
}

/// LOG-B-02: advance `progress_edge` from elapsed transit ticks (no teleport).
fn advance_lot_progress(lot: &mut FreightLot) {
    if lot.remaining_ticks == 0 {
        return;
    }
    lot.remaining_ticks -= 1;
    let edge_count = lot.path.edge_count;
    if edge_count == 0 {
        return;
    }
    let total = freight_transit_ticks(edge_count as usize, lot.movement);
    let elapsed = total.saturating_sub(lot.remaining_ticks);
    lot.progress_edge = match lot.movement {
        FreightMovementModel::Continuous => elapsed.min(edge_count.saturating_sub(1)),
        FreightMovementModel::Batched => (elapsed / 2).min(edge_count.saturating_sub(1)),
    };
}

/// Commit arrivals: deployable tags → [`SiteStagingStock`] (sole credit writer);
/// all other tags → destination [`ResourceFlowNode`] buffers.
pub fn commit_freight_arrivals_system(
    mut ledger: ResMut<InTransitLedger>,
    mut staging: ResMut<SiteStagingStock>,
    mut nodes: Query<&mut ResourceFlowNode>,
) {
    let mut i = 0;
    while i < ledger.lots.len() {
        if ledger.lots[i].remaining_ticks > 0 {
            advance_lot_progress(&mut ledger.lots[i]);
            i += 1;
            continue;
        }
        let lot = ledger.lots[i].clone();
        if is_deployable_staging_buffer_tag(&lot.buffer_tag) {
            // Single writer for SiteStagingStock credit — UI must not mirror this path.
            staging.credit(&lot.buffer_tag, lot.amount);
        } else if let Ok(mut node) = nodes.get_mut(lot.destination) {
            *node
                .buffer_by_tag
                .entry(lot.buffer_tag.clone())
                .or_insert(0.0) += lot.amount;
        }
        ledger.lots.swap_remove(i);
    }
}

/// Apply place-commit staging debits — sole `SiteStagingStock::try_debit` caller
/// (COD-DEPLOYABLE-PLACE-001). Same writer family as arrivals credit.
pub fn apply_pending_site_staging_debits_system(
    mut staging: ResMut<SiteStagingStock>,
    mut pending: ResMut<PendingSiteStagingDebits>,
) {
    if pending.queue.is_empty() {
        return;
    }
    let drained: Vec<_> = pending.queue.drain(..).collect();
    for req in drained {
        let _ok = staging.try_debit(&req.buffer_tag, req.amount);
        debug_assert!(
            _ok,
            "staging debit failed for {} amount {} — gate should have reserved",
            req.buffer_tag, req.amount
        );
    }
}

/// Haul plant-local deployable stock into [`InTransitLedger`] (no teleport / no staging credit).
///
/// Debits [`ManufacturingOutputBuffers`]; arrivals later credit [`SiteStagingStock`].
pub fn dispatch_deployable_from_manufacturing_system(
    mut ledger: ResMut<InTransitLedger>,
    mut buffers: Query<(Entity, &mut ManufacturingOutputBuffers)>,
) {
    let ticks = deployable_haul_transit_ticks();
    debug_assert!(ticks > 0, "deployable haul must not same-tick teleport");
    for (plant, mut buf) in buffers.iter_mut() {
        for tag in DEPLOYABLE_HAUL_TAGS {
            let ship = buf.debit(tag, f32::MAX);
            if ship <= 0.0 {
                continue;
            }
            ledger.lots.push(FreightLot {
                destination: plant,
                buffer_tag: (*tag).to_string(),
                amount: ship,
                route: RouteHandle {
                    id: 0,
                    topology_revision: 0,
                },
                path: RoutePath {
                    first_edge: 0,
                    edge_count: 1,
                },
                progress_edge: 0,
                remaining_ticks: ticks,
                movement: FreightMovementModel::Continuous,
            });
        }
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
        // Deployables use ManufacturingOutputBuffers → dispatch_deployable_*; skip flow edges.
        if is_deployable_staging_buffer_tag(tag) {
            continue;
        }
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
        let route = cached
            .map(|c| c.handle)
            .or(edge.route_handle)
            .unwrap_or(super::types::RouteHandle {
                id: 0,
                topology_revision: solver.topology_revision,
            });
        ledger.lots.push(FreightLot {
            destination: edge.to,
            buffer_tag: tag.clone(),
            amount: ship,
            route,
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    /// PCI-34: `SiteStagingStock::try_debit` has one production caller.
    #[test]
    fn site_staging_try_debit_has_one_caller() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let needle = format!("staging.{}(", "try_debit");
        let mut hits: Vec<String> = Vec::new();
        fn walk(dir: &Path, needle: &str, hits: &mut Vec<String>) {
            let Ok(rd) = std::fs::read_dir(dir) else {
                return;
            };
            for ent in rd.flatten() {
                let path = ent.path();
                if path.is_dir() {
                    walk(&path, needle, hits);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                if text.contains(needle) {
                    hits.push(path.display().to_string().replace('\\', "/"));
                }
            }
        }
        walk(&root, &needle, &mut hits);
        assert_eq!(
            hits.len(),
            1,
            "expected one staging.try_debit call site, found {hits:?}"
        );
        assert!(
            hits[0].ends_with("src/economy/logistics/propagation.rs"),
            "sole debit caller drifted: {}",
            hits[0]
        );
    }
}
