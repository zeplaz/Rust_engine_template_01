//! **G4** — **R8** `TransportNetworkSnapshot` I/O + schema gate (dev/slice before hybrid binary body).
//!
//! **Format policy:** **RON** is the canonical on-disk format for editor/dev transport saves (matches Bevy/tooling
//! patterns: `material_rules.ron`, keybindings RON, etc.). **JSON** remains for legacy fixtures and explicit
//! `.json` paths / human-shared snippets. See `prompts/matrix/transport/runbook/ron_and_persistence_next_steps_v1.md`.
//!
//! Load/save does **not** invent gameplay rules: text → DTO → [`super::snapshot::hydrate_transport_from_snapshot`] only.

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
    /// RON parse or serialize (`Display` carries both `SpannedError` / serialization messages).
    Ron(String),
    Io(std::io::Error),
    BadSchema { found: u32, expected: u32 },
    Hydrate(super::snapshot::HydrateError),
}

fn assert_schema_v1(snap: &TransportNetworkSnapshot) -> Result<(), TransportNetworkPersistenceError> {
    if snap.schema_version != TRANSPORT_NETWORK_SCHEMA_V1 {
        return Err(TransportNetworkPersistenceError::BadSchema {
            found: snap.schema_version,
            expected: TRANSPORT_NETWORK_SCHEMA_V1,
        });
    }
    Ok(())
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

/// Parse **RON**; validates `schema_version` before returning.
pub fn transport_network_snapshot_from_ron_str(s: &str) -> Result<TransportNetworkSnapshot, TransportNetworkPersistenceError> {
    let snap: TransportNetworkSnapshot =
        ron::de::from_str(s).map_err(|e| TransportNetworkPersistenceError::Ron(e.to_string()))?;
    assert_schema_v1(&snap)?;
    Ok(snap)
}

pub fn transport_network_snapshot_to_ron_string(snap: &TransportNetworkSnapshot) -> Result<String, TransportNetworkPersistenceError> {
    let cfg = ron::ser::PrettyConfig::new().depth_limit(8).indentor("    ".into());
    ron::ser::to_string_pretty(snap, cfg).map_err(|e| TransportNetworkPersistenceError::Ron(e.to_string()))
}

/// Canonical dev save — **RON** pretty.
pub fn transport_network_snapshot_save_ron_path(
    snap: &TransportNetworkSnapshot,
    path: impl AsRef<Path>,
) -> Result<(), TransportNetworkPersistenceError> {
    let s = transport_network_snapshot_to_ron_string(snap)?;
    fs::write(path.as_ref(), s)?;
    Ok(())
}

/// Parse JSON only; validates `schema_version`. Prefer RON for new files; keep for fixtures and `.json` interchange.
pub fn transport_network_snapshot_from_json_str(s: &str) -> Result<TransportNetworkSnapshot, TransportNetworkPersistenceError> {
    let snap: TransportNetworkSnapshot = serde_json::from_str(s)?;
    assert_schema_v1(&snap)?;
    Ok(snap)
}

pub fn transport_network_snapshot_from_json_path(path: impl AsRef<Path>) -> Result<TransportNetworkSnapshot, TransportNetworkPersistenceError> {
    let bytes = fs::read_to_string(path.as_ref())?;
    transport_network_snapshot_from_json_str(&bytes)
}

/// Dispatch by extension: `.json` → JSON; `.ron` → RON; **unknown / none** → RON then JSON fallback (older dev paths).
pub fn transport_network_snapshot_from_path(path: &Path) -> Result<TransportNetworkSnapshot, TransportNetworkPersistenceError> {
    let text = fs::read_to_string(path)?;
    let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase());
    match ext.as_deref() {
        Some("json") => transport_network_snapshot_from_json_str(&text),
        Some("ron") => transport_network_snapshot_from_ron_str(&text),
        None | Some(_) => transport_network_snapshot_from_ron_str(&text)
            .or_else(|_| transport_network_snapshot_from_json_str(&text)),
    }
}

pub fn transport_network_snapshot_to_json_string(snap: &TransportNetworkSnapshot) -> Result<String, TransportNetworkPersistenceError> {
    Ok(serde_json::to_string_pretty(snap)?)
}

/// **JSON** save for human-shared snippets / legacy only.
pub fn transport_network_snapshot_save_json_path(
    snap: &TransportNetworkSnapshot,
    path: impl AsRef<Path>,
) -> Result<(), TransportNetworkPersistenceError> {
    let s = transport_network_snapshot_to_json_string(snap)?;
    fs::write(path.as_ref(), s)?;
    Ok(())
}

/// UTF-8 body: **RON** first, then **JSON** (hybrid `.sav` transport body, clipboard paste, etc.).
pub fn hydrate_transport_from_snapshot_text(
    topology: &mut TransportTopology,
    field_store: &mut TransportFieldStore,
    edge_directory: &mut TransportEdgeDirectory,
    text: &str,
) -> Result<TransportNetworkSnapshot, TransportNetworkPersistenceError> {
    let snap = transport_network_snapshot_from_ron_str(text)
        .or_else(|_| transport_network_snapshot_from_json_str(text))?;
    hydrate_transport_from_snapshot(topology, field_store, edge_directory, &snap)?;
    Ok(snap)
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

/// **RON** hydrate (convenience).
pub fn hydrate_transport_from_ron_str(
    topology: &mut TransportTopology,
    field_store: &mut TransportFieldStore,
    edge_directory: &mut TransportEdgeDirectory,
    ron: &str,
) -> Result<TransportNetworkSnapshot, TransportNetworkPersistenceError> {
    let snap = transport_network_snapshot_from_ron_str(ron)?;
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
    mut construction_book: Option<ResMut<crate::strategic::CorridorConstructionBook>>,
) {
    for msg in messages.read() {
        let path = Path::new(msg.path.as_ref());
        match transport_network_snapshot_from_path(path) {
            Ok(snap) => {
                match hydrate_transport_from_snapshot(
                    topology.as_mut(),
                    field_store.as_mut(),
                    edge_directory.as_mut(),
                    &snap,
                ) {
                    Ok(()) => {
                        if let Some(book) = construction_book.as_mut() {
                            crate::strategic::apply_corridor_book_from_transport_snapshot(
                                book,
                                edge_directory.as_ref(),
                                &snap,
                            );
                        }
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

    #[test]
    fn g4_chain_fixture_round_trips_json_to_ron_and_back() {
        let s0 = transport_network_snapshot_from_json_str(fixture_chain_v1_json()).unwrap();
        let ron = transport_network_snapshot_to_ron_string(&s0).unwrap();
        let s1 = transport_network_snapshot_from_ron_str(&ron).unwrap();
        assert_eq!(s0, s1);
    }

    #[test]
    fn g4_fixture_ron_chain_loads_from_path() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/test_fixtures/transport/network_chain_v1.ron");
        let s = transport_network_snapshot_from_path(&path).unwrap();
        assert_eq!(s.edges.len(), 2);
        let json_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/test_fixtures/transport/network_chain_v1.json");
        let sj = transport_network_snapshot_from_path(&json_path).unwrap();
        assert_eq!(
            serde_json::to_value(&s).unwrap(),
            serde_json::to_value(&sj).unwrap()
        );
    }

    #[test]
    fn g4_from_path_respects_json_extension() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/test_fixtures/transport/network_chain_v1.json");
        let s = transport_network_snapshot_from_path(&path).unwrap();
        assert_eq!(s.edges.len(), 2);
    }

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
