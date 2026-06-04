//! **INFRA-E1-001** transport graph resource (minimal B/A shared surface).

use bevy::prelude::*;
use std::collections::HashMap;

use crate::systems::transport::{CorridorClass, TransportEdgeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct TransportNodeId(pub u64);

#[derive(Clone, Debug)]
pub struct TransportNode {
    pub position: Vec3,
    pub junction_kind: super::junction::JunctionKind,
}

#[derive(Clone, Debug)]
pub struct TransportEdge {
    pub head: TransportNodeId,
    pub tail: TransportNodeId,
    pub profile_id: String,
    pub control_points: Vec<[f32; 3]>,
    pub corridor: CorridorClass,
    pub allowed_agents: Vec<String>,
}

#[derive(Resource, Default, Debug)]
pub struct TransportGraph {
    pub nodes: HashMap<TransportNodeId, TransportNode>,
    pub edges: HashMap<TransportEdgeId, TransportEdge>,
    pub adjacency: HashMap<TransportEdgeId, Vec<TransportEdgeId>>,
}

impl TransportGraph {
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.adjacency.clear();
    }

    pub fn insert_edge(&mut self, id: TransportEdgeId, edge: TransportEdge) {
        self.adjacency
            .entry(id)
            .or_default()
            .retain(|_| false);
        self.edges.insert(id, edge);
    }

    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::transport::junction::ensure_edge_endpoints;
    use crate::systems::transport::CorridorClass;

    #[test]
    fn infra_e1_001_insert_three_node_line_graph() {
        let mut graph = TransportGraph::default();
        let e0 = TransportEdgeId(0);
        let e1 = TransportEdgeId(1);
        let (h0, t0) = ensure_edge_endpoints(
            &mut graph,
            e0,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        );
        let (h1, t1) = ensure_edge_endpoints(
            &mut graph,
            e1,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        );
        graph.insert_edge(
            e0,
            TransportEdge {
                head: h0,
                tail: t0,
                profile_id: "default_road".into(),
                control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                corridor: CorridorClass::Road,
                allowed_agents: vec![],
            },
        );
        graph.insert_edge(
            e1,
            TransportEdge {
                head: h1,
                tail: t1,
                profile_id: "default_road".into(),
                control_points: vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
                corridor: CorridorClass::Road,
                allowed_agents: vec![],
            },
        );
        graph.adjacency.insert(e0, vec![e1]);
        graph.adjacency.insert(e1, vec![]);
        assert_eq!(graph.edge_count(), 2);
        assert_eq!(graph.adjacency.get(&e0).map(|v| v.len()), Some(1));
    }
}
