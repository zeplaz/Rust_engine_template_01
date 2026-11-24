//! **INFRA-E2-003** — construction execute → transport graph edge + construction record.

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;
use crate::systems::transport::{
    TransportConstructionRecord, TransportEdgeDirectory, TransportEdgeId,
};

/// View binding: sim path markers reference authoritative transport edge ids.
#[derive(Component, Debug, Clone, Copy)]
pub struct SimCorridorEdgeBinding {
    pub edge_id: TransportEdgeId,
}

#[must_use]
pub fn transport_edge_from_binding(binding: &SimCorridorEdgeBinding) -> TransportEdgeId {
    binding.edge_id
}

#[must_use]
pub fn transport_tile_node_key(tile: BuildSiteTile) -> String {
    format!("t{}_{}", tile.x, tile.z)
}

/// Resolve the edge record connecting two adjacent tiles after bake/hydrate.
#[must_use]
pub fn find_edge_id_for_segment(
    directory: &TransportEdgeDirectory,
    head: BuildSiteTile,
    tail: BuildSiteTile,
) -> Option<TransportEdgeId> {
    let hk = transport_tile_node_key(head);
    let tk = transport_tile_node_key(tail);
    directory
        .by_edge
        .iter()
        .find(|(_, meta)| {
            (meta.head_key == hk && meta.tail_key == tk)
                || (meta.head_key == tk && meta.tail_key == hk)
        })
        .map(|(id, _)| *id)
}

/// R8 construction slice row for a newly planned corridor edge.
#[must_use]
pub fn planned_construction_record(edge_id: TransportEdgeId) -> TransportConstructionRecord {
    TransportConstructionRecord {
        edge_id: edge_id.0,
        phase: "Planned".into(),
        progress: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::{
        bake_snapshot_from_ordered_tile_markers, hydrate_transport_from_snapshot,
        TransportEdgeDirectory, TransportEdgeRecord, TransportFieldStore, TransportTopology,
    };

    #[test]
    fn infra_e2_003_planned_construction_record_shape() {
        let rec = planned_construction_record(TransportEdgeId(7));
        assert_eq!(rec.edge_id, 7);
        assert_eq!(rec.phase, "Planned");
        assert_eq!(rec.progress, 0.0);
    }

    #[test]
    fn infra_e2_003_find_edge_after_bake() {
        let snap =
            bake_snapshot_from_ordered_tile_markers(&[(1, 1), (3, 1)], |_, _| 0.5, 20.0, 0.25);
        assert_eq!(snap.edges.len(), 1);
        let edge: &TransportEdgeRecord = &snap.edges[0];
        assert_eq!(edge.head, "t1_1");
        assert_eq!(edge.tail, "t3_1");

        let mut topo = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut topo, &mut field, &mut dir, &snap).unwrap();

        let eid = find_edge_id_for_segment(
            &dir,
            BuildSiteTile { x: 1, z: 1 },
            BuildSiteTile { x: 3, z: 1 },
        )
        .expect("segment edge");
        assert_eq!(eid, TransportEdgeId(0));
    }

    #[test]
    fn infra_e2_004_rail_edges_carry_train_agents_from_the_network_bake() {
        use crate::construction::corridor_network::{CorridorKind, CorridorNetwork};
        let mut net = CorridorNetwork::default();
        net.add_segment(
            CorridorKind::Rail { profile: "default_rail".into() },
            BuildSiteTile { x: 1, z: 1 },
            BuildSiteTile { x: 6, z: 1 },
        )
        .expect("rail line");
        let snap = net.bake_snapshot(10.25);
        let mut topo = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut topo, &mut field, &mut dir, &snap).unwrap();
        assert!(!dir.by_edge.is_empty());
        for meta in dir.by_edge.values() {
            assert!(meta.allowed_agents.iter().any(|a| a == "train"), "{:?}", meta.allowed_agents);
            assert_eq!(meta.corridor_class, crate::systems::transport::CorridorClass::Rail);
        }
    }
}
