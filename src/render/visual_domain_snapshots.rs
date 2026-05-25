//! Committed domain snapshots for logistics + ecology (Stage 5 spine).

use bevy::prelude::*;

use crate::economy::logistics::ThroughputSolverState;
use crate::render::FireSimulationSnapshot;
use crate::render::{ClimateVisualAggregate, EcologyVisualSnapshot, LogisticsVisualSnapshot};
use crate::strategic::{edge_flow_for_overlay, CorridorConstructionBook, LogisticsGraph};

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
