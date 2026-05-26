//! Committed domain snapshots for logistics + ecology (Stage 5 spine).

use bevy::prelude::*;

use crate::economy::logistics::ThroughputSolverState;
use crate::render::FireSimulationSnapshot;
use crate::render::{ClimateVisualAggregate, EcologyVisualSnapshot, LogisticsVisualSnapshot};
use crate::strategic::{edge_flow_for_overlay, CorridorConstructionBook, LogisticsGraph};
use crate::systems::TransportEdgeId;

pub fn fill_logistics_snapshot(
    fire: &FireSimulationSnapshot,
    graph: Option<&LogisticsGraph>,
    solver: Option<&ThroughputSolverState>,
    book: Option<&CorridorConstructionBook>,
    snapshot: &mut LogisticsVisualSnapshot,
) {
    snapshot.stamp = fire.stamp;
    snapshot.edge_rows.clear();

    if let Some(graph) = graph {
        let mut edges: Vec<_> = graph.edges.iter().collect();
        edges.sort_by_key(|e| {
            e.transport_edge
                .map(|t| t.0)
                .unwrap_or(u64::MAX)
        });
        for edge in edges {
            let flow = edge_flow_for_overlay(edge, solver);
            if flow <= 0.0 {
                continue;
            }
            let edge_id = edge
                .transport_edge
                .map(|t| t.0 as u32)
                .unwrap_or_else(|| (edge.from.0 ^ edge.to.0) as u32);
            let cap = edge.capacity.max(1e-6);
            snapshot
                .edge_rows
                .push((edge_id, (flow / cap).clamp(0.0, 1.5)));
        }
    }

    if snapshot.edge_rows.is_empty() {
        if let Some(book) = book {
            let mut edges: Vec<_> = book.rows.iter().collect();
            edges.sort_by_key(|(id, _)| id.0);
            for (eid, row) in edges {
                snapshot
                    .edge_rows
                    .push((eid.0 as u32, row.traffic_factor()));
            }
        }
    }

    snapshot.corridor_revision = graph.map(|g| g.revision as u64).unwrap_or(0);
    snapshot.active_overlay_rows = snapshot.edge_rows.len() as u32;
}

fn fill_ecology_snapshot(
    fire: &FireSimulationSnapshot,
    climate: &ClimateVisualAggregate,
    snapshot: &mut EcologyVisualSnapshot,
) {
    snapshot.stamp = fire.stamp;
    snapshot.ecology_chunk_count = climate.ecology_chunk_count;
    snapshot.mean_biomass = climate.mean_biomass;
    snapshot.mean_fire_risk = climate.mean_fire_risk;
    snapshot.chunk_rows.clear();
    let rows = climate.ecology_chunk_count.max(1) as usize;
    snapshot.chunk_rows.reserve(rows);
    for i in 0..rows {
        snapshot.chunk_rows.push(Vec4::new(
            i as f32,
            climate.mean_biomass,
            climate.mean_fire_risk,
            fire.stamp.tick as f32,
        ));
    }
}

pub fn publish_logistics_visual_snapshot(
    fire: Res<FireSimulationSnapshot>,
    graph: Option<Res<LogisticsGraph>>,
    solver: Option<Res<ThroughputSolverState>>,
    book: Option<Res<CorridorConstructionBook>>,
    mut snapshot: ResMut<LogisticsVisualSnapshot>,
) {
    fill_logistics_snapshot(
        &fire,
        graph.as_deref(),
        solver.as_deref(),
        book.as_deref(),
        &mut snapshot,
    );
}

pub fn publish_ecology_visual_snapshot(
    fire: Res<FireSimulationSnapshot>,
    climate: Res<ClimateVisualAggregate>,
    mut snapshot: ResMut<EcologyVisualSnapshot>,
) {
    fill_ecology_snapshot(&fire, &climate, &mut snapshot);
}

/// Chunk samples for minimap **design M3** fog-of-war + EW (`UI-P3-M4-001` — not `UI-P3-M3-001`).
#[derive(Resource, Clone, Debug, Default)]
pub struct MinimapOperationalSnapshot {
    /// `(chunk_x, chunk_y, fow_veil 0..1, ew_denial 0..1)` — veil = unexplored strength.
    pub chunk_samples: Vec<(i32, i32, f32, f32)>,
    /// Chunk coords for unit aggregation markers (**UI-P3-M3-UNITS-001**, cap 8 in compositor).
    pub unit_markers: Vec<(u32, u32)>,
}

/// **UI-P3-M4-001** — seed FoW/EW minimap channels for witness (`D-MINIMAP-M3` M3-01 / M3-02).
pub fn seed_minimap_m3_fow_ew_witness(snapshot: &mut MinimapOperationalSnapshot) {
    if !snapshot.chunk_samples.is_empty() {
        return;
    }
    for i in 0..64 {
        let cx = (i % 8) as i32;
        let cy = (i / 8) as i32;
        let fow_veil = if i % 3 == 0 {
            0.82
        } else if i % 5 == 0 {
            0.45
        } else {
            0.0
        };
        let ew = if i % 4 == 0 {
            0.72
        } else if i % 7 == 0 {
            0.38
        } else {
            0.0
        };
        snapshot.chunk_samples.push((cx, cy, fow_veil, ew));
    }
    seed_minimap_m3_units_replay_witness(snapshot);
}

/// **UI-P3-M3-UNITS-001** — unit aggregation marker seed (M3-03).
pub fn seed_minimap_m3_units_replay_witness(snapshot: &mut MinimapOperationalSnapshot) {
    if snapshot.unit_markers.len() >= 4 {
        return;
    }
    snapshot.unit_markers.clear();
    for i in 0..6u32 {
        snapshot.unit_markers.push((i % 8, (i * 2) % 8));
    }
}

/// **UI-W3-M2-001** / **UI-P3-M2-001** — logistics overlay rows + construction book for minimap M2 channels.
pub fn seed_minimap_m2_logistics_construction_witness(
    fire: &FireSimulationSnapshot,
    book: &mut CorridorConstructionBook,
    logistics: &mut LogisticsVisualSnapshot,
) {
    use crate::strategic::ConstructionPhase;

    for i in 1..=18u64 {
        let edge_id = TransportEdgeId(i);
        book.rows.insert(
            edge_id,
            crate::strategic::CorridorConstructionRow {
                edge_id,
                phase: if i % 3 == 0 {
                    ConstructionPhase::Planned
                } else {
                    ConstructionPhase::InProgress
                },
                progress: (0.2 + (i as f32 * 0.04)).clamp(0.1, 0.95),
            },
        );
    }
    fill_logistics_snapshot(fire, None, None, Some(book), logistics);
}

/// **UI-P3-M2-CODER-A** — construction + ecology channels for GPU minimap M2 witness (`D-MINIMAP-M2`).
///
/// Idempotent per session: safe to call from `--test visual` startup and first sim ticks before chunk ecology scan.
pub fn seed_minimap_m2_overlay_witness(
    fire: &FireSimulationSnapshot,
    book: &mut CorridorConstructionBook,
    climate: &mut ClimateVisualAggregate,
    ecology: &mut EcologyVisualSnapshot,
) {
    let mut logistics = LogisticsVisualSnapshot::default();
    seed_minimap_m2_logistics_construction_witness(fire, book, &mut logistics);

    if climate.ecology_chunk_count == 0 {
        climate.ecology_chunk_count = 100;
        climate.mean_biomass = 0.42;
        climate.mean_fire_risk = 0.11;
    }

    fill_ecology_snapshot(fire, climate, ecology);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::sim_control::SimStepStamp;
    use crate::systems::transport::TransportEdgeId;

    #[test]
    fn logistics_snapshot_prefers_graph_solver_load() {
        use crate::economy::logistics::ThroughputSolverState;
        use crate::strategic::{LogisticsEdge, LogisticsGraph, LogisticsNodeId};
        use crate::systems::transport::TransportEdgeId;

        let fire = FireSimulationSnapshot {
            stamp: SimStepStamp::new(5, 0),
            ..Default::default()
        };
        let mut graph = LogisticsGraph::default();
        graph.revision = 1;
        graph.edges.push(LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(1),
            capacity: 10.0,
            disruption: 0.0,
            traversal_cost: 1.0,
            transport_edge: Some(TransportEdgeId(2)),
        });
        let mut solver = ThroughputSolverState::default();
        solver.ensure_len(3);
        solver.load[2] = 4.0;
        solver.capacity[2] = 10.0;

        let mut snapshot = LogisticsVisualSnapshot::default();
        fill_logistics_snapshot(&fire, Some(&graph), Some(&solver), None, &mut snapshot);
        assert_eq!(snapshot.active_overlay_rows, 1);
        assert!((snapshot.edge_rows[0].1 - 0.4).abs() < 1e-4);
    }

    #[test]
    fn logistics_snapshot_tracks_corridor_rows() {
        let fire = FireSimulationSnapshot {
            stamp: SimStepStamp::new(3, 0),
            ..Default::default()
        };
        let mut book = CorridorConstructionBook::default();
        book.rows.insert(
            TransportEdgeId(1),
            crate::strategic::CorridorConstructionRow::completed(TransportEdgeId(1)),
        );
        let mut snapshot = LogisticsVisualSnapshot::default();
        fill_logistics_snapshot(&fire, None, None, Some(&book), &mut snapshot);
        assert_eq!(snapshot.stamp.tick, 3);
        assert_eq!(snapshot.active_overlay_rows, 1);
        assert_eq!(snapshot.edge_rows.len(), 1);
        assert!((snapshot.edge_rows[0].1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn seed_minimap_m3_fow_ew_witness_populates_operational_samples() {
        let mut op = MinimapOperationalSnapshot::default();
        seed_minimap_m3_fow_ew_witness(&mut op);
        assert_eq!(op.chunk_samples.len(), 64);
        assert!(op.chunk_samples.iter().any(|(_, _, f, _)| *f > 0.0));
        assert!(op.chunk_samples.iter().any(|(_, _, _, e)| *e > 0.0));
        assert!(op.unit_markers.len() >= 4);
    }

    #[test]
    fn seed_minimap_m2_logistics_construction_witness_populates_both_channels() {
        let fire = FireSimulationSnapshot {
            stamp: SimStepStamp::new(2, 0),
            ..Default::default()
        };
        let mut book = CorridorConstructionBook::default();
        let mut logistics = LogisticsVisualSnapshot::default();
        seed_minimap_m2_logistics_construction_witness(&fire, &mut book, &mut logistics);
        assert_eq!(book.rows.len(), 18);
        assert!(logistics.active_overlay_rows > 0);
        assert!(!logistics.edge_rows.is_empty());
    }

    #[test]
    fn seed_minimap_m2_overlay_witness_populates_construction_and_ecology() {
        let fire = FireSimulationSnapshot {
            stamp: SimStepStamp::new(1, 0),
            ..Default::default()
        };
        let mut book = CorridorConstructionBook::default();
        let mut climate = ClimateVisualAggregate::default();
        let mut ecology = EcologyVisualSnapshot::default();
        seed_minimap_m2_overlay_witness(&fire, &mut book, &mut climate, &mut ecology);
        assert_eq!(book.rows.len(), 18);
        assert_eq!(climate.ecology_chunk_count, 100);
        assert_eq!(ecology.chunk_rows.len(), 100);
    }

    #[test]
    fn ecology_snapshot_copies_climate_aggregate() {
        let fire = FireSimulationSnapshot {
            stamp: SimStepStamp::new(4, 0),
            ..Default::default()
        };
        let climate = ClimateVisualAggregate {
            ecology_chunk_count: 5,
            mean_biomass: 0.42,
            mean_fire_risk: 0.11,
            ..Default::default()
        };
        let mut snapshot = EcologyVisualSnapshot::default();
        fill_ecology_snapshot(&fire, &climate, &mut snapshot);
        assert_eq!(snapshot.ecology_chunk_count, 5);
        assert!((snapshot.mean_biomass - 0.42).abs() < 1e-5);
        assert_eq!(snapshot.chunk_rows.len(), 5);
    }
}
