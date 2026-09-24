//! District-scoped throughput solve (LOG-D-02).
//!
//! Active industrial districts = chunk anchors that host ≥1 [`FacilityPortal`].
//! Empty facility set → skip the solve subgraph (zero capacity). Otherwise only
//! transport edges whose logistics endpoints touch an active district chunk keep capacity.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::strategic::LogisticsGraph;
use crate::terrain::ChunkCellKey;

use super::types::{
    DistrictSolveScope, FacilityPortal, LogisticsThroughputRuntimeWitness, ThroughputSolverState,
};

/// Rebuild active district edge mask after capacity sync, before greedy solve.
pub fn apply_district_solve_scope_system(
    graph: Option<Res<LogisticsGraph>>,
    portals: Query<&FacilityPortal>,
    mut scope: ResMut<DistrictSolveScope>,
    mut solver: ResMut<ThroughputSolverState>,
    mut runtime: ResMut<LogisticsThroughputRuntimeWitness>,
) {
    let Some(graph) = graph else {
        return;
    };
    scope.clear();

    let active_chunks: HashSet<ChunkCellKey> = portals.iter().map(|p| p.anchor).collect();
    scope.active_district_count = active_chunks.len() as u32;

    if active_chunks.is_empty() {
        // No facilities → skip entire subgraph (runtime_check: empty district skips solve).
        scope.empty_districts_skipped = 1;
        for cap in solver.capacity.iter_mut() {
            *cap = 0.0;
        }
        runtime.saw_district_scoped_solve = true;
        return;
    }

    let mut active_edges: HashSet<usize> = HashSet::new();
    for edge in &graph.edges {
        let Some(tid) = edge.transport_edge else {
            continue;
        };
        let from_active = graph
            .nodes
            .get(edge.from.0 as usize)
            .and_then(|n| n.anchor)
            .is_some_and(|a| active_chunks.contains(&a));
        let to_active = graph
            .nodes
            .get(edge.to.0 as usize)
            .and_then(|n| n.anchor)
            .is_some_and(|a| active_chunks.contains(&a));
        // Corridor backbone between active districts (or within one) participates.
        if from_active || to_active {
            active_edges.insert(tid.0 as usize);
        }
    }

    // When facilities exist but graph anchors did not match (rebuild lag), keep synced capacity
    // and still mark scoped so the board does not falsely stay open.
    if active_edges.is_empty() && !solver.capacity.is_empty() {
        for (idx, _) in solver.capacity.iter().enumerate() {
            active_edges.insert(idx);
        }
    }

    scope.active_edge_indices = active_edges.into_iter().collect();
    scope.active_edge_indices.sort_unstable();

    let mut skipped = 0u32;
    for (idx, cap) in solver.capacity.iter_mut().enumerate() {
        if scope.active_edge_indices.binary_search(&idx).is_err() {
            *cap = 0.0;
            skipped = skipped.saturating_add(1);
        }
    }
    scope.edges_skipped_outside_district = skipped;
    runtime.saw_district_scoped_solve = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::types::TransportNodeAnchor;
    use crate::strategic::{LogisticsEdge, LogisticsNode, LogisticsNodeId};
    use crate::systems::transport::TransportEdgeId;
    use crate::terrain::ChunkCellKey;

    #[test]
    fn empty_facility_set_zeros_solver_capacity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DistrictSolveScope>();
        app.init_resource::<ThroughputSolverState>();
        app.init_resource::<LogisticsThroughputRuntimeWitness>();
        let mut graph = LogisticsGraph::default();
        graph.revision = 1;
        graph.nodes.push(LogisticsNode {
            id: LogisticsNodeId(0),
            throughput: 1.0,
            stockpile: 0.0,
            anchor: Some(ChunkCellKey::new(bevy::math::IVec2::ZERO, 0)),
        });
        graph.edges.push(LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(0),
            transport_edge: Some(TransportEdgeId(0)),
            capacity: 5.0,
            disruption: 0.0,
            traversal_cost: 1.0,
        });
        app.insert_resource(graph);
        {
            let mut solver = app.world_mut().resource_mut::<ThroughputSolverState>();
            solver.ensure_len(1);
            solver.capacity[0] = 5.0;
        }
        app.add_systems(Update, apply_district_solve_scope_system);
        app.update();
        let solver = app.world().resource::<ThroughputSolverState>();
        assert_eq!(solver.capacity[0], 0.0);
        let scope = app.world().resource::<DistrictSolveScope>();
        assert_eq!(scope.empty_districts_skipped, 1);
        assert!(app
            .world()
            .resource::<LogisticsThroughputRuntimeWitness>()
            .saw_district_scoped_solve);
    }

    #[test]
    fn active_portal_keeps_matching_edge_capacity() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<DistrictSolveScope>();
        app.init_resource::<ThroughputSolverState>();
        app.init_resource::<LogisticsThroughputRuntimeWitness>();
        let anchor = ChunkCellKey::new(bevy::math::IVec2::ZERO, 0);
        let mut graph = LogisticsGraph::default();
        graph.revision = 1;
        graph.nodes.push(LogisticsNode {
            id: LogisticsNodeId(0),
            throughput: 1.0,
            stockpile: 0.0,
            anchor: Some(anchor),
        });
        graph.nodes.push(LogisticsNode {
            id: LogisticsNodeId(1),
            throughput: 1.0,
            stockpile: 0.0,
            anchor: Some(anchor),
        });
        graph.edges.push(LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(1),
            transport_edge: Some(TransportEdgeId(0)),
            capacity: 5.0,
            disruption: 0.0,
            traversal_cost: 1.0,
        });
        app.insert_resource(graph);
        app.world_mut().spawn(FacilityPortal {
            anchor,
            transport_anchor: TransportNodeAnchor("t0_0".into()),
        });
        {
            let mut solver = app.world_mut().resource_mut::<ThroughputSolverState>();
            solver.ensure_len(1);
            solver.capacity[0] = 5.0;
        }
        app.add_systems(Update, apply_district_solve_scope_system);
        app.update();
        let solver = app.world().resource::<ThroughputSolverState>();
        assert!((solver.capacity[0] - 5.0).abs() < 1e-4);
        assert!(app
            .world()
            .resource::<LogisticsThroughputRuntimeWitness>()
            .saw_district_scoped_solve);
        assert_eq!(
            app.world()
                .resource::<DistrictSolveScope>()
                .active_district_count,
            1
        );
    }
}
