//! **INFRA-E2-003** — construction execute → transport graph edge + construction record.

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;
use crate::systems::transport::{
    TransportConstructionRecord, TransportEdgeDirectory, TransportEdgeId, CorridorClass,
};

/// Active rail profile id for corridor execute (INFRA-E2-004).
#[derive(Resource, Debug, Clone)]
pub struct RailProfileSelection {
    pub profile_id: String,
}

impl Default for RailProfileSelection {
    fn default() -> Self {
        Self {
            profile_id: "default_rail".into(),
        }
    }
}

/// View binding: sim path markers reference authoritative transport edge ids.
#[derive(Component, Debug, Clone, Copy)]
pub struct SimCorridorEdgeBinding {
    pub edge_id: TransportEdgeId,
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

/// Patch rail corridor metadata after tile-marker bake (profile picker + train agents).
pub fn apply_rail_profile_to_edge(
    directory: &mut TransportEdgeDirectory,
    edge_id: TransportEdgeId,
    profile_id: &str,
    allowed_agents: &[String],
) {
    if let Some(meta) = directory.by_edge.get_mut(&edge_id) {
        meta.profile = profile_id.into();
        meta.corridor_class = CorridorClass::Rail;
        meta.allowed_agents = if allowed_agents.is_empty() {
            vec!["train".into()]
        } else {
            allowed_agents.to_vec()
        };
    }
}

/// Resolve rail profile from registry selection (falls back to default train agents).
pub fn apply_selected_rail_profile_to_edge(
    directory: &mut TransportEdgeDirectory,
    edge_id: TransportEdgeId,
    selection: &RailProfileSelection,
    registry: Option<&crate::infrastructure::ProfileRegistry>,
) {
    let (profile_id, agents) = registry
        .and_then(|r| r.resolve(&selection.profile_id))
        .map(|kind| {
            let agents = match &kind {
                crate::infrastructure::CorridorProfileKind::Rail(p) => p.allowed_agents.clone(),
                _ => vec!["train".into()],
            };
            (selection.profile_id.clone(), agents)
        })
        .unwrap_or_else(|| (selection.profile_id.clone(), vec!["train".into()]));
    apply_rail_profile_to_edge(directory, edge_id, &profile_id, &agents);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::{
        bake_snapshot_from_ordered_tile_markers, hydrate_transport_from_snapshot,
        TransportEdgeRecord, TransportEdgeDirectory, TransportFieldStore, TransportTopology,
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
        let snap = bake_snapshot_from_ordered_tile_markers(
            &[(1, 1), (3, 1)],
            |_, _| 0.5,
            20.0,
            0.25,
        );
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
    fn infra_e2_004_rail_profile_train_agents() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(0),
            crate::systems::transport::TransportEdgeMeta::default(),
        );
        let selection = RailProfileSelection {
            profile_id: "default_rail".into(),
        };
        apply_selected_rail_profile_to_edge(&mut dir, TransportEdgeId(0), &selection, None);
        let meta = &dir.by_edge[&TransportEdgeId(0)];
        assert_eq!(meta.profile, "default_rail");
        assert_eq!(meta.corridor_class, CorridorClass::Rail);
        assert_eq!(meta.allowed_agents, vec!["train".to_string()]);
    }
}
