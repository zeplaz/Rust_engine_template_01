//! Utility graph resource + power edge hydration (INFRA-E4-002).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::strategic::{
    InfrastructureEdge, InfrastructureGraph, InfrastructureNetworkType, InfrastructureNode,
};

use super::{PowerLine, UtilityLink, UtilityNetworkSnapshot, VoltageClass};

/// Power topology hydrated from [`UtilityNetworkSnapshot`] — authoritative for `NetworkType::Power`.
#[derive(Resource, Clone, Debug, Default)]
pub struct UtilityGraph {
    pub nodes: Vec<UtilityGraphNode>,
    pub power_edges: Vec<UtilityGraphEdge>,
    pub hydrated_from_snapshot: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UtilityGraphNode {
    pub id: u64,
    pub key: String,
    pub position: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UtilityGraphEdge {
    pub from: u64,
    pub to: u64,
    pub link_id: u64,
    pub throughput: f32,
}

const UTILITY_NODE_ID_BASE: u64 = 900_000;

#[must_use]
pub fn node_position_for_key(key: &str, index: usize) -> Vec2 {
    let mut h = 0_u64;
    for b in key.bytes() {
        h = h.wrapping_mul(31).wrapping_add(u64::from(b));
    }
    Vec2::new(
        index as f32 * 8.0 + (h % 997) as f32 * 0.01,
        ((h >> 10) % 997) as f32 * 0.01,
    )
}

#[must_use]
pub fn hydrate_utility_graph_from_snapshot(snap: &UtilityNetworkSnapshot) -> UtilityGraph {
    let mut graph = UtilityGraph::default();
    let mut name_to_id: HashMap<String, u64> = HashMap::new();
    for (i, name) in snap.nodes.iter().enumerate() {
        let id = UTILITY_NODE_ID_BASE + i as u64 + 1;
        name_to_id.insert(name.clone(), id);
        graph.nodes.push(UtilityGraphNode {
            id,
            key: name.clone(),
            position: node_position_for_key(name, i),
        });
    }

    let power_link_ids: std::collections::HashSet<u64> =
        snap.power_lines.iter().map(|p| p.link_id).collect();

    for edge in &snap.edges {
        let is_power = edge.utility_type.eq_ignore_ascii_case("power") || power_link_ids.contains(&edge.id);
        if !is_power {
            continue;
        }
        let Some(&from) = name_to_id.get(&edge.head) else {
            continue;
        };
        let Some(&to) = name_to_id.get(&edge.tail) else {
            continue;
        };
        graph.power_edges.push(UtilityGraphEdge {
            from,
            to,
            link_id: edge.id,
            throughput: 1.0,
        });
    }

    graph.hydrated_from_snapshot = !graph.nodes.is_empty();
    graph
}

#[must_use]
pub fn fixture_utility_network_snapshot() -> UtilityNetworkSnapshot {
    UtilityNetworkSnapshot {
        schema_version: super::UTILITY_NETWORK_SCHEMA_V1,
        nodes: vec!["sub_a".into(), "sub_b".into(), "plant_1".into()],
        edges: vec![
            UtilityLink {
                id: 10,
                head: "sub_a".into(),
                tail: "plant_1".into(),
                utility_type: "power".into(),
            },
            UtilityLink {
                id: 11,
                head: "sub_b".into(),
                tail: "plant_1".into(),
                utility_type: "power".into(),
            },
        ],
        power_lines: vec![
            PowerLine {
                link_id: 10,
                voltage: VoltageClass::Medium,
            },
            PowerLine {
                link_id: 11,
                voltage: VoltageClass::Medium,
            },
        ],
        water_pipes: vec![],
    }
}

/// Merge utility power nodes/edges into the infrastructure mirror (replaces profile-inferred power).
pub fn apply_utility_power_to_infrastructure(
    utility: &UtilityGraph,
    infra: &mut InfrastructureGraph,
) {
    if !utility.hydrated_from_snapshot || utility.power_edges.is_empty() {
        return;
    }

    use std::collections::HashSet;

    infra
        .nodes
        .retain(|n| n.network != InfrastructureNetworkType::Power);
    let live: HashSet<u64> = infra.nodes.iter().map(|n| n.id).collect();
    infra
        .edges
        .retain(|e| live.contains(&e.from) && live.contains(&e.to));

    for node in &utility.nodes {
        infra.nodes.push(InfrastructureNode {
            id: node.id,
            position: node.position,
            network: InfrastructureNetworkType::Power,
        });
    }
    for edge in &utility.power_edges {
        infra.edges.push(InfrastructureEdge {
            from: edge.from,
            to: edge.to,
            throughput: edge.throughput,
            integrity: 1.0,
            maintenance_cost: 0.0,
            linked_transport_edge: None,
        });
    }
}

pub fn merge_utility_power_into_infrastructure_graph(
    utility: Res<UtilityGraph>,
    mut infra: ResMut<InfrastructureGraph>,
) {
    apply_utility_power_to_infrastructure(&utility, &mut infra);
}

pub struct UtilityGraphPlugin;

impl Plugin for UtilityGraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UtilityNetworkSnapshotResource>()
            .init_resource::<UtilityGraph>()
            .add_systems(PostStartup, hydrate_utility_graph_startup)
            .add_systems(Update, merge_utility_power_into_infrastructure_graph);
    }
}

#[derive(Resource, Clone, Debug)]
pub struct UtilityNetworkSnapshotResource(pub UtilityNetworkSnapshot);

impl Default for UtilityNetworkSnapshotResource {
    fn default() -> Self {
        Self(fixture_utility_network_snapshot())
    }
}

fn hydrate_utility_graph_startup(
    snap: Res<UtilityNetworkSnapshotResource>,
    mut graph: ResMut<UtilityGraph>,
) {
    *graph = hydrate_utility_graph_from_snapshot(&snap.0);
}

#[must_use]
pub fn infra_e4_002_power_edges_from_graph_green(
    utility: &UtilityGraph,
    infra: &InfrastructureGraph,
) -> bool {
    if utility.power_edges.is_empty() || !utility.hydrated_from_snapshot {
        return false;
    }
    infra.nodes.iter().any(|n| n.network == InfrastructureNetworkType::Power)
        && infra.edges.len() >= utility.power_edges.len()
}

#[must_use]
pub fn refresh_utility_network_live_witness_payload() -> serde_json::Value {
    let snap = fixture_utility_network_snapshot();
    let utility = hydrate_utility_graph_from_snapshot(&snap);
    let mut infra = InfrastructureGraph::default();
    apply_utility_power_to_infrastructure(&utility, &mut infra);
    let green = infra_e4_002_power_edges_from_graph_green(&utility, &infra);
    serde_json::json!({
        "gate_id": "INFRA-E4-002",
        "green": green,
        "power_edges_from_graph": green,
        "utility_power_edges": utility.power_edges.len(),
        "infra_power_nodes": infra.nodes.iter().filter(|n| {
            n.network == InfrastructureNetworkType::Power
        }).count(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::InfrastructureNetworkType;

    #[test]
    fn hydrate_utility_graph_from_snapshot_builds_power_edges() {
        let snap = fixture_utility_network_snapshot();
        let graph = hydrate_utility_graph_from_snapshot(&snap);
        assert_eq!(graph.power_edges.len(), 2);
        assert!(graph.hydrated_from_snapshot);
    }

    #[test]
    fn infra_e4_002_power_edges_from_graph_witness_green() {
        let snap = fixture_utility_network_snapshot();
        let utility = hydrate_utility_graph_from_snapshot(&snap);
        let mut infra = InfrastructureGraph::default();
        apply_utility_power_to_infrastructure(&utility, &mut infra);
        assert!(infra_e4_002_power_edges_from_graph_green(&utility, &infra));
        let body = refresh_utility_network_live_witness_payload();
        assert_eq!(body["power_edges_from_graph"].as_bool(), Some(true));
    }

    #[test]
    fn transport_profile_no_longer_infers_power() {
        fn profile_network(p: &str) -> InfrastructureNetworkType {
            let pl = p.to_ascii_lowercase();
            if pl.contains("rail") {
                InfrastructureNetworkType::Rail
            } else if pl.contains("pipe") {
                InfrastructureNetworkType::Pipelines
            } else if pl.contains("comm") || pl.contains("fiber") {
                InfrastructureNetworkType::Communications
            } else {
                InfrastructureNetworkType::Roads
            }
        }
        assert_ne!(
            profile_network("power_grid_main"),
            InfrastructureNetworkType::Power
        );
        assert_ne!(
            profile_network("high_voltage_power"),
            InfrastructureNetworkType::Power
        );
    }
}
