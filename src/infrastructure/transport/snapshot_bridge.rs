//! **INFRA-E1-004** — snapshot ↔ `TransportGraph` round-trip helpers.

use bevy::prelude::Vec3;

use super::graph::{TransportEdge, TransportGraph, TransportNode, TransportNodeId};
use super::junction::{ensure_edge_endpoints, rebuild_junction_metadata};
use crate::systems::transport::{
    bake_snapshot_from_ordered_markers_with_world_positions,
    corridor_class_from_profile, hydrate_transport_from_snapshot, TransportEdgeDirectory,
    TransportEdgeId, TransportEdgeRecord, TransportFieldStore, TransportNetworkSnapshot,
    TransportNodeRecord, TransportTopology, TRANSPORT_NETWORK_SCHEMA_V1,
};

/// **INFRA-E2-002** — ordered markers → authoritative graph (v2 bake path).
#[must_use]
pub fn bake_transport_graph_from_ordered_markers(
    markers_in_authoring_order: &[(u32, u32, Vec3)],
) -> TransportGraph {
    let snap = bake_snapshot_from_ordered_markers_with_world_positions(markers_in_authoring_order);
    hydrate_transport_graph_from_snapshot(&snap)
}

/// Build an in-memory graph from an R8 snapshot, then junction metadata.
pub fn hydrate_transport_graph_from_snapshot(
    snapshot: &TransportNetworkSnapshot,
) -> TransportGraph {
    let mut graph = TransportGraph::default();
    let mut node_key_to_id: std::collections::HashMap<String, TransportNodeId> =
        std::collections::HashMap::new();
    let mut next_node = 1_u64;

    for node in &snapshot.nodes {
        let id = TransportNodeId(next_node);
        next_node += 1;
        node_key_to_id.insert(node.key.clone(), id);
        graph.nodes.insert(
            id,
            TransportNode {
                position: Vec3::new(node.position[0], node.position[1], node.position[2]),
                junction_kind: super::junction::JunctionKind::Endpoint,
            },
        );
    }

    for edge in &snapshot.edges {
        let edge_id = TransportEdgeId(edge.id);
        let head_pos = node_position(snapshot, &edge.head)
            .or_else(|| edge.control_points.first().copied())
            .map(Vec3::from_array)
            .unwrap_or(Vec3::ZERO);
        let tail_pos = node_position(snapshot, &edge.tail)
            .or_else(|| edge.control_points.last().copied())
            .map(Vec3::from_array)
            .unwrap_or(Vec3::ZERO);

        let head = node_key_to_id
            .get(&edge.head)
            .copied()
            .unwrap_or_else(|| ensure_edge_endpoints(&mut graph, edge_id, head_pos, tail_pos).0);
        let tail = node_key_to_id
            .get(&edge.tail)
            .copied()
            .unwrap_or_else(|| ensure_edge_endpoints(&mut graph, edge_id, head_pos, tail_pos).1);

        let corridor = corridor_class_from_profile(&edge.profile);
        graph.insert_edge(
            edge_id,
            TransportEdge {
                head,
                tail,
                profile_id: edge.profile.clone(),
                control_points: edge.control_points.clone(),
                corridor,
                allowed_agents: edge.allowed_agents.clone(),
            },
        );
        let succ: Vec<TransportEdgeId> = edge.successors.iter().copied().map(TransportEdgeId).collect();
        graph.adjacency.insert(edge_id, succ);
    }

    rebuild_junction_metadata(&mut graph);
    graph
}

fn node_position(snapshot: &TransportNetworkSnapshot, key: &str) -> Option<[f32; 3]> {
    snapshot
        .nodes
        .iter()
        .find(|n| n.key == key)
        .map(|n| n.position)
}

/// Deterministic R8 DTO from graph (sorted node keys / edge ids).
#[must_use]
pub fn transport_network_snapshot_from_graph(graph: &TransportGraph) -> TransportNetworkSnapshot {
    let mut id_to_key: std::collections::HashMap<TransportNodeId, String> =
        std::collections::HashMap::new();
    let mut nodes: Vec<TransportNodeRecord> = Vec::new();

    let mut node_ids: Vec<TransportNodeId> = graph.nodes.keys().copied().collect();
    node_ids.sort_by_key(|n| n.0);
    for nid in node_ids.iter() {
        let key = format!("n{}", nid.0);
        id_to_key.insert(*nid, key.clone());
        let node = &graph.nodes[nid];
        nodes.push(TransportNodeRecord {
            key,
            position: node.position.to_array(),
        });
    }

    let mut edge_ids: Vec<TransportEdgeId> = graph.edges.keys().copied().collect();
    edge_ids.sort_by_key(|e| e.0);
    let mut edges = Vec::with_capacity(edge_ids.len());
    for eid in edge_ids {
        let edge = &graph.edges[&eid];
        let head = id_to_key
            .get(&edge.head)
            .cloned()
            .unwrap_or_else(|| format!("n{}", edge.head.0));
        let tail = id_to_key
            .get(&edge.tail)
            .cloned()
            .unwrap_or_else(|| format!("n{}", edge.tail.0));
        let succ = graph
            .adjacency
            .get(&eid)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|s| s.0)
            .collect();
        edges.push(TransportEdgeRecord {
            id: eid.0,
            head,
            tail,
            successors: succ,
            control_points: edge.control_points.clone(),
            profile: edge.profile_id.clone(),
            allowed_agents: edge.allowed_agents.clone(),
            ..Default::default()
        });
    }

    TransportNetworkSnapshot {
        schema_version: TRANSPORT_NETWORK_SCHEMA_V1,
        nodes,
        edges,
        construction: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::{
        hydrate_transport_from_snapshot, TransportEdgeDirectory, TransportEdgeRecord,
        TransportFieldStore, TransportNetworkSnapshot, TransportNodeRecord, TransportTopology,
        TRANSPORT_NETWORK_SCHEMA_V1,
    };

    fn sample_snapshot() -> TransportNetworkSnapshot {
        TransportNetworkSnapshot {
            schema_version: TRANSPORT_NETWORK_SCHEMA_V1,
            nodes: vec![
                TransportNodeRecord {
                    key: "a".into(),
                    position: [0., 0., 0.],
                },
                TransportNodeRecord {
                    key: "b".into(),
                    position: [1., 0., 0.],
                },
                TransportNodeRecord {
                    key: "c".into(),
                    position: [2., 0., 0.],
                },
            ],
            edges: vec![
                TransportEdgeRecord {
                    id: 10,
                    head: "a".into(),
                    tail: "b".into(),
                    successors: vec![11],
                    control_points: vec![[0., 0., 0.], [1., 0., 0.]],
                    profile: "road_local".into(),
                    allowed_agents: vec!["truck".into()],
                    ..Default::default()
                },
                TransportEdgeRecord {
                    id: 11,
                    head: "b".into(),
                    tail: "c".into(),
                    successors: vec![],
                    control_points: vec![[1., 0., 0.], [2., 0., 0.]],
                    profile: "road_local".into(),
                    allowed_agents: vec!["truck".into()],
                    ..Default::default()
                },
            ],
            construction: Vec::new(),
        }
    }

    #[test]
    fn infra_e2_002_bake_v2_graph_from_ordered_markers() {
        let markers = [
            (0u32, 0u32, Vec3::new(0.0, 0.0, 0.0)),
            (1u32, 0u32, Vec3::new(1.0, 0.0, 0.0)),
            (2u32, 0u32, Vec3::new(2.0, 0.0, 0.0)),
        ];
        let graph = super::bake_transport_graph_from_ordered_markers(&markers);
        assert_eq!(graph.edges.len(), 2);
        let snap = transport_network_snapshot_from_graph(&graph);
        assert_eq!(snap.edges.len(), 2);
    }

    #[test]
    fn transport_network_roundtrip_001_graph_hydrate_sync() {
        let snap = sample_snapshot();
        let graph = hydrate_transport_graph_from_snapshot(&snap);
        let back = transport_network_snapshot_from_graph(&graph);
        let mut topo = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut topo, &mut field, &mut dir, &back).unwrap();
        super::super::sync::sync_transport_runtime_from_graph(
            &graph, &mut topo, &mut field, &mut dir,
        );
        assert_eq!(topo.neighbors.len(), snap.edges.len());
        assert!(crate::infrastructure::transport::infra_e1_001_transport_graph_sync_witness_green());
    }

    #[test]
    fn infra_e1_004_snapshot_graph_snapshot_round_trip() {
        let snap = sample_snapshot();
        let graph = hydrate_transport_graph_from_snapshot(&snap);
        let back = transport_network_snapshot_from_graph(&graph);
        assert_eq!(back.schema_version, snap.schema_version);
        assert_eq!(back.nodes.len(), snap.nodes.len());
        assert_eq!(back.edges.len(), snap.edges.len());
        for (a, b) in back.edges.iter().zip(snap.edges.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.successors, b.successors);
            assert_eq!(a.profile, b.profile);
            assert_eq!(a.control_points, b.control_points);
        }

        let mut topo = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut topo, &mut field, &mut dir, &snap).unwrap();
        hydrate_transport_from_snapshot(&mut topo, &mut field, &mut dir, &back).unwrap();
    }
}
