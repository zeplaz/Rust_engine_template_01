//! **R8** network snapshot DTO + deterministic hydrate into runtime resources (**W2**).
//! See `transport_code_implementation_plan_v1.md` waves **W1–W3**.

use super::types::{
    EdgeFieldState, NavExportEdge, TransportCostCache, TransportEdgeDirectory, TransportEdgeId,
    TransportEdgeMeta, TransportFieldStore, TransportNavExport, TransportTopology,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const TRANSPORT_NETWORK_SCHEMA_V1: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TransportNetworkSnapshot {
    #[serde(default = "default_schema")]
    pub schema_version: u32,
    #[serde(default)]
    pub nodes: Vec<TransportNodeRecord>,
    #[serde(default)]
    pub edges: Vec<TransportEdgeRecord>,
}

fn default_schema() -> u32 {
    TRANSPORT_NETWORK_SCHEMA_V1
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TransportNodeRecord {
    pub key: String,
    pub position: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TransportEdgeRecord {
    pub id: u64,
    pub head: String,
    pub tail: String,
    #[serde(default)]
    pub successors: Vec<u64>,
    #[serde(default)]
    pub control_points: Vec<[f32; 3]>,
    #[serde(default)]
    pub profile: String,
    #[serde(default)]
    pub allowed_agents: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HydrateError {
    WrongSchema(u32),
    UnknownNode(String),
    UnknownSuccessor { from: u64, to: u64 },
    DuplicateEdgeId(u64),
}

/// Clears and repopulates topology, field store, and edge directory from `snapshot`. Does **not** touch cost cache or nav export (those refresh on the next transport tick).
pub fn hydrate_transport_from_snapshot(
    topology: &mut TransportTopology,
    field_store: &mut TransportFieldStore,
    edge_directory: &mut TransportEdgeDirectory,
    snapshot: &TransportNetworkSnapshot,
) -> Result<(), HydrateError> {
    if snapshot.schema_version != TRANSPORT_NETWORK_SCHEMA_V1 {
        return Err(HydrateError::WrongSchema(snapshot.schema_version));
    }

    let node_keys: HashSet<&str> = snapshot.nodes.iter().map(|n| n.key.as_str()).collect();
    let mut edge_ids: HashSet<u64> = HashSet::new();

    topology.neighbors.clear();
    field_store.by_edge.clear();
    edge_directory.by_edge.clear();

    for e in &snapshot.edges {
        if !edge_ids.insert(e.id) {
            return Err(HydrateError::DuplicateEdgeId(e.id));
        }
        if !node_keys.contains(e.head.as_str()) {
            return Err(HydrateError::UnknownNode(e.head.clone()));
        }
        if !node_keys.contains(e.tail.as_str()) {
            return Err(HydrateError::UnknownNode(e.tail.clone()));
        }
    }

    for e in &snapshot.edges {
        for &suc in &e.successors {
            if !snapshot.edges.iter().any(|x| x.id == suc) {
                return Err(HydrateError::UnknownSuccessor { from: e.id, to: suc });
            }
        }
    }

    for e in &snapshot.edges {
        let tid = TransportEdgeId(e.id);
        let succ: Vec<TransportEdgeId> = e.successors.iter().copied().map(TransportEdgeId).collect();
        topology.neighbors.insert(tid, succ);

        let base = polyline_length(&e.control_points).max(0.1);
        field_store.by_edge.insert(tid, EdgeFieldState {
            travel_time_base: base,
            ..Default::default()
        });

        edge_directory.by_edge.insert(
            tid,
            TransportEdgeMeta {
                profile: e.profile.clone(),
                allowed_agents: e.allowed_agents.clone(),
            },
        );
    }

    Ok(())
}

fn polyline_length(points: &[[f32; 3]]) -> f32 {
    if points.len() < 2 {
        return 1.;
    }
    let mut sum = 0_f32;
    for w in points.windows(2) {
        let a = w[0];
        let b = w[1];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let dz = b[2] - a[2];
        sum += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    sum.max(0.01)
}

/// Refresh **R7** export after [`TransportCostCache`] is current.
pub fn refresh_transport_nav_export(
    topology: &TransportTopology,
    cache: &TransportCostCache,
    edge_directory: &TransportEdgeDirectory,
    export: &mut TransportNavExport,
) {
    export.edges.clear();
    let mut ids: Vec<TransportEdgeId> = topology.neighbors.keys().copied().collect();
    ids.sort_by_key(|k| k.0);

    for id in ids {
        let cost = cache.by_edge.get(&id).copied().unwrap_or(10_000.0);
        let succ = topology.neighbors.get(&id).cloned().unwrap_or_default();
        let (profile, allowed_agents) = edge_directory
            .by_edge
            .get(&id)
            .map(|m| (m.profile.clone(), m.allowed_agents.clone()))
            .unwrap_or_else(|| (String::new(), Vec::new()));
        export.edges.push(NavExportEdge {
            id,
            cost,
            successors: succ,
            allowed_agents,
            profile,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                    id: 0,
                    head: "a".into(),
                    tail: "b".into(),
                    successors: vec![1],
                    control_points: vec![[0., 0., 0.], [1., 0., 0.]],
                    profile: "default_road".into(),
                    allowed_agents: vec!["road_vehicle".into()],
                },
                TransportEdgeRecord {
                    id: 1,
                    head: "b".into(),
                    tail: "c".into(),
                    successors: vec![],
                    control_points: vec![[1., 0., 0.], [2., 0., 0.]],
                    profile: "default_road".into(),
                    allowed_agents: vec!["road_vehicle".into()],
                },
            ],
        }
    }

    #[test]
    fn hydrate_round_trip_json() {
        let s0 = sample_snapshot();
        let json = serde_json::to_string(&s0).unwrap();
        let s1: TransportNetworkSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(s0, s1);

        let mut top = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &s1).unwrap();

        assert_eq!(
            top.neighbors.get(&TransportEdgeId(0)).cloned().unwrap(),
            vec![TransportEdgeId(1)]
        );
        assert!(top.neighbors.get(&TransportEdgeId(1)).unwrap().is_empty());
        assert!(field.by_edge.contains_key(&TransportEdgeId(0)));
        let w = field.by_edge[&TransportEdgeId(0)].travel_time_base;
        assert!((w - 1.0).abs() < 0.01);
    }

    #[test]
    fn nav_export_follows_cache() {
        let s = sample_snapshot();
        let mut top = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &s).unwrap();

        let weights = crate::systems::transport::TransportCostWeights::default();
        let mut cache = TransportCostCache::default();
        for (id, state) in field.by_edge.iter() {
            cache.by_edge.insert(
                *id,
                crate::systems::transport::edge_traversal_cost(state, &weights, state.travel_time_base),
            );
        }
        let mut exp = TransportNavExport::default();
        refresh_transport_nav_export(&top, &cache, &dir, &mut exp);
        assert_eq!(exp.edges.len(), 2);
        assert_eq!(exp.edges[0].successors, vec![TransportEdgeId(1)]);
        assert!(!exp.edges[0].allowed_agents.is_empty());
    }
}
