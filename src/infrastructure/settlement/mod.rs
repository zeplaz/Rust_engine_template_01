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

/// Attach settlement to nearest transport node key within `radius` tiles (v1 stub).
#[must_use]
pub fn attach_settlement_to_nearest_transport_node(
    directory: &TransportEdgeDirectory,
    origin: Vec3,
    radius: f32,
) -> Vec<String> {
    let mut best: Vec<(f32, String)> = Vec::new();
    for meta in directory.by_edge.values() {
        for key in [&meta.head_key, &meta.tail_key] {
            if key.is_empty() {
                continue;
            }
            let dist = origin.length(); // v1: coarse stub until node positions indexed
            if dist <= radius.max(1.0) {
                best.push((dist, key.clone()));
            }
        }
    }
    best.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
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

    #[test]
    fn attach_settlement_returns_transport_keys() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(0),
            TransportEdgeMeta {
                head_key: "t0_0".into(),
                tail_key: "t1_0".into(),
                ..Default::default()
            },
        );
        let keys = attach_settlement_to_nearest_transport_node(&dir, Vec3::ZERO, 64.0);
        assert!(!keys.is_empty());
    }
}
