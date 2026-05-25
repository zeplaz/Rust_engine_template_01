//! Facility portals + attachment map (LOG-A-02).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::economy::spatial_district::chunk_key_from_site_tile;
use crate::strategic::{
    LogisticsGraph, LogisticsNode, LogisticsNodeId, PlannedSite, StrategicRasterConfig,
};
use crate::terrain::ChunkCellKey;

use super::routes::tile_node_key;
use super::types::{
    FacilityPortal, FacilityPortalRegistered, PortalAttachmentMap, TransportNodeAnchor,
};

pub fn register_facility_portals_system(
    mut commands: Commands,
    cfg: Option<Res<StrategicRasterConfig>>,
    q: Query<
        (Entity, &PlannedSite),
        (
            With<crate::economy::activation::IndustrialFacilityActivated>,
            Without<FacilityPortalRegistered>,
        ),
    >,
) {
    let cells = cfg
        .map(|c| c.cells_per_chunk)
        .unwrap_or(UVec2::new(32, 32));
    for (entity, planned) in &q {
        let anchor = chunk_key_from_site_tile(planned.origin, cells);
        let transport_anchor = TransportNodeAnchor(tile_node_key(planned.origin));
        commands.entity(entity).insert((
            FacilityPortal {
                anchor,
                transport_anchor,
            },
            FacilityPortalRegistered,
            crate::economy::spatial_district::IndustrialDistrictAnchor(anchor),
        ));
    }
}

/// Rebuild transient facility → graph node ids after derived graph sync (LOG-A-02).
pub fn rebuild_portal_attachment_map_system(
    graph: Option<Res<LogisticsGraph>>,
    mut map: ResMut<PortalAttachmentMap>,
    portals: Query<(Entity, &FacilityPortal)>,
) {
    let Some(graph) = graph else {
        return;
    };
    if graph.nodes.is_empty() {
        return;
    }
    if map.revision == graph.revision && !map.facility_to_graph.is_empty() {
        return;
    }
    map.revision = graph.revision;
    map.facility_to_graph.clear();

    let mut by_anchor: HashMap<ChunkCellKey, LogisticsNodeId> = HashMap::new();
    for node in &graph.nodes {
        if let Some(a) = node.anchor {
            by_anchor.insert(a, node.id);
        }
    }

    for (entity, portal) in &portals {
        if let Some(&nid) = by_anchor.get(&portal.anchor) {
            map.facility_to_graph.insert(entity, nid);
            continue;
        }
        let mut best: Option<(LogisticsNodeId, i32)> = None;
        for node in &graph.nodes {
            let Some(a) = node.anchor else {
                continue;
            };
            let dist = (a.chunk.x - portal.anchor.chunk.x).abs()
                + (a.chunk.y - portal.anchor.chunk.y).abs();
            if best.map(|(_, d)| dist < d).unwrap_or(true) {
                best = Some((node.id, dist));
            }
        }
        if let Some((nid, _)) = best {
            map.facility_to_graph.insert(entity, nid);
        }
    }
}

/// Append portal-only nodes for facilities not colocated with a junction (read-only graph extension at rebuild).
pub fn attach_portal_nodes_to_derived_graph(
    graph: &mut LogisticsGraph,
    portals: &[(Entity, ChunkCellKey)],
) -> HashMap<Entity, LogisticsNodeId> {
    let mut out = HashMap::new();
    let mut by_anchor: HashMap<ChunkCellKey, LogisticsNodeId> = graph
        .nodes
        .iter()
        .filter_map(|n| n.anchor.map(|a| (a, n.id)))
        .collect();

    let mut next_id = graph
        .nodes
        .iter()
        .map(|n| n.id.0)
        .max()
        .unwrap_or(0)
        .saturating_add(1);

    for &(entity, anchor) in portals {
        if let Some(&id) = by_anchor.get(&anchor) {
            out.insert(entity, id);
            continue;
        }
        let id = LogisticsNodeId(next_id);
        next_id += 1;
        graph.nodes.push(LogisticsNode {
            id,
            throughput: 1.0,
            stockpile: 0.0,
            anchor: Some(anchor),
        });
        by_anchor.insert(anchor, id);
        out.insert(entity, id);
    }
    out
}
