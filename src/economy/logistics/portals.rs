//! Facility portals + attachment map (LOG-A-02).
//!
//! **NAV-08** (aig3 4.6.3 *Hierarchical Pathfinding on Exclusions* · 4.6.4 *Strange
//! Effects of Hierarchies on Pathfinding* · LEGACY `pathfinding` aig_e3): abstract
//! portal hops invent odd routes when a facility latches onto a far graph node.
//! Exact-anchor attach wins; nearest fallback is **excluded** beyond
//! [`PORTAL_ATTACH_MAX_CHUNK_MANHATTAN`] (leave unattached rather than a bad hop).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::economy::spatial_district::chunk_key_from_site_tile;
use crate::strategic::{LogisticsGraph, LogisticsNodeId, PlannedSite, StrategicRasterConfig};
use crate::terrain::ChunkCellKey;

use super::routes::tile_node_key;
use super::types::{
    FacilityPortal, FacilityPortalRegistered, PortalAttachmentMap, TransportNodeAnchor,
};

/// NAV-08 exclusion: max chunk-Manhattan distance for nearest-portal fallback.
/// Exact anchor match always wins; beyond this radius the facility stays unattached.
pub const PORTAL_ATTACH_MAX_CHUNK_MANHATTAN: i32 = 2;

#[inline]
#[must_use]
pub fn chunk_manhattan(a: ChunkCellKey, b: ChunkCellKey) -> i32 {
    (a.chunk.x - b.chunk.x).abs() + (a.chunk.y - b.chunk.y).abs()
}

/// Resolve graph node for a portal anchor (exact → nearest within exclusion).
#[must_use]
pub fn resolve_portal_attachment(
    portal_anchor: ChunkCellKey,
    by_anchor: &HashMap<ChunkCellKey, LogisticsNodeId>,
    node_anchors: &[(LogisticsNodeId, ChunkCellKey)],
    max_chunk_manhattan: i32,
) -> Option<LogisticsNodeId> {
    if let Some(&nid) = by_anchor.get(&portal_anchor) {
        return Some(nid);
    }
    let mut best: Option<(LogisticsNodeId, i32)> = None;
    for &(nid, anchor) in node_anchors {
        let dist = chunk_manhattan(anchor, portal_anchor);
        if dist > max_chunk_manhattan {
            continue;
        }
        if best.map(|(_, d)| dist < d).unwrap_or(true) {
            best = Some((nid, dist));
        }
    }
    best.map(|(nid, _)| nid)
}

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
    let cells = cfg.map(|c| c.cells_per_chunk).unwrap_or(UVec2::new(32, 32));
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
    let mut node_anchors: Vec<(LogisticsNodeId, ChunkCellKey)> = Vec::new();
    for node in &graph.nodes {
        if let Some(a) = node.anchor {
            by_anchor.insert(a, node.id);
            node_anchors.push((node.id, a));
        }
    }

    for (entity, portal) in &portals {
        if let Some(nid) = resolve_portal_attachment(
            portal.anchor,
            &by_anchor,
            &node_anchors,
            PORTAL_ATTACH_MAX_CHUNK_MANHATTAN,
        ) {
            map.facility_to_graph.insert(entity, nid);
        }
        // NAV-08: far portals stay excluded from the abstract hierarchy (no latch).
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;

    fn key(cx: i32, cy: i32) -> ChunkCellKey {
        ChunkCellKey::new(IVec2::new(cx, cy), 0)
    }

    #[test]
    fn nav08_exact_anchor_wins() {
        let a = key(0, 0);
        let mut by = HashMap::new();
        by.insert(a, LogisticsNodeId(7));
        let nodes = [(LogisticsNodeId(7), a), (LogisticsNodeId(9), key(1, 0))];
        assert_eq!(
            resolve_portal_attachment(a, &by, &nodes, PORTAL_ATTACH_MAX_CHUNK_MANHATTAN),
            Some(LogisticsNodeId(7))
        );
    }

    #[test]
    fn nav08_nearest_within_exclusion_attaches() {
        let portal = key(0, 0);
        let by = HashMap::new();
        let nodes = [
            (LogisticsNodeId(1), key(1, 0)),
            (LogisticsNodeId(2), key(5, 5)),
        ];
        assert_eq!(
            resolve_portal_attachment(portal, &by, &nodes, PORTAL_ATTACH_MAX_CHUNK_MANHATTAN),
            Some(LogisticsNodeId(1))
        );
    }

    #[test]
    fn nav08_far_portal_excluded_from_hierarchy() {
        let portal = key(0, 0);
        let by = HashMap::new();
        let nodes = [(LogisticsNodeId(2), key(10, 10))];
        assert_eq!(
            resolve_portal_attachment(portal, &by, &nodes, PORTAL_ATTACH_MAX_CHUNK_MANHATTAN),
            None,
            "far abstract latch must be excluded (NAV-08 artefact)"
        );
    }
}
