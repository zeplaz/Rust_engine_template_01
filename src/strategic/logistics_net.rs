//! **Graph → field:** [`LogisticsGraph`](super::LogisticsGraph) edges push effective throughput into
//! [`ChunkStrategicOverlay::logistics_throughput`] and mirror a neutral summary into `logistics_strength[..][0]`.
//!
//! Faction-specific routing can later use other slots; slot **0** is reserved for aggregate infrastructure
//! pressure from nets.

use std::collections::HashMap;

use bevy::prelude::*;

use super::{ChunkStrategicOverlay, LogisticsGraph, LogisticsNode, LogisticsNodeId};
use crate::construction::PowerLineDamageBook;
use crate::infrastructure::utility::{plant_reaches_consumer, UtilityGraph};
use crate::systems::fire::ChunkFireOverlay;
use crate::terrain::generation::Chunk;

use super::schedule::StrategicOverlayCouplingScratch;

/// Clears per-cell net throughput, accumulates **effective** edge flow `capacity × (1 − disruption)` split
/// across endpoints, then copies clamped throughput into `logistics_strength[i][0]` for quick AI/debug reads.
/// Burning heat on an edge cell raises disruption. No burn leaves the edge value.
/// Heat of 1 matches the closed threshold [`open_flow`] already uses.
#[must_use]
pub fn disruption_after_cell_heat(disruption: f32, heat: f32) -> f32 {
    let base = if disruption.is_finite() {
        disruption.clamp(0.0, 1.0)
    } else {
        1.0
    };
    if !heat.is_finite() || heat <= 0.0 {
        base
    } else {
        base.max(heat.clamp(0.0, 1.0))
    }
}

fn edge_open_flow(
    edge: &super::LogisticsEdge,
    solver: Option<&crate::economy::logistics::ThroughputSolverState>,
    disruption: f32,
) -> f32 {
    if let Some(solver) = solver {
        if let Some(tid) = edge.transport_edge {
            let idx = tid.0 as usize;
            return match solver.load.get(idx) {
                Some(&load) => open_flow(load, disruption),
                None => 0.0,
            };
        }
    }
    open_flow(edge.capacity, disruption)
}

#[must_use]
pub fn edge_flow_for_overlay(
    edge: &super::LogisticsEdge,
    solver: Option<&crate::economy::logistics::ThroughputSolverState>,
) -> f32 {
    edge_open_flow(edge, solver, edge.disruption)
}

fn heat_on_logistics_edge(
    edge: &super::LogisticsEdge,
    nodes: &HashMap<LogisticsNodeId, &LogisticsNode>,
    heat_at: &HashMap<(IVec2, u32), f32>,
) -> f32 {
    let mut heat = 0.0f32;
    for id in [edge.from, edge.to] {
        let Some(node) = nodes.get(&id).copied() else {
            continue;
        };
        let Some(anchor) = node.anchor else {
            continue;
        };
        if let Some(cell_heat) = heat_at.get(&(anchor.chunk, anchor.cell_index)) {
            heat = heat.max(*cell_heat);
        }
    }
    heat
}

/// Open flow after disruption. A fully disrupted edge does not move a haul.
/// A negative capacity stays closed.
#[must_use]
pub fn open_flow(capacity: f32, disruption: f32) -> f32 {
    if !capacity.is_finite() || !disruption.is_finite() || capacity <= 0.0 {
        return 0.0;
    }
    let d = disruption.clamp(0.0, 1.0);
    (capacity * (1.0 - d)).max(0.0)
}

/// Haul phase after traversal cost. Cost at or below 1 leaves the phase alone.
/// The overlay inject does not read traversal cost yet; tests lock the curve.
#[cfg(test)]
#[must_use]
pub fn haul_phase_after_cost(phase: f32, traversal_cost: f32) -> f32 {
    let cost = traversal_cost.max(0.001);
    if cost <= 1.0 {
        phase.max(0.0)
    } else {
        (phase.max(0.0) / cost).clamp(0.0, 1.0)
    }
}

/// Share of open flow that lands on each anchored end. A missing end does not cancel the other.
#[must_use]
pub fn endpoint_flow_shares(flow: f32, from_anchored: bool, to_anchored: bool) -> (f32, f32) {
    if flow <= 0.0 {
        return (0.0, 0.0);
    }
    let half = flow * 0.5;
    (
        if from_anchored { half } else { 0.0 },
        if to_anchored { half } else { 0.0 },
    )
}

/// Haul phase along an edge. Full disruption stays at the origin.
#[must_use]
pub fn haul_marker_t_for_edge(capacity: f32, disruption: f32, tick: u64) -> f32 {
    let cap = capacity.max(0.0);
    let frac = if cap <= 1e-6 {
        0.0
    } else {
        open_flow(cap, disruption) / cap
    };
    haul_marker_t(frac, tick)
}

/// One haul's class at a map tile. Absent means the inject does not gate edges.
#[derive(Resource, Clone, Copy, Debug)]
pub struct HaulTrafficClass {
    pub kind: crate::entities::MilitaryCivilian,
    pub clearance: u8,
    pub tile: IVec2,
}

/// Flow that may cross a town. A refused vehicle contributes nothing.
/// Does not write [`super::LogisticsEdge::disruption`].
#[must_use]
pub fn haul_crosses_town_flow(
    flow: f32,
    policy: crate::strategic::settlement::TownAccessPolicy,
    kind: crate::entities::MilitaryCivilian,
    clearance: u8,
) -> f32 {
    if !flow.is_finite() || flow <= 0.0 {
        return 0.0;
    }
    if crate::strategic::settlement::town_allows_vehicle(policy, kind, clearance) {
        flow
    } else {
        0.0
    }
}

fn policy_at_tile(
    towns: &crate::strategic::settlement::TownBook,
    districts: &crate::strategic::settlement::DistrictBook,
    tile: IVec2,
) -> Option<crate::strategic::settlement::TownAccessPolicy> {
    for district in districts.districts.values() {
        if district.contains_tile(tile) {
            return towns.towns.get(&district.town_id).map(|town| town.access);
        }
    }
    None
}

fn gate_haul_through_town(
    flow: f32,
    traffic: Option<&HaulTrafficClass>,
    towns: Option<&crate::strategic::settlement::TownBook>,
    districts: Option<&crate::strategic::settlement::DistrictBook>,
) -> f32 {
    let Some(traffic) = traffic else {
        return flow;
    };
    let (Some(towns), Some(districts)) = (towns, districts) else {
        return flow;
    };
    match policy_at_tile(towns, districts, traffic.tile) {
        Some(policy) => haul_crosses_town_flow(flow, policy, traffic.kind, traffic.clearance),
        None => flow,
    }
}

/// A haul that depends on the plant moves only while that plant still feeds a consumer.
#[must_use]
pub fn haul_flow_while_plant_feeds(flow: f32, plant_feeds: bool) -> f32 {
    if !plant_feeds || !flow.is_finite() || flow <= 0.0 {
        0.0
    } else {
        flow
    }
}

pub fn logistics_net_inject_into_overlays(
    graph: Res<LogisticsGraph>,
    solver: Option<Res<crate::economy::logistics::ThroughputSolverState>>,
    utility: Option<Res<UtilityGraph>>,
    damage: Option<Res<PowerLineDamageBook>>,
    fire: Query<(&Chunk, &ChunkFireOverlay)>,
    mut q: Query<(&Chunk, &mut ChunkStrategicOverlay)>,
    mut scratch: ResMut<StrategicOverlayCouplingScratch>,
    traffic: Option<Res<HaulTrafficClass>>,
    towns: Option<Res<crate::strategic::settlement::TownBook>>,
    districts: Option<Res<crate::strategic::settlement::DistrictBook>>,
) {
    let mut heat_at: HashMap<(IVec2, u32), f32> = HashMap::new();
    for (chunk, overlay) in &fire {
        for (index, heat) in overlay.heat.iter().enumerate() {
            if *heat > 0.0 {
                heat_at.insert((chunk.coord, index as u32), *heat);
            }
        }
    }

    for (_chunk, mut overlay) in q.iter_mut() {
        overlay.logistics_throughput.fill(0.0);
    }

    let plant_feeds = match utility.as_deref() {
        None => true,
        Some(utility) => {
            let blocked = damage
                .as_deref()
                .map(|book| book.destroyed_link_ids())
                .unwrap_or_default();
            plant_reaches_consumer(utility, &blocked)
        }
    };

    let id_to_node: HashMap<LogisticsNodeId, &LogisticsNode> =
        graph.nodes.iter().map(|n| (n.id, n)).collect();

    let mut by_chunk: HashMap<IVec2, HashMap<usize, f32>> = HashMap::new();

    for edge in &graph.edges {
        let heat = heat_on_logistics_edge(edge, &id_to_node, &heat_at);
        let disruption = disruption_after_cell_heat(edge.disruption, heat);
        let eff = gate_haul_through_town(
            haul_flow_while_plant_feeds(
                edge_open_flow(edge, solver.as_deref(), disruption),
                plant_feeds,
            ),
            traffic.as_deref(),
            towns.as_deref(),
            districts.as_deref(),
        );
        if eff <= 0.0 {
            continue;
        }
        let from_n = id_to_node.get(&edge.from).copied();
        let to_n = id_to_node.get(&edge.to).copied();
        let ka = from_n.and_then(|n| n.anchor);
        let kb = to_n.and_then(|n| n.anchor);
        let (from_share, to_share) = endpoint_flow_shares(eff, ka.is_some(), kb.is_some());
        if let Some(ka) = ka {
            scratch.mark_dirty(ka.chunk);
            *by_chunk
                .entry(ka.chunk)
                .or_default()
                .entry(ka.cell_index as usize)
                .or_insert(0.0) += from_share;
        }
        if let Some(kb) = kb {
            scratch.mark_dirty(kb.chunk);
            *by_chunk
                .entry(kb.chunk)
                .or_default()
                .entry(kb.cell_index as usize)
                .or_insert(0.0) += to_share;
        }
    }

    for (chunk, mut overlay) in q.iter_mut() {
        if let Some(cells) = by_chunk.get(&chunk.coord) {
            for (ci, add) in cells {
                if *ci < overlay.logistics_throughput.len() {
                    overlay.logistics_throughput[*ci] += *add;
                }
            }
        }
        for i in 0..overlay.len_cells() {
            overlay.logistics_strength[i][0] = overlay.logistics_throughput[i].clamp(0.0, 1.0);
        }
    }
}

/// Position of a haul along an edge, 0 at the origin and 1 at the destination.
/// Idle edges stay at the origin. The phase repeats every 16 ticks.
#[must_use]
pub fn haul_marker_t(flow_frac: f32, tick: u64) -> f32 {
    if flow_frac <= 0.0 {
        return 0.0;
    }
    let phase = (tick % 16) as f32 / 16.0;
    phase * flow_frac.clamp(0.0, 1.0)
}

/// Tile of a haul mark. Idle flow stays on the edge origin.
/// Even edges advance along x. Odd edges advance along y.
#[must_use]
pub fn haul_marker_cell(edge_id: u32, traffic: f32, haul: f32, width: u32, height: u32) -> (u32, u32) {
    let ww = width.max(1);
    let hh = height.max(1);
    let x = edge_id % ww;
    let y = (edge_id / ww) % hh;
    if traffic <= 0.0 || haul <= 0.0 {
        return (x, y);
    }
    let span = if edge_id % 2 == 0 { ww } else { hh };
    if span <= 1 {
        return (x, y);
    }
    let mut shift = ((haul.clamp(0.0, 1.0) * (span - 1) as f32).round() as u32)
        .max(1)
        .min(span - 1);
    if edge_id % 2 == 1 {
        shift = (shift / 2).max(1).min(span - 1);
    }
    if edge_id % 2 == 0 {
        let nx = x.saturating_add(shift).min(ww - 1);
        (nx, y)
    } else {
        let ny = y.saturating_add(shift).min(hh - 1);
        (x, ny)
    }
}

/// Play-raster cells after `tick` open-haul phases. Idle and closed edges stay off the map.
#[must_use]
pub fn play_haul_cells_at_tick(rows: &[(u32, f32)], tick: u64, span: u32) -> Vec<(i32, i32)> {
    let markers: Vec<f32> = rows.iter().map(|(_, flow)| haul_marker_t(*flow, tick)).collect();
    play_haul_stamp_cells(rows, &markers, span)
}

/// Play-raster cells for open hauls. Closed edges and idle markers are omitted.
#[must_use]
pub fn play_haul_stamp_cells(rows: &[(u32, f32)], markers: &[f32], span: u32) -> Vec<(i32, i32)> {
    if span == 0 {
        return Vec::new();
    }
    rows.iter()
        .enumerate()
        .filter_map(|(i, &(edge, traffic))| {
            let haul = markers.get(i).copied().unwrap_or(0.0);
            if traffic <= 0.0 || haul <= 0.0 {
                return None;
            }
            let (x, y) = haul_marker_cell(edge, traffic, haul, span, span.max(1));
            Some((x as i32, y as i32))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::{LogisticsEdge, LogisticsGraph, LogisticsNode, StrategicFieldsPlugin};
    use crate::systems::terrain::MaterialUnificationPlugin;
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};
    use crate::terrain::ChunkCellKey;
    use bevy::asset::AssetPlugin;
    use bevy::prelude::{IVec2, MinimalPlugins, UVec2};

    #[test]
    fn haul_marker_advances_only_when_flow_is_open() {
        assert_eq!(haul_marker_t(0.0, 8), 0.0);
        let mid = haul_marker_t(1.0, 8);
        assert!((mid - 0.5).abs() < 1e-5, "{mid}");
        assert!(haul_marker_t(1.0, 8) > haul_marker_t(1.0, 0));
        assert_eq!(haul_marker_cell(2, 0.0, 0.8, 16, 16), (2, 0));
        assert_ne!(haul_marker_cell(2, 1.0, 0.8, 16, 16), (2, 0));
        assert!(play_haul_stamp_cells(&[(5, 0.0)], &[0.8], 16).is_empty());
        assert_eq!(play_haul_stamp_cells(&[(5, 1.0)], &[0.0], 16).len(), 0);
        assert_eq!(play_haul_stamp_cells(&[(5, 1.0)], &[0.8], 16).len(), 1);
    }

    #[test]
    fn drain_half_disruption_slows_the_haul_marker() {
        let full = haul_marker_t_for_edge(10.0, 0.0, 8);
        let half = haul_marker_t_for_edge(10.0, 0.5, 8);
        let closed = haul_marker_t_for_edge(10.0, 1.0, 8);
        assert!((full - 0.5).abs() < 1e-5, "{full}");
        assert!((half - 0.25).abs() < 1e-5, "{half}");
        assert_eq!(closed, 0.0);
        assert!(half < full);
        assert!((open_flow(10.0, 0.2) - 8.0).abs() < 1e-5);
    }

    #[test]
    fn live_haul_marker_stops_at_the_map_edge() {
        let east = haul_marker_cell(2, 1.0, 1.0, 4, 4);
        assert_eq!(east, (3, 0));
        let south = haul_marker_cell(13, 1.0, 1.0, 4, 4);
        assert_eq!(south, (1, 3));
    }

    #[test]
    fn live_play_haul_stamp_stays_inside_the_map() {
        let rows = [(2u32, 1.0), (13u32, 1.0)];
        let cells = play_haul_stamp_cells(&rows, &[1.0, 1.0], 4);
        assert!(cells.iter().all(|(x, y)| (0..4).contains(x) && (0..4).contains(y)));
        assert!(cells.contains(&(3, 0)));
        assert!(cells.contains(&(1, 3)));
    }

    #[test]
    fn open_edges_move_on_different_axes() {
        let even = haul_marker_cell(2, 1.0, 0.8, 16, 16);
        let odd = haul_marker_cell(3, 1.0, 0.8, 16, 16);
        assert!(even.0 > 2, "even x {:?}", even);
        assert_eq!(even.1, 0);
        assert_eq!(odd.0, 3);
        assert!(odd.1 > 0, "odd y {:?}", odd);
        assert_ne!(even.0 - 2, odd.1);
    }

    #[test]
    fn play_haul_stamp_advances_with_tick() {
        let rows = [(2u32, 1.0), (3u32, 1.0)];
        assert!(play_haul_cells_at_tick(&rows, 0, 16).is_empty());
        let early = play_haul_cells_at_tick(&rows, 4, 16);
        let later = play_haul_cells_at_tick(&rows, 8, 16);
        assert_eq!(early.len(), 2);
        assert_eq!(later.len(), 2);
        assert_ne!(early, later);
    }

    #[test]
    fn logistics_edge_injects_throughput_at_anchors() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .init_resource::<LogisticsGraph>()
            .init_resource::<crate::strategic::StrategicOverlayCouplingScratch>()
            .init_resource::<crate::strategic::StrategicOverlayDisplayPolicy>()
            .add_systems(Update, logistics_net_inject_into_overlays);

        app.world_mut().insert_resource(LogisticsGraph {
            revision: 1,
            nodes: vec![
                LogisticsNode {
                    id: LogisticsNodeId(0),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 0)),
                },
                LogisticsNode {
                    id: LogisticsNodeId(1),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 1)),
                },
            ],
            edges: vec![LogisticsEdge {
                from: LogisticsNodeId(0),
                to: LogisticsNodeId(1),
                transport_edge: Some(crate::systems::transport::TransportEdgeId(0)),
                capacity: 10.0,
                disruption: 0.2,
                traversal_cost: 1.0,
            }],
        });

        app.world_mut().spawn((
            Chunk {
                coord: IVec2::ZERO,
            },
            ChunkCellMatrix::new(UVec2::new(2, 1)),
            ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(2, 1)),
        ));

        app.update();

        let mut world_query = app.world_mut().query::<(&Chunk, &ChunkStrategicOverlay)>();
        let (_, overlay) = world_query
            .iter(app.world())
            .next()
            .expect("one chunk with overlay");

        assert!((overlay.logistics_throughput[0] - 4.0).abs() < 1e-4);
        assert!((overlay.logistics_throughput[1] - 4.0).abs() < 1e-4);
        assert!((overlay.logistics_strength[0][0] - 1.0).abs() < 1e-4);
        assert!((overlay.logistics_strength[1][0] - 1.0).abs() < 1e-4);
    }

    /// Runbook round — full disruption kills effective flow (`strategic_overlay_runbook_v1` / logistics coupling).
    #[test]
    fn logistics_full_disruption_zeroes_injection() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .init_resource::<LogisticsGraph>()
            .add_plugins(MaterialUnificationPlugin)
            .add_plugins(StrategicFieldsPlugin);

        app.world_mut().insert_resource(LogisticsGraph {
            revision: 1,
            nodes: vec![
                LogisticsNode {
                    id: LogisticsNodeId(0),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 0)),
                },
                LogisticsNode {
                    id: LogisticsNodeId(1),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 1)),
                },
            ],
            edges: vec![LogisticsEdge {
                from: LogisticsNodeId(0),
                to: LogisticsNodeId(1),
                transport_edge: Some(crate::systems::transport::TransportEdgeId(0)),
                capacity: 10.0,
                disruption: 1.0,
                traversal_cost: 1.0,
            }],
        });

        app.world_mut().spawn((
            Chunk {
                coord: IVec2::ZERO,
            },
            ChunkCellMatrix::new(UVec2::new(2, 1)),
        ));

        app.update();

        let mut world_query = app.world_mut().query::<(&Chunk, &ChunkStrategicOverlay)>();
        let (_, overlay) = world_query
            .iter(app.world())
            .next()
            .expect("one chunk with overlay");

        assert!((overlay.logistics_throughput[0]).abs() < 1e-6);
        assert!((overlay.logistics_throughput[1]).abs() < 1e-6);
    }

    #[test]
    fn push_zero_solver_load_does_not_restore_static_capacity() {
        let edge = LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(1),
            capacity: 100.0,
            disruption: 0.0,
            traversal_cost: 1.0,
            transport_edge: Some(crate::systems::transport::TransportEdgeId(0)),
        };
        let mut solver = crate::economy::logistics::ThroughputSolverState::default();
        solver.ensure_len(1);
        solver.load[0] = 0.0;
        let flow = edge_flow_for_overlay(&edge, Some(&solver));
        assert!((flow - 0.0).abs() < 1e-4);
        assert!(flow < edge.capacity);
    }

    #[test]
    fn push_disrupted_solver_load_closes_the_edge() {
        let edge = LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(1),
            capacity: 100.0,
            disruption: 1.0,
            traversal_cost: 1.0,
            transport_edge: Some(crate::systems::transport::TransportEdgeId(0)),
        };
        let mut solver = crate::economy::logistics::ThroughputSolverState::default();
        solver.ensure_len(1);
        solver.load[0] = 2.5;
        assert!((edge_flow_for_overlay(&edge, Some(&solver)) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn next_one_anchor_keeps_its_half_of_the_flow() {
        let (from, to) = endpoint_flow_shares(8.0, true, false);
        assert!((from - 4.0).abs() < 1e-5);
        assert_eq!(to, 0.0);
        let closed = endpoint_flow_shares(0.0, true, true);
        assert_eq!(closed, (0.0, 0.0));
    }

    #[test]
    fn next_costly_haul_advances_slower() {
        let open = haul_marker_t_for_edge(10.0, 0.0, 8);
        let flat = haul_phase_after_cost(open, 1.0);
        let steep = haul_phase_after_cost(open, 2.0);
        assert!((flat - open).abs() < 1e-5);
        assert!(steep < flat);
        assert!((steep - open * 0.5).abs() < 1e-5);
    }

    #[test]
    fn next_open_flow_rejects_negative_capacity() {
        assert_eq!(open_flow(-4.0, 0.0), 0.0);
        assert_eq!(open_flow(0.0, 0.0), 0.0);
        assert!((open_flow(10.0, 0.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn cont_missing_solver_slot_stays_closed() {
        let edge = LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(1),
            capacity: 100.0,
            disruption: 0.0,
            traversal_cost: 1.0,
            transport_edge: Some(crate::systems::transport::TransportEdgeId(5)),
        };
        let mut solver = crate::economy::logistics::ThroughputSolverState::default();
        solver.ensure_len(1);
        assert_eq!(edge_flow_for_overlay(&edge, Some(&solver)), 0.0);
        assert!(edge_flow_for_overlay(&edge, None) > 0.0);
    }

    #[test]
    fn cont_zero_span_stamps_no_haul() {
        assert!(play_haul_stamp_cells(&[(2, 1.0)], &[0.8], 0).is_empty());
        assert_eq!(play_haul_stamp_cells(&[(2, 1.0)], &[0.8], 4).len(), 1);
    }

    #[test]
    fn cont_non_finite_flow_stays_closed() {
        assert_eq!(open_flow(f32::NAN, 0.0), 0.0);
        assert_eq!(open_flow(10.0, f32::NAN), 0.0);
        assert!((open_flow(10.0, 0.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn over_zero_flow_on_a_live_tick_stays_closed() {
        assert_eq!(open_flow(0.0, 0.25), 0.0);
        assert_eq!(haul_marker_t(0.0, 15), 0.0);
        assert!(play_haul_cells_at_tick(&[(6, 0.0)], 12, 16).is_empty());
    }

    #[test]
    fn over_unit_flow_uses_the_whole_phase() {
        assert_eq!(haul_marker_t(1.0, 12), 0.75);
        assert_eq!(haul_marker_t(1.0, 15), 15.0 / 16.0);
        assert!(haul_marker_t(1.0, 12) > haul_marker_t(0.5, 12));
    }

    #[test]
    fn over_sixteenth_tick_restarts_the_haul() {
        assert_eq!(haul_marker_t(1.0, 16), 0.0);
        assert_eq!(haul_marker_t(1.0, 32), 0.0);
        assert!(play_haul_cells_at_tick(&[(2, 1.0)], 16, 16).is_empty());
        assert_eq!(haul_marker_t(1.0, 17), 1.0 / 16.0);
    }

    #[test]
    fn over_full_disruption_sits_on_the_origin() {
        let stalled = haul_marker_t_for_edge(10.0, 1.0, 15);
        assert_eq!(stalled, 0.0);
        assert_eq!(open_flow(4.0, 1.0), 0.0);
        assert_eq!(haul_marker_cell(2, 1.0, stalled, 16, 16), (2, 0));
        assert_eq!(
            haul_marker_cell(3, 1.0, haul_marker_t_for_edge(8.0, 1.0, 7), 16, 16),
            (3, 0)
        );
    }

    #[test]
    fn grid_islanded_plant_stops_the_haul() {
        use crate::infrastructure::utility::graph::{
            fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .init_resource::<LogisticsGraph>()
            .init_resource::<crate::strategic::StrategicOverlayCouplingScratch>()
            .insert_resource(hydrate_utility_graph_from_snapshot(
                &fixture_utility_network_snapshot(),
            ))
            .insert_resource(PowerLineDamageBook::default())
            .add_systems(Update, logistics_net_inject_into_overlays);

        app.world_mut().insert_resource(LogisticsGraph {
            revision: 1,
            nodes: vec![
                LogisticsNode {
                    id: LogisticsNodeId(0),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 0)),
                },
                LogisticsNode {
                    id: LogisticsNodeId(1),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 1)),
                },
            ],
            edges: vec![LogisticsEdge {
                from: LogisticsNodeId(0),
                to: LogisticsNodeId(1),
                transport_edge: Some(crate::systems::transport::TransportEdgeId(0)),
                capacity: 10.0,
                disruption: 0.2,
                traversal_cost: 1.0,
            }],
        });
        app.world_mut().spawn((
            Chunk {
                coord: IVec2::ZERO,
            },
            ChunkCellMatrix::new(UVec2::new(2, 1)),
            ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(2, 1)),
        ));

        app.update();
        let fed = {
            let mut q = app.world_mut().query::<(&Chunk, &ChunkStrategicOverlay)>();
            let (_, overlay) = q.iter(app.world()).next().expect("overlay");
            overlay.logistics_throughput[0]
        };
        assert!((fed - 4.0).abs() < 1e-4, "{fed}");

        {
            let mut book = app.world_mut().resource_mut::<PowerLineDamageBook>();
            assert!(book.cut_segment(10));
            assert!(book.cut_segment(11));
        }
        app.update();
        let mut q = app.world_mut().query::<(&Chunk, &ChunkStrategicOverlay)>();
        let (_, overlay) = q.iter(app.world()).next().expect("overlay");
        assert!(overlay.logistics_throughput[0].abs() < 1e-6);
        assert!(overlay.logistics_throughput[1].abs() < 1e-6);
        assert_eq!(haul_flow_while_plant_feeds(8.0, false), 0.0);
    }

    #[test]
    fn link_fire_on_an_edge_raises_disruption_and_stops_the_haul() {
        let authored = 0.2f32;
        let heat = 1.0f32;
        let disruption = disruption_after_cell_heat(authored, heat);
        assert!(disruption > authored);
        assert_eq!(haul_marker_t_for_edge(10.0, disruption, 8), 0.0);
        assert_eq!(open_flow(10.0, disruption), 0.0);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .init_resource::<LogisticsGraph>()
            .init_resource::<crate::strategic::StrategicOverlayCouplingScratch>()
            .init_resource::<crate::strategic::StrategicOverlayDisplayPolicy>()
            .add_systems(Update, logistics_net_inject_into_overlays);
        app.world_mut().insert_resource(LogisticsGraph {
            revision: 1,
            nodes: vec![
                LogisticsNode {
                    id: LogisticsNodeId(0),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 0)),
                },
                LogisticsNode {
                    id: LogisticsNodeId(1),
                    throughput: 0.0,
                    stockpile: 0.0,
                    anchor: Some(ChunkCellKey::new(IVec2::ZERO, 1)),
                },
            ],
            edges: vec![LogisticsEdge {
                from: LogisticsNodeId(0),
                to: LogisticsNodeId(1),
                transport_edge: Some(crate::systems::transport::TransportEdgeId(0)),
                capacity: 10.0,
                disruption: authored,
                traversal_cost: 1.0,
            }],
        });
        let n = 2usize;
        app.world_mut().spawn((
            Chunk { coord: IVec2::ZERO },
            ChunkCellMatrix::new(UVec2::new(2, 1)),
            ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(2, 1)),
            crate::systems::fire::ChunkFireOverlay {
                heat: vec![heat, 0.0],
                fuel: vec![0.4; n],
                smoke: vec![0.0; n],
                toxic: vec![0.0; n],
                burn_hold: vec![0; n],
            },
        ));
        app.update();
        let stored = app.world().resource::<LogisticsGraph>().edges[0].disruption;
        assert!((stored - authored).abs() < 1e-6);
        let mut q = app.world_mut().query::<(&Chunk, &ChunkStrategicOverlay)>();
        let (_, overlay) = q.iter(app.world()).next().expect("overlay");
        assert!(overlay.logistics_throughput[0].abs() < 1e-6);
        assert!(overlay.logistics_throughput[1].abs() < 1e-6);
    }

    #[test]
    fn link_cold_edge_stays_open() {
        let authored = 0.0f32;
        let disruption = disruption_after_cell_heat(authored, 0.0);
        assert!((disruption - authored).abs() < 1e-6);
        assert!(open_flow(10.0, disruption) > 0.0);
        assert!(haul_marker_t_for_edge(10.0, disruption, 8) > 0.0);
    }
}
