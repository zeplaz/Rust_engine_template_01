//! Transport R8 overlay slot for Wave S manifest composition.

use std::path::Path;

use crate::io::save::manifest::OverlaySnapshotRef;
use crate::systems::transport::{
    migrate_transport_snapshot_to_v2, transport_network_snapshot_from_world,
    TransportEdgeDirectory, TransportNetworkSnapshot, TransportTopology,
    TRANSPORT_NETWORK_SCHEMA_V2,
};

pub const TRANSPORT_OVERLAY_NAME: &str = "transport_r8";

#[must_use]
pub fn transport_overlay_ref(artifact_path: impl Into<String>) -> OverlaySnapshotRef {
    OverlaySnapshotRef {
        overlay_name: TRANSPORT_OVERLAY_NAME.into(),
        artifact_path: artifact_path.into(),
    }
}

/// **INFRA-E3-002** — serialize authoritative transport graph to RON artifact.
pub fn write_transport_snapshot_ron(
    path: impl AsRef<Path>,
    topology: &TransportTopology,
    directory: &TransportEdgeDirectory,
) -> std::io::Result<()> {
    let Some(mut snap) = transport_network_snapshot_from_world(topology, directory) else {
        return Ok(());
    };
    migrate_transport_snapshot_to_v2(&mut snap);
    assert_eq!(snap.schema_version, TRANSPORT_NETWORK_SCHEMA_V2);
    let body = ron::ser::to_string_pretty(&snap, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)
}

/// Load transport overlay artifact (v1 migrates to v2 on read).
pub fn read_transport_snapshot_ron(path: impl AsRef<Path>) -> std::io::Result<TransportNetworkSnapshot> {
    let body = std::fs::read_to_string(path)?;
    let mut snap: TransportNetworkSnapshot = ron::from_str(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    migrate_transport_snapshot_to_v2(&mut snap);
    Ok(snap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::{
        hydrate_transport_from_snapshot, TransportEdgeId, TransportEdgeMeta, TransportFieldStore,
    };

    #[test]
    fn transport_overlay_ref_uses_canonical_name() {
        let overlay = transport_overlay_ref("overlays/transport.ron");
        assert_eq!(overlay.overlay_name, TRANSPORT_OVERLAY_NAME);
    }

    #[test]
    fn transport_overlay_ron_roundtrip() {
        let mut topo = TransportTopology::default();
        topo.neighbors.insert(TransportEdgeId(0), vec![]);
        let mut dir = crate::systems::transport::TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(0),
            TransportEdgeMeta {
                head_key: "a".into(),
                tail_key: "b".into(),
                control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
                profile: "default_road".into(),
                ..Default::default()
            },
        );
        let path = std::env::temp_dir().join("transport_overlay_test.ron");
        write_transport_snapshot_ron(&path, &topo, &dir).unwrap();
        let loaded = read_transport_snapshot_ron(&path).unwrap();
        assert_eq!(loaded.schema_version, TRANSPORT_NETWORK_SCHEMA_V2);
        let _ = std::fs::remove_file(path);
    }
}
