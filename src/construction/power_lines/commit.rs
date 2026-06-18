//! Commit validated power line segments → [`UtilityNetworkSnapshot`] + [`UtilityGraph`].

use bevy::prelude::*;

use crate::infrastructure::utility::{
    hydrate_utility_graph_from_snapshot, PowerLine, UtilityGraph, UtilityLink,
    UtilityNetworkSnapshot, VoltageClass,
};

use super::placement::ActivePowerLinePlacement;

#[must_use]
pub fn node_key_for_world(p: Vec3) -> String {
    format!("pwr_{}_{}", p.x.floor() as i32, p.z.floor() as i32)
}

#[must_use]
pub fn node_position_from_key(key: &str) -> Vec2 {
    if let Some(rest) = key.strip_prefix("pwr_") {
        let mut parts = rest.split('_');
        if let (Some(xs), Some(zs)) = (parts.next(), parts.next()) {
            if let (Ok(x), Ok(z)) = (xs.parse::<i32>(), zs.parse::<i32>()) {
                return Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
            }
        }
    }
    Vec2::ZERO
}

fn next_link_id(snap: &UtilityNetworkSnapshot) -> u64 {
    snap.edges
        .iter()
        .map(|e| e.id)
        .chain(snap.power_lines.iter().map(|p| p.link_id))
        .max()
        .unwrap_or(100)
        .saturating_add(1)
}

fn ensure_node(snap: &mut UtilityNetworkSnapshot, key: &str) {
    if !snap.nodes.iter().any(|n| n == key) {
        snap.nodes.push(key.to_string());
    }
}

/// Enqueue graph edges for each valid preview segment; rehydrates [`UtilityGraph`].
pub fn commit_power_line_to_utility_graph(
    placement: &mut ActivePowerLinePlacement,
    snap: &mut UtilityNetworkSnapshot,
    graph: &mut UtilityGraph,
    voltage: VoltageClass,
) {
    let segments: Vec<_> = placement
        .generated_segments
        .iter()
        .filter(|s| s.valid)
        .cloned()
        .collect();
    if segments.is_empty() {
        return;
    }

    for seg in &segments {
        let head_key = node_key_for_world(seg.start);
        let tail_key = node_key_for_world(seg.end);
        if head_key == tail_key {
            continue;
        }
        ensure_node(snap, &head_key);
        ensure_node(snap, &tail_key);
        let link_id = next_link_id(snap);
        snap.edges.push(UtilityLink {
            id: link_id,
            head: head_key.clone(),
            tail: tail_key.clone(),
            utility_type: "power".into(),
        });
        snap.power_lines.push(PowerLine { link_id, voltage });
    }

    *graph = hydrate_utility_graph_from_snapshot(snap);
    for node in &mut graph.nodes {
        if node.key.starts_with("pwr_") {
            node.position = node_position_from_key(&node.key);
        }
    }

    placement.clear_path();
}

#[must_use]
pub fn power_line_commit_witness_green() -> bool {
    let mut placement = ActivePowerLinePlacement::default();
    placement.generated_segments.push(super::placement::PowerLineSegmentPreview {
        start: Vec3::new(0.5, 0.0, 0.5),
        end: Vec3::new(3.5, 0.0, 0.5),
        valid: true,
    });
    let mut snap = crate::infrastructure::utility::fixture_utility_network_snapshot();
    let mut graph = hydrate_utility_graph_from_snapshot(&snap);
    let edges_before = snap.edges.len();
    commit_power_line_to_utility_graph(
        &mut placement,
        &mut snap,
        &mut graph,
        VoltageClass::Medium,
    );
    edges_before < snap.edges.len() && graph.power_edges.len() >= 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::utility::fixture_utility_network_snapshot;

    #[test]
    fn commit_adds_power_edge_to_snapshot_and_graph() {
        let mut placement = ActivePowerLinePlacement::default();
        placement.generated_segments.push(super::super::placement::PowerLineSegmentPreview {
            start: Vec3::new(1.5, 0.0, 1.5),
            end: Vec3::new(5.5, 0.0, 1.5),
            valid: true,
        });
        let mut snap = fixture_utility_network_snapshot();
        let before = snap.edges.len();
        let mut graph = hydrate_utility_graph_from_snapshot(&snap);
        commit_power_line_to_utility_graph(
            &mut placement,
            &mut snap,
            &mut graph,
            VoltageClass::Medium,
        );
        assert_eq!(snap.edges.len(), before + 1);
        assert!(graph.power_edges.len() > 2);
        assert!(placement.control_points.is_empty());
    }
}
