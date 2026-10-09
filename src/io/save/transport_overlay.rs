//! Transport R8 overlay slot for Wave S manifest composition.

use std::path::Path;

use bevy::prelude::*;

use crate::io::save::manifest::{OverlaySnapshotRef, SaveWorldManifest};
use crate::strategic::{
    apply_corridor_book_from_transport_snapshot, transport_construction_records_from_book,
    CorridorConstructionBook,
};
use crate::systems::transport::{
    hydrate_transport_from_snapshot, migrate_transport_snapshot_to_v2,
    transport_network_snapshot_from_world, transport_network_snapshot_from_world_with_construction,
    TransportEdgeDirectory, TransportFieldStore, TransportLastHydratedSnapshot,
    TransportNetworkSnapshot, TransportTopology, TRANSPORT_NETWORK_SCHEMA_V2,
};

pub const TRANSPORT_OVERLAY_NAME: &str = "transport_r8";
/// Bundle-relative artifact for the R8 transport overlay (roads, rails, corridor construction rows).
pub const TRANSPORT_OVERLAY_REL_PATH: &str = "overlays/transport_r8.ron";

#[must_use]
pub fn transport_overlay_ref(artifact_path: impl Into<String>) -> OverlaySnapshotRef {
    OverlaySnapshotRef {
        overlay_name: TRANSPORT_OVERLAY_NAME.into(),
        artifact_path: artifact_path.into(),
    }
}

#[must_use]
pub fn default_transport_overlay_ref() -> OverlaySnapshotRef {
    transport_overlay_ref(TRANSPORT_OVERLAY_REL_PATH)
}

#[must_use]
pub fn build_transport_overlay_refs() -> Vec<OverlaySnapshotRef> {
    vec![default_transport_overlay_ref()]
}

/// Capture the live network (plus corridor construction rows) as an R8 v2 snapshot.
/// An empty network yields an empty snapshot so loading the save also clears stale roads.
#[must_use]
pub fn capture_transport_overlay_snapshot(
    topology: &TransportTopology,
    directory: &TransportEdgeDirectory,
    book: Option<&CorridorConstructionBook>,
) -> TransportNetworkSnapshot {
    let construction = book
        .map(|b| transport_construction_records_from_book(b, topology))
        .unwrap_or_default();
    let mut snap = transport_network_snapshot_from_world_with_construction(
        topology,
        directory,
        construction,
    )
    .unwrap_or_else(|| TransportNetworkSnapshot {
        schema_version: TRANSPORT_NETWORK_SCHEMA_V2,
        nodes: Vec::new(),
        edges: Vec::new(),
        construction: Vec::new(),
    });
    migrate_transport_snapshot_to_v2(&mut snap);
    snap
}

/// Write the transport overlay artifact into a save bundle.
pub fn write_transport_overlay_to_bundle(
    bundle_dir: &Path,
    snapshot: &TransportNetworkSnapshot,
) -> std::io::Result<OverlaySnapshotRef> {
    let path = bundle_dir.join(TRANSPORT_OVERLAY_REL_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = ron::ser::to_string_pretty(snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)?;
    Ok(default_transport_overlay_ref())
}

/// Read the transport overlay named by `manifest`, if any. `None` when absent or unreadable
/// (older saves predate this overlay — the live network is then left untouched).
#[must_use]
pub fn read_transport_overlay_from_manifest(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
) -> Option<TransportNetworkSnapshot> {
    let entry = manifest
        .overlays
        .iter()
        .find(|o| o.overlay_name == TRANSPORT_OVERLAY_NAME)?;
    read_transport_snapshot_ron(bundle_dir.join(&entry.artifact_path)).ok()
}

/// Write the transport overlay when a save flush is armed. Runs **before**
/// [`crate::io::save::flush_dirty_chunk_save_queue`] so the manifest-referenced artifact exists.
pub fn write_transport_overlay_on_save_flush(
    flush: Res<crate::io::save::SaveFlushRequested>,
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    topology: Option<Res<TransportTopology>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    book: Option<Res<CorridorConstructionBook>>,
) {
    if !flush.0 {
        return;
    }
    let (Some(topology), Some(directory)) = (topology, directory) else {
        return;
    };
    let snap = capture_transport_overlay_snapshot(&topology, &directory, book.as_deref());
    if let Err(err) = write_transport_overlay_to_bundle(&settings.bundle_dir, &snap) {
        bevy::log::warn!(
            "transport overlay write failed at {:?}: {err}",
            settings.bundle_dir.join(TRANSPORT_OVERLAY_REL_PATH)
        );
    }
}

/// Tracks transport overlay hydrate attempts per bundle dir.
#[derive(Resource, Default, Debug)]
pub struct TransportOverlayHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub hydrate_attempted: bool,
    pub hydrated_from_manifest: bool,
}

/// Restore roads/rails (+ corridor construction book) when the manifest references `transport_r8`.
pub fn try_hydrate_transport_overlay_on_bundle_dir(
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    mut state: ResMut<TransportOverlayHydrateState>,
    topology: Option<ResMut<TransportTopology>>,
    fields: Option<ResMut<TransportFieldStore>>,
    directory: Option<ResMut<TransportEdgeDirectory>>,
    last: Option<ResMut<TransportLastHydratedSnapshot>>,
    book: Option<ResMut<CorridorConstructionBook>>,
) {
    let Some(settings) = settings else {
        return;
    };
    let same_dir = state
        .last_bundle_dir
        .as_deref()
        .is_some_and(|p| p == settings.bundle_dir.as_path());
    if !same_dir {
        state.last_bundle_dir = Some(settings.bundle_dir.clone());
        state.hydrate_attempted = false;
        state.hydrated_from_manifest = false;
    }
    if state.hydrate_attempted {
        return;
    }
    let (Some(mut topology), Some(mut fields), Some(mut directory)) = (topology, fields, directory)
    else {
        return;
    };
    state.hydrate_attempted = true;
    let Ok(manifest) = crate::io::save::read_manifest_from_bundle(&settings.bundle_dir) else {
        return;
    };
    let Some(snap) = read_transport_overlay_from_manifest(&settings.bundle_dir, &manifest) else {
        return;
    };
    if let Err(err) =
        hydrate_transport_from_snapshot(&mut topology, &mut fields, &mut directory, &snap)
    {
        bevy::log::warn!("transport overlay hydrate failed: {err:?}");
        return;
    }
    if let Some(mut book) = book {
        apply_corridor_book_from_transport_snapshot(&mut book, &directory, &snap);
    }
    if let Some(mut last) = last {
        last.snapshot = Some(snap);
    }
    state.hydrated_from_manifest = true;
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
    use crate::systems::transport::{TransportEdgeId, TransportEdgeMeta};

    #[test]
    fn empty_network_captures_empty_snapshot() {
        let snap = capture_transport_overlay_snapshot(
            &TransportTopology::default(),
            &TransportEdgeDirectory::default(),
            None,
        );
        assert_eq!(snap.schema_version, TRANSPORT_NETWORK_SCHEMA_V2);
        assert!(snap.edges.is_empty());
    }

    #[test]
    fn transport_overlay_manifest_roundtrip_hydrates_runtime() {
        use crate::io::save::manifest::build_save_world_manifest;
        use crate::io::save::pipeline::write_manifest_atomic;

        let mut topo = TransportTopology::default();
        topo.neighbors.insert(TransportEdgeId(3), vec![]);
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(3),
            TransportEdgeMeta {
                head_key: "t0_0".into(),
                tail_key: "t4_0".into(),
                control_points: vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]],
                profile: "default_road".into(),
                ..Default::default()
            },
        );
        let snap = capture_transport_overlay_snapshot(&topo, &dir, None);
        assert_eq!(snap.edges.len(), 1);

        let bundle = std::env::temp_dir().join(format!(
            "transport_overlay_rt_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        write_transport_overlay_to_bundle(&bundle, &snap).expect("write overlay");
        let manifest =
            build_save_world_manifest(9, Vec::new(), Vec::new(), build_transport_overlay_refs());
        write_manifest_atomic(&bundle, &manifest).expect("manifest");
        let loaded_manifest = crate::io::save::read_manifest_from_bundle(&bundle).expect("read");
        let loaded = read_transport_overlay_from_manifest(&bundle, &loaded_manifest);
        let _ = std::fs::remove_dir_all(&bundle);
        let loaded = loaded.expect("overlay present");

        let mut topo2 = TransportTopology::default();
        let mut field2 = TransportFieldStore::default();
        let mut dir2 = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut topo2, &mut field2, &mut dir2, &loaded)
            .expect("hydrate");
        assert!(topo2.neighbors.contains_key(&TransportEdgeId(3)));
        assert_eq!(
            dir2.by_edge.get(&TransportEdgeId(3)).map(|m| m.profile.as_str()),
            Some("default_road")
        );
    }

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
