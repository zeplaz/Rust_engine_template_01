//! Utility network (power / water) overlay slot for Wave S manifest composition.
//!
//! Player-built power lines live in [`UtilityNetworkSnapshotResource`]; without this overlay they were
//! lost on save/load. Hydrate replaces the snapshot and rebuilds [`UtilityGraph`] from it.

use std::path::Path;

use bevy::prelude::*;

use crate::infrastructure::utility::graph::UtilityNetworkSnapshotResource;
use crate::infrastructure::utility::{hydrate_utility_graph_from_snapshot, UtilityGraph};
use crate::infrastructure::{UtilityNetworkSnapshot, UTILITY_NETWORK_SCHEMA_V1};
use crate::io::save::manifest::{OverlaySnapshotRef, SaveWorldManifest};

pub const UTILITY_OVERLAY_NAME: &str = "utility_network";
pub const UTILITY_OVERLAY_REL_PATH: &str = "overlays/utility_network.ron";

#[must_use]
pub fn utility_overlay_ref(artifact_path: impl Into<String>) -> OverlaySnapshotRef {
    OverlaySnapshotRef {
        overlay_name: UTILITY_OVERLAY_NAME.into(),
        artifact_path: artifact_path.into(),
    }
}

#[must_use]
pub fn default_utility_overlay_ref() -> OverlaySnapshotRef {
    utility_overlay_ref(UTILITY_OVERLAY_REL_PATH)
}

#[must_use]
pub fn build_utility_overlay_refs() -> Vec<OverlaySnapshotRef> {
    vec![default_utility_overlay_ref()]
}

pub fn write_utility_network_ron(
    path: impl AsRef<Path>,
    snapshot: &UtilityNetworkSnapshot,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = ron::ser::to_string_pretty(snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)
}

pub fn read_utility_network_ron(path: impl AsRef<Path>) -> std::io::Result<UtilityNetworkSnapshot> {
    let body = std::fs::read_to_string(path)?;
    let snap: UtilityNetworkSnapshot = ron::from_str(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    if snap.schema_version != UTILITY_NETWORK_SCHEMA_V1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("utility network schema {} unsupported", snap.schema_version),
        ));
    }
    Ok(snap)
}

/// Write the utility overlay artifact into a save bundle.
pub fn write_utility_overlay_to_bundle(
    bundle_dir: &Path,
    snapshot: &UtilityNetworkSnapshot,
) -> std::io::Result<OverlaySnapshotRef> {
    write_utility_network_ron(bundle_dir.join(UTILITY_OVERLAY_REL_PATH), snapshot)?;
    Ok(default_utility_overlay_ref())
}

/// Read the utility overlay named by `manifest`; `None` when absent (older saves) or unreadable.
#[must_use]
pub fn read_utility_overlay_from_manifest(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
) -> Option<UtilityNetworkSnapshot> {
    let entry = manifest
        .overlays
        .iter()
        .find(|o| o.overlay_name == UTILITY_OVERLAY_NAME)?;
    read_utility_network_ron(bundle_dir.join(&entry.artifact_path)).ok()
}

/// Write the utility overlay when a save flush is armed (before the manifest flush).
pub fn write_utility_overlay_on_save_flush(
    flush: Res<crate::io::save::SaveFlushRequested>,
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    snap: Option<Res<UtilityNetworkSnapshotResource>>,
) {
    if !flush.0 {
        return;
    }
    let Some(snap) = snap else {
        return;
    };
    if let Err(err) = write_utility_overlay_to_bundle(&settings.bundle_dir, &snap.0) {
        bevy::log::warn!(
            "utility overlay write failed at {:?}: {err}",
            settings.bundle_dir.join(UTILITY_OVERLAY_REL_PATH)
        );
    }
}

/// Tracks utility overlay hydrate attempts per bundle dir.
#[derive(Resource, Default, Debug)]
pub struct UtilityOverlayHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub hydrate_attempted: bool,
    pub hydrated_from_manifest: bool,
}

/// Restore the utility snapshot + rebuild [`UtilityGraph`] when the manifest references it.
pub fn try_hydrate_utility_overlay_on_bundle_dir(
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    mut state: ResMut<UtilityOverlayHydrateState>,
    snap_res: Option<ResMut<UtilityNetworkSnapshotResource>>,
    graph: Option<ResMut<UtilityGraph>>,
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
    let (Some(mut snap_res), Some(mut graph)) = (snap_res, graph) else {
        return;
    };
    state.hydrate_attempted = true;
    let Ok(manifest) = crate::io::save::read_manifest_from_bundle(&settings.bundle_dir) else {
        return;
    };
    let Some(snap) = read_utility_overlay_from_manifest(&settings.bundle_dir, &manifest) else {
        return;
    };
    *graph = hydrate_utility_graph_from_snapshot(&snap);
    snap_res.0 = snap;
    state.hydrated_from_manifest = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::utility::fixture_utility_network_snapshot;
    use crate::infrastructure::{PowerLine, UtilityLink, VoltageClass};
    use crate::io::save::manifest::build_save_world_manifest;
    use crate::io::save::pipeline::write_manifest_atomic;

    fn temp_bundle(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "utility_overlay_{tag}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn utility_overlay_ref_uses_canonical_name() {
        let r = default_utility_overlay_ref();
        assert_eq!(r.overlay_name, UTILITY_OVERLAY_NAME);
        assert_eq!(r.artifact_path, UTILITY_OVERLAY_REL_PATH);
    }

    #[test]
    fn utility_overlay_manifest_roundtrip_restores_player_line() {
        let mut snap = fixture_utility_network_snapshot();
        snap.nodes.push("pole_7".into());
        snap.edges.push(UtilityLink {
            id: 77,
            head: "sub_a".into(),
            tail: "pole_7".into(),
            utility_type: "power".into(),
        });
        snap.power_lines.push(PowerLine {
            link_id: 77,
            voltage: VoltageClass::High,
        });
        let dir = temp_bundle("rt");
        write_utility_overlay_to_bundle(&dir, &snap).expect("write overlay");
        let manifest = build_save_world_manifest(7, Vec::new(), Vec::new(), build_utility_overlay_refs());
        write_manifest_atomic(&dir, &manifest).expect("manifest");
        let loaded_manifest = crate::io::save::read_manifest_from_bundle(&dir).expect("read manifest");
        let loaded = read_utility_overlay_from_manifest(&dir, &loaded_manifest).expect("overlay");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(loaded, snap);
        let graph = hydrate_utility_graph_from_snapshot(&loaded);
        assert!(graph.power_edges.iter().any(|e| e.link_id == 77));
    }

    #[test]
    fn utility_overlay_absent_from_manifest_is_none() {
        let dir = temp_bundle("absent");
        let manifest = build_save_world_manifest(7, Vec::new(), Vec::new(), Vec::new());
        assert!(read_utility_overlay_from_manifest(&dir, &manifest).is_none());
    }
}
