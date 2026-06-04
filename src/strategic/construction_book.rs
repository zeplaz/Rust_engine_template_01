//! **P2** construction ledger keyed by [`TransportEdgeId`](crate::systems::transport::TransportEdgeId).
//!
//! **Operational sites** use [`super::site::SiteConstructionBook`] and [`SiteConstructionPhase`](super::site::SiteConstructionPhase) per runbook §10.
//!
//! Drives strategic corridor entities and logistics capacity multipliers per
//! [`infrastructure_construction_runbook_v1.md`](../../prompts/guides/infrastructure_construction_runbook_v1.md) §10 (corridor phase set; sites use full enum in `site`).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::systems::sim_control::SimControlState;
use crate::systems::transport::TransportEdgeId;

pub type CorridorEdgeId = TransportEdgeId;

/// High-level phase for a corridor span (edge).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ConstructionPhase {
    Planned,
    InProgress,
    #[default]
    Completed,
}

pub type CorridorConstructionPhase = ConstructionPhase;

/// Authoritative per-edge construction row (book storage only).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CorridorConstructionRow {
    pub edge_id: CorridorEdgeId,
    pub phase: ConstructionPhase,
    pub progress: f32,
}

impl CorridorConstructionRow {
    #[must_use]
    pub fn completed(edge_id: CorridorEdgeId) -> Self {
        Self {
            edge_id,
            phase: ConstructionPhase::Completed,
            progress: 1.0,
        }
    }

    #[must_use]
    pub fn planned(edge_id: CorridorEdgeId) -> Self {
        Self {
            edge_id,
            phase: ConstructionPhase::Planned,
            progress: 0.0,
        }
    }

    /// Derived operational multiplier (0 = not open).
    #[inline]
    pub fn traffic_factor(&self) -> f32 {
        match self.phase {
            ConstructionPhase::Planned => 0.0,
            ConstructionPhase::InProgress => self.progress.clamp(0.0, 1.0),
            ConstructionPhase::Completed => 1.0,
        }
    }
}

/// ECS mirror copied from the book during GraphSync (not authoritative).
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub struct CorridorConstructionStatus {
    pub phase: ConstructionPhase,
    pub progress: f32,
}

impl Default for CorridorConstructionStatus {
    fn default() -> Self {
        Self {
            phase: ConstructionPhase::Completed,
            progress: 1.0,
        }
    }
}

impl From<CorridorConstructionRow> for CorridorConstructionStatus {
    fn from(row: CorridorConstructionRow) -> Self {
        Self {
            phase: row.phase,
            progress: row.progress,
        }
    }
}

impl CorridorConstructionStatus {
    #[inline]
    pub fn traffic_factor(&self) -> f32 {
        CorridorConstructionRow {
            edge_id: TransportEdgeId(0),
            phase: self.phase,
            progress: self.progress,
        }
        .traffic_factor()
    }
}

/// Authoritative construction snapshot per transport edge. Missing entries ⇒ **completed** (legacy / baked edges).
#[derive(Resource, Debug, Default)]
pub struct CorridorConstructionBook {
    pub rows: HashMap<CorridorEdgeId, CorridorConstructionRow>,
}

/// Per-**sim-tick** construction progression (P2 MVP). Wall-clock is not used.
#[derive(Resource, Debug, Clone)]
pub struct CorridorConstructionTickConfig {
    pub progress_per_tick: f32,
}

impl Default for CorridorConstructionTickConfig {
    fn default() -> Self {
        Self {
            progress_per_tick: 0.1,
        }
    }
}

impl CorridorConstructionBook {
    #[inline]
    pub fn traffic_factor(&self, eid: CorridorEdgeId) -> f32 {
        self.rows
            .get(&eid)
            .map(CorridorConstructionRow::traffic_factor)
            .unwrap_or(1.0)
    }

    pub fn plan_edge(&mut self, eid: CorridorEdgeId) {
        self.rows.insert(eid, CorridorConstructionRow::planned(eid));
    }
}

/// Advance one row by a single sim tick (deterministic; no wall-clock).
pub fn advance_corridor_construction_row(row: &mut CorridorConstructionRow, progress_per_tick: f32) {
    match row.phase {
        ConstructionPhase::Planned => {
            row.phase = ConstructionPhase::InProgress;
        }
        ConstructionPhase::InProgress => {
            row.progress += progress_per_tick;
            if row.progress >= 1.0 {
                row.progress = 1.0;
                row.phase = ConstructionPhase::Completed;
            }
        }
        ConstructionPhase::Completed => {}
    }
}

/// Sole writer for corridor construction progress.
pub fn advance_corridor_construction_book_on_sim_tick(
    sim: Res<SimControlState>,
    config: Res<CorridorConstructionTickConfig>,
    mut book: ResMut<CorridorConstructionBook>,
) {
    if !sim.should_tick() {
        return;
    }
    let step = config.progress_per_tick;
    let mut edge_ids: Vec<_> = book.rows.keys().copied().collect();
    edge_ids.sort_by_key(|id| id.0);
    for edge_id in edge_ids {
        let Some(row) = book.rows.get_mut(&edge_id) else {
            continue;
        };
        advance_corridor_construction_row(row, step);
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
    book.rows.retain(|eid, _| directory.by_edge.contains_key(eid));
    for eid in directory.by_edge.keys() {
        book
            .rows
            .entry(*eid)
            .or_insert_with(|| CorridorConstructionRow::completed(*eid));
    }
}

use crate::systems::transport::{TransportConstructionRecord, TransportNetworkSnapshot};
use crate::systems::transport::{TransportEdgeDirectory, TransportTopology};

/// Wire enum → snapshot string (stable for R8 / RON).
pub fn corridor_phase_to_wire(phase: ConstructionPhase) -> &'static str {
    match phase {
        ConstructionPhase::Planned => "Planned",
        ConstructionPhase::InProgress => "InProgress",
        ConstructionPhase::Completed => "Completed",
    }
}

pub fn corridor_phase_from_wire(s: &str) -> Option<ConstructionPhase> {
    match s {
        "Planned" => Some(ConstructionPhase::Planned),
        "InProgress" => Some(ConstructionPhase::InProgress),
        "Completed" => Some(ConstructionPhase::Completed),
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
            book.rows.get(&eid).map(|row| TransportConstructionRecord {
                edge_id: eid.0,
                phase: corridor_phase_to_wire(row.phase).to_string(),
                progress: row.progress,
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
    book.rows.clear();
    for r in &snap.construction {
        let eid = TransportEdgeId(r.edge_id);
        if directory.by_edge.contains_key(&eid) {
            let phase = corridor_phase_from_wire(&r.phase).unwrap_or(ConstructionPhase::Completed);
            book.rows.insert(
                eid,
                CorridorConstructionRow {
                    edge_id: eid,
                    phase,
                    progress: r.progress,
                },
            );
        }
    }
    align_corridor_book_with_transport_directory(directory, book);
}

#[must_use]
pub fn corridor_r8_roundtrip_witness_green() -> bool {
    corridor_r8_roundtrip_self_check().is_ok()
}

fn corridor_r8_roundtrip_self_check() -> Result<(), &'static str> {
    let mut dir = crate::systems::transport::TransportEdgeDirectory::default();
    dir.by_edge.insert(
        TransportEdgeId(1),
        crate::systems::transport::TransportEdgeMeta::default(),
    );
    let mut book = CorridorConstructionBook::default();
    let snap = crate::systems::transport::TransportNetworkSnapshot {
        schema_version: 1,
        nodes: vec![],
        edges: vec![],
        construction: vec![crate::systems::transport::TransportConstructionRecord {
            edge_id: 1,
            phase: "Planned".into(),
            progress: 0.0,
        }],
    };
    apply_corridor_book_from_transport_snapshot(&mut book, &dir, &snap);
    if book.rows.get(&TransportEdgeId(1)).map(|r| r.phase) != Some(ConstructionPhase::Planned) {
        return Err("phase");
    }
    Ok(())
}

#[must_use]
pub fn corridor_sim_tick_writer_witness_green() -> bool {
    corridor_sim_tick_writer_self_check().is_ok()
}

fn corridor_sim_tick_writer_self_check() -> Result<(), &'static str> {
    let mut row = CorridorConstructionRow::planned(TransportEdgeId(3));
    advance_corridor_construction_row(&mut row, 0.1);
    if row.phase == ConstructionPhase::Planned {
        return Err("should_advance");
    }
    Ok(())
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
        book.rows.insert(TransportEdgeId(99), CorridorConstructionRow::planned(TransportEdgeId(99)));
        align_corridor_book_with_transport_directory(&dir, &mut book);
        assert!(!book.rows.contains_key(&TransportEdgeId(99)));
        assert_eq!(
            book.rows.get(&TransportEdgeId(1)).map(|row| row.phase),
            Some(ConstructionPhase::Completed)
        );
    }

    #[test]
    fn align_preserves_existing_phase_when_edge_still_present() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(TransportEdgeId(2), TransportEdgeMeta::default());
        let mut book = CorridorConstructionBook::default();
        book.rows.insert(TransportEdgeId(2), CorridorConstructionRow::planned(TransportEdgeId(2)));
        align_corridor_book_with_transport_directory(&dir, &mut book);
        assert_eq!(
            book.rows.get(&TransportEdgeId(2)).map(|row| row.phase),
            Some(ConstructionPhase::Planned)
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
            construction: vec![crate::systems::transport::TransportConstructionRecord {
                edge_id: 1,
                phase: "Planned".into(),
                progress: 0.0,
            }],
        };
        apply_corridor_book_from_transport_snapshot(&mut book, &dir, &snap);
        assert_eq!(
            book.rows.get(&TransportEdgeId(1)).map(|row| row.phase),
            Some(ConstructionPhase::Planned)
        );
    }

    #[test]
    fn planned_corridor_advances_to_completed_in_eleven_ticks() {
        let mut row = CorridorConstructionRow::planned(TransportEdgeId(1));
        for _ in 0..11 {
            advance_corridor_construction_row(&mut row, 0.1);
        }
        assert_eq!(row.phase, ConstructionPhase::Completed);
        assert!((row.progress - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sim_tick_advances_book_when_sim_runs() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimControlState>()
            .init_resource::<CorridorConstructionBook>()
            .init_resource::<CorridorConstructionTickConfig>()
            .add_systems(
                Update,
                advance_corridor_construction_book_on_sim_tick
                    .in_set(crate::systems::sim_control::SimControlSystemSet::AdvanceSimTick),
            );

        app.world_mut()
            .resource_mut::<CorridorConstructionBook>()
            .plan_edge(TransportEdgeId(7));

        for _ in 0..11 {
            app.update();
        }

        let row = app
            .world()
            .resource::<CorridorConstructionBook>()
            .rows
            .get(&TransportEdgeId(7))
            .expect("edge row");
        assert_eq!(row.phase, ConstructionPhase::Completed);

        app.world_mut().resource_mut::<SimControlState>().paused = true;
        let before = app.world().resource::<CorridorConstructionBook>().rows.clone();
        app.update();
        let after = app.world().resource::<CorridorConstructionBook>().rows.clone();
        assert_eq!(before, after);
    }
}
