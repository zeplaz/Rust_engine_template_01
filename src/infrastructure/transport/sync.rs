//! **INFRA-E1-001** — push authoritative [`TransportGraph`] into runtime transport resources.

use super::graph::TransportGraph;
use crate::systems::transport::{
    EdgeFieldState, TransportEdgeDirectory, TransportEdgeId, TransportEdgeMeta, TransportFieldStore,
    TransportTopology,
};

fn polyline_length(points: &[[f32; 3]]) -> f32 {
    if points.len() < 2 {
        return 0.;
    }
    let mut sum = 0.0_f32;
    for w in points.windows(2) {
        let a = w[0];
        let b = w[1];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        sum += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    sum
}

fn node_key(id: super::graph::TransportNodeId) -> String {
    format!("n{}", id.0)
}

/// Replace topology / field / directory from graph (graph is authoritative when non-empty).
pub fn sync_transport_runtime_from_graph(
    graph: &TransportGraph,
    topology: &mut TransportTopology,
    field_store: &mut TransportFieldStore,
    edge_directory: &mut TransportEdgeDirectory,
) {
    topology.neighbors.clear();
    field_store.by_edge.clear();
    edge_directory.by_edge.clear();

    let mut edge_ids: Vec<TransportEdgeId> = graph.edges.keys().copied().collect();
    edge_ids.sort_by_key(|e| e.0);

    for eid in edge_ids {
        let edge = match graph.edges.get(&eid) {
            Some(e) => e,
            None => continue,
        };
        let succ = graph
            .adjacency
            .get(&eid)
            .cloned()
            .unwrap_or_default();
        topology.neighbors.insert(eid, succ);

        let base = polyline_length(&edge.control_points).max(0.1);
        field_store.by_edge.insert(
            eid,
            EdgeFieldState {
                travel_time_base: base,
                ..Default::default()
            },
        );

        edge_directory.by_edge.insert(
            eid,
            TransportEdgeMeta {
                profile: edge.profile_id.clone(),
                corridor_class: edge.corridor,
                allowed_agents: edge.allowed_agents.clone(),
                head_key: node_key(edge.head),
                tail_key: node_key(edge.tail),
                control_points: edge.control_points.clone(),
            },
        );
    }
}

#[must_use]
pub fn infra_e1_001_transport_graph_sync_witness_green() -> bool {
    use super::graph::{TransportEdge, TransportGraph};
    use super::junction::{ensure_edge_endpoints, rebuild_junction_metadata};
    use crate::systems::transport::CorridorClass;

    let mut graph = TransportGraph::default();
    let e1 = TransportEdgeId(1);
    let e2 = TransportEdgeId(2);
    let (h1, t1) = ensure_edge_endpoints(
        &mut graph,
        e1,
        bevy::prelude::Vec3::new(0.0, 0.0, 0.0),
        bevy::prelude::Vec3::new(1.0, 0.0, 0.0),
    );
    let (h2, t2) = ensure_edge_endpoints(
        &mut graph,
        e2,
        bevy::prelude::Vec3::new(1.0, 0.0, 0.0),
        bevy::prelude::Vec3::new(2.0, 0.0, 0.0),
    );
    graph.insert_edge(
        e1,
        TransportEdge {
            head: h1,
            tail: t1,
            profile_id: "default_road".into(),
            control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            corridor: CorridorClass::Road,
            allowed_agents: vec!["truck".into()],
        },
    );
    graph.insert_edge(
        e2,
        TransportEdge {
            head: h2,
            tail: t2,
            profile_id: "default_road".into(),
            control_points: vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            corridor: CorridorClass::Road,
            allowed_agents: vec!["truck".into()],
        },
    );
    graph.adjacency.insert(e1, vec![e2]);
    graph.adjacency.insert(e2, vec![]);
    rebuild_junction_metadata(&mut graph);

    let mut topo = TransportTopology::default();
    let mut field = TransportFieldStore::default();
    let mut dir = TransportEdgeDirectory::default();
    sync_transport_runtime_from_graph(&graph, &mut topo, &mut field, &mut dir);

    topo.neighbors.get(&e1).map(|s| s.as_slice()) == Some(&[e2])
        && topo.neighbors.get(&e2).map(|s| s.is_empty()).unwrap_or(false)
        && dir.by_edge.len() == 2
        && field.by_edge.len() == 2
        && graph.nodes.values().any(|n| n.position.x >= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infra_e1_001_three_node_line_adjacency_round_trip() {
        assert!(infra_e1_001_transport_graph_sync_witness_green());
    }
}
