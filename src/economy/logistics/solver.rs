//! Throughput solver + corridor pressure (LOG-C).

use bevy::prelude::*;

use crate::strategic::LogisticsGraph;
use crate::systems::transport::{TransportEdgeDirectory, TransportFieldStore};

use super::routes::topology_revision_u32;
use super::types::{LogisticsDiagnostics, RouteProof, ThroughputSolverState};
use super::types::{RoutePathStore, RouteCache};
use crate::construction::ConstructionWorldRevision;
use crate::economy::resource_flow::ResourceFlowRegistry;

/// LOG-C-02: every edge reservation must stay within capacity after solve.
#[must_use]
pub fn reservations_within_capacity(solver: &ThroughputSolverState) -> bool {
    solver
        .capacity
        .iter()
        .zip(solver.reserved.iter())
        .all(|(&cap, &res)| res <= cap + 1e-4)
}

pub fn sync_solver_capacity_from_graph_system(
    graph: Option<Res<LogisticsGraph>>,
    directory: Res<TransportEdgeDirectory>,
    construction_rev: Option<Res<ConstructionWorldRevision>>,
    mut solver: ResMut<ThroughputSolverState>,
) {
    let Some(graph) = graph else {
        return;
    };
    let max_id = directory
        .by_edge
        .keys()
        .map(|k| k.0 as usize)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    solver.topology_revision = topology_revision_u32(
        graph.revision,
        construction_rev.map(|r| r.revision).unwrap_or(0),
    );
    solver.ensure_len(max_id);
    solver.clear_tick();

    for edge in &graph.edges {
        let Some(tid) = edge.transport_edge else {
            continue;
        };
        let idx = tid.0 as usize;
        if idx < solver.capacity.len() {
            solver.capacity[idx] = edge.capacity * (1.0 - edge.disruption.clamp(0.0, 1.0));
        }
    }
}

pub fn solve_throughput_greedy_system(
    flow: Res<ResourceFlowRegistry>,
    route_cache: Res<RouteCache>,
    path_store: Res<RoutePathStore>,
    mut solver: ResMut<ThroughputSolverState>,
    mut diagnostics: ResMut<LogisticsDiagnostics>,
) {
    for edge in flow.edges.iter() {
        if !edge.path_open {
            continue;
        }
        let Some(handle) = edge.route_handle else {
            continue;
        };
        if handle.topology_revision != solver.topology_revision {
            continue;
        }
        let Some(cached) = route_cache.routes.get(&(edge.from, edge.to)) else {
            continue;
        };
        let path_edges = path_store.edge_slice(cached.path);
        let mut amount = edge.max_rate;
        let mut blocked: Option<crate::systems::transport::TransportEdgeId> = None;
        for &tid in path_edges {
            let idx = tid.0 as usize;
            if idx >= solver.capacity.len() {
                amount = 0.0;
                blocked = Some(tid);
                break;
            }
            let cap = solver.capacity[idx];
            let used = solver.load[idx] + solver.reserved[idx];
            let avail = (cap - used).max(0.0);
            if avail < amount {
                amount = avail;
                if amount <= 0.0 {
                    blocked = Some(tid);
                }
            }
        }
        if amount > 0.0 {
            for &tid in path_edges {
                let idx = tid.0 as usize;
                if idx < solver.reserved.len() {
                    solver.reserved[idx] += amount;
                    solver.load[idx] += amount;
                    let cap = solver.capacity[idx].max(1e-6);
                    solver.edge_pressure[idx] = (solver.load[idx] / cap).clamp(0.0, 2.0);
                }
            }
        }
        let req_id = diagnostics.request_id_seq.saturating_add(1);
        diagnostics.request_id_seq = req_id;
        diagnostics.proofs.push(RouteProof {
            request_id: req_id,
            from_catalog: String::new(),
            to_catalog: String::new(),
            requested: edge.max_rate,
            delivered: amount,
            blocked_at: blocked,
            bottleneck_capacity: cached.bottleneck_capacity,
        });
        if diagnostics.proofs.len() > 64 {
            diagnostics.proofs.drain(0..32);
        }
    }
}

pub fn feedback_congestion_from_load_system(
    solver: Res<ThroughputSolverState>,
    fields: Option<ResMut<TransportFieldStore>>,
    directory: Option<Res<TransportEdgeDirectory>>,
) {
    let (Some(mut fields), Some(directory)) = (fields, directory) else {
        return;
    };
    for (&eid, _) in directory.by_edge.iter() {
        let idx = eid.0 as usize;
        if idx >= solver.load.len() {
            continue;
        }
        let saturation = solver.edge_pressure.get(idx).copied().unwrap_or(0.0);
        if saturation <= 0.0 {
            continue;
        }
        if let Some(state) = fields.by_edge.get_mut(&eid) {
            state.congestion = (state.congestion + 0.12 * saturation).min(1.0);
        }
    }
}

pub fn propagate_corridor_pressure_system(
    mut solver: ResMut<ThroughputSolverState>,
    nav: Option<Res<crate::systems::transport::TransportNavExport>>,
    mut next_pressure: Local<Vec<f32>>,
) {
    let Some(nav) = nav else {
        return;
    };
    let n = solver.edge_pressure.len();
    if n == 0 {
        return;
    }
    next_pressure.resize(n, 0.0);
    for v in next_pressure.iter_mut() {
        *v = 0.0;
    }
    for e in nav.edges.iter() {
        let idx = e.id.0 as usize;
        if idx >= n {
            continue;
        }
        let p = solver.edge_pressure[idx];
        if p <= 0.01 {
            continue;
        }
        for &s in &e.successors {
            let si = s.0 as usize;
            if si < n {
                next_pressure[si] = next_pressure[si].max(p * 0.35);
            }
        }
    }
    for (i, &np) in next_pressure.iter().enumerate().take(n) {
        if np > solver.edge_pressure[i] {
            solver.edge_pressure[i] = np;
        }
    }
}
