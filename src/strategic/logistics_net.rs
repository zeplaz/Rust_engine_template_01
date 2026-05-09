//! **Graph → field:** [`LogisticsGraph`](super::LogisticsGraph) edges push effective throughput into
//! [`ChunkStrategicOverlay::logistics_throughput`] and mirror a neutral summary into `logistics_strength[..][0]`.
//!
//! Faction-specific routing can later use other slots; slot **0** is reserved for aggregate infrastructure
//! pressure from nets.

use std::collections::HashMap;

use bevy::prelude::*;

use super::{ChunkStrategicOverlay, LogisticsGraph, LogisticsNode, LogisticsNodeId};
use crate::terrain::generation::Chunk;

/// Clears per-cell net throughput, accumulates **effective** edge flow `capacity × (1 − disruption)` split
/// across endpoints, then copies clamped throughput into `logistics_strength[i][0]` for quick AI/debug reads.
pub fn logistics_net_inject_into_overlays(
    graph: Res<LogisticsGraph>,
    mut q: Query<(&Chunk, &mut ChunkStrategicOverlay)>,
) {
    for (_chunk, mut overlay) in q.iter_mut() {
        overlay.logistics_throughput.fill(0.0);
    }

    let id_to_node: HashMap<LogisticsNodeId, &LogisticsNode> =
        graph.nodes.iter().map(|n| (n.id, n)).collect();

    let mut by_chunk: HashMap<IVec2, HashMap<usize, f32>> = HashMap::new();

    for edge in &graph.edges {
        let eff = edge.capacity * (1.0 - edge.disruption.clamp(0.0, 1.0));
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
        *by_chunk
            .entry(ka.chunk)
            .or_default()
            .entry(ka.cell_index as usize)
            .or_insert(0.0) += half;
        *by_chunk
            .entry(kb.chunk)
            .or_default()
            .entry(kb.cell_index as usize)
            .or_insert(0.0) += half;
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
    fn logistics_edge_injects_throughput_at_anchors() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<WorldGenParams>()
            .init_resource::<LogisticsGraph>()
            .add_plugins(MaterialUnificationPlugin)
            .add_plugins(StrategicFieldsPlugin);

        app.world_mut().insert_resource(LogisticsGraph {
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
}
