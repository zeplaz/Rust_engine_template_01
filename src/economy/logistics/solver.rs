//! Throughput solver + corridor pressure (LOG-C).
//!
//! **CLN-P0-P10 / EC-LOG:** route solve allocates `HashMap`/`HashSet` per invocation — acceptable
//! because `solve_throughput_greedy_system` runs on-demand (graph revision / freight dispatch), not
//! every sim frame. Pool or reuse if it ever moves into a per-frame hot path.

use bevy::prelude::*;

use crate::strategic::LogisticsGraph;
use crate::systems::transport::{TransportEdgeDirectory, TransportFieldStore};

use super::routes::topology_revision_u32;
use super::types::{
    FreightReservation, FreightReservationBook, LogisticsDiagnostics, RouteProof,
    ThroughputSolverState,
};
use super::types::{RouteCache, RoutePathStore};
use crate::construction::ConstructionWorldRevision;
use crate::economy::resource_flow::{ResourceFlowEdge, ResourceFlowRegistry};
use crate::systems::sim_attention::{AttentionTier, SimAttentionFrame};

/// LOG-C-01: SoA vectors share one edge-index space (no HashMap on the hot path).
#[must_use]
pub fn soa_solver_aligned(solver: &ThroughputSolverState) -> bool {
    let n = solver.capacity.len();
    n > 0 && solver.load.len() == n && solver.reserved.len() == n && solver.edge_pressure.len() == n
}

/// LOG-C-02: every edge reservation must stay within capacity after solve.
#[must_use]
pub fn reservations_within_capacity(solver: &ThroughputSolverState) -> bool {
    solver
        .capacity
        .iter()
        .zip(solver.reserved.iter())
        .all(|(&cap, &res)| res <= cap + 1e-4)
}

/// Book sparse sums must equal SoA `reserved` (single solve writer).
#[must_use]
pub fn book_matches_soa_reserved(
    book: &FreightReservationBook,
    solver: &ThroughputSolverState,
) -> bool {
    let sums = book.reserved_sums(solver.reserved.len());
    sums.iter()
        .zip(solver.reserved.iter())
        .all(|(a, b)| (a - b).abs() <= 1e-3)
}

/// Default per-tick edge budget before [`AttentionTier::work_scale`].
pub const THROUGHPUT_SOLVE_BASE_EDGE_BUDGET: u32 = 64;

#[must_use]
pub fn throughput_solve_max_edges(work_scale: f32, base_budget: u32) -> usize {
    if work_scale <= 0.0 {
        return 0;
    }
    ((base_budget as f32 * work_scale).ceil() as u32)
        .max(1)
        .min(u32::MAX) as usize
}

#[must_use]
fn solve_target_matches_flow(solver: &ThroughputSolverState, flow: &ResourceFlowRegistry) -> bool {
    if solver.solve_target_topology != solver.topology_revision {
        return false;
    }
    if solver.solve_target_path_open.len() != flow.edges.len()
        || solver.solve_target_max_rate.len() != flow.edges.len()
    {
        return false;
    }
    flow.edges.iter().enumerate().all(|(i, edge)| {
        solver.solve_target_path_open[i] == edge.path_open
            && solver.solve_target_max_rate[i] == edge.max_rate
    })
}

#[must_use]
fn solve_pass_in_progress(solver: &ThroughputSolverState, flow: &ResourceFlowRegistry) -> bool {
    solver.solve_flow_cursor < flow.edges.len() && solve_target_matches_flow(solver, flow)
}

fn begin_solve_pass(
    solver: &mut ThroughputSolverState,
    flow: &ResourceFlowRegistry,
    book: &mut FreightReservationBook,
    diagnostics: &mut LogisticsDiagnostics,
) {
    solver.clear_tick();
    book.clear();
    diagnostics.proofs.clear();
    solver.solve_flow_cursor = 0;
    solver.solve_target_topology = solver.topology_revision;
    solver.solve_target_path_open.clear();
    solver.solve_target_max_rate.clear();
    solver
        .solve_target_path_open
        .extend(flow.edges.iter().map(|e| e.path_open));
    solver
        .solve_target_max_rate
        .extend(flow.edges.iter().map(|e| e.max_rate));
}

fn solve_greedy_one_flow_edge(
    edge: &ResourceFlowEdge,
    solver: &mut ThroughputSolverState,
    book: &mut FreightReservationBook,
    diagnostics: &mut LogisticsDiagnostics,
    route_cache: &RouteCache,
    path_store: &RoutePathStore,
) {
    if !edge.path_open {
        return;
    }
    let Some(handle) = edge.route_handle else {
        return;
    };
    if handle.topology_revision != solver.topology_revision {
        return;
    }
    let Some(cached) = route_cache.routes.get(&(edge.from, edge.to)) else {
        return;
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
    let req_id = diagnostics.request_id_seq.saturating_add(1);
    diagnostics.request_id_seq = req_id;
    if amount > 0.0 {
        for &tid in path_edges {
            let idx = tid.0 as usize;
            if idx < solver.reserved.len() {
                solver.reserved[idx] += amount;
                solver.load[idx] += amount;
                let cap = solver.capacity[idx].max(1e-6);
                solver.edge_pressure[idx] = (solver.load[idx] / cap).clamp(0.0, 2.0);
                book.push(FreightReservation {
                    edge_index: idx,
                    amount,
                    request_id: req_id,
                });
            }
        }
    }
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
    // Load/reserved are cleared by the solve pass itself; clearing here every tick left them
    // zero whenever the solver reused its cached result (LOG-C-02 regression).
    let mut capacity_changed = false;

    for edge in &graph.edges {
        let Some(tid) = edge.transport_edge else {
            continue;
        };
        let idx = tid.0 as usize;
        if idx < solver.capacity.len() {
            let cap = edge.capacity * (1.0 - edge.disruption.clamp(0.0, 1.0));
            if (solver.capacity[idx] - cap).abs() > 1e-6 {
                capacity_changed = true;
            }
            solver.capacity[idx] = cap;
        }
    }
    if capacity_changed {
        // Cached reservations may exceed the new capacity — force a fresh solve.
        solver.solved_revision = None;
    }
}

pub fn solve_throughput_greedy_system(
    flow: Res<ResourceFlowRegistry>,
    route_cache: Res<RouteCache>,
    path_store: Res<RoutePathStore>,
    attention: Option<Res<SimAttentionFrame>>,
    mut solver: ResMut<ThroughputSolverState>,
    mut book: ResMut<FreightReservationBook>,
    mut diagnostics: ResMut<LogisticsDiagnostics>,
) {
    if flow_snapshot_unchanged(&solver, &flow) {
        return;
    }
    let tier = attention.as_ref().map_or(AttentionTier::Full, |a| {
        if a.player_theater_active {
            AttentionTier::Full
        } else if a.foci.is_empty() {
            AttentionTier::Aggregate
        } else {
            AttentionTier::Aggregate
        }
    });
    let base = attention
        .as_ref()
        .map(|a| a.budget_path_units)
        .unwrap_or(THROUGHPUT_SOLVE_BASE_EDGE_BUDGET);
    let max_iter = throughput_solve_max_edges(tier.work_scale(), base);
    let in_progress = solve_pass_in_progress(&solver, &flow);
    if max_iter == 0 {
        return;
    }
    if !in_progress {
        begin_solve_pass(&mut solver, &flow, &mut book, &mut diagnostics);
    }
    let mut processed = 0usize;
    while solver.solve_flow_cursor < flow.edges.len() && processed < max_iter {
        let edge = &flow.edges[solver.solve_flow_cursor];
        solve_greedy_one_flow_edge(
            edge,
            &mut solver,
            &mut book,
            &mut diagnostics,
            &route_cache,
            &path_store,
        );
        debug_assert!(soa_solver_aligned(&solver));
        debug_assert!(reservations_within_capacity(&solver));
        debug_assert!(book_matches_soa_reserved(&book, &solver));
        solver.solve_flow_cursor += 1;
        processed += 1;
    }
    if solver.solve_flow_cursor >= flow.edges.len() {
        solver.solved_revision = Some(solver.topology_revision);
        solver.solved_path_open.clear();
        solver.solved_max_rate.clear();
        solver
            .solved_path_open
            .extend(flow.edges.iter().map(|edge| edge.path_open));
        solver
            .solved_max_rate
            .extend(flow.edges.iter().map(|edge| edge.max_rate));
    }
}

fn flow_snapshot_unchanged(solver: &ThroughputSolverState, flow: &ResourceFlowRegistry) -> bool {
    let Some(solved) = solver.solved_revision else {
        return false;
    };
    if solved != solver.topology_revision {
        return false;
    }
    if solver.solved_path_open.len() != flow.edges.len()
        || solver.solved_max_rate.len() != flow.edges.len()
    {
        return false;
    }
    flow.edges.iter().enumerate().all(|(i, edge)| {
        solver.solved_path_open[i] == edge.path_open && solver.solved_max_rate[i] == edge.max_rate
    })
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

#[cfg(test)]
mod anytime_tests {
    use super::*;

    #[test]
    fn throughput_solve_max_edges_scales_with_attention() {
        assert_eq!(throughput_solve_max_edges(1.0, 64), 64);
        assert_eq!(throughput_solve_max_edges(0.5, 64), 32);
        assert_eq!(throughput_solve_max_edges(0.0, 64), 0);
        assert_eq!(throughput_solve_max_edges(0.1, 10), 1);
    }
}
