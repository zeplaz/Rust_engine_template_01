//! **INFRA-E3-WIT-001** — `debug_runs/transport_network_live.json` lib refresh.

use serde_json::{json, Value};

use crate::io::save::{read_transport_snapshot_ron, write_transport_snapshot_ron};
use crate::systems::transport::{
    migrate_transport_snapshot_to_v2, transport_network_snapshot_from_world,
    TransportEdgeDirectory, TransportEdgeId, TransportEdgeMeta, TransportTopology,
    TRANSPORT_NETWORK_SCHEMA_V2,
};

pub const TRANSPORT_NETWORK_LIVE_JSON: &str = "debug_runs/transport_network_live.json";

#[must_use]
pub fn build_transport_network_live_payload(
    topology: &TransportTopology,
    directory: &TransportEdgeDirectory,
) -> Value {
    let edge_count = topology.neighbors.len();
    let node_count = directory
        .by_edge
        .values()
        .flat_map(|m| [&m.head_key, &m.tail_key])
        .filter(|k| !k.is_empty())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let mut profile_histogram: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();
    for meta in directory.by_edge.values() {
        *profile_histogram.entry(meta.profile.clone()).or_insert(0) += 1;
    }
    let schema_version = transport_network_snapshot_from_world(topology, directory)
        .map(|s| s.schema_version)
        .unwrap_or(TRANSPORT_NETWORK_SCHEMA_V2);
    let hybrid_save_ok = transport_overlay_roundtrip_witness_green();
    let infra_e1_001 = crate::infrastructure::transport::infra_e1_001_transport_graph_sync_witness_green();
    let infra_e1_002 = crate::infrastructure::transport::infra_e1_002_spline_subdivide_witness_green();
    let graph_roundtrip = transport_network_graph_roundtrip_witness_green();
    json!({
        "gate": "INFRA-E3-WIT-001",
        "green": edge_count > 0 && hybrid_save_ok && infra_e1_001 && infra_e1_002 && graph_roundtrip,
        "schema_version": schema_version,
        "node_count": node_count,
        "edge_count": edge_count,
        "profile_histogram": profile_histogram,
        "hybrid_save_ok": hybrid_save_ok,
        "transport_overlay_name": "transport_r8",
        "infra_e1_001_graph_sync": infra_e1_001,
        "infra_e1_002_spline_subdivide": infra_e1_002,
        "transport_network_roundtrip_001": graph_roundtrip,
    })
}

#[must_use]
fn transport_overlay_roundtrip_witness_green() -> bool {
    let mut topo = TransportTopology::default();
    topo.neighbors.insert(TransportEdgeId(0), vec![]);
    let mut dir = TransportEdgeDirectory::default();
    dir.by_edge.insert(
        TransportEdgeId(0),
        TransportEdgeMeta {
            head_key: "n0".into(),
            tail_key: "n1".into(),
            control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            profile: "default_road".into(),
            ..Default::default()
        },
    );
    let path = std::env::temp_dir().join("transport_network_live_proof.ron");
    if write_transport_snapshot_ron(&path, &topo, &dir).is_err() {
        return false;
    }
    let loaded = read_transport_snapshot_ron(&path).ok();
    let _ = std::fs::remove_file(&path);
    loaded
        .map(|mut s| {
            migrate_transport_snapshot_to_v2(&mut s);
            s.schema_version == TRANSPORT_NETWORK_SCHEMA_V2 && !s.edges.is_empty()
        })
        .unwrap_or(false)
}

#[must_use]
pub fn transport_network_graph_roundtrip_witness_green() -> bool {
    use crate::infrastructure::transport::{
        hydrate_transport_graph_from_snapshot, transport_network_snapshot_from_graph,
    };
    use crate::systems::transport::{
        hydrate_transport_from_snapshot, TransportEdgeDirectory, TransportEdgeRecord,
        TransportFieldStore, TransportNetworkSnapshot, TransportNodeRecord,
        TRANSPORT_NETWORK_SCHEMA_V1,
    };

    let snap = TransportNetworkSnapshot {
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
        ],
        edges: vec![TransportEdgeRecord {
            id: 1,
            head: "a".into(),
            tail: "b".into(),
            successors: vec![],
            control_points: vec![[0., 0., 0.], [1., 0., 0.]],
            profile: "default_road".into(),
            allowed_agents: vec!["truck".into()],
            ..Default::default()
        }],
        construction: vec![],
    };
    let graph = hydrate_transport_graph_from_snapshot(&snap);
    let back = transport_network_snapshot_from_graph(&graph);
    let mut topo = TransportTopology::default();
    let mut field = TransportFieldStore::default();
    let mut dir = TransportEdgeDirectory::default();
    hydrate_transport_from_snapshot(&mut topo, &mut field, &mut dir, &back).is_ok()
        && !graph.edges.is_empty()
        && back.edges.len() == snap.edges.len()
}

/// Lib refresh — writes envelope JSON for transport network witness.
#[cfg(test)]
#[must_use]
pub fn refresh_transport_network_live_witness() -> bool {
    let mut topo = TransportTopology::default();
    topo.neighbors.insert(TransportEdgeId(1), vec![TransportEdgeId(2)]);
    topo.neighbors.insert(TransportEdgeId(2), vec![]);
    let mut dir = TransportEdgeDirectory::default();
    dir.by_edge.insert(
        TransportEdgeId(1),
        TransportEdgeMeta {
            profile: "default_road".into(),
            head_key: "a".into(),
            tail_key: "b".into(),
            control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            ..Default::default()
        },
    );
    dir.by_edge.insert(
        TransportEdgeId(2),
        TransportEdgeMeta {
            profile: "default_rail".into(),
            head_key: "b".into(),
            tail_key: "c".into(),
            control_points: vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            allowed_agents: vec!["train".into()],
            ..Default::default()
        },
    );
    let body = build_transport_network_live_payload(&topo, &dir);
    let green = body.get("green").and_then(|v| v.as_bool()) == Some(true);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "TRANSPORT_NETWORK",
        "refresh_transport_network_live_witness",
        TRANSPORT_NETWORK_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(TRANSPORT_NETWORK_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_network_live_witness_refresh_green() {
        assert!(refresh_transport_network_live_witness());
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(TRANSPORT_NETWORK_LIVE_JSON);
        let json: Value =
            serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("parse");
        assert_eq!(json.get("edge_count").and_then(|v| v.as_u64()), Some(2));
        assert_eq!(json.get("green").and_then(|v| v.as_bool()), Some(true));
    }
}
