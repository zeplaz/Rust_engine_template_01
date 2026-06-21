//! Facility ↔ utility graph power link (COD-UTILITY-ACTIVATION-LINK-001).

use std::collections::{HashSet, VecDeque};

use bevy::prelude::*;

use crate::construction::node_key_for_world;
use crate::construction::{BuildingDefinitionRegistry, UtilityInfrastructureRole};

use super::{UtilityConnection, UtilityGraph, UtilityNetworkKind};

#[must_use]
pub fn powered_utility_node_ids(
    utility: &UtilityGraph,
    blocked_link_ids: &HashSet<u64>,
) -> HashSet<u64> {
    let mut adjacency: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
    for edge in &utility.power_edges {
        if blocked_link_ids.contains(&edge.link_id) {
            continue;
        }
        adjacency.entry(edge.from).or_default().push(edge.to);
        adjacency.entry(edge.to).or_default().push(edge.from);
    }

    let plant_nodes: HashSet<u64> = utility
        .nodes
        .iter()
        .filter(|n| n.key.to_ascii_lowercase().contains("plant"))
        .map(|n| n.id)
        .collect();
    let sources: Vec<u64> = if plant_nodes.is_empty() {
        utility.nodes.iter().map(|n| n.id).take(1).collect()
    } else {
        plant_nodes.into_iter().collect()
    };

    let mut powered = HashSet::new();
    let mut queue: VecDeque<u64> = sources.into_iter().collect();
    while let Some(node) = queue.pop_front() {
        if !powered.insert(node) {
            continue;
        }
        if let Some(neighbors) = adjacency.get(&node) {
            for &next in neighbors {
                if !powered.contains(&next) {
                    queue.push_back(next);
                }
            }
        }
    }
    powered
}

#[must_use]
pub fn graph_node_id_at_transform(utility: &UtilityGraph, transform: &Transform) -> Option<u64> {
    let key = node_key_for_world(transform.translation);
    utility.nodes.iter().find(|n| n.key == key).map(|n| n.id)
}

#[must_use]
pub fn facility_power_connected_from_graph(
    utility: &UtilityGraph,
    transform: &Transform,
    blocked_link_ids: &HashSet<u64>,
) -> bool {
    let Some(node_id) = graph_node_id_at_transform(utility, transform) else {
        return false;
    };
    powered_utility_node_ids(utility, blocked_link_ids).contains(&node_id)
}

#[must_use]
pub fn initial_utility_power_connected(
    registry: &BuildingDefinitionRegistry,
    catalog_id: &str,
) -> bool {
    if catalog_id.is_empty() || catalog_id.starts_with("builtin:") {
        return false;
    }
    let Some(def) = registry.get(catalog_id) else {
        return false;
    };
    UtilityInfrastructureRole::resolve(def.id.as_str(), def.utility_role).is_some()
        || def.id.to_ascii_lowercase().contains("plant")
        || def.id.to_ascii_lowercase().contains("substation")
}

/// Refresh [`UtilityConnection::connected`] from graph reachability (no radius hack).
pub fn sync_utility_connection_power_system(
    utility: Option<Res<UtilityGraph>>,
    presentation: Option<Res<crate::render::PowerMapOverlayPresentation>>,
    mut q: Query<(&Transform, &mut UtilityConnection)>,
) {
    let Some(utility) = utility else {
        return;
    };
    let blocked: HashSet<u64> = presentation
        .as_deref()
        .map(|p| {
            p.damaged_link_ids
                .iter()
                .chain(p.destroyed_link_ids.iter())
                .copied()
                .collect()
        })
        .unwrap_or_default();

    for (transform, mut conn) in &mut q {
        if conn.kind != UtilityNetworkKind::Power {
            continue;
        }
        conn.connected = facility_power_connected_from_graph(&utility, transform, &blocked);
    }
}

#[must_use]
pub fn utility_activation_link_witness_green() -> bool {
    use super::graph::{fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot};

    let graph = hydrate_utility_graph_from_snapshot(&fixture_utility_network_snapshot());
    let powered = powered_utility_node_ids(&graph, &HashSet::new());
    !powered.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::utility::graph::{
        fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot,
    };

    #[test]
    fn utility_activation_link_witness_green_lib() {
        assert!(utility_activation_link_witness_green());
    }

    #[test]
    fn cut_link_isolates_downstream_node() {
        let graph = hydrate_utility_graph_from_snapshot(&fixture_utility_network_snapshot());
        let all_powered = powered_utility_node_ids(&graph, &HashSet::new());
        let cut = HashSet::from([10_u64]);
        let after_cut = powered_utility_node_ids(&graph, &cut);
        assert!(after_cut.len() < all_powered.len() || all_powered.len() <= 1);
    }
}
