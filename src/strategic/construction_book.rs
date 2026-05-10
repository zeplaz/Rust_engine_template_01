//! **P2** construction ledger keyed by [`TransportEdgeId`](crate::systems::transport::TransportEdgeId).
//!
//! Drives strategic corridor entities and logistics capacity multipliers per
//! [`infrastructure_construction_runbook_v1.md`](../../prompts/guides/infrastructure_construction_runbook_v1.md) §10 (construction states).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::systems::transport::TransportEdgeId;

/// High-level phase for a corridor span (edge). Matches runbook “plan → build → operate” at coarse granularity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CorridorConstructionPhase {
    /// Routed / authorized; no operational traffic, no wear from transport fields.
    Planned,
    /// Partially open — `progress` scales traffic and wear.
    InProgress,
    /// Fully open — full capacity and transport-linked wear.
    #[default]
    Completed,
}

/// Per-edge construction snapshot (gameplay / construction systems write into [`CorridorConstructionBook`]).
#[derive(Clone, Copy, Debug, Component)]
pub struct CorridorConstructionStatus {
    pub phase: CorridorConstructionPhase,
    /// Only used when [`CorridorConstructionPhase::InProgress`]; ignored otherwise. Range **0..=1**.
    pub progress: f32,
}

impl Default for CorridorConstructionStatus {
    fn default() -> Self {
        Self {
            phase: CorridorConstructionPhase::Completed,
            progress: 1.0,
        }
    }
}

impl CorridorConstructionStatus {
    /// Multiplier for **operational** capacity, transport coupling, and wear (0 = not open).
    #[inline]
    pub fn traffic_factor(&self) -> f32 {
        match self.phase {
            CorridorConstructionPhase::Planned => 0.0,
            CorridorConstructionPhase::InProgress => self.progress.clamp(0.0, 1.0),
            CorridorConstructionPhase::Completed => 1.0,
        }
    }
}

/// Authoritative construction snapshot per transport edge. Missing entries ⇒ **completed** (legacy / baked edges).
#[derive(Resource, Debug, Default)]
pub struct CorridorConstructionBook {
    pub by_edge: HashMap<TransportEdgeId, CorridorConstructionStatus>,
}

impl CorridorConstructionBook {
    #[inline]
    pub fn traffic_factor(&self, eid: TransportEdgeId) -> f32 {
        self.by_edge
            .get(&eid)
            .map(CorridorConstructionStatus::traffic_factor)
            .unwrap_or(1.0)
    }
}

/// Stable fingerprint of **which** edges exist (order-independent). Used to detect bake / G4 loads.
pub fn transport_directory_edge_signature(directory: &crate::systems::transport::TransportEdgeDirectory) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut ids: Vec<_> = directory.by_edge.keys().copied().collect();
    ids.sort_by_key(|k| k.0);
    let mut h = DefaultHasher::new();
    for id in ids {
        id.0.hash(&mut h);
    }
    h.finish()
}

/// After transport **rehydrate** (editor bake, G4 JSON, hybrid): drop stale book rows; add **Completed** for new edge ids.
/// Existing entries for edges that remain are **preserved** (playtest can keep `Planned` / `InProgress` on specific ids).
pub fn align_corridor_book_with_transport_directory(
    directory: &crate::systems::transport::TransportEdgeDirectory,
    book: &mut CorridorConstructionBook,
) {
    book.by_edge.retain(|eid, _| directory.by_edge.contains_key(eid));
    for eid in directory.by_edge.keys() {
        book.by_edge.entry(*eid).or_insert(CorridorConstructionStatus::default());
    }
}

use crate::systems::transport::{TransportConstructionRecord, TransportNetworkSnapshot};
use crate::systems::transport::{TransportEdgeDirectory, TransportTopology};

/// Wire enum → snapshot string (stable for R8 / RON).
pub fn corridor_phase_to_wire(phase: CorridorConstructionPhase) -> &'static str {
    match phase {
        CorridorConstructionPhase::Planned => "Planned",
        CorridorConstructionPhase::InProgress => "InProgress",
        CorridorConstructionPhase::Completed => "Completed",
    }
}

pub fn corridor_phase_from_wire(s: &str) -> Option<CorridorConstructionPhase> {
    match s {
        "Planned" => Some(CorridorConstructionPhase::Planned),
        "InProgress" => Some(CorridorConstructionPhase::InProgress),
        "Completed" => Some(CorridorConstructionPhase::Completed),
        _ => None,
    }
}

/// Build R8 construction slice from the live book (only edges that exist in `topology`).
pub fn transport_construction_records_from_book(
    book: &CorridorConstructionBook,
    topology: &TransportTopology,
) -> Vec<TransportConstructionRecord> {
    let mut ids: Vec<_> = topology.neighbors.keys().copied().collect();
    ids.sort_by_key(|k| k.0);
    ids
        .into_iter()
        .filter_map(|eid| {
            book.by_edge.get(&eid).map(|st| TransportConstructionRecord {
                edge_id: eid.0,
                phase: corridor_phase_to_wire(st.phase).to_string(),
                progress: st.progress,
            })
        })
        .collect()
}

/// After **G4** hydrate: restore book from snapshot, or align from directory when `construction` is empty.
pub fn apply_corridor_book_from_transport_snapshot(
    book: &mut CorridorConstructionBook,
    directory: &TransportEdgeDirectory,
    snap: &TransportNetworkSnapshot,
) {
    if snap.construction.is_empty() {
        align_corridor_book_with_transport_directory(directory, book);
        return;
    }
    book.by_edge.clear();
    for r in &snap.construction {
        let eid = TransportEdgeId(r.edge_id);
        if directory.by_edge.contains_key(&eid) {
            let phase =
                corridor_phase_from_wire(&r.phase).unwrap_or(CorridorConstructionPhase::Completed);
            book.by_edge.insert(
                eid,
                CorridorConstructionStatus {
                    phase,
                    progress: r.progress,
                },
            );
        }
    }
    align_corridor_book_with_transport_directory(directory, book);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeMeta};

    #[test]
    fn align_adds_completed_for_new_edges_and_drops_removed() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(1),
            TransportEdgeMeta {
                head_key: "a".into(),
                tail_key: "b".into(),
                ..Default::default()
            },
        );
        let mut book = CorridorConstructionBook::default();
        book.by_edge.insert(
            TransportEdgeId(99),
            CorridorConstructionStatus {
                phase: CorridorConstructionPhase::Planned,
                progress: 0.0,
            },
        );
        align_corridor_book_with_transport_directory(&dir, &mut book);
        assert!(!book.by_edge.contains_key(&TransportEdgeId(99)));
        assert_eq!(
            book.by_edge.get(&TransportEdgeId(1)).map(|s| s.phase),
            Some(CorridorConstructionPhase::Completed)
        );
    }

    #[test]
    fn align_preserves_existing_phase_when_edge_still_present() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(2),
            TransportEdgeMeta::default(),
        );
        let mut book = CorridorConstructionBook::default();
        book.by_edge.insert(
            TransportEdgeId(2),
            CorridorConstructionStatus {
                phase: CorridorConstructionPhase::Planned,
                progress: 0.0,
            },
        );
        align_corridor_book_with_transport_directory(&dir, &mut book);
        assert_eq!(
            book.by_edge.get(&TransportEdgeId(2)).map(|s| s.phase),
            Some(CorridorConstructionPhase::Planned)
        );
    }

    #[test]
    fn apply_from_snapshot_restores_planned_phase() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(TransportEdgeId(1), TransportEdgeMeta::default());
        let mut book = CorridorConstructionBook::default();
        let snap = crate::systems::transport::TransportNetworkSnapshot {
            schema_version: 1,
            nodes: vec![],
            edges: vec![],
            construction: vec![TransportConstructionRecord {
                edge_id: 1,
                phase: "Planned".into(),
                progress: 0.0,
            }],
        };
        apply_corridor_book_from_transport_snapshot(&mut book, &dir, &snap);
        assert_eq!(
            book.by_edge.get(&TransportEdgeId(1)).map(|s| s.phase),
            Some(CorridorConstructionPhase::Planned)
        );
    }
}
