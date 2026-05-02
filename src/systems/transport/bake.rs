//! **W1 / R9** — polyline snapshot from tile markers in **authoring order** (map editor **M4**).
//!
//! Do **not** lexicographically sort tiles when building transport edges — that scrambles designer intent.
//! Order comes from [`MapEditorRoadMarkerV1::placement_seq`](`crate::gui::editor::map_editor::MapEditorRoadMarkerV1`) (or any ordered slice you pass in).

use super::snapshot::{
    TransportEdgeRecord, TransportNetworkSnapshot, TransportNodeRecord, TRANSPORT_NETWORK_SCHEMA_V1,
};

pub const DEFAULT_ROAD_PROFILE: &str = "default_road";

fn node_key(x: u32, z: u32) -> String {
    format!("t{x}_{z}")
}

fn collapse_consecutive_duplicate_tiles(points: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for &p in points {
        if out.last() == Some(&p) {
            continue;
        }
        out.push(p);
    }
    out
}

/// **`markers`** must be in **authoring order** (e.g. sorted by `placement_seq`). Only **consecutive**
/// duplicate tiles are removed (double-click same tile). Builds an **R8** snapshot: `control_points`,
/// `profile`, `allowed_agents`.
pub fn bake_snapshot_from_ordered_tile_markers(
    markers_in_authoring_order: &[(u32, u32)],
    height_normalized_at: impl Fn(u32, u32) -> f32,
    y_world_scale: f32,
    y_marker_bias: f32,
) -> TransportNetworkSnapshot {
    let pts = collapse_consecutive_duplicate_tiles(markers_in_authoring_order);

    if pts.len() < 2 {
        return TransportNetworkSnapshot {
            schema_version: TRANSPORT_NETWORK_SCHEMA_V1,
            nodes: vec![],
            edges: vec![],
        };
    }

    let mut nodes: Vec<TransportNodeRecord> = Vec::with_capacity(pts.len());
    for &(x, z) in &pts {
        let hn = height_normalized_at(x, z);
        let y = hn * y_world_scale + y_marker_bias;
        nodes.push(TransportNodeRecord {
            key: node_key(x, z),
            position: [x as f32, y, z as f32],
        });
    }

    let n_edge = pts.len() - 1;
    let mut edges: Vec<TransportEdgeRecord> = Vec::with_capacity(n_edge);
    for i in 0..n_edge {
        let (x0, z0) = pts[i];
        let (x1, z1) = pts[i + 1];
        let h0 = height_normalized_at(x0, z0);
        let h1 = height_normalized_at(x1, z1);
        let y0 = h0 * y_world_scale + y_marker_bias;
        let y1 = h1 * y_world_scale + y_marker_bias;
        let p0 = [x0 as f32, y0, z0 as f32];
        let p1 = [x1 as f32, y1, z1 as f32];
        let successors = if i + 1 < n_edge {
            vec![(i + 1) as u64]
        } else {
            vec![]
        };
        edges.push(TransportEdgeRecord {
            id: i as u64,
            head: node_key(x0, z0),
            tail: node_key(x1, z1),
            successors,
            control_points: vec![p0, p1],
            profile: DEFAULT_ROAD_PROFILE.into(),
            allowed_agents: vec!["road_vehicle".into()],
        });
    }

    TransportNetworkSnapshot {
        schema_version: TRANSPORT_NETWORK_SCHEMA_V1,
        nodes,
        edges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::transport::hydrate_transport_from_snapshot;
    use crate::systems::transport::{TransportEdgeDirectory, TransportFieldStore, TransportTopology};

    #[test]
    fn bake_three_markers_chain_ordered() {
        let markers = vec![(0u32, 0u32), (1u32, 0u32), (2u32, 0u32)];
        let snap = bake_snapshot_from_ordered_tile_markers(&markers, |_, _| 0.5_f32, 20., 0.25);
        assert_eq!(snap.edges.len(), 2);
        assert_eq!(snap.nodes.len(), 3);
        let mut top = TransportTopology::default();
        let mut field = TransportFieldStore::default();
        let mut dir = TransportEdgeDirectory::default();
        hydrate_transport_from_snapshot(&mut top, &mut field, &mut dir, &snap).unwrap();
        assert_eq!(
            top.neighbors[&crate::systems::transport::TransportEdgeId(0)],
            vec![crate::systems::transport::TransportEdgeId(1)]
        );
    }

    /// Lexicographic sort would connect (0,0)→(1,0)→(2,0). Authoring order must not.
    #[test]
    fn bake_preserves_authoring_order_not_lexicographic() {
        let markers = vec![(2u32, 0u32), (0u32, 0u32), (1u32, 0u32)];
        let snap = bake_snapshot_from_ordered_tile_markers(&markers, |_, _| 0.5_f32, 20., 0.25);
        assert_eq!(snap.edges.len(), 2);
        assert_eq!(snap.edges[0].control_points[0][0], 2.0);
        assert_eq!(snap.edges[0].control_points[1][0], 0.0);
        assert_eq!(snap.edges[1].control_points[0][0], 0.0);
        assert_eq!(snap.edges[1].control_points[1][0], 1.0);
    }

    #[test]
    fn collapse_consecutive_duplicates_only() {
        let markers = vec![(0u32, 0u32), (0u32, 0u32), (1u32, 0u32)];
        let snap = bake_snapshot_from_ordered_tile_markers(&markers, |_, _| 0.5_f32, 20., 0.25);
        assert_eq!(snap.edges.len(), 1);
    }
}
