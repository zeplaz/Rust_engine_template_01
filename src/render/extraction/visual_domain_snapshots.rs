//! Committed domain snapshots for logistics + ecology (Stage 5 spine).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::economy::logistics::ThroughputSolverState;
use crate::render::FireSimulationSnapshot;
use crate::render::{ClimateVisualAggregate, EcologyVisualSnapshot, LogisticsVisualSnapshot};
use crate::strategic::{
    edge_flow_for_overlay, CorridorConstructionBook, LogisticsGraph, LogisticsNodeId, PlannedSite,
};
use crate::strategic::StrategicRasterConfig;
use crate::systems::transport::TransportEdgeDirectory;
use crate::systems::TransportEdgeId;
use crate::terrain::ChunkCellKey;

/// Snapshot cap (compositor paints at [`crate::render::minimap_compositor::composite::M3_UNIT_MARKER_CAP`]).
pub const MINIMAP_UNIT_MARKER_SNAPSHOT_CAP: usize = 32;

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
    registry: Option<Res<crate::substrate::WorldSubstrateRegistry>>,
    mut snapshot: ResMut<LogisticsVisualSnapshot>,
) {
    fill_logistics_snapshot(
        &fire,
        graph.as_deref(),
        solver.as_deref(),
        book.as_deref(),
        &mut snapshot,
    );
    crate::substrate::apply_slab_traction_to_logistics_snapshot(registry, snapshot);
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
}

/// **UI-P3-M3-UNITS-001** — legacy witness seed (tests only; sim uses [`fill_minimap_unit_markers_from_logistics`]).
pub fn seed_minimap_m3_units_replay_witness(snapshot: &mut MinimapOperationalSnapshot) {
    if snapshot.unit_markers.len() >= 4 {
        return;
    }
    snapshot.unit_markers.clear();
    for i in 0..6u32 {
        snapshot.unit_markers.push((i % 8, (i * 2) % 8));
    }
}

fn parse_transport_tile_key(key: &str) -> Option<(u32, u32)> {
    let rest = key.strip_prefix('t')?;
    let (xs, zs) = rest.split_once('_')?;
    Some((xs.parse().ok()?, zs.parse().ok()?))
}

fn tile_to_chunk(tx: u32, tz: u32, cells: UVec2) -> IVec2 {
    let sx = cells.x.max(1);
    let sy = cells.y.max(1);
    IVec2::new((tx / sx) as i32, (tz / sy) as i32)
}

fn chunk_to_marker(chunk: IVec2) -> Option<(u32, u32)> {
    if chunk.x < 0 || chunk.y < 0 {
        return None;
    }
    Some((chunk.x as u32, chunk.y as u32))
}

/// Aggregate strategic/logistics pressure into minimap unit marker chunk coords (not seed lattice).
pub fn fill_minimap_unit_markers_from_logistics<'a>(
    graph: Option<&LogisticsGraph>,
    solver: Option<&ThroughputSolverState>,
    directory: Option<&TransportEdgeDirectory>,
    raster: Option<&StrategicRasterConfig>,
    sites: impl Iterator<Item = &'a PlannedSite>,
    out: &mut Vec<(u32, u32)>,
) -> &'static str {
    out.clear();
    let cells = raster.map(|c| c.cells_per_chunk).unwrap_or(UVec2::ONE);
    let mut mass_by_chunk: HashMap<IVec2, f32> = HashMap::new();

    if let Some(graph) = graph.filter(|g| !g.edges.is_empty()) {
        let id_to_node: HashMap<LogisticsNodeId, _> =
            graph.nodes.iter().map(|n| (n.id, n)).collect();
        for edge in &graph.edges {
            let eff = edge_flow_for_overlay(edge, solver);
            if eff <= 0.0 {
                continue;
            }
            let Some(from_n) = id_to_node.get(&edge.from).copied() else {
                continue;
            };
            let Some(to_n) = id_to_node.get(&edge.to).copied() else {
                continue;
            };
            let Some(ka) = from_n.anchor else {
                continue;
            };
            let Some(kb) = to_n.anchor else {
                continue;
            };
            let half = eff * 0.5;
            *mass_by_chunk.entry(ka.chunk).or_insert(0.0) += half;
            *mass_by_chunk.entry(kb.chunk).or_insert(0.0) += half;
        }
    }

    for site in sites {
        let chunk = tile_to_chunk(site.origin.x, site.origin.z, cells);
        *mass_by_chunk.entry(chunk).or_insert(0.0) += 1.0;
    }

    let source = if !mass_by_chunk.is_empty() {
        if graph.is_some_and(|g| !g.edges.is_empty()) {
            "logistics_graph"
        } else {
            "planned_sites"
        }
    } else if let Some(dir) = directory.filter(|d| !d.by_edge.is_empty()) {
        for meta in dir.by_edge.values() {
            for key in [&meta.head_key, &meta.tail_key] {
                if let Some((tx, tz)) = parse_transport_tile_key(key) {
                    let chunk = tile_to_chunk(tx, tz, cells);
                    *mass_by_chunk.entry(chunk).or_insert(0.0) += 0.5;
                }
            }
        }
        if mass_by_chunk.is_empty() {
            "none"
        } else {
            "transport_directory"
        }
    } else {
        "none"
    };

    let mut ranked: Vec<(IVec2, f32)> = mass_by_chunk.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (chunk, _) in ranked.into_iter().take(MINIMAP_UNIT_MARKER_SNAPSHOT_CAP) {
        if let Some(marker) = chunk_to_marker(chunk) {
            out.push(marker);
        }
    }
    source
}

/// Single writer for [`MinimapOperationalSnapshot::unit_markers`] during Simulation.
pub fn publish_minimap_operational_unit_markers_system(
    graph: Option<Res<LogisticsGraph>>,
    solver: Option<Res<ThroughputSolverState>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    raster: Option<Res<StrategicRasterConfig>>,
    sites: Query<&PlannedSite>,
    mut operational: ResMut<MinimapOperationalSnapshot>,
) {
    fill_minimap_unit_markers_from_logistics(
        graph.as_deref(),
        solver.as_deref(),
        directory.as_deref(),
        raster.as_deref(),
        sites.iter(),
        &mut operational.unit_markers,
    );
}

/// Witness: unit markers are sourced from logistics/transport/sites, not the legacy seed lattice.
#[must_use]
pub fn unit_markers_real_reader_witness_green() -> bool {
    unit_markers_real_reader_self_check().is_ok()
}

fn unit_markers_real_reader_self_check() -> Result<(), &'static str> {
    use crate::strategic::{
        BuildSiteTile, FootprintTiles, LayerType, LogisticsNode, SiteArchetype,
    };

    let mut graph = LogisticsGraph::default();
    graph.nodes = vec![
        LogisticsNode {
            id: LogisticsNodeId(0),
            throughput: 1.0,
            stockpile: 0.0,
            anchor: Some(ChunkCellKey::new(IVec2::new(2, 3), 0)),
        },
        LogisticsNode {
            id: LogisticsNodeId(1),
            throughput: 1.0,
            stockpile: 0.0,
            anchor: Some(ChunkCellKey::new(IVec2::new(4, 3), 0)),
        },
    ];
    graph.edges.push(crate::strategic::LogisticsEdge {
        from: LogisticsNodeId(0),
        to: LogisticsNodeId(1),
        transport_edge: Some(TransportEdgeId(1)),
        capacity: 4.0,
        disruption: 0.0,
        traversal_cost: 1.0,
    });

    let site = PlannedSite {
        site_id: crate::strategic::SiteId(1),
        origin: BuildSiteTile { x: 40, z: 12 },
        footprint: FootprintTiles {
            width: 2,
            depth: 2,
        },
        archetype: SiteArchetype::Factory,
        layer: LayerType::Surface,
        catalog_id: None,
        placement: None,
    };

    let mut markers = Vec::new();
    let source = fill_minimap_unit_markers_from_logistics(
        Some(&graph),
        None,
        None,
        None,
        std::iter::once(&site),
        &mut markers,
    );
    if markers.is_empty() {
        return Err("no_markers");
    }
    if source == "none" {
        return Err("unexpected_source");
    }
    if markers.iter().any(|&(x, y)| y == (x * 2) % 8 && x < 8) {
        return Err("seed_lattice_pattern");
    }
    Ok(())
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
        assert!(op.unit_markers.is_empty());
    }

    #[test]
    fn ui_p3_m3_units_real_reader_from_logistics_not_seed_lattice() {
        use crate::strategic::{LogisticsEdge, LogisticsGraph, LogisticsNode, LogisticsNodeId};
        use crate::terrain::ChunkCellKey;

        let mut graph = LogisticsGraph::default();
        graph.nodes = vec![
            LogisticsNode {
                id: LogisticsNodeId(0),
                throughput: 1.0,
                stockpile: 0.0,
                anchor: Some(ChunkCellKey::new(IVec2::new(5, 7), 0)),
            },
            LogisticsNode {
                id: LogisticsNodeId(1),
                throughput: 1.0,
                stockpile: 0.0,
                anchor: Some(ChunkCellKey::new(IVec2::new(9, 7), 0)),
            },
        ];
        graph.edges.push(LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(1),
            transport_edge: Some(TransportEdgeId(3)),
            capacity: 2.0,
            disruption: 0.0,
            traversal_cost: 1.0,
        });
        let mut markers = Vec::new();
        let source = fill_minimap_unit_markers_from_logistics(
            Some(&graph),
            None,
            None,
            None,
            [].into_iter(),
            &mut markers,
        );
        assert_eq!(source, "logistics_graph");
        assert!(!markers.is_empty());
        assert!(!markers.iter().all(|&(x, y)| y == (x * 2) % 8));
        assert!(unit_markers_real_reader_witness_green());
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
