//! **G4** — **R8** `TransportNetworkSnapshot` JSON I/O + schema gate (dev/slice before hybrid binary body).
//!
//! Load/save does **not** invent gameplay rules: bytes → DTO → [`super::snapshot::hydrate_transport_from_snapshot`] only.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use super::snapshot::{
    hydrate_transport_from_snapshot, TransportNetworkSnapshot, TRANSPORT_NETWORK_SCHEMA_V1,
};
use super::types::{TransportEdgeDirectory, TransportFieldStore, TransportTopology};
use bevy::prelude::*;

/// Last snapshot successfully hydrated (editor bake or **G4** load). Intended **save** anchor for the transport slice until **M5** owns full world snapshots.
#[derive(Resource, Clone, Debug, Default)]
pub struct TransportLastHydratedSnapshot {
    pub snapshot: Option<TransportNetworkSnapshot>,
}

#[derive(Debug)]
pub enum TransportNetworkPersistenceError {
    Json(serde_json::Error),
    Io(std::io::Error),
    BadSchema { found: u32, expected: u32 },
    Hydrate(super::snapshot::HydrateError),
}

impl From<serde_json::Error> for TransportNetworkPersistenceError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<std::io::Error> for TransportNetworkPersistenceError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<super::snapshot::HydrateError> for TransportNetworkPersistenceError {
    fn from(e: super::snapshot::HydrateError) -> Self {
        Self::Hydrate(e)
    }
}

/// Parse JSON only; validates `schema_version` before hydrate.
pub fn transport_network_snapshot_from_json_str(s: &str) -> Result<TransportNetworkSnapshot, TransportNetworkPersistenceError> {
    let snap: TransportNetworkSnapshot = serde_json::from_str(s)?;
    if snap.schema_version != TRANSPORT_NETWORK_SCHEMA_V1 {
        return Err(TransportNetworkPersistenceError::BadSchema {
            found: snap.schema_version,
            expected: TRANSPORT_NETWORK_SCHEMA_V1,
        });
    }
    Ok(snap)
}

pub fn transport_network_snapshot_from_json_path(path: impl AsRef<Path>) -> Result<TransportNetworkSnapshot, TransportNetworkPersistenceError> {
    let bytes = fs::read_to_string(path.as_ref())?;
    transport_network_snapshot_from_json_str(&bytes)
}

pub fn transport_network_snapshot_to_json_string(snap: &TransportNetworkSnapshot) -> Result<String, TransportNetworkPersistenceError> {
    Ok(serde_json::to_string_pretty(snap)?)
}

pub fn transport_network_snapshot_save_json_path(
    snap: &TransportNetworkSnapshot,
    path: impl AsRef<Path>,
) -> Result<(), TransportNetworkPersistenceError> {
    let s = transport_network_snapshot_to_json_string(snap)?;
    fs::write(path.as_ref(), s)?;
    Ok(())
}

/// Apply a JSON snapshot to ECS transport resources (main-thread **G4** boundary).
pub fn hydrate_transport_from_json_str(
    topology: &mut TransportTopology,
    field_store: &mut TransportFieldStore,
    edge_directory: &mut TransportEdgeDirectory,
    json: &str,
) -> Result<TransportNetworkSnapshot, TransportNetworkPersistenceError> {
    let snap = transport_network_snapshot_from_json_str(json)?;
    hydrate_transport_from_snapshot(topology, field_store, edge_directory, &snap)?;
    Ok(snap)
}

/// Bevy message: load **R8** transport network from disk (relative paths resolved by caller).
#[derive(Clone, Debug, Message)]
pub struct LoadTransportNetworkSnapshotFromDisk {
    pub path: Arc<str>,
}

fn transport_network_persistence_on_load(
    mut messages: MessageReader<LoadTransportNetworkSnapshotFromDisk>,
    mut topology: ResMut<TransportTopology>,
    mut field_store: ResMut<TransportFieldStore>,
    mut edge_directory: ResMut<TransportEdgeDirectory>,
    mut last: ResMut<TransportLastHydratedSnapshot>,
) {
    for msg in messages.read() {
        let path = Path::new(msg.path.as_ref());
        match transport_network_snapshot_from_json_path(path) {
            Ok(snap) => {
                match hydrate_transport_from_snapshot(
                    topology.as_mut(),
                    field_store.as_mut(),
                    edge_directory.as_mut(),
                    &snap,
                ) {
                    Ok(()) => {
                        last.snapshot = Some(snap);
                    }
                    Err(e) => {
                        warn!("Transport G4: hydrate failed for {}: {e:?}", path.display());
                    }
                }
            }
            Err(e) => warn!("Transport G4: load failed for {}: {e:?}", path.display()),
        }
    }
}

/// Registers **G4** load path: `LoadTransportNetworkSnapshotFromDisk` → hydrate + [`TransportLastHydratedSnapshot`].
pub struct TransportNetworkPersistencePlugin;

impl Plugin for TransportNetworkPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LoadTransportNetworkSnapshotFromDisk>().add_systems(
            Update,
            transport_network_persistence_on_load,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::snapshot::transport_network_snapshot_from_world;
    use std::path::PathBuf;

    fn fixture_chain_v1_json() -> &'static str {
        include_str!("../../../assets/test_fixtures/transport/network_chain_v1.json")
    }

    #[test]
    fn g4_fixture_load_hydrate_nonempty_topology() {
        let json = fixture_chain_v1_json();
        let snap = transport_network_snapshot_from_json_str(json).unwrap();
        let mut top = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &snap).unwrap();
        assert_eq!(top.neighbors.len(), 2);
        assert!(dir.by_edge.contains_key(&crate::systems::transport::TransportEdgeId(0)));
    }

    #[test]
    fn g4_round_trip_json_file_from_manifest() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/test_fixtures/transport/network_chain_v1.json");
        let json = fs::read_to_string(&path).unwrap();
        let s0 = transport_network_snapshot_from_json_str(&json).unwrap();
        let mut top = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &s0).unwrap();
        let s1 = transport_network_snapshot_from_world(&top, &dir).unwrap();
        let js0 = serde_json::to_value(&s0).unwrap();
        let js1 = serde_json::to_value(&s1).unwrap();
        assert_eq!(js0, js1);
    }

    /// **W4 / T-LANE-001** slice: one node fans out to two edges (junction seed).
    #[test]
    fn g4_fork_fixture_hydrate_two_edges_from_one_head() {
        let json = include_str!("../../../assets/test_fixtures/transport/network_fork_v1.json");
        let snap = transport_network_snapshot_from_json_str(json).unwrap();
        assert_eq!(snap.edges.len(), 2);
        assert!(snap.edges.iter().all(|e| e.head == "a"));
        let mut top = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &snap).unwrap();
        assert_eq!(top.neighbors.len(), 2);
        assert_eq!(top.neighbors[&crate::systems::transport::TransportEdgeId(0)].len(), 0);
        assert_eq!(top.neighbors[&crate::systems::transport::TransportEdgeId(1)].len(), 0);
    }
}
