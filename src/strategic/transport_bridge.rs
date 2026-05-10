//! **Transport → strategic bridge:** rebuild [`LogisticsGraph`](super::LogisticsGraph) from live transport
//! resources, maintain per-edge [`InfrastructureCorridor`](super::sim::InfrastructureCorridor) entities,
//! and paint **routing congestion / EW-like denial** into [`ChunkStrategicOverlay`](super::ChunkStrategicOverlay).

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::construction_book::{CorridorConstructionBook, CorridorConstructionStatus};
use super::runbook_rounds::corridor::CorridorType;
use super::sim::{InfrastructureCorridor, StrategicTransportCorridor};
use super::{ChunkStrategicOverlay, LogisticsEdge, LogisticsGraph, LogisticsNode, LogisticsNodeId};
use crate::systems::transport::{
    edge_traversal_cost, TransportCostWeights, TransportEdgeDirectory, TransportEdgeId,
    TransportFieldStore,
};
use crate::terrain::generation::Chunk;
use crate::terrain::ChunkCellKey;

/// Row-major chunk size for tile indices → [`ChunkCellKey`] (align with primary [`ChunkCellMatrix`](crate::terrain::generation::ChunkCellMatrix) when known).
#[derive(Resource, Clone, Debug)]
pub struct StrategicRasterConfig {
    pub cells_per_chunk: UVec2,
}

impl Default for StrategicRasterConfig {
    fn default() -> Self {
        Self {
            cells_per_chunk: UVec2::new(32, 32),
        }
    }
}

fn parse_tile_node_key(key: &str) -> Option<(u32, u32)> {
    let rest = key.strip_prefix('t')?;
    let (xs, zs) = rest.split_once('_')?;
    Some((xs.parse().ok()?, zs.parse().ok()?))
}

fn tile_to_chunk_key(tx: u32, tz: u32, cells: UVec2) -> ChunkCellKey {
    let sx = cells.x.max(1);
    let sy = cells.y.max(1);
    let cx = (tx / sx) as i32;
    let cz = (tz / sy) as i32;
    let lx = tx % sx;
    let lz = tz % sy;
    let cell = lz * sx + lx;
    ChunkCellKey::new(IVec2::new(cx, cz), cell)
}

fn corridor_type_for_profile(profile: &str) -> CorridorType {
    let p = profile.to_ascii_lowercase();
    if p.contains("rail") {
        CorridorType::Rail
    } else if p.contains("pipe") {
        CorridorType::Pipeline
    } else if p.contains("power") || p.contains("grid") {
        CorridorType::PowerTransmission
    } else if p.contains("military") || p.contains("supply") {
        CorridorType::MilitarySupply
    } else if p.contains("highway") || p.contains("road") || p == "default_road" {
        CorridorType::Highway
    } else {
        CorridorType::Logistics
    }
}

/// Pure rebuild used by tests and the sync system.
#[must_use]
pub fn rebuild_logistics_graph_from_transport(
    directory: &TransportEdgeDirectory,
    fields: &TransportFieldStore,
    weights: &TransportCostWeights,
    cells: &StrategicRasterConfig,
    book: &CorridorConstructionBook,
) -> LogisticsGraph {
    let mut graph = LogisticsGraph::default();
    if directory.by_edge.is_empty() {
        return graph;
    }

    let per_chunk = cells.cells_per_chunk.max(UVec2::ONE);
    let mut node_map: HashMap<ChunkCellKey, LogisticsNodeId> = HashMap::new();
    let mut next_raw: u32 = 0;

    let register_node =
        |key: ChunkCellKey,
         node_map: &mut HashMap<ChunkCellKey, LogisticsNodeId>,
         nodes: &mut Vec<LogisticsNode>,
         next: &mut u32|
         -> LogisticsNodeId {
            if let Some(id) = node_map.get(&key) {
                return *id;
            }
            let id = LogisticsNodeId(*next);
            *next += 1;
            node_map.insert(key, id);
            nodes.push(LogisticsNode {
                id,
                throughput: 0.0,
                stockpile: 0.0,
                anchor: Some(key),
            });
            id
        };

    let mut nodes: Vec<LogisticsNode> = Vec::new();
    let mut edges_out: Vec<LogisticsEdge> = Vec::with_capacity(directory.by_edge.len());

    for (&eid, meta) in directory.by_edge.iter() {
        let Some((tx0, tz0)) = parse_tile_node_key(&meta.head_key) else {
            continue;
        };
        let Some((tx1, tz1)) = parse_tile_node_key(&meta.tail_key) else {
            continue;
        };
        let k0 = tile_to_chunk_key(tx0, tz0, per_chunk);
        let k1 = tile_to_chunk_key(tx1, tz1, per_chunk);
        let from = register_node(k0, &mut node_map, &mut nodes, &mut next_raw);
        let to = register_node(k1, &mut node_map, &mut nodes, &mut next_raw);

        let state = fields.by_edge.get(&eid).cloned().unwrap_or_default();
        let cost = edge_traversal_cost(&state, weights, state.travel_time_base);
        let tf = book.traffic_factor(eid);
        let capacity = ((2.0 / cost.max(0.08)).min(3.0)) * tf;
        let disruption =
            (state.damage + state.congestion * 0.45 + state.danger * 0.25).clamp(0.0, 1.0);

        edges_out.push(LogisticsEdge {
            from,
            to,
            capacity,
            disruption,
            traversal_cost: cost,
        });
    }

    graph.nodes = nodes;
    graph.edges = edges_out;
    graph
}

pub fn sync_logistics_graph_from_transport(
    directory: Res<TransportEdgeDirectory>,
    fields: Res<TransportFieldStore>,
    weights: Res<TransportCostWeights>,
    cells: Res<StrategicRasterConfig>,
    book: Res<CorridorConstructionBook>,
    mut graph: ResMut<LogisticsGraph>,
) {
    if directory.by_edge.is_empty() {
        return;
    }
    *graph = rebuild_logistics_graph_from_transport(&directory, &fields, &weights, &cells, &book);
}

/// Keeps [`CorridorConstructionStatus`] on corridor entities aligned with [`CorridorConstructionBook`].
pub fn apply_corridor_construction_book_to_entities(
    book: Res<CorridorConstructionBook>,
    mut q: Query<(&StrategicTransportCorridor, &mut CorridorConstructionStatus)>,
) {
    for (link, mut st) in &mut q {
        let next = book
            .by_edge
            .get(&link.edge_id)
            .copied()
            .unwrap_or_default();
        *st = next;
    }
}

/// Mean transport **damage** 0..1 — feeds [`super::sim::LogisticsAiRuntime`].
pub fn transport_mean_damage(fields: &TransportFieldStore) -> f32 {
    if fields.by_edge.is_empty() {
        return 0.0;
    }
    let mut s = 0.0f32;
    for st in fields.by_edge.values() {
        s += st.damage.clamp(0.0, 1.0);
    }
    s / fields.by_edge.len() as f32
}

/// Push per-cell **routing congestion** and **EW denial** scalars from transport edge endpoints.
pub fn inject_transport_scalar_fields_into_overlays(
    directory: Res<TransportEdgeDirectory>,
    fields: Res<TransportFieldStore>,
    cells: Res<StrategicRasterConfig>,
    book: Res<CorridorConstructionBook>,
    policy: Res<super::schedule::StrategicOverlayDisplayPolicy>,
    mut scratch: ResMut<super::schedule::StrategicOverlayCouplingScratch>,
    mut q: Query<(&Chunk, &mut ChunkStrategicOverlay)>,
) {
    let per_chunk = cells.cells_per_chunk.max(UVec2::ONE);
    let mut bumps: HashMap<IVec2, Vec<(usize, f32, f32)>> = HashMap::new();

    if !directory.by_edge.is_empty() {
        for (&eid, meta) in directory.by_edge.iter() {
            let tf = book.traffic_factor(eid);
            if tf <= 0.0 {
                continue;
            }
            let Some((tx0, tz0)) = parse_tile_node_key(&meta.head_key) else {
                continue;
            };
            let Some((tx1, tz1)) = parse_tile_node_key(&meta.tail_key) else {
                continue;
            };
            let state = fields.by_edge.get(&eid);
            let congestion = state.map(|s| s.congestion).unwrap_or(0.0);
            let danger = state.map(|s| s.danger).unwrap_or(0.0);
            let heat = state.map(|s| s.heat).unwrap_or(0.0);

            for (tx, tz) in [(tx0, tz0), (tx1, tz1)] {
                let ck = tile_to_chunk_key(tx, tz, per_chunk);
                let c_add = congestion * 0.5 * tf;
                let ew_add = (danger * 0.35 + heat * 0.2 + congestion * 0.1) * tf;
                bumps.entry(ck.chunk).or_default().push((
                    ck.cell_index as usize,
                    c_add,
                    ew_add,
                ));
            }
        }
    }

    for (chunk, mut overlay) in q.iter_mut() {
        if policy.apply_routing_congestion {
            overlay.routing_congestion.fill(0.0);
        }
        if policy.apply_ew_denial {
            overlay.ew_denial.fill(0.0);
        }
        let Some(list) = bumps.get(&chunk.coord) else {
            continue;
        };
        scratch.mark_dirty(chunk.coord);
        for &(i, c, e) in list {
            if i < overlay.len_cells() {
                if policy.apply_routing_congestion {
                    overlay.routing_congestion[i] =
                        (overlay.routing_congestion[i] + c).min(1.0);
                }
                if policy.apply_ew_denial {
                    overlay.ew_denial[i] = (overlay.ew_denial[i] + e).min(1.0);
                }
            }
        }
    }
}

pub fn maintain_strategic_corridor_entities(
    directory: Res<TransportEdgeDirectory>,
    book: Res<CorridorConstructionBook>,
    mut commands: Commands,
    q: Query<(Entity, &StrategicTransportCorridor)>,
) {
    if directory.by_edge.is_empty() {
        return;
    }
    let active: HashSet<TransportEdgeId> = directory.by_edge.keys().copied().collect();
    let mut by_edge: HashMap<TransportEdgeId, Entity> = HashMap::new();
    for (ent, link) in q.iter() {
        by_edge.insert(link.edge_id, ent);
    }

    for (eid, ent) in &by_edge {
        if !active.contains(eid) {
            commands.entity(*ent).despawn();
        }
    }

    for eid in active {
        if by_edge.contains_key(&eid) {
            continue;
        }
        let ctype = directory
            .by_edge
            .get(&eid)
            .map(|m| corridor_type_for_profile(&m.profile))
            .unwrap_or(CorridorType::Logistics);
        let construction = book
            .by_edge
            .get(&eid)
            .copied()
            .unwrap_or_default();
        commands.spawn((
            InfrastructureCorridor::new(ctype),
            StrategicTransportCorridor { edge_id: eid },
            construction,
            Name::new(format!("strategic_corridor_{}", eid.0)),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::terrain::MaterialUnificationPlugin;
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};
    use bevy::asset::AssetPlugin;

    #[test]
    fn rebuild_graph_from_directory() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(0),
            crate::systems::transport::TransportEdgeMeta {
                profile: "default_road".into(),
                head_key: "t0_0".into(),
                tail_key: "t1_0".into(),
                ..default()
            },
        );
        let mut fields = TransportFieldStore::default();
        fields.by_edge.insert(
            TransportEdgeId(0),
            crate::systems::transport::EdgeFieldState {
                travel_time_base: 1.0,
                congestion: 0.2,
                damage: 0.1,
                ..default()
            },
        );
        let weights = TransportCostWeights::default();
        let cells = StrategicRasterConfig {
            cells_per_chunk: UVec2::new(4, 4),
        };
        let g = rebuild_logistics_graph_from_transport(&dir, &fields, &weights, &cells, &CorridorConstructionBook::default());
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
    }

    #[test]
    fn construction_book_scales_logistics_capacity() {
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(0),
            crate::systems::transport::TransportEdgeMeta {
                profile: "default_road".into(),
                head_key: "t0_0".into(),
                tail_key: "t1_0".into(),
                ..default()
            },
        );
        let mut fields = TransportFieldStore::default();
        fields.by_edge.insert(
            TransportEdgeId(0),
            crate::systems::transport::EdgeFieldState {
                travel_time_base: 1.0,
                ..default()
            },
        );
        let weights = TransportCostWeights::default();
        let cells = StrategicRasterConfig {
            cells_per_chunk: UVec2::new(4, 4),
        };
        let baseline = rebuild_logistics_graph_from_transport(
            &dir,
            &fields,
            &weights,
            &cells,
            &CorridorConstructionBook::default(),
        );
        let base_cap = baseline.edges[0].capacity;
        assert!(base_cap > 0.0);

        let mut planned_book = CorridorConstructionBook::default();
        planned_book.by_edge.insert(
            TransportEdgeId(0),
            CorridorConstructionStatus {
                phase: crate::strategic::CorridorConstructionPhase::Planned,
                progress: 0.0,
            },
        );
        let g2 = rebuild_logistics_graph_from_transport(&dir, &fields, &weights, &cells, &planned_book);
        assert!(g2.edges[0].capacity < 1e-5);

        let mut half_book = CorridorConstructionBook::default();
        half_book.by_edge.insert(
            TransportEdgeId(0),
            CorridorConstructionStatus {
                phase: crate::strategic::CorridorConstructionPhase::InProgress,
                progress: 0.5,
            },
        );
        let g3 = rebuild_logistics_graph_from_transport(&dir, &fields, &weights, &cells, &half_book);
        assert!(g3.edges[0].capacity > 0.0);
        assert!(g3.edges[0].capacity < base_cap * 0.51);
    }

    #[test]
    fn inject_marks_overlay_cells() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .add_plugins(MaterialUnificationPlugin)
            .insert_resource(StrategicRasterConfig {
                cells_per_chunk: UVec2::new(8, 8),
            })
            .insert_resource({
                let mut d = TransportEdgeDirectory::default();
                d.by_edge.insert(
                    TransportEdgeId(0),
                    crate::systems::transport::TransportEdgeMeta {
                        head_key: "t0_0".into(),
                        tail_key: "t1_0".into(),
                        ..default()
                    },
                );
                d
            })
            .insert_resource({
                let mut f = TransportFieldStore::default();
                f.by_edge.insert(
                    TransportEdgeId(0),
                    crate::systems::transport::EdgeFieldState {
                        congestion: 0.8,
                        danger: 0.5,
                        ..default()
                    },
                );
                f
            })
            .init_resource::<crate::strategic::StrategicOverlayDisplayPolicy>()
            .init_resource::<crate::strategic::StrategicOverlayCouplingScratch>()
            .init_resource::<CorridorConstructionBook>()
            .add_systems(Update, inject_transport_scalar_fields_into_overlays);

        app.world_mut().spawn((
            Chunk {
                coord: IVec2::ZERO,
            },
            ChunkCellMatrix::new(UVec2::new(8, 8)),
            ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(8, 8)),
        ));

        app.update();
        let mut q = app.world_mut().query::<&ChunkStrategicOverlay>();
        let ov = q.iter(app.world()).next().expect("overlay");
        assert!(ov.routing_congestion[0] > 0.0);
        assert!(ov.ew_denial[0] > 0.0);
    }
}
