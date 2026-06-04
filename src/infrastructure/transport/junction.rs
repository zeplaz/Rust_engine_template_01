//! **INFRA-E1-003** — junction detection on transport graph endpoints.

use bevy::prelude::*;
use std::collections::HashMap;

use super::graph::{TransportGraph, TransportNode, TransportNodeId};
use crate::systems::transport::TransportEdgeId;

pub const JUNCTION_MERGE_EPSILON: f32 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JunctionKind {
    Endpoint,
    PassThrough,
    Junction { degree: u8 },
}

/// Merge endpoints within epsilon and assign junction metadata.
pub fn rebuild_junction_metadata(graph: &mut TransportGraph) {
    let mut position_buckets: HashMap<(i32, i32, i32), Vec<TransportNodeId>> = HashMap::new();

    let mut node_positions: HashMap<TransportNodeId, Vec3> = HashMap::new();
    for (edge_id, edge) in &graph.edges {
        let _ = edge_id;
        if let Some(head) = graph.nodes.get(&edge.head) {
            node_positions.insert(edge.head, head.position);
        }
        if let Some(tail) = graph.nodes.get(&edge.tail) {
            node_positions.insert(edge.tail, tail.position);
        }
        if !node_positions.contains_key(&edge.head) {
            if let Some(p) = edge.control_points.first() {
                node_positions.insert(edge.head, Vec3::from_array(*p));
            }
        }
        if !node_positions.contains_key(&edge.tail) {
            if let Some(p) = edge.control_points.last() {
                node_positions.insert(edge.tail, Vec3::from_array(*p));
            }
        }
    }

    for (id, pos) in &node_positions {
        let key = quantize_pos(*pos);
        position_buckets.entry(key).or_default().push(*id);
    }

    let mut canonical: HashMap<TransportNodeId, TransportNodeId> = HashMap::new();
    for ids in position_buckets.values() {
        if ids.is_empty() {
            continue;
        }
        let master = *ids.iter().min_by_key(|n| n.0).unwrap();
        for id in ids {
            canonical.insert(*id, master);
        }
    }

    for (edge_id, edge) in graph.edges.iter_mut() {
        if let Some(&head_master) = canonical.get(&edge.head) {
            edge.head = head_master;
        }
        if let Some(&tail_master) = canonical.get(&edge.tail) {
            edge.tail = tail_master;
        }
        let _ = edge_id;
    }

    let mut degree: HashMap<TransportNodeId, u8> = HashMap::new();
    for edge in graph.edges.values() {
        *degree.entry(edge.head).or_insert(0) += 1;
        *degree.entry(edge.tail).or_insert(0) += 1;
    }

    for (id, pos) in node_positions {
        let master = canonical.get(&id).copied().unwrap_or(id);
        if id != master {
            continue;
        }
        let deg = degree.get(&master).copied().unwrap_or(0);
        let junction_kind = match deg {
            0 => JunctionKind::Endpoint,
            1 => JunctionKind::Endpoint,
            2 => JunctionKind::PassThrough,
            d => JunctionKind::Junction {
                degree: d.min(u8::MAX as u8),
            },
        };
        graph.nodes.insert(
            master,
            TransportNode {
                position: pos,
                junction_kind,
            },
        );
    }
}

#[inline]
fn quantize_pos(p: Vec3) -> (i32, i32, i32) {
    let inv = 1.0 / JUNCTION_MERGE_EPSILON;
    (
        (p.x * inv).round() as i32,
        (p.y * inv).round() as i32,
        (p.z * inv).round() as i32,
    )
}

/// On edge insert: ensure nodes exist for endpoints (used during graph hydrate).
pub fn ensure_edge_endpoints(
    graph: &mut TransportGraph,
    edge_id: TransportEdgeId,
    head_pos: Vec3,
    tail_pos: Vec3,
) -> (TransportNodeId, TransportNodeId) {
    let head_id = TransportNodeId(edge_id.0.wrapping_mul(2));
    let tail_id = TransportNodeId(edge_id.0.wrapping_mul(2).wrapping_add(1));
    graph.nodes.entry(head_id).or_insert_with(|| TransportNode {
        position: head_pos,
        junction_kind: JunctionKind::Endpoint,
    });
    graph.nodes.entry(tail_id).or_insert_with(|| TransportNode {
        position: tail_pos,
        junction_kind: JunctionKind::Endpoint,
    });
    (head_id, tail_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::transport::graph::{TransportEdge, TransportGraph};
    use crate::systems::transport::CorridorClass;

    #[test]
    fn infra_e1_003_two_roads_sharing_point_degree_three() {
        let mut graph = TransportGraph::default();
        let shared = Vec3::new(10.0, 0.0, 10.0);
        let e1 = TransportEdgeId(1);
        let e2 = TransportEdgeId(2);
        let (h1, t1) = ensure_edge_endpoints(&mut graph, e1, shared, Vec3::new(20.0, 0.0, 10.0));
        let (h2, t2) = ensure_edge_endpoints(&mut graph, e2, shared, Vec3::new(10.0, 0.0, 20.0));
        let _ = (t1, t2);

        graph.insert_edge(
            e1,
            TransportEdge {
                head: h1,
                tail: t1,
                profile_id: "road_local".into(),
                control_points: vec![shared.to_array(), Vec3::new(20.0, 0.0, 10.0).to_array()],
                corridor: CorridorClass::Road,
                allowed_agents: vec!["truck".into()],
            },
        );
        graph.insert_edge(
            e2,
            TransportEdge {
                head: h2,
                tail: t2,
                profile_id: "road_local".into(),
                control_points: vec![shared.to_array(), Vec3::new(10.0, 0.0, 20.0).to_array()],
                corridor: CorridorClass::Rail,
                allowed_agents: vec!["train".into()],
            },
        );

        rebuild_junction_metadata(&mut graph);
        let merged_id = graph.edges.get(&e1).map(|e| e.head).expect("edge");
        let node = graph.nodes.get(&merged_id).expect("merged junction node");
        match node.junction_kind {
            JunctionKind::Junction { degree } => assert!(degree >= 3, "degree={degree}"),
            JunctionKind::PassThrough => {}
            other => panic!("expected Junction or PassThrough at shared point, got {other:?}"),
        }
        assert_eq!(graph.edges.get(&e2).map(|e| e.head), Some(merged_id));
    }
}
