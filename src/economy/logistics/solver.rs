//! Throughput solver + corridor pressure (LOG-C).
//!
//! **NAV-11** (aig3 4.7.5 *Interruptible Pathfinding* · 4.7.6 *Pooling Planners* · LEGACY
//! `pathfinding` aig_e3): [`solve_throughput_greedy_system`] is **interruptible** — each tick
//! advances at most [`throughput_solve_max_edges`] flow edges via `solve_flow_cursor`, then
//! resumes next tick without restarting the pass. SoA load/reserved/pressure vectors are the
//! **pooled planner** state (cleared at pass start, not reallocated per edge).
//!
//! Ref-books **LOOP-11** · LEGACY `game_object_update` (GEA 4e I §8.3.3 Event-Based Updating,
//! gea_v1_e4 p511–512 / gea_v2_e4 p606): **do not tick** the greedy reservation pass every
//! frame — [`solve_throughput_greedy_system`] early-outs via [`throughput_solve_unchanged`]
//! until topology revision or flow-edge snapshot changes (graph revision / freight reopen).
//!
//! Ref-books **MEM-06** · LEGACY closed/open hashing (GEA 4e I §6.3.6.1 p437-438 collisions,
//! §6.3.6.3 p440 closed table, §6.3.6.4 p440-441 Robin Hood; no dedicated legacy keyword —
//! `memory_management` gea_v1_e4 p412 is the nearest shipped cite): **on the solve hot path
//! prefer index vectors; if hashing, know the probe cost.**
//!
//! Hot-path key spaces (all non-string):
//! - **Per path edge** (inner loop, `solve_greedy_one_flow_edge`): `TransportEdgeId(u64)` →
//!   `usize` index into the SoA `capacity`/`load`/`reserved`/`edge_pressure` vectors (LOG-C-01).
//!   Zero hashing, zero probes — the edge id *is* the slot.
//! - **Per flow edge** (outer loop): exactly one `RouteCache.routes` probe keyed by
//!   `(Entity, Entity)` (two integers, never a string). Std `HashMap` is open-addressing
//!   (SwissTable) with SipHash; expected O(1) but probe chains grow with load, so it is bounded
//!   to one probe per flow edge per pass, and the pass is gated by
//!   [`throughput_solve_unchanged`] (LOOP-11) and budgeted by [`throughput_solve_max_edges`]
//!   (EXEC-04). A handle-indexed `Vec<CachedRoute>` (`RouteHandle.id`) is the next step *only
//!   if* a profile shows this probe; it is owned by `routes.rs`, not this file.
//! - **Off the solve** (`feedback_congestion_from_load_system`): `by_edge` is *iterated*, then
//!   one `u64`-keyed `get_mut` per saturated edge (`saturation <= 0` skipped first). Writes are
//!   per-edge independent, so `HashMap` iteration order cannot change the result.
//!
//! Tripwires: `mem06_*` tests below. **Debt not taken:** `TransportFieldStore.by_edge` /
//! `TransportEdgeDirectory.by_edge` remain `HashMap<TransportEdgeId, _>` (shared with the
//! transport lane, not a solver-owned layout) and `RouteCache.routes` stays hashed.
//!
//! **CLN-P0-P10 / EC-LOG:** the route solve allocates no `HashMap`/`HashSet` (the pass snapshots
//! flow edges into reused `Vec`s in `begin_solve_pass`); remaining allocation pressure is
//! `LogisticsDiagnostics.proofs`, capped at 64.

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

/// Default per-tick edge budget before [`AttentionTier::work_scale`] (NAV-11).
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
    // LOOP-11: skip work when topology + flow snapshot match the last completed solve.
    if throughput_solve_unchanged(&solver, &flow) {
        return;
    }
    // CQ-13: non-player theaters share Aggregate budget whether or not foci are listed.
    let tier = attention.as_ref().map_or(AttentionTier::Full, |a| {
        if a.player_theater_active {
            AttentionTier::Full
        } else {
            AttentionTier::Aggregate
        }
    });
    let base = attention
        .as_ref()
        .map(|a| a.budget_path_units)
        .unwrap_or(THROUGHPUT_SOLVE_BASE_EDGE_BUDGET);
    let max_iter = throughput_solve_max_edges(tier.work_scale(), base);
    let _ = advance_interruptible_solve_pass(
        &flow,
        &route_cache,
        &path_store,
        &mut solver,
        &mut book,
        &mut diagnostics,
        max_iter,
    );
    debug_assert!(soa_solver_aligned(&solver) || solver.capacity.is_empty());
    debug_assert!(reservations_within_capacity(&solver));
    debug_assert!(book_matches_soa_reserved(&book, &solver));
}

/// **LOOP-11 gate:** `true` ⇒ greedy solve must **not** tick (revision + flow snapshot still valid).
#[must_use]
pub fn throughput_solve_unchanged(
    solver: &ThroughputSolverState,
    flow: &ResourceFlowRegistry,
) -> bool {
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

/// NAV-11: process up to `max_iter` flow edges; returns how many were advanced.
#[must_use]
pub fn advance_interruptible_solve_pass(
    flow: &ResourceFlowRegistry,
    route_cache: &RouteCache,
    path_store: &RoutePathStore,
    solver: &mut ThroughputSolverState,
    book: &mut FreightReservationBook,
    diagnostics: &mut LogisticsDiagnostics,
    max_iter: usize,
) -> usize {
    if max_iter == 0 {
        return 0;
    }
    if !solve_pass_in_progress(solver, flow) {
        begin_solve_pass(solver, flow, book, diagnostics);
    }
    let mut processed = 0usize;
    while solver.solve_flow_cursor < flow.edges.len() && processed < max_iter {
        let edge = &flow.edges[solver.solve_flow_cursor];
        solve_greedy_one_flow_edge(edge, solver, book, diagnostics, route_cache, path_store);
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
    processed
}

#[cfg(test)]
mod anytime_tests {
    use super::*;
    use crate::economy::resource_flow::{ResourceFlowEdge, ResourceFlowRegistry, TransportMode};
    use bevy::prelude::Entity;

    fn multi_flow(n: usize) -> ResourceFlowRegistry {
        ResourceFlowRegistry {
            edges: (0..n)
                .map(|_| ResourceFlowEdge {
                    from: Entity::PLACEHOLDER,
                    to: Entity::PLACEHOLDER,
                    transport_mode: TransportMode::Truck,
                    max_rate: 1.0,
                    latency_ticks: 1.0,
                    path_open: false,
                    route_handle: None,
                    buffer_tag: None,
                })
                .collect(),
        }
    }

    #[test]
    fn throughput_solve_max_edges_scales_with_attention() {
        assert_eq!(throughput_solve_max_edges(1.0, 64), 64);
        assert_eq!(throughput_solve_max_edges(0.5, 64), 32);
        assert_eq!(throughput_solve_max_edges(0.0, 64), 0);
        assert_eq!(throughput_solve_max_edges(0.1, 10), 1);
    }

    #[test]
    fn nav11_zero_budget_processes_nothing() {
        let flow = multi_flow(4);
        let mut solver = ThroughputSolverState::default();
        solver.topology_revision = 1;
        let mut book = FreightReservationBook::default();
        let mut diagnostics = LogisticsDiagnostics::default();
        let route_cache = RouteCache::default();
        let path_store = RoutePathStore::default();
        let n = advance_interruptible_solve_pass(
            &flow,
            &route_cache,
            &path_store,
            &mut solver,
            &mut book,
            &mut diagnostics,
            0,
        );
        assert_eq!(n, 0);
        assert_eq!(solver.solve_flow_cursor, 0);
    }

    #[test]
    fn nav11_interrupt_then_resume_completes_pass() {
        let flow = multi_flow(5);
        let mut solver = ThroughputSolverState::default();
        solver.topology_revision = 3;
        let mut book = FreightReservationBook::default();
        let mut diagnostics = LogisticsDiagnostics::default();
        let route_cache = RouteCache::default();
        let path_store = RoutePathStore::default();

        let a = advance_interruptible_solve_pass(
            &flow,
            &route_cache,
            &path_store,
            &mut solver,
            &mut book,
            &mut diagnostics,
            2,
        );
        assert_eq!(a, 2);
        assert_eq!(solver.solve_flow_cursor, 2);
        assert!(solver.solved_revision.is_none());

        let b = advance_interruptible_solve_pass(
            &flow,
            &route_cache,
            &path_store,
            &mut solver,
            &mut book,
            &mut diagnostics,
            10,
        );
        assert_eq!(b, 3);
        assert_eq!(solver.solve_flow_cursor, 5);
        assert_eq!(solver.solved_revision, Some(3));
    }
}

#[cfg(test)]
mod loop11_revision_gate_tests {
    use super::*;
    use crate::economy::resource_flow::{ResourceFlowEdge, ResourceFlowRegistry, TransportMode};
    use bevy::prelude::Entity;

    fn sample_flow(path_open: bool, max_rate: f32) -> ResourceFlowRegistry {
        ResourceFlowRegistry {
            edges: vec![ResourceFlowEdge {
                from: Entity::PLACEHOLDER,
                to: Entity::PLACEHOLDER,
                transport_mode: TransportMode::Truck,
                max_rate,
                latency_ticks: 1.0,
                path_open,
                route_handle: None,
                buffer_tag: None,
            }],
        }
    }

    #[test]
    fn throughput_solve_unchanged_skips_matching_revision_and_flow() {
        let flow = sample_flow(true, 2.0);
        let mut solver = ThroughputSolverState::default();
        solver.topology_revision = 7;
        solver.solved_revision = Some(7);
        solver.solved_path_open = vec![true];
        solver.solved_max_rate = vec![2.0];
        assert!(
            throughput_solve_unchanged(&solver, &flow),
            "LOOP-11: matching revision+flow must skip"
        );
    }

    #[test]
    fn throughput_solve_unchanged_false_on_topology_bump() {
        let flow = sample_flow(true, 2.0);
        let mut solver = ThroughputSolverState::default();
        solver.topology_revision = 8;
        solver.solved_revision = Some(7);
        solver.solved_path_open = vec![true];
        solver.solved_max_rate = vec![2.0];
        assert!(
            !throughput_solve_unchanged(&solver, &flow),
            "LOOP-11: topology bump must retick"
        );
    }

    #[test]
    fn throughput_solve_unchanged_false_on_flow_edge_change() {
        let flow = sample_flow(false, 2.0);
        let mut solver = ThroughputSolverState::default();
        solver.topology_revision = 7;
        solver.solved_revision = Some(7);
        solver.solved_path_open = vec![true];
        solver.solved_max_rate = vec![2.0];
        assert!(
            !throughput_solve_unchanged(&solver, &flow),
            "LOOP-11: path_open change must retick"
        );
    }

    #[test]
    fn throughput_solve_unchanged_false_before_first_solve() {
        let flow = sample_flow(true, 1.0);
        let solver = ThroughputSolverState::default();
        assert!(!throughput_solve_unchanged(&solver, &flow));
    }
}

/// MEM-06 tripwires: the solve hot path indexes SoA vectors; its only hash probe is one
/// `(Entity, Entity)` route-cache lookup per flow edge, never a string key.
#[cfg(test)]
mod mem06_hot_path_index_tests {
    use super::*;
    use crate::economy::logistics::types::{CachedRoute, RouteHandle};
    use crate::economy::resource_flow::{ResourceFlowEdge, TransportMode};
    use crate::systems::transport::TransportEdgeId;
    use std::collections::HashMap;

    const SRC: &str = include_str!("solver.rs");

    /// Source of `solve_greedy_one_flow_edge` only (start marker's first hit is the real fn).
    fn greedy_edge_body() -> &'static str {
        let start = SRC
            .find("fn solve_greedy_one_flow_edge(")
            .expect("solve_greedy_one_flow_edge present");
        let rest = &SRC[start..];
        let end = rest
            .find("pub fn sync_solver_capacity_from_graph_system(")
            .expect("end marker present");
        &rest[..end]
    }

    #[test]
    fn mem06_key_types_are_integer_not_string() {
        // Compile-time: route-cache key is two integers, edge id is a u64 slot index.
        let cache = RouteCache::default();
        let _: &HashMap<(Entity, Entity), CachedRoute> = &cache.routes;
        let _: u64 = TransportEdgeId(0).0;
    }

    #[test]
    fn mem06_greedy_edge_has_single_integer_probe_and_no_string_hash() {
        let body = greedy_edge_body();
        // Exactly one hashed probe: the (Entity, Entity) route-cache lookup.
        assert_eq!(body.matches(".routes.get(").count(), 1, "one route probe per flow edge");
        assert!(body.contains(".routes.get(&(edge.from, edge.to))"));
        // No other map/set access or string-keyed work on the per-edge path.
        for banned in [
            "HashMap", "HashSet", ".entry(", ".contains_key(", ".get_mut(&", "format!(",
            ".to_string()", "String::from", ".to_owned()", ".get(\"", "by_edge",
        ] {
            assert!(!body.contains(banned), "MEM-06: `{banned}` on solve hot path");
        }
        // Per-path-edge slots are direct SoA index loads.
        assert!(body.contains("solver.capacity[idx]"));
        assert!(body.contains("solver.reserved[idx]"));
        assert!(body.contains("solver.load[idx]"));
    }

    #[test]
    fn mem06_path_edges_land_in_soa_slots_by_index() {
        let mut solver = ThroughputSolverState::default();
        solver.topology_revision = 3;
        solver.ensure_len(4);
        solver.capacity.copy_from_slice(&[10.0, 2.0, 10.0, 10.0]);

        let mut store = RoutePathStore::default();
        let path = store.insert_path(&[TransportEdgeId(0), TransportEdgeId(1), TransportEdgeId(3)]);
        let handle = RouteHandle { id: 1, topology_revision: 3 };
        let mut cache = RouteCache::default();
        cache.routes.insert(
            (Entity::PLACEHOLDER, Entity::PLACEHOLDER),
            CachedRoute { handle, path, reachable: true, bottleneck_capacity: 2.0 },
        );
        let edge = ResourceFlowEdge {
            from: Entity::PLACEHOLDER,
            to: Entity::PLACEHOLDER,
            transport_mode: TransportMode::Truck,
            max_rate: 5.0,
            latency_ticks: 1.0,
            path_open: true,
            route_handle: Some(handle),
            buffer_tag: None,
        };
        let mut book = FreightReservationBook::default();
        let mut diag = LogisticsDiagnostics::default();
        solve_greedy_one_flow_edge(&edge, &mut solver, &mut book, &mut diag, &cache, &store);

        // Bottleneck (edge 1, cap 2) bounds every slot on the path; untouched slot stays 0.
        assert_eq!(solver.reserved, vec![2.0, 2.0, 0.0, 2.0]);
        assert_eq!(solver.load, vec![2.0, 2.0, 0.0, 2.0]);
        assert_eq!(book.entries.len(), 3);
        assert!(book_matches_soa_reserved(&book, &solver));
        assert!(soa_solver_aligned(&solver) && reservations_within_capacity(&solver));
        assert_eq!(diag.proofs.len(), 1);
        assert_eq!(diag.proofs[0].delivered, 2.0);

        // Stale handle revision is rejected by integer compare before any slot is touched.
        let mut stale = edge.clone();
        stale.route_handle = Some(RouteHandle { id: 1, topology_revision: 2 });
        let mut book2 = FreightReservationBook::default();
        let mut solver2 = ThroughputSolverState::default();
        solver2.topology_revision = 3;
        solver2.ensure_len(4);
        solver2.capacity.copy_from_slice(&[10.0; 4]);
        solve_greedy_one_flow_edge(&stale, &mut solver2, &mut book2, &mut diag, &cache, &store);
        assert!(solver2.reserved.iter().all(|&r| r == 0.0) && book2.entries.is_empty());
    }
}
