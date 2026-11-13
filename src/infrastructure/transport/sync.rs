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

#[cfg(test)]
mod tests {
    

}
