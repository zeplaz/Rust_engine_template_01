//! Settlement node attachment to transport graph (INFRA-E5-001).

use bevy::prelude::*;

use crate::strategic::TownId;
use crate::systems::transport::TransportEdgeDirectory;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SettlementId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettlementKind {
    Town,
    Port,
    Depot,
}

#[derive(Clone, Debug)]
pub struct SettlementNode {
    pub id: SettlementId,
    pub kind: SettlementKind,
    pub town_id: Option<TownId>,
    pub position: Vec3,
    pub attached_transport_nodes: Vec<String>,
}

/// Attach settlement to the nearest transport node keys within `radius` (world units, same frame
/// as the edge `control_points`). Node positions are the first / last control point of each edge
/// (head / tail). Returns up to three distinct keys, nearest first.
/// **INFRA-E5-001** (board: `plan_infrastructure_world_layers_exec_001_v1.md`).
#[must_use]
pub fn attach_settlement_to_nearest_transport_node(
    directory: &TransportEdgeDirectory,
    origin: Vec3,
    radius: f32,
) -> Vec<String> {
    let radius = radius.max(1.0);
    let mut best: Vec<(f32, String)> = Vec::new();
    for meta in directory.by_edge.values() {
        let ends = [
            (&meta.head_key, meta.control_points.first()),
            (&meta.tail_key, meta.control_points.last()),
        ];
        for (key, pos) in ends {
            if key.is_empty() {
                continue;
            }
            let Some(p) = pos else {
                continue;
            };
            let dist = origin.distance(Vec3::from_array(*p));
            if dist <= radius && !best.iter().any(|(_, k)| k == key) {
                best.push((dist, key.clone()));
            }
        }
    }
    best.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.cmp(&b.1))
    });
    best.into_iter().take(3).map(|(_, k)| k).collect()
}

#[must_use]
pub fn settlement_node_for_town(
    id: SettlementId,
    town_id: TownId,
    position: Vec3,
    attached: Vec<String>,
) -> SettlementNode {
    SettlementNode {
        id,
        kind: SettlementKind::Town,
        town_id: Some(town_id),
        position,
        attached_transport_nodes: attached,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::{TransportEdgeId, TransportEdgeMeta};

    fn edge(head: &str, tail: &str, a: [f32; 3], b: [f32; 3]) -> TransportEdgeMeta {
        TransportEdgeMeta {
            head_key: head.into(),
            tail_key: tail.into(),
            control_points: vec![a, b],
            ..Default::default()
        }
    }

    #[test]
    fn attach_settlement_returns_transport_keys() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge
            .insert(TransportEdgeId(0), edge("t0_0", "t1_0", [0.0; 3], [1.0, 0.0, 0.0]));
        let keys = attach_settlement_to_nearest_transport_node(&dir, Vec3::ZERO, 64.0);
        assert_eq!(keys.first().map(String::as_str), Some("t0_0"));
    }

    #[test]
    fn attach_settlement_orders_by_real_distance_and_respects_radius() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(0),
            edge("near", "mid", [2.0, 0.0, 0.0], [5.0, 0.0, 0.0]),
        );
        dir.by_edge.insert(
            TransportEdgeId(1),
            edge("mid", "far", [5.0, 0.0, 0.0], [500.0, 0.0, 0.0]),
        );
        let keys = attach_settlement_to_nearest_transport_node(&dir, Vec3::ZERO, 10.0);
        assert_eq!(keys, vec!["near".to_string(), "mid".to_string()]);
        let none = attach_settlement_to_nearest_transport_node(&dir, Vec3::new(0.0, 0.0, 300.0), 10.0);
        assert!(none.is_empty());
    }
}
